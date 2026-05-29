use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};

use crate::config::{
    default_index_policy, default_install_policy, IndexPolicy, InstallPolicy,
    SkillShelfStorageLayout,
};
use crate::model::{
    GroupCreateResult, GroupDeleteResult, GroupListItem, GroupSkillIndexEntry, GroupSkillsResult,
    GroupUpdateResult, ManagedGroupRecord, ReclassifyResult, RegistryIssue, SkillInstallEntry,
    SkillInstallFailure, SkillInstallResult, SkillInstallSkipped, SkillMeta,
    SkillNeedsClassification, SkillRecord, SkillValidationIssue, SkillValidationPassed,
    SkillValidationResult,
};
use crate::parser::parse_skill_file;

const SKILL_FILENAME: &str = "SKILL.md";
const EXCLUDED_MARKDOWN_BASENAMES: &[&str] = &[
    "readme",
    "contributing",
    "license",
    "upstream",
    "catalog",
    "agent-list",
    "quickstart",
    "executive-brief",
];

#[derive(Debug)]
pub struct SkillRegistry {
    layout: SkillShelfStorageLayout,
    install_policy: InstallPolicy,
    index_policy: IndexPolicy,
    managed_groups: Vec<ManagedGroupRecord>,
    skill_records: BTreeMap<String, SkillRecord>,
    group_list: BTreeMap<String, GroupListItem>,
    group_skills: BTreeMap<String, Vec<GroupSkillIndexEntry>>,
    issues: Vec<RegistryIssue>,
}

impl SkillRegistry {
    pub fn new(layout: SkillShelfStorageLayout) -> Self {
        Self::with_policies(layout, default_install_policy(), default_index_policy())
    }

    pub fn with_policies(
        layout: SkillShelfStorageLayout,
        install_policy: InstallPolicy,
        index_policy: IndexPolicy,
    ) -> Self {
        Self {
            layout,
            install_policy,
            index_policy,
            managed_groups: Vec::new(),
            skill_records: BTreeMap::new(),
            group_list: BTreeMap::new(),
            group_skills: BTreeMap::new(),
            issues: Vec::new(),
        }
    }

    pub fn rebuild(&mut self) -> Result<()> {
        self.managed_groups = load_managed_groups(&self.layout)?;
        self.skill_records.clear();
        self.group_list.clear();
        self.group_skills.clear();
        self.issues.clear();
        self.rebuild_groups_from_skills();

        for skill_file_path in collect_skill_files(&self.layout.packages_root)? {
            let group_from_path =
                derive_group_from_path(&skill_file_path, &self.layout.packages_root);
            match parse_skill_file(
                &skill_file_path,
                &self.managed_groups,
                self.index_policy.max_keywords_per_skill,
                group_from_path.as_deref(),
            ) {
                Ok(parsed) => self.upsert_in_memory(parsed),
                Err(error) => self.issues.push(RegistryIssue {
                    path: skill_file_path.to_string_lossy().into_owned(),
                    message: error.to_string(),
                }),
            }
        }

        self.rebuild_groups_from_skills();
        self.persist_indexes()?;
        self.persist_package_metadata()?;
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.skill_records.len()
    }

    pub fn list_skill_records(&self) -> Vec<SkillRecord> {
        self.skill_records.values().cloned().collect()
    }

    pub fn list_managed_groups(&self) -> Vec<ManagedGroupRecord> {
        let mut groups = self.managed_groups.clone();
        groups.sort_by(|a, b| a.group.cmp(&b.group));
        groups
    }

    pub fn list_groups(&self) -> Vec<GroupListItem> {
        self.group_list.values().cloned().collect()
    }

    pub fn list_issues(&self) -> Vec<RegistryIssue> {
        self.issues.clone()
    }

    pub fn get_by_id(&self, id: &str) -> Option<SkillRecord> {
        if let Some(record) = self.skill_records.get(id) {
            return Some(record.clone());
        }

        let bare_id = id.split('/').next_back().unwrap_or(id);
        self.skill_records
            .values()
            .find(|record| record.skill_id == bare_id)
            .cloned()
    }

    pub fn get_by_name(&self, skill_name: &str) -> Option<SkillRecord> {
        let normalized = skill_name.trim().to_lowercase();
        self.skill_records
            .values()
            .find(|record| record.skill_name.trim().to_lowercase() == normalized)
            .cloned()
    }

