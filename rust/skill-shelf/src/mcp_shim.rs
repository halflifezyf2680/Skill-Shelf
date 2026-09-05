use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::config::{load_config, resolve_storage_layout, SkillShelfStorageLayout};
use crate::ipc::{connect_and_handshake, read_framed_json, send_framed_json, DaemonState};
use crate::lifecycle::{current_parent_pid, ParentDeathWatcher, ShutdownPoller, ShutdownReason};
use crate::model::{GroupListItem, ManagedGroupRecord};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub open_world_hint: bool,
    pub idempotent_hint: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: String,
    pub annotations: ToolAnnotations,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

static TOOLS: OnceLock<Vec<ToolDefinition>> = OnceLock::new();

fn build_tools() -> Vec<ToolDefinition> {
    let runtime_config = load_config();
    vec![
        ToolDefinition {
            name: "browse_shelf",
            description: build_browse_shelf_description(),
        annotations: ToolAnnotations {
            read_only_hint: true,
            destructive_hint: false,
            open_world_hint: false,
            idempotent_hint: true,
        },
        input_schema: json!({
            "type": "object",
            "properties": {
                "group": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Optional group id. When provided, returns skills inside that group instead of the group catalog."
                },
                "query": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Optional filter within the selected group (only used when group is set)."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 50,
                    "default": 20,
                    "description": "Maximum number of skill summaries to return (only used when group is set)."
                }
            },
            "additionalProperties": false
        }),
        },
        ToolDefinition {
            name: "search_skills",
            description: "Step 2 of the skill-shelf routing protocol. Search all skills by query when browse_shelf does not fully narrow the target. Returns top matches ranked by relevance. IMPORTANT: Always try the user's language first. If no relevant results found, retry with English keywords. For CJK queries, separate words with spaces (e.g. '品牌 视觉 设计', NOT '品牌设计视觉').".to_string(),
        annotations: ToolAnnotations {
            read_only_hint: true,
            destructive_hint: false,
            open_world_hint: false,
            idempotent_hint: true,
        },
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Search query derived from the current user task."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 20,
                    "default": runtime_config.index_policy.default_search_result_limit,
                    "description": "Maximum number of candidate skills to return."
                }
            },
            "required": ["query"],
            "additionalProperties": false
        }),
        },
        ToolDefinition {
            name: "read_skill",
            description: "Step 3 of the skill-shelf routing protocol. Read a skill after browse_shelf or search_skills has identified the target skill by name or skill id. Returns a 80-line summary by default; pass full=true to load the complete body. Avoid reading multiple full skills unless one is clearly insufficient.".to_string(),
        annotations: ToolAnnotations {
            read_only_hint: true,
            destructive_hint: false,
            open_world_hint: false,
            idempotent_hint: true,
        },
        input_schema: json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Skill name or skill id returned by browse_shelf or search_skills."
                },
                "full": {
                    "type": "boolean",
                    "default": false,
                    "description": "Set to true to return the full skill body instead of the 80-line summary."
                }
            },
            "required": ["skill"],
            "additionalProperties": false
        }),
        },
        ToolDefinition {
            name: "install_skills",
            description: "Write tool. Install one skill package directory or a directory containing multiple skill packages into the shelf packages store. This mutates the formal packages store, may overwrite existing package ids, and rebuilds shelf indexes.".to_string(),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            open_world_hint: false,
            idempotent_hint: false,
        },
        input_schema: json!({
            "type": "object",
            "properties": {
                "sourcePath": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Absolute or relative path to a skill package directory or parent directory."
                },
                "group": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Target group id. If omitted and SKILL.md has no frontmatter group, the tool returns skill descriptions and available groups for you to classify. Then call again with the chosen group."
                }
            },
            "required": ["sourcePath"],
            "additionalProperties": false
        }),
        },
        ToolDefinition {
            name: "validate_skills",
            description: "Governance tool. Validate installed skills for missing SKILL.md, invalid frontmatter, duplicate names, and generic-group review cases. Pass clean=true to automatically delete skills with blocked severity (missing files, broken frontmatter).".to_string(),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            open_world_hint: false,
            idempotent_hint: false,
        },
        input_schema: json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "description": "Optional skill id or skill name. When omitted, validate the whole shelf."
                },
                "clean": {
                    "type": "boolean",
                    "default": false,
                    "description": "When true, automatically delete skills with blocked severity issues."
                }
            },
            "additionalProperties": false
        }),
        },
        ToolDefinition {
            name: "manage_group",
            description: "Write tool. Manage skill groups: create a new group, update an existing group (description, keywords, aliases, rename), or delete an empty custom group. Builtin groups cannot be deleted.".to_string(),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: true,
            open_world_hint: false,
            idempotent_hint: false,
        },
        input_schema: json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["create", "update", "delete"],
                    "description": "Operation mode."
                },
                "group": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Group id (kebab-case). For update/delete, must be an existing group."
                },
                "groupDescription": {
                    "type": "string",
                    "description": "Group description. Required for create, optional for update."
                },
                "newGroup": {
                    "type": "string",
                    "description": "New group id for rename (update mode only)."
                },
                "keywords": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Routing keywords (replaces existing on update)."
                },
                "aliases": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Alternative names (replaces existing on update)."
                }
            },
            "required": ["mode", "group"],
            "additionalProperties": false
        }),
        },
        ToolDefinition {
            name: "reclassify_skill",
            description: "Move an installed skill to a different group. Updates the skill directory location and SKILL.md frontmatter, then rebuilds the index. Use when a skill was imported into the wrong group.".to_string(),
        annotations: ToolAnnotations {
            read_only_hint: false,
            destructive_hint: false,
            open_world_hint: false,
            idempotent_hint: false,
        },
        input_schema: json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Skill id or skill name to reclassify."
                },
                "target_group": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Target group id to move the skill into."
                }
            },
            "required": ["skill", "target_group"],
            "additionalProperties": false
        }),
        },
    ]
}

