use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[cfg(not(windows))]
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context};
use skill_shelf::daemon::{
    spawn_daemon_ipc_server_with_shutdown, DaemonState as RuntimeDaemonState,
};
use skill_shelf::ipc::{
    cleanup_stale_state_file, default_lock_path, default_state_path, generate_token,
    is_state_stale, plan_daemon_autostart, read_state_file, request_daemon_shutdown,
    shutdown_request_frame, update_state_after_shutdown_ack, write_state_file, DaemonAutostartPlan,
    DaemonState as IpcDaemonState,
};
use skill_shelf::lock::acquire_daemon_lock;
use skill_shelf::mcp_shim::{
    default_context, run_stdio_loop, IpcForwarder, McpStdioShim, ShimCli, ShimCommand,
};

fn main() {
    let cli = ShimCli::parse();

    match cli.command {
        ShimCommand::Mcp => {
            let state = match ensure_mcp_daemon_state() {
                Ok(state) => state,
                Err(error) => {
                    eprintln!("failed to prepare skill-shelf daemon: {error:#}");
                    std::process::exit(1);
                }
            };
            let shim = McpStdioShim::new(
                default_context(),
                IpcForwarder::new(state, Duration::from_secs(30)),
            );
            if let Err(error) = run_stdio_loop(io::stdin(), io::stdout(), &shim) {
                eprintln!("mcp stdio loop failed: {error:#}");
                std::process::exit(1);
            }
        }
        ShimCommand::Daemon => {
            if let Err(error) = run_daemon_command() {
                eprintln!("daemon failed: {error:#}");
                std::process::exit(1);
            }
        }
        ShimCommand::Status => {
            let state_path = default_state_path();
            match read_state_file(&state_path) {
                Ok(state) => {
                    let stale = is_state_stale(&state, Duration::from_millis(250)).unwrap_or(true);
                    println!(
                        "pid={} port={} version={} stale={}",
                        state.pid, state.port, state.version, stale
                    );
                }
                Err(_error) if !state_path.exists() => {
                    println!("daemon is not running");
                }
                Err(error) => {
                    eprintln!("failed to read skill-shelf daemon state: {error:#}");
                    std::process::exit(1);
                }
            }
        }
        ShimCommand::Stop => {
            if let Err(error) = run_stop_command() {
                eprintln!("stop failed: {error:#}");
                std::process::exit(1);
            }
        }
    }
}

fn run_daemon_command() -> anyhow::Result<()> {
    detach_daemon_stdio();
    let _lock = acquire_daemon_lock(&default_lock_path())?;
    let token = generate_token();
    let runtime = RuntimeDaemonState::new();
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let server = spawn_daemon_ipc_server_with_shutdown(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        &token,
        runtime,
        move || {
            let _ = shutdown_tx.send(());
        },
    )?;
    let state = IpcDaemonState {
        pid: std::process::id(),
        port: server.local_addr().port(),
        token,
        version: env!("CARGO_PKG_VERSION").to_string(),
        started_at_ms: unix_time_ms(),
    };
    write_state_file(&default_state_path(), &state)?;
    eprintln!(
        "skill-shelf daemon ready | pid={} | port={}",
        state.pid, state.port
    );

    let _ = shutdown_rx.recv();
    Ok(())
}

#[cfg(windows)]
fn detach_daemon_stdio() {
    use std::ffi::c_void;
    use std::ptr;

    type Handle = *mut c_void;

    const STD_INPUT_HANDLE: u32 = -10_i32 as u32;
    const STD_OUTPUT_HANDLE: u32 = -11_i32 as u32;
    const STD_ERROR_HANDLE: u32 = -12_i32 as u32;
    const INVALID_HANDLE_VALUE: Handle = -1_isize as Handle;

    #[link(name = "kernel32")]
    extern "system" {
        fn CloseHandle(handle: Handle) -> i32;
        fn GetStdHandle(std_handle: u32) -> Handle;
        fn SetStdHandle(std_handle: u32, handle: Handle) -> i32;
    }

    for std_handle in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        unsafe {
            let handle = GetStdHandle(std_handle);
            if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(handle);
            }
            let _ = SetStdHandle(std_handle, ptr::null_mut());
        }
    }
}

