use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use skill_shelf::daemon::spawn_daemon_ipc_server_with_shutdown;
use skill_shelf::daemon::{spawn_daemon_ipc_server, DaemonRequest, DaemonResponse, DaemonState};
use skill_shelf::ipc::{
    request_daemon_shutdown, shutdown_request_frame, DaemonState as IpcDaemonState,
};
use skill_shelf::mcp_shim::{
    ForwardRequestEnvelope, ForwardingContext, IpcForwarder, RequestForwarder,
};
use skill_shelf::workspace::{WorkspaceKey, WorkspaceManager};
use tokio::sync::oneshot;
use tokio::sync::Barrier;
use tokio::time::sleep;

fn workspace_key(root: &str, hash: &str) -> WorkspaceKey {
    WorkspaceKey::new(PathBuf::from(root), hash.to_string())
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    fs::create_dir_all(dst).unwrap();

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let target = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn write_file(path: impl AsRef<std::path::Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn forwarding_context(shelf: PathBuf) -> ForwardingContext {
    ForwardingContext {
        session_id: "session-ipc".to_string(),
        shelf_root: shelf,
        config_hash: "cfg-ipc".to_string(),
    }
}

#[tokio::test]
async fn same_key_reuses_workspace() {
    let manager = WorkspaceManager::new();
    let key = workspace_key("C:/shelf-a", "cfg-1");

    let first = manager.get_or_create(key.clone()).await;
    let second = manager.get_or_create(key).await;

    assert_eq!(first.id(), second.id());
}

#[tokio::test]
async fn different_keys_get_distinct_workspaces() {
    let manager = WorkspaceManager::new();

    let first = manager
        .get_or_create(workspace_key("C:/shelf-a", "cfg-1"))
        .await;
    let second = manager
        .get_or_create(workspace_key("C:/shelf-a", "cfg-2"))
        .await;

    assert_ne!(first.id(), second.id());
}

#[tokio::test]
async fn write_operations_are_serialized() {
    let manager = WorkspaceManager::new();
    let workspace = manager
        .get_or_create(workspace_key("C:/shelf-a", "cfg-1"))
        .await;
    let gate = Arc::new(Barrier::new(2));
    let active_writers = Arc::new(AtomicUsize::new(0));
    let peak_writers = Arc::new(AtomicUsize::new(0));

    let first_workspace = workspace.clone();
    let first_gate = gate.clone();
    let first_active = active_writers.clone();
    let first_peak = peak_writers.clone();
    let first = tokio::spawn(async move {
        first_workspace
            .with_write(|| async move {
                let current = first_active.fetch_add(1, Ordering::SeqCst) + 1;
                first_peak.fetch_max(current, Ordering::SeqCst);
                first_gate.wait().await;
                sleep(Duration::from_millis(25)).await;
                first_active.fetch_sub(1, Ordering::SeqCst);
            })
            .await;
    });

    let second_workspace = workspace.clone();
    let second_gate = gate.clone();
    let second_active = active_writers.clone();
    let second_peak = peak_writers.clone();
    let second = tokio::spawn(async move {
        second_gate.wait().await;
        second_workspace
            .with_write(|| async move {
                let current = second_active.fetch_add(1, Ordering::SeqCst) + 1;
                second_peak.fetch_max(current, Ordering::SeqCst);
                second_active.fetch_sub(1, Ordering::SeqCst);
            })
            .await;
    });

    first.await.unwrap();
    second.await.unwrap();

    assert_eq!(peak_writers.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn unload_creates_new_workspace() {
    let manager = WorkspaceManager::new();
    let key = workspace_key("C:/shelf-a", "cfg-1");

    let first = manager.get_or_create(key.clone()).await;
    assert!(manager.unload(&key).await);

    let second = manager.get_or_create(key).await;
    assert_ne!(first.id(), second.id());
}

#[tokio::test]
async fn idle_workspace_can_be_removed() {
    let manager = WorkspaceManager::new();
    let key = workspace_key("C:/shelf-a", "cfg-1");

    let first = manager.get_or_create(key.clone()).await;
    sleep(Duration::from_millis(20)).await;
    assert_eq!(manager.remove_idle(Duration::from_millis(5)).await, 1);

    let second = manager.get_or_create(key).await;
    assert_ne!(first.id(), second.id());
}

#[tokio::test]
async fn long_write_prevents_idle_removal_and_reuses_workspace() {
    let manager = WorkspaceManager::new();
    let key = workspace_key("C:/shelf-a", "cfg-1");
    let workspace = manager.get_or_create(key.clone()).await;
    let original_id = workspace.id();

    let (started_tx, started_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let held_workspace = workspace.clone();
    let write_task = tokio::spawn(async move {
        held_workspace
            .with_write(|| async move {
                started_tx.send(()).unwrap();
                release_rx.await.unwrap();
            })
            .await;
    });

    started_rx.await.unwrap();
    sleep(Duration::from_millis(20)).await;

    assert_eq!(manager.remove_idle(Duration::from_millis(5)).await, 0);

    let reused = manager.get_or_create(key.clone()).await;
    assert_eq!(reused.id(), original_id);

    release_tx.send(()).unwrap();
    write_task.await.unwrap();

    sleep(Duration::from_millis(20)).await;
    assert_eq!(manager.remove_idle(Duration::from_millis(5)).await, 1);

    let replacement = manager.get_or_create(key).await;
    assert_ne!(replacement.id(), original_id);
}

#[tokio::test]
async fn attached_session_prevents_idle_removal_until_detached() {
    let manager = WorkspaceManager::new();
    let key = workspace_key("C:/shelf-a", "cfg-1");
    let workspace = manager.get_or_create(key.clone()).await;
    let original_id = workspace.id();
    let session = workspace.attach_session();

    sleep(Duration::from_millis(20)).await;
    assert_eq!(manager.remove_idle(Duration::from_millis(5)).await, 0);

    let reused = manager.get_or_create(key.clone()).await;
    assert_eq!(reused.id(), original_id);

    drop(session);
    sleep(Duration::from_millis(20)).await;

    assert_eq!(manager.remove_idle(Duration::from_millis(5)).await, 1);

    let replacement = manager.get_or_create(key).await;
    assert_ne!(replacement.id(), original_id);
}

#[tokio::test]
async fn daemon_routes_same_workspace_key_and_loads_registry_records() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let daemon = DaemonState::new();
    let search_request = DaemonRequest::tool_call(
        "session-a",
        shelf.clone(),
        "cfg-1",
        "search_skills",
        json!({
            "query": "rust",
            "limit": 8
        }),
    );

    let first = daemon.dispatch(search_request).await.unwrap();
    let first_workspace = daemon
        .workspace_manager()
        .get_or_create(WorkspaceKey::new(shelf.clone(), "cfg-1".to_string()))
        .await;
    let second_workspace = daemon
        .workspace_manager()
        .get_or_create(WorkspaceKey::new(shelf.clone(), "cfg-1".to_string()))
        .await;

    assert_eq!(first_workspace.id(), second_workspace.id());

    let DaemonResponse::ToolResult { structured_content } = first;
    assert_eq!(structured_content["skills"].as_array().unwrap().len(), 1);
    assert_eq!(structured_content["skills"][0]["skillId"], "rust-helper");
    assert_eq!(structured_content["skills"][0]["group"], "engineering");
}

#[tokio::test]
async fn daemon_read_dispatch_refreshes_snapshot_after_package_tree_delete() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let daemon = DaemonState::new();
    daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "search_skills",
            json!({
                "query": "rust",
                "limit": 8
            }),
        ))
        .await
        .unwrap();

    fs::remove_dir_all(shelf.join("packages")).unwrap();

    let search = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "search_skills",
            json!({
                "query": "rust",
                "limit": 8
            }),
        ))
        .await
        .unwrap();
    let read_error = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "read_skill",
            json!({
                "skill": "rust-helper"
            }),
        ))
        .await
        .unwrap_err();
    let status = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf,
            "cfg-1",
            "get_shelf_status",
            json!({}),
        ))
        .await
        .unwrap();

    let DaemonResponse::ToolResult {
        structured_content: search_content,
    } = search;
    assert_eq!(search_content["skills"].as_array().unwrap().len(), 0);
    assert!(read_error.to_string().contains("unknown skill"));

    let DaemonResponse::ToolResult {
        structured_content: status_content,
    } = status;
    assert_eq!(status_content["skillsCount"], 0);
    assert_eq!(status_content["groupsCount"], 1);
    assert_eq!(status_content["watcherStatus"]["running"], true);
    assert!(status_content["watcherStatus"]["lastEventAtMs"].is_number());
}

