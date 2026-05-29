use std::io::Cursor;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use serde_json::{json, Value};
use skill_shelf::ipc::{
    plan_daemon_autostart, read_framed_json, send_framed_json, spawn_token_server,
    write_state_file, AutostartReason, DaemonState,
};
use skill_shelf::lifecycle::{ParentDeathWatcher, ShutdownPoller, ShutdownReason, StdinEofWatcher};
use skill_shelf::mcp_shim::{
    run_stdio_loop, run_stdio_loop_with_shutdown_poll, run_stdio_loop_with_shutdown_poller,
    tool_names, tools, ForwardRequestEnvelope, ForwardingContext, IpcForwarder, McpStdioShim,
    RequestForwarder, ShimCli, ShimCommand,
};
use sysinfo::{Pid, System};

#[derive(Clone, Default)]
struct MockForwarder {
    forwarded: Arc<Mutex<Vec<ForwardRequestEnvelope>>>,
    result: Arc<Mutex<Value>>,
}

impl MockForwarder {
    fn with_result(result: Value) -> Self {
        Self {
            forwarded: Arc::new(Mutex::new(Vec::new())),
            result: Arc::new(Mutex::new(result)),
        }
    }

    fn take_forwarded(&self) -> Vec<ForwardRequestEnvelope> {
        self.forwarded.lock().unwrap().clone()
    }
}

impl RequestForwarder for MockForwarder {
    fn forward(&self, envelope: ForwardRequestEnvelope) -> Result<Value> {
        self.forwarded.lock().unwrap().push(envelope);
        Ok(self.result.lock().unwrap().clone())
    }
}

#[derive(Clone, Default)]
struct FailingForwarder;

impl RequestForwarder for FailingForwarder {
    fn forward(&self, _envelope: ForwardRequestEnvelope) -> Result<Value> {
        anyhow::bail!("simulated forward failure")
    }
}

fn test_context() -> ForwardingContext {
    ForwardingContext {
        session_id: "session-123".to_string(),
        shelf_root: PathBuf::from("C:/SkillShelf"),
        config_hash: "cfg-abc".to_string(),
    }
}

fn find_missing_pid() -> u32 {
    for pid in (u16::MAX as u32..u32::MAX).rev().take(4096) {
        let system = System::new_all();
        if system.process(Pid::from_u32(pid)).is_none() {
            return pid;
        }
    }

    panic!("failed to find a PID that is not currently in use");
}

fn framed_message(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn parse_framed_messages(output: &[u8]) -> Vec<Value> {
    let mut cursor = Cursor::new(output);
    let mut messages = Vec::new();

    while (cursor.position() as usize) < output.len() {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            std::io::BufRead::read_line(&mut cursor, &mut line).unwrap();
            if line.is_empty() {
                return messages;
            }

            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }

            let (name, value) = trimmed.split_once(':').unwrap();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = Some(value.trim().parse::<usize>().unwrap());
            }
        }

        let length = content_length.expect("missing Content-Length header");
        let mut body = vec![0_u8; length];
        std::io::Read::read_exact(&mut cursor, &mut body).unwrap();
        messages.push(serde_json::from_slice(&body).unwrap());
    }

    messages
}

#[test]
fn tools_call_forward_failure_returns_json_rpc_error_and_loop_continues() {
    let input = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":\"bad-call\",\"method\":\"tools/call\",\"params\":{\"name\":\"read_skill\",\"arguments\":{\"skill\":\"missing\"}}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":\"after-error\",\"method\":\"tools/list\",\"params\":{}}\n",
    );
    let shim = McpStdioShim::new(test_context(), FailingForwarder);
    let mut output = Vec::new();

    run_stdio_loop(Cursor::new(input.as_bytes()), &mut output, &shim).unwrap();

    let lines: Vec<Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["id"], "bad-call");
    assert_eq!(lines[0]["error"]["code"], -32000);
    assert!(lines[0]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("simulated forward failure"));
    assert_eq!(lines[1]["id"], "after-error");
    assert_eq!(
        lines[1]["result"]["tools"],
        serde_json::to_value(tools()).unwrap()
    );
}

#[test]
fn tool_list_exposes_current_skill_shelf_tools() {
    let names = tool_names();

    assert_eq!(
        names,
        &[
            "browse_shelf",
            "list_group_skills",
            "search_skills",
            "read_skill",
            "install_skills",
            "validate_skills",
            "manage_group",
            "get_shelf_status",
        ]
    );
}