#[cfg(not(windows))]
fn detach_daemon_stdio() {}

fn ensure_mcp_daemon_state() -> anyhow::Result<IpcDaemonState> {
    let state_path = default_state_path();
    let current_exe = std::env::current_exe().context("failed to resolve current executable")?;
    if let Some(plan) =
        plan_daemon_autostart(&state_path, &current_exe, Duration::from_millis(250))?
    {
        execute_daemon_autostart(&plan)?;
    }

    wait_for_daemon_state(&state_path, Duration::from_secs(3))
}

fn execute_daemon_autostart(plan: &DaemonAutostartPlan) -> anyhow::Result<()> {
    spawn_daemon_process(plan).with_context(|| {
        format!(
            "failed to spawn daemon via {}",
            plan.command.program.display()
        )
    })
}

#[cfg(not(windows))]
fn spawn_daemon_process(plan: &DaemonAutostartPlan) -> anyhow::Result<()> {
    let mut command = Command::new(&plan.command.program);
    command.args(&plan.command.args);
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    configure_daemon_process(&mut command);
    for (key, value) in &plan.command.env {
        command.env(key, value);
    }
    command.spawn()?;
    Ok(())
}

#[cfg(unix)]
fn configure_daemon_process(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
}

#[cfg(not(any(unix, windows)))]
fn configure_daemon_process(_command: &mut Command) {}