const TOOL_NAMES: [&str; 7] = [
    "browse_shelf",
    "search_skills",
    "read_skill",
    "install_skills",
    "validate_skills",
    "manage_group",
    "reclassify_skill",
];

pub fn tools() -> &'static [ToolDefinition] {
    TOOLS.get_or_init(build_tools).as_slice()
}

pub fn tool_names() -> &'static [&'static str] {
    &TOOL_NAMES
}

pub fn tool_by_name(name: &str) -> Option<&'static ToolDefinition> {
    tools().iter().find(|tool| tool.name == name)
}

fn build_browse_shelf_description() -> String {
    let layout = resolve_storage_layout(default_shelf_root());
    build_browse_shelf_description_for_layout(&layout)
}

fn build_browse_shelf_description_for_layout(layout: &SkillShelfStorageLayout) -> String {
    let mut lines = vec![
        "Entry point to discover available skill groups. Call this first to see what domains the shelf covers, then drill into a group or use search_skills.",
        "",
        "CURRENT SHELF CATALOG:",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    let catalog_lines = load_current_shelf_catalog_lines(layout);
    if catalog_lines.is_empty() {
        lines.push("(no indexed groups found)".to_string());
    } else {
        lines.extend(catalog_lines);
    }

    lines.join("\n")
}

fn load_current_shelf_catalog_lines(layout: &SkillShelfStorageLayout) -> Vec<String> {
    let mut groups = load_managed_group_catalog_lines(layout);
    if groups.is_empty() {
        groups = load_index_group_catalog_lines(layout);
    }

    groups
        .into_iter()
        .map(|(group, description)| {
            let skill_count = count_skill_files(&layout.packages_root.join(&group));
            format!("{} ({}): {}", group, skill_count, description)
        })
        .collect()
}

fn load_managed_group_catalog_lines(layout: &SkillShelfStorageLayout) -> BTreeMap<String, String> {
    fs::read_to_string(&layout.group_catalog_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Vec<ManagedGroupRecord>>(&raw).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|group| (group.group, group.group_description))
        .collect()
}

fn load_index_group_catalog_lines(layout: &SkillShelfStorageLayout) -> BTreeMap<String, String> {
    fs::read_to_string(&layout.group_list_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Vec<GroupListItem>>(&raw).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|group| (group.group, group.group_description))
        .collect()
}

fn count_skill_files(root: &Path) -> usize {
    let Ok(entries) = fs::read_dir(root) else {
        return 0;
    };

    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                count_skill_files(&path)
            } else if path.file_name().is_some_and(|name| name == "SKILL.md") {
                1
            } else {
                0
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::config::resolve_storage_layout;
    use crate::mcp_shim::build_browse_shelf_description_for_layout;

    #[test]
    fn browse_shelf_description_reads_managed_groups_without_index() {
        let temp = tempfile::tempdir().unwrap();
        let shelf_root = temp.path().join("hub");
        let layout = resolve_storage_layout(&shelf_root);

        fs::create_dir_all(&layout.config_root).unwrap();
        fs::create_dir_all(layout.packages_root.join("engineering/example")).unwrap();
        fs::write(
            &layout.group_catalog_path,
            r#"[
  {
    "group": "engineering",
    "groupDescription": "Engineering skills",
    "keywords": [],
    "aliases": [],
    "source": "builtin"
  },
  {
    "group": "design",
    "groupDescription": "Design skills",
    "keywords": [],
    "aliases": [],
    "source": "builtin"
  }
]"#,
        )
        .unwrap();
        fs::write(
            layout.packages_root.join("engineering/example/SKILL.md"),
            "---\nname: Example\ndescription: Example skill\n---\n",
        )
        .unwrap();

        let description = build_browse_shelf_description_for_layout(&layout);

        assert!(description.contains("CURRENT SHELF CATALOG:"));
        assert!(description.contains("design (0): Design skills"));
        assert!(description.contains("engineering (1): Engineering skills"));
        assert!(!description.contains("(no indexed groups found)"));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForwardingContext {
    pub session_id: String,
    pub shelf_root: PathBuf,
    pub config_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardRequestEnvelope {
    pub request_id: String,
    pub session_id: String,
    pub shelf_root: PathBuf,
    pub config_hash: String,
    pub method: String,
    pub params: Value,
}

impl ForwardRequestEnvelope {
    pub fn new(
        context: &ForwardingContext,
        request_id: impl Into<String>,
        method: impl Into<String>,
        params: Value,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            session_id: context.session_id.clone(),
            shelf_root: context.shelf_root.clone(),
            config_hash: context.config_hash.clone(),
            method: method.into(),
            params,
        }
    }
}

pub trait RequestForwarder {
    fn forward(&self, envelope: ForwardRequestEnvelope) -> Result<Value>;
}

#[derive(Clone, Debug)]
pub struct McpStdioShim<F> {
    context: ForwardingContext,
    forwarder: F,
}

impl<F> McpStdioShim<F>
where
    F: RequestForwarder,
{
    pub fn new(context: ForwardingContext, forwarder: F) -> Self {
        Self { context, forwarder }
    }

    pub fn build_request(
        &self,
        request_id: impl Into<String>,
        method: impl Into<String>,
        params: Value,
    ) -> ForwardRequestEnvelope {
        ForwardRequestEnvelope::new(&self.context, request_id, method, params)
    }

    pub fn forward(
        &self,
        request_id: impl Into<String>,
        method: impl Into<String>,
        params: Value,
    ) -> Result<Value> {
        self.forwarder
            .forward(self.build_request(request_id, method, params))
    }

    pub fn context(&self) -> &ForwardingContext {
        &self.context
    }
}

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "skill-shelf", about = "Skill Shelf Rust daemon shim CLI")]
pub struct ShimCli {
    #[command(subcommand)]
    pub command: ShimCommand,
}