    pub fn list_related_skills(&self, skill_id: &str, limit: usize) -> Vec<GroupSkillIndexEntry> {
        let Some(record) = self.get_by_id(skill_id) else {
            return Vec::new();
        };

        let mut related = self
            .group_skills
            .get(&record.group)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|entry| entry.skill_id != record.skill_id)
            .collect::<Vec<_>>();
        related.sort_by(|a, b| a.skill_name.cmp(&b.skill_name));
        related.truncate(limit);
        related
    }

    pub fn list_group_skills(&self, group: &str, query: Option<&str>) -> Option<GroupSkillsResult> {
        let group_record = self.group_list.get(group)?;
        let mut skills = self.group_skills.get(group).cloned().unwrap_or_default();

        if let Some(query) = query.map(str::trim).filter(|query| !query.is_empty()) {
            let normalized_query = query.to_lowercase();
            let query_tokens = normalized_query
                .split(|c: char| !c.is_ascii_alphanumeric() && !is_cjk(c))
                .filter(|token| !token.is_empty())
                .map(|token| token.to_string())
                .collect::<Vec<_>>();

            skills.retain(|skill| {
                let text = format!(
                    "{} {}",
                    skill.skill_name.to_lowercase(),
                    skill.description.to_lowercase()
                );
                text.contains(&normalized_query)
                    || query_tokens.iter().any(|token| text.contains(token))
            });
        }

        skills.sort_by(|a, b| a.skill_name.cmp(&b.skill_name));
        Some(GroupSkillsResult {
            group: group_record.group.clone(),
            group_description: group_record.group_description.clone(),
            skills,
        })
    }

    pub fn install_skills(
        &mut self,
        source_path: &str,
        explicit_group: Option<&str>,
    ) -> Result<SkillInstallResult> {
        self.managed_groups = load_managed_groups(&self.layout)?;
        let source_path = PathBuf::from(source_path);
        let source_path = if source_path.is_absolute() {
            source_path
        } else {
            std::env::current_dir()?.join(source_path)
        };

        let installables = collect_installables(&source_path, &self.install_policy)?;
        let mut result = SkillInstallResult {
            installed: Vec::new(),
            skipped: Vec::<SkillInstallSkipped>::new(),
            failed: Vec::new(),
            needs_classification: Vec::new(),
            classification_hint: None,
        };

        if installables.is_empty() {
            result.failed.push(SkillInstallFailure {
                source_path: source_path.to_string_lossy().into_owned(),
                status: "failed".to_string(),
                message:
                    "no installable skill package or raw markdown candidate found under sourcePath"
                        .to_string(),
            });
            return Ok(result);
        }

        for installable in installables {
            match self.install_skill_candidate(&installable, explicit_group) {
                Ok(InstallOutcome::Installed(entry)) => result.installed.push(entry),
                Ok(InstallOutcome::NeedsClassification(entry)) => {
                    result.needs_classification.push(entry)
                }
                Err(error) => result.failed.push(SkillInstallFailure {
                    source_path: installable.source_path.to_string_lossy().into_owned(),
                    status: "failed".to_string(),
                    message: error.to_string(),
                }),
            }
        }

        if !result.installed.is_empty() {
            self.rebuild()?;
        }

        if !result.needs_classification.is_empty() {
            let prompts = result
                .needs_classification
                .iter()
                .map(|entry| {
                    build_classification_prompt(
                        &entry.skill_id,
                        &entry.description,
                        &self.list_managed_groups(),
                    )
                })
                .collect::<Vec<_>>();
            result.classification_hint = Some(format!(
                "{}\n\nPlease call install_skills again with the `group` parameter for each skill listed above.",
                prompts.join("\n\n---\n\n")
            ));
        }

        Ok(result)
    }

    pub fn validate_skills(&self, skill: Option<&str>) -> Result<SkillValidationResult> {
        let records = if let Some(skill) = skill {
            vec![self.resolve_single_skill(skill)?]
        } else {
            self.list_skill_records()
        };

        let mut issues = Vec::new();
        let mut name_to_records: BTreeMap<String, Vec<SkillRecord>> = BTreeMap::new();

        for record in &records {
            name_to_records
                .entry(record.skill_name.trim().to_lowercase())
                .or_default()
                .push(record.clone());

            let skill_path = Path::new(&record.skill_path);
            if !skill_path.exists() {
                issues.push(SkillValidationIssue {
                    skill_id: Some(record.skill_id.clone()),
                    path: record.skill_path.clone(),
                    code: "missing_skill_file".to_string(),
                    severity: "blocked".to_string(),
                    message: "SKILL.md is missing from package path".to_string(),
                });
                continue;
            }

            match parse_frontmatter_fields(&fs::read_to_string(skill_path)?) {
                Ok(frontmatter) => {
                    let name = frontmatter
                        .get("name")
                        .map(|value| value.trim())
                        .unwrap_or("");
                    let description = frontmatter
                        .get("description")
                        .map(|value| value.trim())
                        .unwrap_or("");
                    if name.is_empty() || description.is_empty() {
                        issues.push(SkillValidationIssue {
                            skill_id: Some(record.skill_id.clone()),
                            path: record.skill_path.clone(),
                            code: "invalid_frontmatter".to_string(),
                            severity: "blocked".to_string(),
                            message: "frontmatter missing required name/description".to_string(),
                        });
                    }
                }
                Err(error) => issues.push(SkillValidationIssue {
                    skill_id: Some(record.skill_id.clone()),
                    path: record.skill_path.clone(),
                    code: "invalid_frontmatter".to_string(),
                    severity: "blocked".to_string(),
                    message: error.to_string(),
                }),
            }

            if record.group == "general" {
                issues.push(SkillValidationIssue {
                    skill_id: Some(record.skill_id.clone()),
                    path: record.skill_path.clone(),
                    code: "generic_group".to_string(),
                    severity: "review_required".to_string(),
                    message:
                        "skill resolved to generic group and may need manual grouping refinement"
                            .to_string(),
                });
            }
        }

        for duplicates in name_to_records.values() {
            if duplicates.len() <= 1 {
                continue;
            }
            for record in duplicates {
                issues.push(SkillValidationIssue {
                    skill_id: Some(record.skill_id.clone()),
                    path: record.skill_path.clone(),
                    code: "duplicate_skill_name".to_string(),
                    severity: "review_required".to_string(),
                    message: format!("duplicate skillName detected: {}", record.skill_name),
                });
            }
        }

        let blocked = issues
            .iter()
            .filter(|issue| issue.severity == "blocked")
            .cloned()
            .collect::<Vec<_>>();
        let review_required = issues
            .iter()
            .filter(|issue| issue.severity == "review_required")
            .cloned()
            .collect::<Vec<_>>();
        let blocked_ids = blocked
            .iter()
            .filter_map(|issue| issue.skill_id.clone())
            .collect::<Vec<_>>();
        let review_ids = review_required
            .iter()
            .filter_map(|issue| issue.skill_id.clone())
            .collect::<Vec<_>>();

        let passed = records
            .into_iter()
            .filter(|record| {
                !blocked_ids.iter().any(|id| id == &record.skill_id)
                    && !review_ids.iter().any(|id| id == &record.skill_id)
            })
            .map(|record| SkillValidationPassed {
                skill_id: record.skill_id,
                skill_path: record.skill_path,
            })
            .collect::<Vec<_>>();

        Ok(SkillValidationResult {
            passed,
            review_required,
            blocked,
            issues,
        })
    }

    pub fn create_group(
        &mut self,
        group: &str,
        group_description: &str,
        keywords: Vec<String>,
        aliases: Vec<String>,
    ) -> Result<GroupCreateResult> {
        fs::create_dir_all(self.layout.packages_root.join(group))?;

        let new_group = ManagedGroupRecord {
            group: group.to_string(),
            group_description: group_description.to_string(),
            keywords,
            aliases,
            source: "custom".to_string(),
        };
        self.managed_groups.push(new_group.clone());
        self.managed_groups.sort_by(|a, b| a.group.cmp(&b.group));
        persist_managed_groups(&self.layout, &self.managed_groups)?;
        self.rebuild()?;

        Ok(GroupCreateResult {
            action: "created".to_string(),
            group: new_group,
        })
    }

    pub fn update_group(
        &mut self,
        group: &str,
        new_group: Option<&str>,
        group_description: Option<&str>,
        keywords: Option<Vec<String>>,
        aliases: Option<Vec<String>>,
    ) -> Result<GroupUpdateResult> {
        let next_group = new_group.unwrap_or(group);
        if new_group.is_some() {
            let old_dir = self.layout.packages_root.join(group);
            let new_dir = self.layout.packages_root.join(next_group);
            if old_dir.exists() {
                fs::rename(old_dir, new_dir)?;
            }
        }

        let managed = self
            .managed_groups
            .iter_mut()
            .find(|entry| entry.group == group)
            .ok_or_else(|| anyhow!("unknown group: {}", group))?;
        managed.group = next_group.to_string();
        if let Some(group_description) = group_description {
            managed.group_description = group_description.to_string();
        }
        if let Some(keywords) = keywords {
            managed.keywords = keywords;
        }
        if let Some(aliases) = aliases {
            managed.aliases = aliases;
        }

        self.managed_groups.sort_by(|a, b| a.group.cmp(&b.group));
        let updated = self
            .managed_groups
            .iter()
            .find(|entry| entry.group == next_group)
            .cloned()
            .ok_or_else(|| anyhow!("updated group missing: {}", next_group))?;
        persist_managed_groups(&self.layout, &self.managed_groups)?;
        self.rebuild()?;

        Ok(GroupUpdateResult {
            action: "updated".to_string(),
            previous_group: group.to_string(),
            group: updated,
        })
    }

    pub fn delete_group(&mut self, group: &str) -> Result<GroupDeleteResult> {
        let managed = self
            .managed_groups
            .iter()
            .find(|entry| entry.group == group)
            .cloned()
            .ok_or_else(|| anyhow!("unknown group: {}", group))?;
        let skill_count = self
            .group_skills
            .get(&managed.group)
            .map(|v| v.len())
            .unwrap_or(0);
        if skill_count > 0 {
            bail!(
                "group is not empty: {} ({} skills)",
                managed.group,
                skill_count
            );
        }

        self.managed_groups.retain(|entry| entry.group != group);
        persist_managed_groups(&self.layout, &self.managed_groups)?;
        remove_path_if_exists(&self.layout.packages_root.join(group))?;
        self.rebuild()?;

        Ok(GroupDeleteResult {
            action: "deleted".to_string(),
            group: group.to_string(),
        })
    }

    fn persist_indexes(&self) -> Result<()> {
        fs::create_dir_all(&self.layout.index_root)?;
        let groups: Vec<GroupListItem> = self.group_list.values().cloned().collect();
        let stage_root = self
            .layout
            .index_root
            .join(format!(".staging-{}", unique_suffix()));
        let stage_groups_root = stage_root.join("groups");
        let stage_skills_root = stage_root.join("skills");

        fs::create_dir_all(&stage_groups_root)?;
        fs::create_dir_all(&stage_skills_root)?;
        write_json_atomic(&stage_root.join("group-list.json"), &groups)?;

        for group in &groups {
            if let Some(skills) = self.group_skills.get(&group.group) {
                let payload = GroupSkillsResult {
                    group: group.group.clone(),
                    group_description: group.group_description.clone(),
                    skills: skills.clone(),
                };
                write_json_atomic(
                    &stage_groups_root.join(format!("{}.json", group.group)),
                    &payload,
                )?;
            }
        }

        for record in self.skill_records.values() {
            let file_name = format!("{}--{}.json", record.group, record.skill_id);
            write_json_atomic(&stage_skills_root.join(file_name), record)?;
        }

        commit_staged_indexes(
            &self.layout.index_root,
            &stage_root,
            &self.layout.group_list_path,
            &self.layout.groups_root,
            &self.layout.skills_root,
        )?;
        Ok(())
    }

    fn persist_package_metadata(&self) -> Result<()> {
        for record in self.skill_records.values() {
            let meta = SkillMeta {
                skill_id: record.skill_id.clone(),
                skill_name: record.skill_name.clone(),
                description: record.description.clone(),
                group: record.group.clone(),
                group_description: record.group_description.clone(),
                keywords: record.keywords.clone(),
                updated_at_ms: record.updated_at_ms,
                status: record.status.clone(),
            };
            let meta_path = Path::new(&record.skill_path)
                .parent()
                .map(|parent| parent.join("meta.json"))
                .context("skill path missing parent directory")?;
            write_skill_meta_atomic_if_changed(&meta_path, &meta)?;
        }
        Ok(())
    }

    fn upsert_in_memory(&mut self, parsed: crate::model::ParsedSkill) {
        let skill_id = Path::new(&parsed.skill_path)
            .parent()
            .and_then(|parent| parent.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let composite_key = format!("{}/{}", parsed.group, skill_id);

        self.skill_records.insert(
            composite_key,
            SkillRecord {
                skill_id: skill_id.clone(),
                skill_name: parsed.skill_name,
                description: parsed.description,
                group: parsed.group,
                group_description: parsed.group_description,
                keywords: parsed.keywords,
                skill_path: parsed.skill_path,
                updated_at_ms: parsed.updated_at_ms,
                status: "ready".to_string(),
            },
        );

        self.rebuild_groups_from_skills();
    }

    fn rebuild_groups_from_skills(&mut self) {
        self.group_list.clear();
        self.group_skills.clear();

        for managed_group in &self.managed_groups {
            self.group_list.insert(
                managed_group.group.clone(),
                GroupListItem {
                    group: managed_group.group.clone(),
                    group_description: managed_group.group_description.clone(),
                },
            );
            self.group_skills
                .entry(managed_group.group.clone())
                .or_default();
        }

        for record in self.skill_records.values() {
            self.group_list
                .entry(record.group.clone())
                .or_insert_with(|| GroupListItem {
                    group: record.group.clone(),
                    group_description: record.group_description.clone(),
                });
            self.group_skills
                .entry(record.group.clone())
                .or_default()
                .push(GroupSkillIndexEntry {
                    skill_id: record.skill_id.clone(),
                    skill_name: record.skill_name.clone(),
                    description: record.description.clone(),
                    keywords: record.keywords.clone(),
                    skill_path: record.skill_path.clone(),
                });
        }
    }

    fn install_skill_candidate(
        &self,
        installable: &InstallableSource,
        explicit_group: Option<&str>,
    ) -> Result<InstallOutcome> {
        if installable.skill_id.is_empty() {
            bail!("skill id is empty after normalization");
        }

        if let Some(group) = explicit_group
            .map(str::trim)
            .filter(|group| !group.is_empty())
        {
            self.require_managed_group(group)?;
            return self.install_candidate_into_group(installable, group);
        }

        let raw = fs::read_to_string(installable.skill_file_path())?;
        let frontmatter = parse_frontmatter_fields(&raw)?;
        if matches!(installable.kind, InstallableKind::Package) {
            if let Some(group) = frontmatter
                .get("group")
                .map(|value| value.trim())
                .filter(|group| !group.is_empty())
            {
                self.require_managed_group(group)?;
                return self.install_candidate_into_group(installable, group);
            }
        }

        let description = frontmatter
            .get("description")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        Ok(InstallOutcome::NeedsClassification(
            SkillNeedsClassification {
                skill_id: installable.skill_id.clone(),
                source_path: installable.source_path.to_string_lossy().into_owned(),
                description,
            },
        ))
    }

    fn install_candidate_into_group(
        &self,
        installable: &InstallableSource,
        group: &str,
    ) -> Result<InstallOutcome> {
        let destination_dir = self
            .layout
            .packages_root
            .join(group)
            .join(&installable.skill_id);
        remove_path_if_exists(&destination_dir)?;
        fs::create_dir_all(&destination_dir)?;
        match installable.kind {
            InstallableKind::Package => copy_directory(&installable.source_path, &destination_dir)?,
            InstallableKind::Markdown => {
                write_markdown_candidate_as_package(&installable.source_path, &destination_dir)?
            }
        }

        Ok(InstallOutcome::Installed(SkillInstallEntry {
            source_path: installable.source_path.to_string_lossy().into_owned(),
            skill_id: installable.skill_id.clone(),
            installed_path: destination_dir.to_string_lossy().into_owned(),
            status: "installed".to_string(),
        }))
    }

    fn require_managed_group(&self, group: &str) -> Result<&ManagedGroupRecord> {
        self.managed_groups
            .iter()
            .find(|record| record.group == group)
            .ok_or_else(|| {
                anyhow!(
                    "unknown group: {group}. create it first with manage_group create before calling install_skills again"
                )
            })
    }

    pub fn reclassify_skill(&mut self, skill: &str, target_group: &str) -> Result<ReclassifyResult> {
        let record = self.resolve_single_skill(skill)?;
        let source_group = &record.group;

        if source_group == target_group {
            return Ok(ReclassifyResult {
                skill_id: record.skill_id.clone(),
                skill_name: record.skill_name.clone(),
                from_group: source_group.clone(),
                to_group: target_group.to_string(),
                status: "skipped".to_string(),
                message: "skill is already in the target group".to_string(),
            });
        }

        // Validate target group exists
        if !self.managed_groups.iter().any(|g| g.group == target_group) {
            let valid = self
                .managed_groups
                .iter()
                .map(|g| g.group.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "unknown target group: {target_group}. valid groups: {valid}"
            );
        }

        let old_path = Path::new(&record.skill_path);
        let skill_dir = old_path
            .parent()
            .ok_or_else(|| anyhow!("skill path has no parent: {}", record.skill_path))?;
        let skill_dir_name = skill_dir
            .file_name()
            .ok_or_else(|| anyhow!("skill dir has no name: {}", skill_dir.display()))?;

        let packages_root = &self.layout.packages_root;
        let new_dir = packages_root.join(target_group).join(skill_dir_name);

        if new_dir.exists() {
            bail!(
                "target directory already exists: {}. remove it first or rename the skill.",
                new_dir.display()
            );
        }

        // Ensure target group directory exists
        let target_group_dir = packages_root.join(target_group);
        if !target_group_dir.exists() {
            fs::create_dir_all(&target_group_dir).with_context(|| {
                format!("failed to create target group dir: {}", target_group_dir.display())
            })?;
        }

        // Update SKILL.md frontmatter group field
        update_skill_md_group(old_path, target_group)?;

        // Move the directory
        fs::rename(skill_dir, &new_dir).with_context(|| {
            format!(
                "failed to move {} to {}",
                skill_dir.display(),
                new_dir.display()
            )
        })?;

        // Rebuild index
        self.rebuild()?;

        Ok(ReclassifyResult {
            skill_id: record.skill_id,
            skill_name: record.skill_name,
            from_group: source_group.clone(),
            to_group: target_group.to_string(),
            status: "moved".to_string(),
            message: format!(
                "moved from {} to {}",
                source_group, target_group
            ),
        })
    }

    fn resolve_single_skill(&self, skill: &str) -> Result<SkillRecord> {
        self.get_by_id(skill)
            .or_else(|| self.get_by_name(skill))
            .ok_or_else(|| anyhow!("unknown skill: {}", skill))
    }
}

