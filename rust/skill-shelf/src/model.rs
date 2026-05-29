use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillGroupRecord {
    pub group: String,
    pub group_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagedGroupRecord {
    pub group: String,
    pub group_description: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default = "default_group_source")]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupSkillIndexEntry {
    pub skill_id: String,
    pub skill_name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub skill_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SkillRecord {
    pub skill_id: String,
    pub skill_name: String,
    pub description: String,
    pub group: String,
    pub group_description: String,
    pub keywords: Vec<String>,
    pub skill_path: String,
    pub updated_at_ms: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SkillMeta {
    pub skill_id: String,
    pub skill_name: String,
    pub description: String,
    pub group: String,
    pub group_description: String,
    pub keywords: Vec<String>,
    pub updated_at_ms: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupListItem {
    pub group: String,
    pub group_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupSkillsResult {
    pub group: String,
    pub group_description: String,
    pub skills: Vec<GroupSkillIndexEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSkill {
    pub skill_name: String,
    pub description: String,
    pub group: String,
    pub group_description: String,
    pub keywords: Vec<String>,
    pub skill_path: String,
    pub updated_at_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallEntry {
    pub source_path: String,
    pub skill_id: String,
    pub installed_path: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallFailure {
    pub source_path: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillNeedsClassification {
    pub skill_id: String,
    pub source_path: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallSkipped {
    pub source_path: String,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallResult {
    pub installed: Vec<SkillInstallEntry>,
    pub skipped: Vec<SkillInstallSkipped>,
    pub failed: Vec<SkillInstallFailure>,
    pub needs_classification: Vec<SkillNeedsClassification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillValidationIssue {
    pub skill_id: Option<String>,
    pub path: String,
    pub code: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillValidationPassed {
    pub skill_id: String,
    pub skill_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillValidationResult {
    pub passed: Vec<SkillValidationPassed>,
    pub review_required: Vec<SkillValidationIssue>,
    pub blocked: Vec<SkillValidationIssue>,
    pub issues: Vec<SkillValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupCreateResult {
    pub action: String,
    pub group: ManagedGroupRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupUpdateResult {
    pub action: String,
    pub previous_group: String,
    pub group: ManagedGroupRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GroupDeleteResult {
    pub action: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillSearchResult {
    pub skill_id: String,
    pub skill_name: String,
    pub description: String,
    pub group: String,
    pub score: i64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatcherStatus {
    pub running: bool,
    pub last_event_at_ms: Option<u64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShelfStatus {
    pub groups_count: usize,
    pub skills_count: usize,
    pub import_count: usize,
    pub index_updated_at: Option<u64>,
    pub watcher_status: WatcherStatus,
    pub issue_count: usize,
}

fn default_group_source() -> String {
    "builtin".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReclassifyResult {
    pub skill_id: String,
    pub skill_name: String,
    pub from_group: String,
    pub to_group: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillCleanEntry {
    pub skill_id: String,
    pub path: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillCleanResult {
    pub deleted: Vec<SkillCleanEntry>,
    pub remaining: Vec<SkillValidationIssue>,
}