impl ShimCli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    pub fn try_parse_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        <Self as Parser>::try_parse_from(itr)
    }
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ShimCommand {
    Mcp,
    Daemon,
    Status,
    Stop,
}

#[derive(Default)]
pub struct NullForwarder;

impl RequestForwarder for NullForwarder {
    fn forward(&self, envelope: ForwardRequestEnvelope) -> Result<Value> {
        Ok(json!({
            "forwarded": true,
            "requestId": envelope.request_id,
            "sessionId": envelope.session_id,
            "method": envelope.method,
            "params": envelope.params
        }))
    }
}

#[derive(Clone, Debug)]
pub struct IpcForwarder {
    state: DaemonState,
    timeout: Duration,
}

impl IpcForwarder {
    pub fn new(state: DaemonState, timeout: Duration) -> Self {
        Self { state, timeout }
    }
}

impl RequestForwarder for IpcForwarder {
    fn forward(&self, envelope: ForwardRequestEnvelope) -> Result<Value> {
        let mut stream = connect_and_handshake(&self.state, self.timeout)?;
        send_framed_json(&mut stream, &envelope)?;
        let reply = read_framed_json::<Value>(&mut stream)?;
        // The daemon reports business errors as an ipcError payload instead of
        // dropping the connection; surface that text to the MCP client.
        if let Some(error) = reply.get("ipcError").and_then(Value::as_str) {
            bail!("{error}");
        }
        Ok(reply)
    }
}