fn load_managed_groups(layout: &SkillShelfStorageLayout) -> Result<Vec<ManagedGroupRecord>> {
    if !layout.group_catalog_path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&layout.group_catalog_path)
        .with_context(|| format!("failed to read {}", layout.group_catalog_path.display()))?;
    let groups: Vec<ManagedGroupRecord> = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", layout.group_catalog_path.display()))?;
    Ok(groups)
}

fn persist_managed_groups(
    layout: &SkillShelfStorageLayout,
    groups: &[ManagedGroupRecord],
) -> Result<()> {
    write_json_atomic(&layout.group_catalog_path, groups)
}

fn collect_skill_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !root.exists() {
        return Ok(files);
    }
    walk(root, &mut files)?;
    Ok(files)
}

fn walk(current: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            walk(&path, files)?;
        } else if file_type.is_file() && entry.file_name() == SKILL_FILENAME {
            files.push(path);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallableKind {
    Package,
    Markdown,
}

#[derive(Debug, Clone)]
struct InstallableSource {
    kind: InstallableKind,
    source_path: PathBuf,
    skill_id: String,
}

impl InstallableSource {
    fn skill_file_path(&self) -> PathBuf {
        match self.kind {
            InstallableKind::Package => self.source_path.join(SKILL_FILENAME),
            InstallableKind::Markdown => self.source_path.clone(),
        }
    }
}

enum InstallOutcome {
    Installed(SkillInstallEntry),
    NeedsClassification(SkillNeedsClassification),
}

fn collect_installables(
    source_path: &Path,
    install_policy: &InstallPolicy,
) -> Result<Vec<InstallableSource>> {
    let metadata = fs::metadata(source_path)?;
    if metadata.is_file() {
        if source_path.file_name().and_then(|name| name.to_str()) == Some(SKILL_FILENAME) {
            if !install_policy.accept_package_directories {
                return Ok(Vec::new());
            }
            let source_dir = source_path
                .parent()
                .ok_or_else(|| anyhow!("skill file missing parent directory"))?;
            return Ok(vec![InstallableSource {
                kind: InstallableKind::Package,
                source_path: source_dir.to_path_buf(),
                skill_id: sanitize_id(
                    &source_dir
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default(),
                ),
            }]);
        }

        if install_policy.accept_raw_markdown
            && is_markdown_candidate(source_path)
            && (!install_policy.raw_markdown_requires_frontmatter
                || has_required_frontmatter(source_path)?)
        {
            return Ok(vec![InstallableSource {
                kind: InstallableKind::Markdown,
                source_path: source_path.to_path_buf(),
                skill_id: sanitize_id(
                    source_path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default(),
                ),
            }]);
        }

        return Ok(Vec::new());
    }

    let direct_skill_path = source_path.join(SKILL_FILENAME);
    if install_policy.accept_package_directories && direct_skill_path.exists() {
        return Ok(vec![InstallableSource {
            kind: InstallableKind::Package,
            source_path: source_path.to_path_buf(),
            skill_id: sanitize_id(
                source_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default(),
            ),
        }]);
    }

    let mut installables = Vec::new();
    collect_installables_walk(source_path, &mut installables, install_policy)?;
    Ok(dedupe_installables(installables))
}

fn collect_installables_walk(
    current: &Path,
    installables: &mut Vec<InstallableSource>,
    install_policy: &InstallPolicy,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }

        if file_type.is_dir() {
            if install_policy.accept_package_directories && path.join(SKILL_FILENAME).exists() {
                installables.push(InstallableSource {
                    kind: InstallableKind::Package,
                    source_path: path.clone(),
                    skill_id: sanitize_id(
                        path.file_name()
                            .and_then(|value| value.to_str())
                            .unwrap_or_default(),
                    ),
                });
            }
            collect_installables_walk(&path, installables, install_policy)?;
            continue;
        }

        if path.file_name().and_then(|value| value.to_str()) == Some(SKILL_FILENAME) {
            continue;
        }

        if !install_policy.accept_raw_markdown
            || !is_markdown_candidate(&path)
            || (install_policy.raw_markdown_requires_frontmatter
                && !has_required_frontmatter(&path)?)
        {
            continue;
        }

        installables.push(InstallableSource {
            kind: InstallableKind::Markdown,
            source_path: path.clone(),
            skill_id: sanitize_id(
                path.file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default(),
            ),
        });
    }
    Ok(())
}