#[tokio::test]
async fn daemon_refreshes_workspace_snapshot_when_watcher_detects_skill_change() {
    let temp = tempfile::tempdir().unwrap();
    let shelf = temp.path().join("hub");
    copy_dir_all(
        PathBuf::from("tests/fixtures/rebuild_shelf").as_path(),
        shelf.as_path(),
    );
    let daemon = DaemonState::new();

    let first = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-a",
            "search_skills",
            json!({ "query": "rust", "limit": 1 }),
        ))
        .await
        .unwrap()
        .into_structured_content();
    assert_eq!(first["skills"][0]["skillName"], "Rust Helper");

    write_file(
        shelf.join("packages/engineering/rust-helper/SKILL.md"),
        "---\nname: Python Helper\ndescription: Helps with Python automation\n---\nUpdated body\n",
    );

    let second = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-a",
            "search_skills",
            json!({ "query": "python", "limit": 1 }),
        ))
        .await
        .unwrap()
        .into_structured_content();
    assert_eq!(second["skills"][0]["skillName"], "Python Helper");

    let status = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf,
            "cfg-a",
            "get_shelf_status",
            json!({}),
        ))
        .await
        .unwrap()
        .into_structured_content();
    assert_eq!(status["watcherStatus"]["running"], true);
    assert!(status["watcherStatus"]["lastEventAtMs"].is_number());
    assert!(status["watcherStatus"]["lastError"].is_null());
}