pub fn default_context() -> ForwardingContext {
    let shelf_root = default_shelf_root();
    forwarding_context_for_shelf(shelf_root, generate_session_id())
}

pub fn forwarding_context_for_shelf(
    shelf_root: PathBuf,
    session_id: impl Into<String>,
) -> ForwardingContext {
    let config_hash = config_hash_for_shelf(&shelf_root);
    ForwardingContext {
        session_id: session_id.into(),
        shelf_root,
        config_hash,
    }
}

fn default_shelf_root() -> PathBuf {
    load_config().storage.shelf_root
}

fn config_hash_for_shelf(shelf_root: &PathBuf) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    shelf_root.hash(&mut hasher);
    "default".hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn generate_session_id() -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::process::id().hash(&mut hasher);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    format!("session-{:016x}", hasher.finish())
}

pub fn run_stdio_loop<R, W, F>(reader: R, writer: W, shim: &McpStdioShim<F>) -> Result<()>
where
    R: std::io::Read,
    W: Write,
    F: RequestForwarder,
{
    let parent_death = current_parent_pid().map(ParentDeathWatcher::new);
    let shutdown_poller = ShutdownPoller::new(None, parent_death.as_ref());
    run_stdio_loop_with_shutdown_poller(reader, writer, shim, &shutdown_poller)
}

pub fn run_stdio_loop_with_shutdown_poller<R, W, F>(
    reader: R,
    writer: W,
    shim: &McpStdioShim<F>,
    shutdown_poller: &ShutdownPoller<'_>,
) -> Result<()>
where
    R: std::io::Read,
    W: Write,
    F: RequestForwarder,
{
    run_stdio_loop_with_shutdown_poll(reader, writer, shim, || shutdown_poller.poll())
}

pub fn run_stdio_loop_with_shutdown_poll<R, W, F, P>(
    reader: R,
    mut writer: W,
    shim: &McpStdioShim<F>,
    mut shutdown_poll: P,
) -> Result<()>
where
    R: std::io::Read,
    W: Write,
    F: RequestForwarder,
    P: FnMut() -> Option<ShutdownReason>,
{
    let mut reader = BufReader::new(reader);

    loop {
        if shutdown_poll().is_some() {
            return Ok(());
        }

        let Some((transport, request)) = read_json_rpc_message(&mut reader)? else {
            return Ok(());
        };
        if request
            .get("method")
            .and_then(Value::as_str)
            .is_some_and(|method| method.starts_with("notifications/"))
        {
            continue;
        }
        if let Some(response) = handle_json_rpc_request(shim, request)? {
            write_json_rpc_response(&mut writer, transport, &response)?;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StdioTransport {
    LineDelimited,
    ContentLength,
}

fn read_json_rpc_message<R: BufRead>(reader: &mut R) -> Result<Option<(StdioTransport, Value)>> {
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(None);
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }

        let compact = trimmed.trim_start();
        if compact.starts_with('{') || compact.starts_with('[') {
            let request =
                serde_json::from_str(compact).context("failed to parse json-rpc request")?;
            return Ok(Some((StdioTransport::LineDelimited, request)));
        }

        let mut content_length = None;
        parse_mcp_header(trimmed, &mut content_length)?;

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                bail!("unexpected EOF while reading MCP headers");
            }

            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }

            parse_mcp_header(trimmed, &mut content_length)?;
        }

        let content_length = content_length.context("missing Content-Length header")?;
        let mut body = vec![0_u8; content_length];
        reader.read_exact(&mut body)?;
        let request =
            serde_json::from_slice(&body).context("failed to parse content-length json-rpc request")?;
        return Ok(Some((StdioTransport::ContentLength, request)));
    }
}