fn dedupe_installables(installables: Vec<InstallableSource>) -> Vec<InstallableSource> {
    let mut by_skill_id: BTreeMap<String, InstallableSource> = BTreeMap::new();
    for installable in installables {
        match by_skill_id.get(&installable.skill_id) {
            None => {
                by_skill_id.insert(installable.skill_id.clone(), installable);
            }
            Some(existing)
                if existing.kind == InstallableKind::Markdown
                    && installable.kind == InstallableKind::Package =>
            {
                by_skill_id.insert(installable.skill_id.clone(), installable);
            }
            _ => {}
        }
    }
    by_skill_id.into_values().collect()
}

fn build_classification_prompt(
    skill_id: &str,
    description: &str,
    groups: &[ManagedGroupRecord],
) -> String {
    let group_list = groups
        .iter()
        .map(|group| format!("- **{}**: {}", group.group, group.group_description))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Skill \"{}\" needs group classification.\nDescription: {}\n\nAvailable groups:\n{}\n\nPlease call install_skills again with the most appropriate `group` parameter.",
        skill_id, description, group_list
    )
}

fn has_required_frontmatter(file_path: &Path) -> Result<bool> {
    let raw = fs::read_to_string(file_path)?;
    let frontmatter = match parse_frontmatter_fields(&raw) {
        Ok(frontmatter) => frontmatter,
        Err(_) => return Ok(false),
    };
    let name = frontmatter
        .get("name")
        .map(|value| value.trim())
        .unwrap_or("");
    let description = frontmatter
        .get("description")
        .map(|value| value.trim())
        .unwrap_or("");
    Ok(!name.is_empty() && !description.is_empty())
}