#[cfg(windows)]
fn spawn_daemon_process(plan: &DaemonAutostartPlan) -> anyhow::Result<()> {
    use std::ffi::{c_void, OsStr, OsString};
    use std::mem;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    type Bool = i32;
    type Dword = u32;
    type Handle = *mut c_void;
    type Lpcwstr = *const u16;
    type Lpwstr = *mut u16;
    type Lpvoid = *mut c_void;

    const CREATE_NEW_PROCESS_GROUP: Dword = 0x0000_0200;
    const CREATE_UNICODE_ENVIRONMENT: Dword = 0x0000_0400;
    const DETACHED_PROCESS: Dword = 0x0000_0008;
    const CREATE_NO_WINDOW: Dword = 0x0800_0000;

    #[repr(C)]
    struct StartupInfoW {
        cb: Dword,
        lp_reserved: Lpwstr,
        lp_desktop: Lpwstr,
        lp_title: Lpwstr,
        dw_x: Dword,
        dw_y: Dword,
        dw_x_size: Dword,
        dw_y_size: Dword,
        dw_x_count_chars: Dword,
        dw_y_count_chars: Dword,
        dw_fill_attribute: Dword,
        dw_flags: Dword,
        w_show_window: u16,
        cb_reserved2: u16,
        lp_reserved2: *mut u8,
        h_std_input: Handle,
        h_std_output: Handle,
        h_std_error: Handle,
    }

    #[repr(C)]
    struct ProcessInformation {
        h_process: Handle,
        h_thread: Handle,
        dw_process_id: Dword,
        dw_thread_id: Dword,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CloseHandle(handle: Handle) -> Bool;
        fn CreateProcessW(
            lp_application_name: Lpcwstr,
            lp_command_line: Lpwstr,
            lp_process_attributes: Lpvoid,
            lp_thread_attributes: Lpvoid,
            b_inherit_handles: Bool,
            dw_creation_flags: Dword,
            lp_environment: Lpvoid,
            lp_current_directory: Lpcwstr,
            lp_startup_info: *mut StartupInfoW,
            lp_process_information: *mut ProcessInformation,
        ) -> Bool;
    }

    let mut application_name = os_str_to_wide(plan.command.program.as_os_str());
    let mut command_line = command_line_for_windows(&plan.command.program, &plan.command.args);
    let mut environment = environment_block_for_windows(&plan.command.env);
    let mut startup_info: StartupInfoW = unsafe { mem::zeroed() };
    startup_info.cb = mem::size_of::<StartupInfoW>() as Dword;
    let mut process_info: ProcessInformation = unsafe { mem::zeroed() };

    let created = unsafe {
        CreateProcessW(
            application_name.as_mut_ptr(),
            command_line.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            DETACHED_PROCESS
                | CREATE_NEW_PROCESS_GROUP
                | CREATE_NO_WINDOW
                | CREATE_UNICODE_ENVIRONMENT,
            environment.as_mut_ptr().cast::<c_void>(),
            ptr::null(),
            &mut startup_info,
            &mut process_info,
        )
    };

    if created == 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    unsafe {
        let _ = CloseHandle(process_info.h_thread);
        let _ = CloseHandle(process_info.h_process);
    }

    fn command_line_for_windows(program: &Path, args: &[String]) -> Vec<u16> {
        let mut parts = Vec::with_capacity(args.len() + 1);
        parts.push(quote_windows_arg(program.as_os_str()));
        parts.extend(args.iter().map(|arg| quote_windows_arg(OsStr::new(arg))));
        os_str_to_wide(OsString::from(parts.join(" ")).as_os_str())
    }

    fn environment_block_for_windows(
        overrides: &std::collections::BTreeMap<String, String>,
    ) -> Vec<u16> {
        let mut vars = std::env::vars_os().collect::<Vec<(OsString, OsString)>>();
        vars.retain(|(key, _)| {
            !overrides
                .keys()
                .any(|override_key| key.to_string_lossy().eq_ignore_ascii_case(override_key))
        });
        vars.extend(
            overrides
                .iter()
                .map(|(key, value)| (OsString::from(key), OsString::from(value))),
        );
        vars.sort_by(|(left, _), (right, _)| {
            left.to_string_lossy()
                .to_ascii_uppercase()
                .cmp(&right.to_string_lossy().to_ascii_uppercase())
        });

        let mut block = Vec::new();
        for (key, value) in vars {
            block.extend(key.encode_wide());
            block.push('=' as u16);
            block.extend(value.encode_wide());
            block.push(0);
        }
        block.push(0);
        block
    }

    fn quote_windows_arg(arg: &OsStr) -> String {
        let raw = arg.to_string_lossy();
        if raw.is_empty() {
            return "\"\"".to_string();
        }
        if !raw.chars().any(|ch| ch == ' ' || ch == '\t' || ch == '"') {
            return raw.into_owned();
        }

        let mut quoted = String::from("\"");
        let mut backslashes = 0;
        for ch in raw.chars() {
            match ch {
                '\\' => backslashes += 1,
                '"' => {
                    quoted.extend(std::iter::repeat('\\').take(backslashes * 2 + 1));
                    quoted.push('"');
                    backslashes = 0;
                }
                _ => {
                    quoted.extend(std::iter::repeat('\\').take(backslashes));
                    quoted.push(ch);
                    backslashes = 0;
                }
            }
        }
        quoted.extend(std::iter::repeat('\\').take(backslashes * 2));
        quoted.push('"');
        quoted
    }

    fn os_str_to_wide(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }

    Ok(())
}

fn wait_for_daemon_state(
    state_path: &std::path::Path,
    timeout: Duration,
) -> anyhow::Result<IpcDaemonState> {
    let started = std::time::Instant::now();
    loop {
        if let Ok(state) = read_state_file(state_path) {
            if !is_state_stale(&state, Duration::from_millis(250)).unwrap_or(true) {
                return Ok(state);
            }
        }

        if started.elapsed() >= timeout {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    Err(anyhow!(
        "daemon did not become ready within {} ms",
        timeout.as_millis()
    ))
}

fn run_stop_command() -> anyhow::Result<()> {
    let state_path = default_state_path();
    if cleanup_stale_state_file(&state_path, Duration::from_millis(250))? {
        println!("removed stale daemon state");
        return Ok(());
    }

    let state = match read_state_file(&state_path) {
        Ok(state) => state,
        Err(_error) if !state_path.exists() => {
            println!("daemon is not running");
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    let request_id = format!("stop-{}", unix_time_ms());
    let response = request_daemon_shutdown(
        &state,
        Duration::from_secs(1),
        shutdown_request_frame(&request_id, "cli-stop"),
    )?;
    let removed = update_state_after_shutdown_ack(&state_path, &response)?;
    println!(
        "shutdown_requested={} state_removed={}",
        response.accepted, removed
    );
    Ok(())
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