#[test]
fn tool_definitions_expose_descriptions_annotations_and_input_schema() {
    let tool_definitions = serde_json::to_value(tools()).unwrap();
    let tools = tool_definitions.as_array().unwrap();
    assert_eq!(tools.len(), 8);

    let browse = &tools[0];
    assert_eq!(browse["name"], "browse_shelf");
    assert!(browse["description"]
        .as_str()
        .unwrap()
        .contains("CURRENT SHELF CATALOG:"));
    assert_eq!(browse["inputSchema"]["additionalProperties"], false);

    let list_group = &tools[1];
    assert_eq!(list_group["name"], "list_group_skills");
    assert_eq!(list_group["inputSchema"]["required"], json!(["group"]));
    assert!(list_group["inputSchema"]["properties"]["query"].is_object());

    let search = &tools[2];
    assert_eq!(search["name"], "search_skills");
    assert!(search["description"]
        .as_str()
        .unwrap()
        .contains("browse_shelf"));

    let read = &tools[3];
    assert_eq!(read["name"], "read_skill");
    assert_eq!(read["inputSchema"]["properties"]["full"]["default"], false);
    assert_eq!(read["inputSchema"]["required"], json!(["skill"]));

    assert_eq!(tools[4]["name"], "install_skills");
    assert_eq!(tools[5]["name"], "validate_skills");
    assert_eq!(tools[6]["name"], "manage_group");
    assert_eq!(tools[7]["name"], "get_shelf_status");
}

#[test]
fn cli_parses_mcp_and_daemon_commands() {
    let mcp = ShimCli::try_parse_from(["skill-shelf", "mcp"]).unwrap();
    assert!(matches!(mcp.command, ShimCommand::Mcp));

    let daemon = ShimCli::try_parse_from(["skill-shelf", "daemon"]).unwrap();
    assert!(matches!(daemon.command, ShimCommand::Daemon));

    let status = ShimCli::try_parse_from(["skill-shelf", "status"]).unwrap();
    assert!(matches!(status.command, ShimCommand::Status));

    let stop = ShimCli::try_parse_from(["skill-shelf", "stop"]).unwrap();
    assert!(matches!(stop.command, ShimCommand::Stop));
}

#[test]
fn forward_request_envelope_carries_daemon_forwarding_fields() {
    let context = test_context();

    let envelope = ForwardRequestEnvelope::new(
        &context,
        "req-789",
        "tools/call",
        json!({
            "name": "search_skills",
            "arguments": {
                "query": "rust"
            }
        }),
    );

    assert_eq!(envelope.request_id, "req-789");
    assert_eq!(envelope.session_id, "session-123");
    assert_eq!(envelope.shelf_root, PathBuf::from("C:/SkillShelf"));
    assert_eq!(envelope.config_hash, "cfg-abc");
    assert_eq!(envelope.method, "tools/call");
    assert_eq!(
        envelope.params,
        json!({
            "name": "search_skills",
            "arguments": {
                "query": "rust"
            }
        })
    );
}

#[test]
fn json_rpc_stdio_loop_handles_initialize_list_and_call_with_valid_response_shapes() {
    let input = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"tester\",\"version\":\"0.0.0\"}}}\n",
        "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\",\"params\":{}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"search_skills\",\"arguments\":{\"query\":\"rust\",\"limit\":3}}}\n",
    );
    let forwarder = MockForwarder::with_result(json!({
        "query": "rust",
        "returned": 1,
        "skills": [
            { "name": "Rust MCP", "score": 0.99 }
        ]
    }));
    let shim = McpStdioShim::new(test_context(), forwarder.clone());
    let mut output = Vec::new();

    run_stdio_loop(Cursor::new(input.as_bytes()), &mut output, &shim).unwrap();

    let lines: Vec<Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["jsonrpc"], "2.0");
    assert_eq!(lines[0]["id"], 1);
    assert_eq!(lines[0]["result"]["serverInfo"]["name"], "skill-shelf");
    assert!(lines[0]["result"]["capabilities"]["tools"].is_object());

    assert_eq!(lines[1]["jsonrpc"], "2.0");
    assert_eq!(lines[1]["id"], 2);
    assert_eq!(lines[1]["result"]["tools"].as_array().unwrap().len(), 8);
    assert_eq!(lines[1]["result"]["tools"][0]["name"], "browse_shelf");
    assert_eq!(lines[1]["result"]["tools"][1]["name"], "list_group_skills");
    assert_eq!(lines[1]["result"]["tools"][3]["name"], "read_skill");

    assert_eq!(lines[2]["jsonrpc"], "2.0");
    assert_eq!(lines[2]["id"], 3);
    assert_eq!(lines[2]["result"]["structuredContent"]["query"], "rust");
    assert_eq!(
        lines[2]["result"]["content"],
        json!([
            {
                "type": "text",
                "text": "{\n  \"query\": \"rust\",\n  \"returned\": 1,\n  \"skills\": [\n    {\n      \"name\": \"Rust MCP\",\n      \"score\": 0.99\n    }\n  ]\n}"
            }
        ])
    );
}