fn is_markdown_candidate(file_path: &Path) -> bool {
    if file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| !ext.eq_ignore_ascii_case("md"))
        .unwrap_or(true)
    {
        return false;
    }
    let basename = file_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_lowercase();
    !EXCLUDED_MARKDOWN_BASENAMES.contains(&basename.as_str())
}

fn write_markdown_candidate_as_package(source_file: &Path, destination_dir: &Path) -> Result<()> {
    let source_raw = fs::read_to_string(source_file)?;
    let frontmatter = parse_frontmatter_fields(&source_raw)?;
    let name = frontmatter
        .get("name")
        .map(|value| value.trim())
        .unwrap_or("");
    let description = frontmatter
        .get("description")
        .map(|value| value.trim())
        .unwrap_or("");
    if name.is_empty() || description.is_empty() {
        bail!("raw markdown candidate is missing required frontmatter name/description");
    }

    fs::write(destination_dir.join(SKILL_FILENAME), source_raw)?;
    write_json_atomic(
        &destination_dir.join("SOURCE.json"),
        &serde_json::json!({
            "sourcePath": source_file.to_string_lossy(),
            "kind": "markdown"
        }),
    )
}

fn copy_directory(source_dir: &Path, destination_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let destination_path = destination_dir.join(entry.file_name());
        if file_type.is_dir() {
            fs::create_dir_all(&destination_path)?;
            copy_directory(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}

fn sanitize_id(value: &str) -> String {
    let mut result = String::new();
    let mut last_was_dash = false;
    for ch in value.chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            result.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            result.push('-');
            last_was_dash = true;
        }
    }
    result.trim_matches('-').to_string()
}

