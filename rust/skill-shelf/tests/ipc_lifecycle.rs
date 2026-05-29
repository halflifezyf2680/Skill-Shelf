use std::io::Write;
use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use skill_shelf::ipc::{
    cleanup_stale_state_file, connect_and_handshake, is_state_stale, read_framed_json,
    read_state_file, request_daemon_shutdown, send_framed_json, shutdown_request_frame,
    spawn_token_server, update_state_after_shutdown_ack, write_state_file, DaemonControlRequest,
    DaemonControlResponse, DaemonState,
};
use skill_shelf::lifecycle::{
    process_exists, ParentDeathWatcher, ShutdownPoller, ShutdownReason, StdinEofWatcher,
};
use skill_shelf::lock::{acquire_daemon_lock, LockError};
use skill_shelf::mcp_shim::ForwardRequestEnvelope;
use sysinfo::{Pid, System};

fn sample_state(port: u16, token: &str) -> DaemonState {
    DaemonState {
        pid: std::process::id(),
        port,
        token: token.to_string(),
        version: "0.1.0-test".to_string(),
        started_at_ms: 1_717_171_717,
    }
}

fn reserve_local_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn find_missing_pid() -> u32 {
    for pid in (u16::MAX as u32..u32::MAX).rev().take(4096) {
        if !process_exists(pid) {
            return pid;
        }
    }

    panic!("failed to find a PID that is not currently in use");
}

#[test]
fn daemon_state_roundtrip_uses_atomic_write() {
    let temp = tempfile::tempdir().unwrap();
    let state_path = temp.path().join("daemon-state.json");
    let state = sample_state(43123, "roundtrip-token");

    write_state_file(&state_path, &state).unwrap();

    let raw = std::fs::read_to_string(&state_path).unwrap();
    assert!(raw.contains("\"startedAtMs\":1717171717"));

    let reloaded = read_state_file(&state_path).unwrap();
    assert_eq!(reloaded, state);

    let dir_entries = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(dir_entries, vec!["daemon-state.json".to_string()]);
}

#[test]
fn daemon_state_overwrite_replaces_contents_without_temp_residue() {
    let temp = tempfile::tempdir().unwrap();
    let state_path = temp.path().join("daemon-state.json");
    let first = sample_state(43123, "state-a");
    let second = sample_state(43124, "state-b");

    write_state_file(&state_path, &first).unwrap();
    write_state_file(&state_path, &second).unwrap();

    let reloaded = read_state_file(&state_path).unwrap();
    assert_eq!(reloaded, second);

    let dir_entries = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(dir_entries, vec!["daemon-state.json".to_string()]);
}

#[test]
fn state_is_stale_when_port_refuses_connections() {
    let port = reserve_local_port();
    let state = sample_state(port, "stale-token");

    assert!(is_state_stale(&state, Duration::from_millis(150)).unwrap());
}

#[test]
fn token_handshake_rejects_wrong_token_and_accepts_correct_token() {
    let server = spawn_token_server(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        |_| Ok(()),
    )
    .unwrap();
    let accepted = sample_state(server.local_addr().port(), "expected-token");
    let rejected = sample_state(server.local_addr().port(), "wrong-token");

    let rejected_err = connect_and_handshake(&rejected, Duration::from_secs(1)).unwrap_err();
    assert!(rejected_err.to_string().contains("token"));

    let mut stream = connect_and_handshake(&accepted, Duration::from_secs(1)).unwrap();
    stream.write_all(b"ping").unwrap();
}