fn parse_mcp_header(line: &str, content_length: &mut Option<usize>) -> Result<()> {
    let (name, value) = line
        .split_once(':')
        .with_context(|| format!("invalid MCP header line: {line}"))?;

    if name.eq_ignore_ascii_case("content-length") {
        *content_length = Some(
            value
                .trim()
                .parse::<usize>()
                .with_context(|| format!("invalid Content-Length value: {}", value.trim()))?,
        );
    }

    Ok(())
}

fn write_json_rpc_response<W: Write>(
    writer: &mut W,
    transport: StdioTransport,
    response: &Value,
) -> Result<()> {
    match transport {
        StdioTransport::LineDelimited => {
            serde_json::to_writer(&mut *writer, response)?;
            writer.write_all(b"\n")?;
        }
        StdioTransport::ContentLength => {
            let body = serde_json::to_vec(response)?;
            write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
            writer.write_all(&body)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn handle_command(command: &ShimCommand) -> &'static str {
    match command {
        ShimCommand::Mcp => "MCP stdio shim ready.",
        ShimCommand::Daemon => "Daemon entrypoint reserved for background service wiring.",
        ShimCommand::Status => "Status entrypoint reserved for daemon state inspection.",
        ShimCommand::Stop => "Stop entrypoint reserved for daemon shutdown signaling.",
    }
}

fn handle_json_rpc_request<F>(shim: &McpStdioShim<F>, request: Value) -> Result<Option<Value>>
where
    F: RequestForwarder,
{
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .context("json-rpc request missing method")?;

    let id = request.get("id").cloned();
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

    let response = match method {
        "initialize" => id.map(|request_id| {
            json_rpc_result(
                request_id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "skill-shelf",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            )
        }),
        "tools/list" => {
            id.map(|request_id| json_rpc_result(request_id, json!({ "tools": tools() })))
        }
        "tools/call" => {
            let request_id = id.context("tools/call request missing id")?;
            let tool_name = params
                .get("name")
                .and_then(Value::as_str)
                .context("tools/call missing tool name")?;
            if tool_by_name(tool_name).is_none() {
                bail!("unknown tool: {tool_name}");
            }
            let forwarded = match shim.forward(
                json_rpc_id_to_forward_id(&request_id),
                "tools/call",
                params.clone(),
            ) {
                Ok(value) => value,
                Err(error) => {
                    return Ok(Some(json_rpc_error(
                        request_id,
                        -32000,
                        format!("Tool call failed: {error:#}"),
                    )));
                }
            };
            Some(json_rpc_result(request_id, wrap_tool_result(forwarded)))
        }
        _ => id.map(|request_id| {
            json_rpc_error(request_id, -32601, format!("Method not found: {method}"))
        }),
    };

    Ok(response)
}

fn json_rpc_id_to_forward_id(id: &Value) -> String {
    match id {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

fn wrap_tool_result(forwarded: Value) -> Value {
    match forwarded {
        Value::Object(mut map) => {
            let structured = Value::Object(map.clone());
            map.entry("structuredContent".to_string())
                .or_insert_with(|| structured.clone());
            map.entry("content".to_string()).or_insert_with(|| {
                json!([
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&structured).unwrap_or_else(|_| structured.to_string())
                    }
                ])
            });
            Value::Object(map)
        }
        other => json!({
            "structuredContent": other.clone(),
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&other).unwrap_or_else(|_| other.to_string())
                }
            ]
        }),
    }
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn json_rpc_error(id: Value, code: i64, message: String) -> Value {
    let mut error = Map::new();
    error.insert("code".to_string(), json!(code));
    error.insert("message".to_string(), json!(message));
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": Value::Object(error)
    })
}