fn parse_frontmatter_fields(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut lines = raw.lines();
    if lines.next() != Some("---") {
        bail!("missing frontmatter start");
    }

    let mut frontmatter = String::new();
    let mut closed = false;
    for line in lines {
        if line == "---" {
            closed = true;
            break;
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }

    if !closed {
        bail!("missing frontmatter end");
    }

    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&frontmatter).with_context(|| "failed to parse YAML frontmatter")?;
    let mapping = yaml
        .as_mapping()
        .ok_or_else(|| anyhow!("frontmatter must be a YAML mapping"))?;
    let mut map = BTreeMap::new();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            continue;
        };
        map.insert(key.trim().to_string(), yaml_value_to_string(value)?);
    }

    Ok(map)
}

fn yaml_value_to_string(value: &serde_yaml::Value) -> Result<String> {
    Ok(match value {
        serde_yaml::Value::Null => String::new(),
        serde_yaml::Value::Bool(value) => value.to_string(),
        serde_yaml::Value::Number(value) => value.to_string(),
        serde_yaml::Value::String(value) => value.clone(),
        other => serde_yaml::to_string(other)
            .with_context(|| "failed to stringify YAML frontmatter value")?
            .trim()
            .to_string(),
    })
}

fn derive_group_from_path(skill_file_path: &Path, packages_root: &Path) -> Option<String> {
    let skill_dir = skill_file_path.parent()?;
    let group_dir = skill_dir.parent()?;
    let packages_root = packages_root
        .canonicalize()
        .unwrap_or_else(|_| packages_root.to_path_buf());
    let group_dir_resolved = group_dir
        .canonicalize()
        .unwrap_or_else(|_| group_dir.to_path_buf());
    if group_dir_resolved == packages_root {
        return None;
    }
    group_dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

fn write_json_atomic<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let parent = path
        .parent()
        .context("target path missing parent directory")?;
    fs::create_dir_all(parent)?;

    let temp_name = format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("index"),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let temp_path = parent.join(temp_name);
    let json = serde_json::to_vec_pretty(value)?;

    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(&json)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }

    // Windows does not replace existing targets with fs::rename. The staging
    // index swap protects index readers; this helper uses the same
    // replace-existing rule for per-skill meta files.
    remove_path_if_exists(path)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