#[tokio::test]
async fn daemon_read_skill_and_status_use_registry_related_skills_and_issues() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);
    write_file(
        shelf.join("packages/engineering/alpha-helper/SKILL.md"),
        r#"---
name: Alpha Helper
description: Helps with adjacent Rust work
---
Body
"#,
    );
    write_file(
        shelf.join("packages/engineering/invalid-helper/SKILL.md"),
        r#"---
name Invalid Helper
description: broken
"#,
    );

    let daemon = DaemonState::new();
    let read = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "read_skill",
            json!({
                "skill": "rust-helper"
            }),
        ))
        .await
        .unwrap();
    let status = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf,
            "cfg-1",
            "get_shelf_status",
            json!({}),
        ))
        .await
        .unwrap();

    let DaemonResponse::ToolResult {
        structured_content: read_content,
    } = read;
    assert_eq!(read_content["skillId"], "rust-helper");
    assert_eq!(read_content["relatedSkills"][0]["skillId"], "alpha-helper");

    let DaemonResponse::ToolResult {
        structured_content: status_content,
    } = status;
    assert_eq!(status_content["skillsCount"], 2);
    assert_eq!(status_content["issueCount"], 1);
}

#[tokio::test]
async fn daemon_browse_list_and_read_skill_support_progressive_disclosure() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let body = (0..100)
        .map(|index| format!("Line {index:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    write_file(
        shelf.join("packages/engineering/long-helper/SKILL.md"),
        &format!(
            "---\nname: Long Helper\ndescription: Exercises summary truncation\n---\n{}\n",
            body
        ),
    );

    let daemon = DaemonState::new();

    let browse = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "browse_shelf",
            json!({}),
        ))
        .await
        .unwrap()
        .into_structured_content();
    assert_eq!(browse["totalSkills"], 2);
    assert_eq!(browse["groups"].as_array().unwrap().len(), 1);
    assert_eq!(browse["groups"][0]["group"], "engineering");
    assert_eq!(browse["groups"][0]["skillCount"], 2);
    assert!(browse["groups"][0].get("skills").is_none());

    let list = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "list_group_skills",
            json!({
                "group": "engineering",
                "limit": 1
            }),
        ))
        .await
        .unwrap()
        .into_structured_content();
    assert_eq!(list["group"], "engineering");
    assert_eq!(list["description"], "Engineering skills");
    assert_eq!(list["skills"].as_array().unwrap().len(), 1);
    assert_eq!(list["skills"][0]["skillId"], "long-helper");
    assert_eq!(list["skills"][0]["skillName"], "Long Helper");

    let summary = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "read_skill",
            json!({
                "skill": "long-helper"
            }),
        ))
        .await
        .unwrap()
        .into_structured_content();
    assert_eq!(summary["skillId"], "long-helper");
    assert_eq!(summary["truncated"], true);
    assert_eq!(summary["totalLines"], 101);
    assert_eq!(
        summary["contents"].as_str().unwrap().split('\n').count(),
        80
    );

    let full = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf,
            "cfg-1",
            "read_skill",
            json!({
                "skill": "long-helper",
                "full": true
            }),
        ))
        .await
        .unwrap()
        .into_structured_content();
    assert_eq!(full["skillId"], "long-helper");
    assert_eq!(full["truncated"], false);
    assert_eq!(full["totalLines"], 101);
    assert_eq!(full["contents"].as_str().unwrap().split('\n').count(), 101);
}

