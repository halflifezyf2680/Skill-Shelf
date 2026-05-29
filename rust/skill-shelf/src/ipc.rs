use std::collections::BTreeMap;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::lifecycle::process_exists;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonState {
    pub pid: u32,
    pub port: u16,
    pub token: String,
    pub version: String,
    pub started_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutostartReason {
    MissingState,
    StaleState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonCommandPlan {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonAutostartPlan {
    pub reason: AutostartReason,
    pub command: DaemonCommandPlan,
    pub cleaned_stale_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonControlRequest {
    pub request_id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonControlResponse {
    pub request_id: String,
    pub accepted: bool,
}

pub fn read_state_file(path: &Path) -> Result<DaemonState> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read state file {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse state file {}", path.display()))
}

pub fn write_state_file(path: &Path, state: &DaemonState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory {}", parent.display()))?;
    }

    let temp_path = temp_state_path(path);
    let payload = serde_json::to_vec(state).context("failed to serialize daemon state")?;
    let mut temp_file = File::create(&temp_path)
        .with_context(|| format!("failed to create temp state file {}", temp_path.display()))?;
    temp_file
        .write_all(&payload)
        .context("failed to write daemon state")?;
    temp_file
        .sync_all()
        .context("failed to sync daemon state")?;
    drop(temp_file);

    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("failed to replace state file {}", path.display()))?;
    }
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "failed to rename temp state file {} to {}",
            temp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

pub fn is_state_stale(state: &DaemonState, timeout: Duration) -> Result<bool> {
    if !process_exists(state.pid) {
        return Ok(true);
    }

    Ok(connect_and_handshake(state, timeout).is_err())
}

pub fn cleanup_stale_state_file(path: &Path, timeout: Duration) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let state = read_state_file(path)?;
    if !is_state_stale(&state, timeout)? {
        return Ok(false);
    }

    fs::remove_file(path)
        .with_context(|| format!("failed to remove stale state file {}", path.display()))?;
    Ok(true)
}

pub fn plan_daemon_autostart(
    state_path: &Path,
    current_exe: &Path,
    timeout: Duration,
) -> Result<Option<DaemonAutostartPlan>> {
    if !state_path.exists() {
        return Ok(Some(build_autostart_plan(
            state_path,
            current_exe,
            AutostartReason::MissingState,
            false,
        )));
    }

    let cleaned_stale_state = cleanup_stale_state_file(state_path, timeout)?;
    if cleaned_stale_state {
        return Ok(Some(build_autostart_plan(
            state_path,
            current_exe,
            AutostartReason::StaleState,
            true,
        )));
    }

    Ok(None)
}

pub fn connect_and_handshake(state: &DaemonState, timeout: Duration) -> Result<TcpStream> {
    let addr = SocketAddr::from(([127, 0, 0, 1], state.port));
    let mut stream =
        TcpStream::connect_timeout(&addr, timeout).with_context(|| format!("connect {addr}"))?;
    stream
        .set_read_timeout(Some(timeout))
        .context("failed to set read timeout")?;
    stream
        .set_write_timeout(Some(timeout))
        .context("failed to set write timeout")?;

    stream
        .write_all(format!("{}\n", state.token).as_bytes())
        .context("failed to send handshake token")?;
    stream.flush().context("failed to flush handshake token")?;

    let mut response = String::new();
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .context("failed to clone handshake stream")?,
    );
    reader
        .read_line(&mut response)
        .context("failed to read handshake response")?;

    if response.trim() != "OK" {
        bail!("token handshake rejected: {}", response.trim());
    }

    Ok(stream)
}

pub fn shutdown_request_frame(request_id: &str, reason: &str) -> DaemonControlRequest {
    DaemonControlRequest {
        request_id: request_id.to_string(),
        method: "daemon/shutdown".to_string(),
        params: json!({ "reason": reason }),
    }
}

pub fn request_daemon_shutdown(
    state: &DaemonState,
    timeout: Duration,
    request: DaemonControlRequest,
) -> Result<DaemonControlResponse> {
    let mut stream = connect_and_handshake(state, timeout)?;
    send_framed_json(&mut stream, &request)?;
    let response: DaemonControlResponse = read_framed_json(&mut stream)?;
    if response.request_id != request.request_id {
        bail!(
            "daemon shutdown response request_id mismatch: expected {}, got {}",
            request.request_id,
            response.request_id
        );
    }
    Ok(response)
}

pub fn update_state_after_shutdown_ack(
    state_path: &Path,
    response: &DaemonControlResponse,
) -> Result<bool> {
    if !response.accepted || !state_path.exists() {
        return Ok(false);
    }

    fs::remove_file(state_path)
        .with_context(|| format!("failed to remove state file {}", state_path.display()))?;
    Ok(true)
}

pub fn send_framed_json<T>(writer: &mut impl Write, value: &T) -> Result<()>
where
    T: Serialize,
{
    let payload = serde_json::to_vec(value).context("failed to serialize framed json")?;
    writer
        .write_all(&payload)
        .context("failed to write framed payload")?;
    writer
        .write_all(b"\n")
        .context("failed to write frame delimiter")?;
    writer.flush().context("failed to flush framed payload")?;
    Ok(())
}

pub fn read_framed_json<T>(reader: &mut impl Read) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut frame = Vec::new();
    let mut byte = [0_u8; 1];
    let mut saw_delimiter = false;

    loop {
        match reader.read(&mut byte) {
            Ok(0) => {
                if frame.is_empty() {
                    bail!("ipc frame ended before payload");
                }
                break;
            }
            Ok(_) => {
                if byte[0] == b'\n' {
                    saw_delimiter = true;
                    break;
                }
                frame.push(byte[0]);
            }
            Err(err) => {
                return Err(err).context("failed to read ipc frame");
            }
        }
    }

    if !saw_delimiter {
        bail!("ipc frame missing newline delimiter");
    }

    if frame.is_empty() {
        bail!("ipc frame payload is empty");
    }

    serde_json::from_slice(&frame).context("failed to parse ipc frame json")
}

