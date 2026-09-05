use std::collections::HashMap;
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::{load_config, resolve_storage_layout, SkillShelfRuntimeConfig};
use crate::ipc::{
    read_framed_json, send_framed_json, spawn_token_server, DaemonControlRequest,
    DaemonControlResponse, TokenServerHandle,
};
use crate::mcp_shim::ForwardRequestEnvelope;
use crate::model::SkillRecord;
use crate::registry::SkillRegistry;
use crate::search::search_skills;
use crate::workspace::{WorkspaceKey, WorkspaceManager, WorkspaceSnapshot};

#[derive(Clone, Debug)]
pub struct DaemonState {
    workspace_manager: Arc<WorkspaceManager>,
    runtime_config: Arc<SkillShelfRuntimeConfig>,
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            workspace_manager: Arc::new(WorkspaceManager::new()),
            runtime_config: Arc::new(load_config()),
        }
    }

    pub fn with_config(runtime_config: SkillShelfRuntimeConfig) -> Self {
        Self {
            workspace_manager: Arc::new(WorkspaceManager::new()),
            runtime_config: Arc::new(runtime_config),
        }
    }

    pub fn workspace_manager(&self) -> Arc<WorkspaceManager> {
        Arc::clone(&self.workspace_manager)
    }

    pub async fn dispatch(&self, request: DaemonRequest) -> Result<DaemonResponse> {
        let runtime_config = Arc::clone(&self.runtime_config);
        let key = WorkspaceKey::new(request.shelf_root.clone(), request.config_hash.clone());
        let workspace = self.workspace_manager.get_or_create(key).await;
        let _session = workspace.attach_session();
        workspace.mark_used();

        match request.method.as_str() {
            "browse_shelf" => {
                let group_param = request
                    .params
                    .get("group")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                if let Some(group) = group_param {
                    let query = request
                        .params
                        .get("query")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let limit = request
                        .params
                        .get("limit")
                        .and_then(Value::as_u64)
                        .unwrap_or(20) as usize;
                    let result = self
                        .with_registry_read(&workspace, runtime_config.as_ref(), move |registry| {
                            let group_result = registry
                                .list_group_skills(&group, query.as_deref())
                                .ok_or_else(|| anyhow!("unknown group: {group}"))?;
                            let skills = group_result
                                .skills
                                .into_iter()
                                .take(limit)
                                .map(|skill| {
                                    json!({
                                        "skillId": skill.skill_id,
                                        "skillName": skill.skill_name,
                                        "description": skill.description,
                                    })
                                })
                                .collect::<Vec<_>>();
                            Ok(json!({
                                "group": group_result.group,
                                "description": group_result.group_description,
                                "skills": skills,
                            }))
                        })
                        .await?;
                    Ok(DaemonResponse::tool_result(result))
                } else {
                    self.ensure_snapshot_loaded(&workspace, runtime_config.as_ref())
                        .await?;
                    workspace.mark_used();
                    let state = workspace.read_state();
                    let watcher_status = workspace.watcher_status();
                    let result = self
                        .with_registry_read(&workspace, runtime_config.as_ref(), |registry| {
                            let groups = registry
                                .list_groups()
                                .into_iter()
                                .map(|group| {
                                    let group_skills =
                                        registry
                                            .list_group_skills(&group.group, None)
                                            .ok_or_else(|| anyhow!("unknown group: {}", group.group))?;
                                    Ok(json!({
                                        "group": group_skills.group,
                                        "description": group_skills.group_description,
                                        "skillCount": group_skills.skills.len(),
                                    }))
                                })
                                .collect::<Result<Vec<_>>>()?;
                            Ok(json!({
                                "totalSkills": registry.size(),
                                "groupsCount": state.groups_count,
                                "groups": groups,
                                "indexUpdatedAt": state.index_updated_at,
                                "watcherStatus": watcher_status,
                                "issueCount": state.issue_count,
                            }))
                        })
                        .await?;
                    Ok(DaemonResponse::tool_result(result))
                }
            }
            "search_skills" => {
                self.ensure_snapshot_loaded(&workspace, runtime_config.as_ref())
                    .await?;
                workspace.mark_used();
                let state = workspace.read_state();
                let query = request
                    .params
                    .get("query")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("search_skills requires string query"))?;
                let limit = request
                    .params
                    .get("limit")
                    .and_then(Value::as_u64)
                    .unwrap_or(runtime_config.index_policy.default_search_result_limit as u64)
                    as usize;
                let results = search_skills(&state.records, query, limit);
                Ok(DaemonResponse::tool_result(json!({
                    "query": query,
                    "returned": results.len(),
                    "skills": results,
                })))
            }
            "read_skill" => {
                self.ensure_snapshot_loaded(&workspace, runtime_config.as_ref())
                    .await?;
                workspace.mark_used();
                let state = workspace.read_state();
                let skill = request
                    .params
                    .get("skill")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("read_skill requires string skill"))?;
                let full = request
                    .params
                    .get("full")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let skill_key = normalize_skill_key(skill);
                let payload = state
                    .read_models
                    .get(&skill_key)
                    .cloned()
                    .ok_or_else(|| anyhow!("unknown skill: {skill}"))?;
                Ok(DaemonResponse::tool_result(render_skill_read_model(
                    payload, full,
                )))
            }
            "install_skills" => {
                let source_path = request
                    .params
                    .get("sourcePath")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("install_skills requires string sourcePath"))?
                    .to_string();
                let group = request
                    .params
                    .get("group")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let result = self
                    .with_registry_write(&workspace, runtime_config.as_ref(), move |registry| {
                        registry.install_skills(&source_path, group.as_deref())
                    })
                    .await?;
                Ok(DaemonResponse::tool_result(serde_json::to_value(result)?))
            }
            "validate_skills" => {
                let skill = request
                    .params
                    .get("skill")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let clean = request
                    .params
                    .get("clean")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if clean {
                    let result = self
                        .with_registry_write(&workspace, runtime_config.as_ref(), move |registry| {
                            registry.validate_and_clean(skill.as_deref())
                        })
                        .await?;
                    Ok(DaemonResponse::tool_result(serde_json::to_value(result)?))
                } else {
                    self.ensure_snapshot_loaded(&workspace, runtime_config.as_ref())
                        .await?;
                    let result = self
                        .with_registry_read(&workspace, runtime_config.as_ref(), move |registry| {
                            registry.validate_skills(skill.as_deref())
                        })
                        .await?;
                    Ok(DaemonResponse::tool_result(serde_json::to_value(result)?))
                }
            }
            "manage_group" => {
                let mode = request
                    .params
                    .get("mode")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("manage_group requires string mode"))?
                    .to_string();
                let group = request
                    .params
                    .get("group")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("manage_group requires string group"))?
                    .to_string();
                let group_description = request
                    .params
                    .get("groupDescription")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let new_group = request
                    .params
                    .get("newGroup")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let keywords = string_array_param(&request.params, "keywords");
                let aliases = string_array_param(&request.params, "aliases");

                let result = self
                    .with_registry_write(&workspace, runtime_config.as_ref(), move |registry| {
                        match mode.as_str() {
                            "create" => {
                                let description =
                                    group_description.as_deref().ok_or_else(|| {
                                        anyhow!("groupDescription is required for create mode")
                                    })?;
                                serde_json::to_value(registry.create_group(
                                    &group,
                                    description,
                                    keywords.unwrap_or_default(),
                                    aliases.unwrap_or_default(),
                                )?)
                                .map_err(Into::into)
                            }
                            "update" => serde_json::to_value(registry.update_group(
                                &group,
                                new_group.as_deref(),
                                group_description.as_deref(),
                                keywords,
                                aliases,
                            )?)
                            .map_err(Into::into),
                            "delete" => serde_json::to_value(registry.delete_group(&group)?)
                                .map_err(Into::into),
                            other => Err(anyhow!("unknown manage_group mode: {other}")),
                        }
                    })
                    .await?;
                Ok(DaemonResponse::tool_result(result))
            }
            "reclassify_skill" => {
                let skill = request
                    .params
                    .get("skill")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("reclassify_skill requires string skill"))?
                    .to_string();
                let target_group = request
                    .params
                    .get("target_group")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("reclassify_skill requires string target_group"))?
                    .to_string();
                let result = self
                    .with_registry_write(&workspace, runtime_config.as_ref(), move |registry| {
                        registry.reclassify_skill(&skill, &target_group)
                    })
                    .await?;
                Ok(DaemonResponse::tool_result(serde_json::to_value(result)?))
            }
            method => Err(anyhow!("unknown tool method: {method}")),
        }
    }

    async fn ensure_snapshot_loaded(
        &self,
        workspace: &Arc<crate::workspace::WorkspaceHandle>,
        runtime_config: &SkillShelfRuntimeConfig,
    ) -> Result<()> {
        if workspace.read_state().loaded && !workspace.poll_watcher_dirty() {
            return Ok(());
        }

        let shelf_root = workspace.key().shelf_root.clone();
        let workspace_for_update = Arc::clone(workspace);
        workspace
            .with_write(|| async move {
                let state = workspace_for_update.read_state();
                let loaded = state.loaded;
                let dirty = workspace_for_update.poll_watcher_dirty();
                if loaded && !dirty {
                    return Ok(());
                }

                let snapshot = build_workspace_snapshot(&shelf_root, runtime_config, !dirty)?;
                workspace_for_update.replace_state(snapshot);
                workspace_for_update.mark_watcher_clean();
                Ok(())
            })
            .await
    }

    async fn with_registry_read<T, F>(
        &self,
        workspace: &Arc<crate::workspace::WorkspaceHandle>,
        runtime_config: &SkillShelfRuntimeConfig,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut SkillRegistry) -> Result<T>,
    {
        let shelf_root = workspace.key().shelf_root.clone();
        let layout = resolve_storage_layout(&shelf_root);
        let mut registry = SkillRegistry::with_policies(
            layout,
            runtime_config.install_policy.clone(),
            runtime_config.index_policy.clone(),
        );
        registry.rebuild()?;
        operation(&mut registry)
    }

    async fn with_registry_write<T, F>(
        &self,
        workspace: &Arc<crate::workspace::WorkspaceHandle>,
        runtime_config: &SkillShelfRuntimeConfig,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut SkillRegistry) -> Result<T>,
    {
        let shelf_root = workspace.key().shelf_root.clone();
        let workspace_for_update = Arc::clone(workspace);
        workspace
            .with_write(|| async move {
                let layout = resolve_storage_layout(&shelf_root);
                let mut registry = SkillRegistry::with_policies(
                    layout,
                    runtime_config.install_policy.clone(),
                    runtime_config.index_policy.clone(),
                );
                registry.rebuild()?;
                let result = operation(&mut registry)?;
                let snapshot = build_workspace_snapshot(&shelf_root, runtime_config, false)?;
                workspace_for_update.replace_state(snapshot);
                workspace_for_update.mark_watcher_clean();
                Ok(result)
            })
            .await
    }
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonRequest {
    pub session_id: String,
    pub shelf_root: PathBuf,
    pub config_hash: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

impl DaemonRequest {
    pub fn tool_call(
        session_id: impl Into<String>,
        shelf_root: PathBuf,
        config_hash: impl Into<String>,
        method: impl Into<String>,
        params: Value,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            shelf_root,
            config_hash: config_hash.into(),
            method: method.into(),
            params,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DaemonResponse {
    ToolResult { structured_content: Value },
}

impl DaemonResponse {
    fn tool_result(structured_content: Value) -> Self {
        Self::ToolResult { structured_content }
    }

    pub fn into_structured_content(self) -> Value {
        match self {
            Self::ToolResult { structured_content } => structured_content,
        }
    }
}

pub fn spawn_daemon_ipc_server(
    bind_addr: SocketAddr,
    expected_token: &str,
    daemon: DaemonState,
) -> Result<TokenServerHandle> {
    spawn_daemon_ipc_server_with_shutdown(bind_addr, expected_token, daemon, || {})
}

pub fn spawn_daemon_ipc_server_with_shutdown<F>(
    bind_addr: SocketAddr,
    expected_token: &str,
    daemon: DaemonState,
    on_shutdown: F,
) -> Result<TokenServerHandle>
where
    F: Fn() + Send + Sync + 'static,
{
    let on_shutdown: Arc<dyn Fn() + Send + Sync> = Arc::new(on_shutdown);
    spawn_token_server(bind_addr, expected_token, move |stream| {
        serve_forward_stream_with_shutdown(stream, daemon.clone(), Arc::clone(&on_shutdown))
    })
}

pub fn serve_forward_stream(stream: TcpStream, daemon: DaemonState) -> Result<()> {
    serve_forward_stream_with_shutdown(stream, daemon, Arc::new(|| {}))
}

fn serve_forward_stream_with_shutdown(
    mut stream: TcpStream,
    daemon: DaemonState,
    on_shutdown: Arc<dyn Fn() + Send + Sync>,
) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("failed to create daemon runtime")?;
    loop {
        let frame = match read_framed_json::<Value>(&mut stream) {
            Ok(frame) => frame,
            Err(error) if error.to_string().contains("ended before payload") => return Ok(()),
            Err(error) => return Err(error),
        };
        if frame.get("method").and_then(Value::as_str) == Some("daemon/shutdown") {
            let request: DaemonControlRequest = serde_json::from_value(frame)
                .context("failed to parse daemon shutdown control frame")?;
            send_framed_json(
                &mut stream,
                &DaemonControlResponse {
                    request_id: request.request_id,
                    accepted: true,
                },
            )?;
            on_shutdown();
            return Ok(());
        }

        let envelope: ForwardRequestEnvelope =
            serde_json::from_value(frame).context("failed to parse forward request envelope")?;
        let request = forward_envelope_to_daemon_request(envelope)?;
        // Business errors (unknown skill/group, install guards, invalid params)
        // must reach the client as an error frame — dropping the connection
        // here would surface as an opaque "ipc frame ended before payload".
        let response = match runtime.block_on(daemon.dispatch(request)) {
            Ok(response) => response.into_structured_content(),
            Err(error) => json!({ "ipcError": format!("{error:#}") }),
        };
        send_framed_json(&mut stream, &response)?;
    }
}

fn forward_envelope_to_daemon_request(envelope: ForwardRequestEnvelope) -> Result<DaemonRequest> {
    if envelope.method == "tools/call" {
        let tool_name = envelope
            .params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("tools/call missing tool name"))?;
        let arguments = envelope
            .params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        return Ok(DaemonRequest::tool_call(
            envelope.session_id,
            envelope.shelf_root,
            envelope.config_hash,
            tool_name,
            arguments,
        ));
    }

    Ok(DaemonRequest::tool_call(
        envelope.session_id,
        envelope.shelf_root,
        envelope.config_hash,
        envelope.method,
        envelope.params,
    ))
}

fn string_array_param(params: &Value, key: &str) -> Option<Vec<String>> {
    params.get(key).and_then(Value::as_array).map(|values| {
        values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    })
}

fn build_workspace_snapshot(
    shelf_root: &Path,
    runtime_config: &SkillShelfRuntimeConfig,
    allow_cache: bool,
) -> Result<WorkspaceSnapshot> {
    let layout = resolve_storage_layout(shelf_root);
    let mut registry = SkillRegistry::with_policies(
        layout.clone(),
        runtime_config.install_policy.clone(),
        runtime_config.index_policy.clone(),
    );
    if !allow_cache || !registry.load_from_cache()? {
        registry.rebuild()?;
    }
    let mut records = registry.list_skill_records();
    let read_models = build_read_models(
        &registry,
        &records,
        runtime_config.index_policy.max_related_skills,
    )
    .or_else(|error| {
        if !allow_cache {
            return Err(error);
        }

        registry.rebuild()?;
        records = registry.list_skill_records();
        build_read_models(
            &registry,
            &records,
            runtime_config.index_policy.max_related_skills,
        )
    })?;

    Ok(WorkspaceSnapshot {
        loaded: true,
        groups_count: registry.list_groups().len(),
        skills_count: records.len(),
        import_count: count_directories(&layout.staging_imports_root),
        index_updated_at: read_index_updated_at(&layout.group_list_path),
        issue_count: registry.list_issues().len(),
        records,
        read_models,
    })
}

fn build_read_models(
    registry: &SkillRegistry,
    records: &[SkillRecord],
    related_limit: usize,
) -> Result<HashMap<String, Value>> {
    let mut models = HashMap::new();
    for record in records {
        let raw = fs::read_to_string(&record.skill_path)
            .with_context(|| format!("failed to read {}", record.skill_path))?;
        let contents = strip_frontmatter(&raw);
        let skill_dir = Path::new(&record.skill_path)
            .parent()
            .context("skill path missing parent directory")?;
        let assets = list_relative_files(&skill_dir.join("assets"))?;
        let references = list_relative_files(&skill_dir.join("references"))?;

        let related_skills = registry.list_related_skills(&record.skill_id, related_limit);
        let payload = json!({
            "skillId": record.skill_id,
            "skillName": record.skill_name,
            "description": record.description,
            "group": record.group,
            "keywords": record.keywords,
            "contents": contents,
            "totalLines": contents.split('\n').count(),
            "truncated": false,
            "assets": assets,
            "references": references,
            "relatedSkills": related_skills,
        });
        models.insert(normalize_skill_key(&record.skill_id), payload.clone());
        models.insert(normalize_skill_key(&record.skill_name), payload);
    }
    Ok(models)
}

fn render_skill_read_model(mut payload: Value, full: bool) -> Value {
    let contents = payload
        .get("contents")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let total_lines = payload
        .get("totalLines")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| contents.split('\n').count() as u64) as usize;

    if full {
        payload["truncated"] = Value::Bool(false);
        payload["totalLines"] = json!(total_lines);
        return payload;
    }

    let truncated = total_lines > 80;
    payload["contents"] = Value::String(if truncated {
        contents.split('\n').take(80).collect::<Vec<_>>().join("\n")
    } else {
        contents
    });
    payload["totalLines"] = json!(total_lines);
    payload["truncated"] = Value::Bool(truncated);
    payload
}

fn strip_frontmatter(body: &str) -> String {
    if let Some(rest) = body.strip_prefix("---\n") {
        if let Some((_, tail)) = rest.split_once("\n---\n") {
            return tail.to_string();
        }
    }
    if let Some(rest) = body.strip_prefix("---\r\n") {
        if let Some((_, tail)) = rest.split_once("\r\n---\r\n") {
            return tail.to_string();
        }
    }
    body.to_string()
}

fn list_relative_files(root: &Path) -> Result<Vec<String>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    walk_relative_files(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_relative_files(base: &Path, current: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        if file_type.is_dir() {
            walk_relative_files(base, &path, files)?;
        } else if file_type.is_file() {
            files.push(
                path.strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}

fn read_index_updated_at(group_list_path: &Path) -> Option<u64> {
    fs::metadata(group_list_path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|dur| dur.as_millis() as u64)
}

fn count_directories(root: &Path) -> usize {
    fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(|entry| entry.ok()))
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .count()
}

fn normalize_skill_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