fn write_skill_meta_atomic_if_changed(path: &Path, value: &SkillMeta) -> Result<()> {
    if let Ok(existing_raw) = fs::read_to_string(path) {
        if let Ok(existing_value) = serde_json::from_str::<SkillMeta>(&existing_raw) {
            if skill_meta_semantically_equal(&existing_value, value) {
                return Ok(());
            }
        }
    }

    write_json_atomic(path, value)
}

fn skill_meta_semantically_equal(left: &SkillMeta, right: &SkillMeta) -> bool {
    left.skill_id == right.skill_id
        && left.skill_name == right.skill_name
        && left.description == right.description
        && left.group == right.group
        && left.group_description == right.group_description
        && left.keywords == right.keywords
        && left.status == right.status
        && (left.updated_at_ms - right.updated_at_ms).abs() < 0.001
}

fn commit_staged_indexes(
    index_root: &Path,
    stage_root: &Path,
    group_list_path: &Path,
    groups_root: &Path,
    skills_root: &Path,
) -> Result<()> {
    let txn = unique_suffix();
    let operations = vec![
        ReplaceOp {
            live_path: group_list_path.to_path_buf(),
            staged_path: stage_root.join("group-list.json"),
            backup_path: index_root.join(format!(".backup-group-list-{}", txn)),
        },
        ReplaceOp {
            live_path: groups_root.to_path_buf(),
            staged_path: stage_root.join("groups"),
            backup_path: index_root.join(format!(".backup-groups-{}", txn)),
        },
        ReplaceOp {
            live_path: skills_root.to_path_buf(),
            staged_path: stage_root.join("skills"),
            backup_path: index_root.join(format!(".backup-skills-{}", txn)),
        },
    ];
    let mut completed = Vec::new();

    for op in &operations {
        remove_path_if_exists(&op.backup_path)?;
        if op.live_path.exists() {
            fs::rename(&op.live_path, &op.backup_path).with_context(|| {
                format!(
                    "failed to move existing path {} to backup {}",
                    op.live_path.display(),
                    op.backup_path.display()
                )
            })?;
        }

        if let Err(error) = fs::rename(&op.staged_path, &op.live_path) {
            restore_replaced_paths(&completed)?;
            restore_live_path(op)?;
            let _ = fs::remove_dir_all(stage_root);
            return Err(error).with_context(|| {
                format!(
                    "failed to promote staged path {} to {}",
                    op.staged_path.display(),
                    op.live_path.display()
                )
            });
        }

        completed.push(op.clone());
    }

    for op in &completed {
        remove_path_if_exists(&op.backup_path)?;
    }
    if stage_root.exists() {
        fs::remove_dir_all(stage_root)?;
    }
    Ok(())
}