#[tokio::test]
async fn daemon_dispatches_install_validate_and_manage_group_write_tools() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);
    let incoming = temp.path().join("incoming/new-helper");
    write_file(
        incoming.join("SKILL.md"),
        r#"---
name: New Helper
description: Helps with newly installed Rust workflows
---
Body
"#,
    );

    let daemon = DaemonState::new();
    let created = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "manage_group",
            json!({
                "mode": "create",
                "group": "custom-tools",
                "groupDescription": "Custom tools",
                "keywords": ["custom"],
                "aliases": ["tools"]
            }),
        ))
        .await
        .unwrap();
    let DaemonResponse::ToolResult {
        structured_content: created_content,
    } = created;
    assert_eq!(created_content["action"], "created");
    assert!(shelf.join("packages/custom-tools").exists());

    let installed = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "install_skills",
            json!({
                "sourcePath": incoming,
                "group": "custom-tools"
            }),
        ))
        .await
        .unwrap();
    let DaemonResponse::ToolResult {
        structured_content: installed_content,
    } = installed;
    assert_eq!(installed_content["installed"].as_array().unwrap().len(), 1);
    assert!(shelf
        .join("packages/custom-tools/new-helper/SKILL.md")
        .exists());

    let validation = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "validate_skills",
            json!({
                "skill": "New Helper"
            }),
        ))
        .await
        .unwrap();
    let DaemonResponse::ToolResult {
        structured_content: validation_content,
    } = validation;
    assert_eq!(validation_content["passed"][0]["skillId"], "new-helper");

    let deleted_non_empty = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf,
            "cfg-1",
            "manage_group",
            json!({
                "mode": "delete",
                "group": "custom-tools"
            }),
        ))
        .await
        .unwrap_err();
    assert!(deleted_non_empty
        .to_string()
        .contains("group is not empty: custom-tools (1 skills)"));
}

#[tokio::test]
async fn daemon_unknown_method_returns_clear_error() {
    let daemon = DaemonState::new();

    let response = daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            PathBuf::from("C:/shelf-a"),
            "cfg-1",
            "totally_unknown",
            json!({}),
        ))
        .await
        .unwrap_err();

    assert!(response.to_string().contains("unknown tool method"));
    assert!(response.to_string().contains("totally_unknown"));
}

#[tokio::test]
async fn daemon_dispatch_and_session_lifecycle_keep_idle_cleanup_working() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let daemon = DaemonState::new();
    let manager = daemon.workspace_manager();
    let key = WorkspaceKey::new(shelf.clone(), "cfg-1".to_string());

    daemon
        .dispatch(DaemonRequest::tool_call(
            "session-a",
            shelf.clone(),
            "cfg-1",
            "get_shelf_status",
            json!({}),
        ))
        .await
        .unwrap();

    let workspace = manager.get_or_create(key.clone()).await;
    let original_id = workspace.id();
    let session = workspace.attach_session();

    sleep(Duration::from_millis(20)).await;
    assert_eq!(manager.remove_idle(Duration::from_millis(5)).await, 0);

    drop(session);
    sleep(Duration::from_millis(20)).await;
    assert_eq!(manager.remove_idle(Duration::from_millis(5)).await, 1);

    let replacement = manager.get_or_create(key).await;
    assert_ne!(replacement.id(), original_id);
}

#[test]
fn daemon_ipc_server_dispatches_mcp_tools_call_to_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let daemon = DaemonState::new();
    let server = spawn_daemon_ipc_server(
        std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        daemon,
    )
    .unwrap();
    let forwarder = IpcForwarder::new(
        IpcDaemonState {
            pid: std::process::id(),
            port: server.local_addr().port(),
            token: "expected-token".to_string(),
            version: "0.1.0-test".to_string(),
            started_at_ms: 1,
        },
        Duration::from_secs(5),
    );

    let response = forwarder
        .forward(ForwardRequestEnvelope::new(
            &forwarding_context(shelf),
            "req-daemon-ipc",
            "tools/call",
            json!({
                "name": "search_skills",
                "arguments": {
                    "query": "rust",
                    "limit": 8
                }
            }),
        ))
        .unwrap();

    assert_eq!(response["query"], "rust");
    assert_eq!(response["returned"], 1);
    assert_eq!(response["skills"][0]["skillId"], "rust-helper");
}

#[test]
fn daemon_ipc_server_accepts_shutdown_control_frame() {
    let shutdown_called = Arc::new(AtomicBool::new(false));
    let shutdown_called_for_server = Arc::clone(&shutdown_called);
    let server = spawn_daemon_ipc_server_with_shutdown(
        std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
        "expected-token",
        DaemonState::new(),
        move || {
            shutdown_called_for_server.store(true, Ordering::SeqCst);
        },
    )
    .unwrap();
    let response = request_daemon_shutdown(
        &IpcDaemonState {
            pid: std::process::id(),
            port: server.local_addr().port(),
            token: "expected-token".to_string(),
            version: "0.1.0-test".to_string(),
            started_at_ms: 1,
        },
        Duration::from_secs(1),
        shutdown_request_frame("shutdown-test", "test"),
    )
    .unwrap();

    assert!(response.accepted);
    assert!(shutdown_called.load(Ordering::SeqCst));
}