#[test]
fn json_rpc_stdio_loop_supports_content_length_framed_mcp_messages() {
    let input = [
        framed_message(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"tester","version":"0.0.0"}}}"#,
        ),
        framed_message(r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#),
        framed_message(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#),
        framed_message(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search_skills","arguments":{"query":"rust","limit":3}}}"#,
        ),
    ]
    .join("");
    let forwarder = MockForwarder::with_result(json!({
        "query": "rust",
        "returned": 1,
        "skills": [
            { "name": "Rust MCP", "score": 0.99 }
        ]
    }));
    let shim = McpStdioShim::new(test_context(), forwarder.clone());
    let mut output = Vec::new();

    run_stdio_loop(Cursor::new(input.into_bytes()), &mut output, &shim).unwrap();

    let messages = parse_framed_messages(&output);

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0]["id"], 1);
    assert_eq!(messages[0]["result"]["serverInfo"]["name"], "skill-shelf");
    assert_eq!(messages[1]["id"], 2);
    assert_eq!(messages[1]["result"]["tools"].as_array().unwrap().len(), 8);
    assert_eq!(messages[2]["id"], 3);
    assert_eq!(messages[2]["result"]["structuredContent"]["query"], "rust");
    assert_eq!(forwarder.take_forwarded().len(), 1);
}

#[test]
fn tools_call_forwards_method_name_arguments_and_original_request_id() {
    let input = "{\"jsonrpc\":\"2.0\",\"id\":\"req-42\",\"method\":\"tools/call\",\"params\":{\"name\":\"manage_group\",\"arguments\":{\"mode\":\"delete\",\"group\":\"legacy\"}}}\n";
    let forwarder = MockForwarder::with_result(json!({"ok": true}));
    let shim = McpStdioShim::new(test_context(), forwarder.clone());
    let mut output = Vec::new();

    run_stdio_loop(Cursor::new(input.as_bytes()), &mut output, &shim).unwrap();

    let forwarded = forwarder.take_forwarded();
    assert_eq!(forwarded.len(), 1);
    assert_eq!(forwarded[0].request_id, "req-42");
    assert_eq!(forwarded[0].session_id, "session-123");
    assert_eq!(forwarded[0].method, "tools/call");
    assert_eq!(
        forwarded[0].params,
        json!({
            "name": "manage_group",
            "arguments": {
                "mode": "delete",
                "group": "legacy"
            }
        })
    );
}

#[test]
fn stdio_loop_terminates_cleanly_on_eof() {
    let forwarder = MockForwarder::with_result(json!({"ok": true}));
    let shim = McpStdioShim::new(test_context(), forwarder);
    let mut output = Vec::new();

    run_stdio_loop(Cursor::new(Vec::<u8>::new()), &mut output, &shim).unwrap();

    assert!(output.is_empty());
}

#[test]
fn stdio_loop_stops_before_second_request_when_shutdown_poll_trips() {
    let input = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":\"req-1\",\"method\":\"tools/call\",\"params\":{\"name\":\"search_skills\",\"arguments\":{\"query\":\"rust\"}}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":\"req-2\",\"method\":\"tools/call\",\"params\":{\"name\":\"read_skill\",\"arguments\":{\"skill\":\"rust\"}}}\n",
    );
    let forwarder = MockForwarder::with_result(json!({
        "ok": true,
        "items": []
    }));
    let shim = McpStdioShim::new(test_context(), forwarder.clone());
    let mut output = Vec::new();
    let poll_count = Arc::new(AtomicUsize::new(0));
    let poll_count_for_loop = Arc::clone(&poll_count);

    run_stdio_loop_with_shutdown_poll(
        Cursor::new(input.as_bytes()),
        &mut output,
        &shim,
        move || match poll_count_for_loop.fetch_add(1, Ordering::SeqCst) {
            0 => None,
            _ => Some(ShutdownReason::ParentDeath),
        },
    )
    .unwrap();

    let lines: Vec<Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["id"], "req-1");
    assert_eq!(lines[0]["result"]["structuredContent"]["ok"], true);
    assert_eq!(forwarder.take_forwarded().len(), 1);
    assert_eq!(poll_count.load(Ordering::SeqCst), 2);
}