pub struct TokenServerHandle {
    local_addr: SocketAddr,
    shutdown_tx: Sender<()>,
    join_handle: Option<JoinHandle<()>>,
}

pub fn default_daemon_dir() -> PathBuf {
    if let Some(value) = std::env::var_os("SKILL_SHELF_DAEMON_DIR") {
        return PathBuf::from(value);
    }

    if cfg!(windows) {
        if let Some(value) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(value).join("skill-shelf");
        }
    }

    if let Some(value) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(value).join("skill-shelf");
    }

    if let Some(value) = std::env::var_os("HOME") {
        return PathBuf::from(value)
            .join(".local")
            .join("share")
            .join("skill-shelf");
    }

    std::env::temp_dir().join("skill-shelf")
}

pub fn default_state_path() -> PathBuf {
    default_daemon_dir().join("daemon-state.json")
}

pub fn default_lock_path() -> PathBuf {
    default_daemon_dir().join("daemon.lock")
}

pub fn generate_token() -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::process::id().hash(&mut hasher);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

impl TokenServerHandle {
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for TokenServerHandle {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

pub fn spawn_token_server(
    bind_addr: SocketAddr,
    expected_token: &str,
    handler: impl Fn(TcpStream) -> Result<()> + Send + Sync + 'static,
) -> Result<TokenServerHandle> {
    let listener =
        TcpListener::bind(bind_addr).with_context(|| format!("failed to bind {bind_addr}"))?;
    let local_addr = listener
        .local_addr()
        .context("failed to resolve local addr")?;
    listener
        .set_nonblocking(true)
        .context("failed to mark listener nonblocking")?;

    let expected_token = expected_token.to_string();
    let handler = Arc::new(handler);
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let join_handle = thread::spawn(move || loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let expected_token = expected_token.clone();
                let handler = Arc::clone(&handler);
                thread::spawn(move || {
                    let stream = match stream.set_nonblocking(false) {
                        Ok(()) => stream,
                        Err(_) => return,
                    };
                    if let Ok(stream) = authenticate_client(stream, &expected_token) {
                        let _ = handler(stream);
                    }
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                match shutdown_rx.try_recv() {
                    Ok(()) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(20)),
                }
            }
            Err(_) => break,
        }
    });

    Ok(TokenServerHandle {
        local_addr,
        shutdown_tx,
        join_handle: Some(join_handle),
    })
}

fn authenticate_client(mut stream: TcpStream, expected_token: &str) -> Result<TcpStream> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .context("failed to set server read timeout")?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .context("failed to set server write timeout")?;

    let mut token = String::new();
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .context("failed to clone server stream")?,
    );
    reader
        .read_line(&mut token)
        .context("failed to read client token")?;

    if token.trim() != expected_token {
        stream
            .write_all(b"ERR invalid token\n")
            .context("failed to write rejection")?;
        stream.flush().context("failed to flush rejection")?;
        return Err(anyhow!("invalid token"));
    }

    stream
        .write_all(b"OK\n")
        .context("failed to write acceptance")?;
    stream.flush().context("failed to flush acceptance")?;

    Ok(stream)
}

fn build_autostart_plan(
    state_path: &Path,
    current_exe: &Path,
    reason: AutostartReason,
    cleaned_stale_state: bool,
) -> DaemonAutostartPlan {
    let daemon_dir = state_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(default_daemon_dir);
    let mut env = BTreeMap::new();
    env.insert(
        "SKILL_SHELF_DAEMON_DIR".to_string(),
        daemon_dir.to_string_lossy().into_owned(),
    );

    DaemonAutostartPlan {
        reason,
        command: DaemonCommandPlan {
            program: current_exe.to_path_buf(),
            args: vec!["daemon".to_string()],
            env,
        },
        cleaned_stale_state,
    }
}

fn temp_state_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "daemon-state.json".to_string());
    path.with_file_name(format!("{file_name}.tmp-{}-{nanos}", std::process::id()))
}