fn restore_replaced_paths(completed: &[ReplaceOp]) -> Result<()> {
    for op in completed.iter().rev() {
        restore_live_path(op)?;
    }
    Ok(())
}

fn restore_live_path(op: &ReplaceOp) -> Result<()> {
    if op.backup_path.exists() {
        remove_path_if_exists(&op.live_path)?;
        fs::rename(&op.backup_path, &op.live_path).with_context(|| {
            format!(
                "failed to restore backup {} to {}",
                op.backup_path.display(),
                op.live_path.display()
            )
        })?;
    }
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn is_cjk(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string()
}

#[derive(Clone)]
struct ReplaceOp {
    live_path: PathBuf,
    staged_path: PathBuf,
    backup_path: PathBuf,
}

/// Update the `group` field in SKILL.md frontmatter.
/// Handles both YAML frontmatter with and without existing `group` field.
fn update_skill_md_group(skill_md_path: &Path, new_group: &str) -> Result<()> {
    let content = fs::read_to_string(skill_md_path)
        .with_context(|| format!("failed to read {}", skill_md_path.display()))?;

    if !content.starts_with("---") {
        // No frontmatter at all — prepend one
        let new_content = format!("---\ngroup: {new_group}\n---\n{content}");
        fs::write(skill_md_path, new_content)
            .with_context(|| format!("failed to write {}", skill_md_path.display()))?;
        return Ok(());
    }

    // Find closing ---
    let rest = &content[3..];
    let close_pos = rest
        .find("\n---")
        .ok_or_else(|| anyhow!("unclosed frontmatter in {}", skill_md_path.display()))?;

    let frontmatter = &rest[..close_pos];

    // Replace or insert group field
    let new_frontmatter = if let Some(line_start) = frontmatter.find("\ngroup:") {
        // Replace existing group line
        let line_end = frontmatter[line_start + 1..]
            .find('\n')
            .map(|i| line_start + 1 + i)
            .unwrap_or(frontmatter.len());
        format!(
            "{}\ngroup: {new_group}{}",
            &frontmatter[..line_start],
            &frontmatter[line_end..]
        )
    } else {
        // Insert group field at end of frontmatter
        format!("{frontmatter}\ngroup: {new_group}")
    };

    let after_frontmatter = &rest[close_pos + 4..]; // skip \n---
    let new_content = format!("---\n{new_frontmatter}\n---{after_frontmatter}");

    fs::write(skill_md_path, new_content)
        .with_context(|| format!("failed to write {}", skill_md_path.display()))?;
    Ok(())
}