#[test]
fn stdio_loop_stops_immediately_when_shutdown_poller_reports_parent_death() {
    let input = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"search_skills\",\"arguments\":{\"query\":\"rust\"}}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"initialize\",\"params\":{}}\n",
    );
    let forwarder = MockForwarder::with_result(json!({
        "ok": true
    }));
    let shim = McpStdioShim::new(test_context(), forwarder.clone());
    let (stdin_watcher, _notifier) = StdinEofWatcher::new();
    let parent_watcher = ParentDeathWatcher::new(find_missing_pid());
    let poller = ShutdownPoller::new(Some(&stdin_watcher), Some(&parent_watcher));
    let mut output = Vec::new();

    run_stdio_loop_with_shutdown_poller(Cursor::new(input.as_bytes()), &mut output, &shim, &poller)
        .unwrap();

    assert!(output.is_empty());
    assert!(forwarder.take_forwarded().is_empty());
}

#[test]
fn ipc_forwarder_sends_envelope_to_authenticated_daemon_and_reads_response() {
    let server = spawn_token_server(
        SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        |mut stream| {
            let envelope: ForwardRequestEnvelope = read_framed_json(&mut stream).unwrap();
            assert_eq!(envelope.request_id, "req-ipc");
            assert_eq!(envelope.session_id, "session-123");
            assert_eq!(envelope.method, "tools/call");
            assert_eq!(
                envelope.params,
                json!({
                    "name": "search_skills",
                    "arguments": {
                        "query": "rust"
                    }
                })
            );
            send_framed_json(
                &mut stream,
                &json!({
                    "query": "rust",
                    "returned": 0,
                    "skills": []
                }),
            )
            .unwrap();
            Ok(())
        },
    )
    .unwrap();
    let state = DaemonState {
        pid: std::process::id(),
        port: server.local_addr().port(),
        token: "expected-token".to_string(),
        version: "0.1.0-test".to_string(),
        started_at_ms: 1,
    };
    let forwarder = IpcForwarder::new(state, Duration::from_secs(1));

    let response = forwarder
        .forward(ForwardRequestEnvelope::new(
            &test_context(),
            "req-ipc",
            "tools/call",
            json!({
                "name": "search_skills",
                "arguments": {
                    "query": "rust"
                }
            }),
        ))
        .unwrap();

    assert_eq!(response["query"], "rust");
    assert_eq!(response["returned"], 0);
}

#[test]
fn autostart_plan_targets_daemon_subcommand_when_state_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let state_path = temp.path().join("daemon-state.json");
    let current_exe = temp.path().join("skill-shelf-test.exe");

    let plan = plan_daemon_autostart(&state_path, &current_exe, Duration::from_millis(50))
        .unwrap()
        .expect("missing state should require autostart");

    assert_eq!(plan.reason, AutostartReason::MissingState);
    assert_eq!(plan.command.program, current_exe);
    assert_eq!(plan.command.args, vec!["daemon".to_string()]);
    assert_eq!(
        plan.command
            .env
            .get("SKILL_SHELF_DAEMON_DIR")
            .map(String::as_str),
        Some(temp.path().to_string_lossy().as_ref())
    );
    assert!(!plan.cleaned_stale_state);
}

#[test]
fn autostart_plan_cleans_stale_state_before_returning_command() {
    let temp = tempfile::tempdir().unwrap();
    let state_path = temp.path().join("daemon-state.json");
    let current_exe = temp.path().join("skill-shelf-test.exe");
    let stale_state = DaemonState {
        pid: u32::MAX,
        port: 6553,
        token: "stale-token".to_string(),
        version: "0.1.0-test".to_string(),
        started_at_ms: 1,
    };
    write_state_file(&state_path, &stale_state).unwrap();

    let plan = plan_daemon_autostart(&state_path, &current_exe, Duration::from_millis(50))
        .unwrap()
        .expect("stale state should require autostart");

    assert_eq!(plan.reason, AutostartReason::StaleState);
    assert!(plan.cleaned_stale_state);
    assert!(!state_path.exists(), "stale state file should be removed");
}