#[test]
fn token_server_accepts_multiple_authenticated_clients_concurrently() {
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let accepted_ports = Arc::new(Mutex::new(Vec::new()));
    let accepted_ports_for_server = Arc::clone(&accepted_ports);
    let release_rx_for_server = Arc::clone(&release_rx);

    let server = spawn_token_server(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        move |stream| {
            accepted_ports_for_server
                .lock()
                .unwrap()
                .push(stream.peer_addr().unwrap().port());
            release_rx_for_server.lock().unwrap().recv().unwrap();
            Ok(())
        },
    )
    .unwrap();

    let first_state = sample_state(server.local_addr().port(), "expected-token");
    let second_state = sample_state(server.local_addr().port(), "expected-token");

    let first = thread::spawn(move || connect_and_handshake(&first_state, Duration::from_secs(1)));
    let second =
        thread::spawn(move || connect_and_handshake(&second_state, Duration::from_secs(1)));

    let first_stream = first.join().unwrap().unwrap();
    let second_stream = second.join().unwrap().unwrap();

    for _ in 0..20 {
        if accepted_ports.lock().unwrap().len() == 2 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    assert_eq!(accepted_ports.lock().unwrap().len(), 2);

    drop(first_stream);
    drop(second_stream);
    release_tx.send(()).unwrap();
    release_tx.send(()).unwrap();
}

#[test]
fn authenticated_client_can_send_forward_request_and_receive_response_with_same_request_id() {
    let server = spawn_token_server(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        |mut stream| {
            let envelope: ForwardRequestEnvelope = read_framed_json(&mut stream).unwrap();
            let response = json!({
                "request_id": envelope.request_id,
                "ok": true,
                "echo_method": envelope.method,
            });
            send_framed_json(&mut stream, &response).unwrap();
            Ok(())
        },
    )
    .unwrap();

    let state = sample_state(server.local_addr().port(), "expected-token");
    let mut stream = connect_and_handshake(&state, Duration::from_secs(1)).unwrap();
    let request = ForwardRequestEnvelope {
        request_id: "req-123".to_string(),
        session_id: "session-a".to_string(),
        shelf_root: std::env::temp_dir(),
        config_hash: "cfg-1".to_string(),
        method: "search_skills".to_string(),
        params: json!({ "query": "rust" }),
    };

    send_framed_json(&mut stream, &request).unwrap();
    let response: Value = read_framed_json(&mut stream).unwrap();

    assert_eq!(response["request_id"], "req-123");
    assert_eq!(response["ok"], true);
    assert_eq!(response["echo_method"], "search_skills");
}

#[test]
fn malformed_frame_returns_clear_error_without_killing_server() {
    let first_connection = Arc::new(AtomicBool::new(true));
    let first_connection_for_server = Arc::clone(&first_connection);
    let server = spawn_token_server(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        move |mut stream| {
            if first_connection_for_server.swap(false, Ordering::SeqCst) {
                let bad_frame_err =
                    read_framed_json::<ForwardRequestEnvelope>(&mut stream).unwrap_err();
                assert!(bad_frame_err.to_string().contains("frame"));
                return Ok(());
            }

            let envelope: ForwardRequestEnvelope = read_framed_json(&mut stream).unwrap();
            let response = json!({
                "request_id": envelope.request_id,
                "ok": true,
            });
            send_framed_json(&mut stream, &response).unwrap();
            Ok(())
        },
    )
    .unwrap();

    let state = sample_state(server.local_addr().port(), "expected-token");
    let mut stream = connect_and_handshake(&state, Duration::from_secs(1)).unwrap();

    stream.write_all(b"not-json\n").unwrap();
    drop(stream);

    let request = ForwardRequestEnvelope {
        request_id: "req-after-bad".to_string(),
        session_id: "session-b".to_string(),
        shelf_root: std::env::temp_dir(),
        config_hash: "cfg-2".to_string(),
        method: "read_skill".to_string(),
        params: json!({ "skill": "ipc" }),
    };

    let mut second_stream = connect_and_handshake(&state, Duration::from_secs(1)).unwrap();
    send_framed_json(&mut second_stream, &request).unwrap();
    let second_response: Value = read_framed_json(&mut second_stream).unwrap();
    assert_eq!(second_response["request_id"], "req-after-bad");
}

#[test]
fn multiple_authenticated_clients_can_send_framed_requests_concurrently() {
    let server = spawn_token_server(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        |mut stream| {
            let envelope: ForwardRequestEnvelope = read_framed_json(&mut stream).unwrap();
            let response = json!({
                "request_id": envelope.request_id,
                "session_id": envelope.session_id,
            });
            send_framed_json(&mut stream, &response).unwrap();
            Ok(())
        },
    )
    .unwrap();

    let port = server.local_addr().port();
    let handles = (0..4)
        .map(|idx| {
            thread::spawn(move || {
                let state = sample_state(port, "expected-token");
                let mut stream = connect_and_handshake(&state, Duration::from_secs(1)).unwrap();
                let request = ForwardRequestEnvelope {
                    request_id: format!("req-{idx}"),
                    session_id: format!("session-{idx}"),
                    shelf_root: std::env::temp_dir(),
                    config_hash: "cfg-concurrent".to_string(),
                    method: "validate_skills".to_string(),
                    params: json!({ "skill": format!("skill-{idx}") }),
                };

                send_framed_json(&mut stream, &request).unwrap();
                let response: Value = read_framed_json(&mut stream).unwrap();
                (
                    request.request_id,
                    request.session_id,
                    response["request_id"].as_str().unwrap().to_string(),
                    response["session_id"].as_str().unwrap().to_string(),
                )
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        let (request_id, session_id, echoed_request_id, echoed_session_id) = handle.join().unwrap();
        assert_eq!(echoed_request_id, request_id);
        assert_eq!(echoed_session_id, session_id);
    }
}

#[test]
fn daemon_lock_is_exclusive_until_drop() {
    let temp = tempfile::tempdir().unwrap();
    let lock_path = temp.path().join("daemon.lock");

    let first = acquire_daemon_lock(&lock_path).unwrap();
    let second = acquire_daemon_lock(&lock_path).unwrap_err();
    assert!(matches!(second, LockError::AlreadyRunning), "{second:?}");

    drop(first);

    let third = acquire_daemon_lock(&lock_path).unwrap();
    drop(third);
}

#[test]
fn stdin_eof_watcher_only_fires_when_main_loop_reports_eof() {
    let (watcher, notifier) = StdinEofWatcher::new();

    notifier.notify_read(1);
    assert_eq!(watcher.poll(), None);

    notifier.notify_eof();
    assert_eq!(watcher.poll(), Some(ShutdownReason::StdinEof));
}

#[test]
fn parent_death_watcher_treats_start_time_mismatch_as_dead_parent() {
    let parent_pid = std::process::id();
    let system = System::new_all();
    let process = system
        .process(Pid::from_u32(parent_pid))
        .expect("current process should exist");
    let actual_start_time = process.start_time();

    let watcher = ParentDeathWatcher::with_expected_start_time(parent_pid, actual_start_time + 1);

    assert!(!watcher.is_parent_alive());
    assert_eq!(watcher.poll(), Some(ShutdownReason::ParentDeath));
}

#[test]
fn parent_death_watcher_reports_parent_death_when_pid_does_not_exist() {
    let missing_pid = find_missing_pid();
    let watcher = ParentDeathWatcher::new(missing_pid);

    assert!(!watcher.is_parent_alive());
    assert_eq!(watcher.poll(), Some(ShutdownReason::ParentDeath));
}

#[test]
fn shutdown_poller_returns_stdin_eof_before_parent_death() {
    let (stdin_watcher, notifier) = StdinEofWatcher::new();
    let parent_watcher = ParentDeathWatcher::new(find_missing_pid());
    let poller = ShutdownPoller::new(Some(&stdin_watcher), Some(&parent_watcher));

    notifier.notify_eof();

    assert_eq!(poller.poll(), Some(ShutdownReason::StdinEof));
}

#[test]
fn shutdown_poller_returns_parent_death_when_stdin_is_still_open() {
    let (stdin_watcher, _notifier) = StdinEofWatcher::new();
    let parent_watcher = ParentDeathWatcher::new(find_missing_pid());
    let poller = ShutdownPoller::new(Some(&stdin_watcher), Some(&parent_watcher));

    assert_eq!(poller.poll(), Some(ShutdownReason::ParentDeath));
}

#[test]
fn shutdown_request_uses_control_frame_contract_and_clears_state_on_ack() {
    let temp = tempfile::tempdir().unwrap();
    let state_path = temp.path().join("daemon-state.json");
    let expected_request_id = "stop-req-1";
    let server = spawn_token_server(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        move |mut stream| {
            let request: DaemonControlRequest = read_framed_json(&mut stream).unwrap();
            assert_eq!(request.request_id, expected_request_id);
            assert_eq!(request.method, "daemon/shutdown");
            assert_eq!(request.params, json!({ "reason": "cli-stop" }));
            send_framed_json(
                &mut stream,
                &DaemonControlResponse {
                    request_id: request.request_id,
                    accepted: true,
                },
            )
            .unwrap();
            Ok(())
        },
    )
    .unwrap();

    let state = sample_state(server.local_addr().port(), "expected-token");
    write_state_file(&state_path, &state).unwrap();

    let response = request_daemon_shutdown(
        &state,
        Duration::from_secs(1),
        shutdown_request_frame(expected_request_id, "cli-stop"),
    )
    .unwrap();
    assert!(response.accepted);

    assert!(update_state_after_shutdown_ack(&state_path, &response).unwrap());
    assert!(!state_path.exists());
}

#[test]
fn cleanup_stale_state_file_removes_dead_daemon_state() {
    let temp = tempfile::tempdir().unwrap();
    let state_path = temp.path().join("daemon-state.json");
    let state = DaemonState {
        pid: find_missing_pid(),
        port: reserve_local_port(),
        token: "dead-token".to_string(),
        version: "0.1.0-test".to_string(),
        started_at_ms: 99,
    };
    write_state_file(&state_path, &state).unwrap();

    assert!(cleanup_stale_state_file(&state_path, Duration::from_millis(50)).unwrap());
    assert!(!state_path.exists());
}
