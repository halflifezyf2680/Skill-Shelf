use std::fs;
use std::path::{Path, PathBuf};

use skill_shelf::config::resolve_storage_layout;
use skill_shelf::model::{
    GroupCreateResult, GroupDeleteResult, GroupListItem, GroupUpdateResult, RegistryIssue,
    SkillInstallResult, SkillValidationResult,
};
use skill_shelf::registry::SkillRegistry;

fn write_file(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn make_shelf() -> (tempfile::TempDir, PathBuf, SkillRegistry) {
    let temp = tempfile::tempdir().unwrap();
    let shelf = temp.path().join("hub");
    fs::create_dir_all(shelf.join("config")).unwrap();
    fs::create_dir_all(shelf.join("packages")).unwrap();
    write_file(
        shelf.join("config/groups.json"),
        r#"[
  {
    "group": "engineering",
    "groupDescription": "Engineering skills",
    "keywords": ["rust", "code"],
    "aliases": ["backend"],
    "source": "builtin"
  },
  {
    "group": "general",
    "groupDescription": "General skills",
    "keywords": ["general"],
    "aliases": [],
    "source": "builtin"
  }
]
"#,
    );
    let layout = resolve_storage_layout(&shelf);
    let registry = SkillRegistry::new(layout);
    (temp, shelf, registry)
}

#[test]
fn install_skills_supports_package_skill_file_and_raw_markdown_sources() {
    let (_temp, shelf, mut registry) = make_shelf();

    let package_dir = shelf.join("incoming/pkg-skill");
    write_file(
        package_dir.join("SKILL.md"),
        r#"---
name: Package Skill
description: Helps with package installs
group: general
---
Body
"#,
    );
    write_file(package_dir.join("notes.txt"), "payload");

    let installed_from_dir: SkillInstallResult = registry
        .install_skills(package_dir.to_str().unwrap(), Some("engineering"))
        .unwrap();
    assert_eq!(installed_from_dir.installed.len(), 1);
    assert!(installed_from_dir.failed.is_empty());
    assert!(shelf
        .join("packages/engineering/pkg-skill/SKILL.md")
        .exists());
    assert_eq!(registry.list_skill_records().len(), 1);
    assert_eq!(registry.list_skill_records()[0].group, "engineering");

    let skill_file_package = shelf.join("incoming/file-skill");
    write_file(
        skill_file_package.join("SKILL.md"),
        r#"---
name: File Skill
description: Installs from a direct skill file path
group: engineering
---
Body
"#,
    );

    let installed_from_skill_file = registry
        .install_skills(skill_file_package.join("SKILL.md").to_str().unwrap(), None)
        .unwrap();
    assert_eq!(installed_from_skill_file.installed.len(), 1);
    assert!(shelf
        .join("packages/engineering/file-skill/SKILL.md")
        .exists());

    let raw_markdown = shelf.join("incoming/raw-markdown.md");
    write_file(
        &raw_markdown,
        r#"---
name: Raw Markdown
description: Installs from markdown with source metadata
---
Body
"#,
    );

    let needs_classification = registry
        .install_skills(raw_markdown.to_str().unwrap(), None)
        .unwrap();
    assert!(needs_classification.installed.is_empty());
    assert_eq!(needs_classification.needs_classification.len(), 1);
    assert!(needs_classification.classification_hint.is_some());

    let installed_raw = registry
        .install_skills(raw_markdown.to_str().unwrap(), Some("engineering"))
        .unwrap();
    assert_eq!(installed_raw.installed.len(), 1);
    assert!(shelf
        .join("packages/engineering/raw-markdown/SOURCE.json")
        .exists());

    let classified_raw_with_frontmatter_group = shelf.join("incoming/raw-frontmatter-group.md");
    write_file(
        &classified_raw_with_frontmatter_group,
        r#"---
name: Raw Frontmatter Group
description: Raw markdown should still need classification
group: engineering
---
Body
"#,
    );

    let raw_group_result = registry
        .install_skills(
            classified_raw_with_frontmatter_group.to_str().unwrap(),
            None,
        )
        .unwrap();
    assert!(raw_group_result.installed.is_empty());
    assert_eq!(raw_group_result.needs_classification.len(), 1);
    assert!(raw_group_result.classification_hint.is_some());
}

#[test]
fn install_skills_requires_managed_groups_or_classifies_unlabeled_sources() {
    let (_temp, shelf, mut registry) = make_shelf();

    let unlabeled_raw = shelf.join("incoming/unlabeled.md");
    write_file(
        &unlabeled_raw,
        r#"---
name: Unlabeled Raw
description: Needs classification
---
Body
"#,
    );

    let needs_classification = registry
        .install_skills(unlabeled_raw.to_str().unwrap(), None)
        .unwrap();
    assert!(needs_classification.installed.is_empty());
    assert_eq!(needs_classification.needs_classification.len(), 1);
    assert!(needs_classification.classification_hint.is_some());

    let explicit_missing = shelf.join("incoming/explicit-missing");
    write_file(
        explicit_missing.join("SKILL.md"),
        r#"---
name: Explicit Missing
description: Explicit group should fail
---
Body
"#,
    );

    let explicit_failure = registry
        .install_skills(explicit_missing.to_str().unwrap(), Some("missing-group"))
        .unwrap();
    assert!(explicit_failure.installed.is_empty());
    assert_eq!(explicit_failure.failed.len(), 1);
    assert!(explicit_failure.failed[0]
        .message
        .contains("manage_group create"));

    let frontmatter_missing = shelf.join("incoming/frontmatter-missing");
    write_file(
        frontmatter_missing.join("SKILL.md"),
        r#"---
name: Frontmatter Missing
description: Frontmatter group should fail
group: missing-group
---
Body
"#,
    );

    let frontmatter_failure = registry
        .install_skills(frontmatter_missing.to_str().unwrap(), None)
        .unwrap();
    assert!(frontmatter_failure.installed.is_empty());
    assert_eq!(frontmatter_failure.failed.len(), 1);
    assert!(frontmatter_failure.failed[0]
        .message
        .contains("unknown group: missing-group"));
}

#[test]
fn install_skills_overwrites_destination_and_reports_no_installable_candidates() {
    let (_temp, shelf, mut registry) = make_shelf();

    let destination = shelf.join("packages/engineering/pkg-skill");
    write_file(destination.join("SKILL.md"), "stale");
    write_file(destination.join("obsolete.txt"), "stale");

    let package_dir = shelf.join("incoming/pkg-skill");
    write_file(
        package_dir.join("SKILL.md"),
        r#"---
name: Package Skill
description: Fresh package payload
group: engineering
---
Fresh
"#,
    );

    let result = registry
        .install_skills(package_dir.to_str().unwrap(), None)
        .unwrap();
    assert_eq!(result.installed.len(), 1);
    assert!(!destination.join("obsolete.txt").exists());
    assert!(fs::read_to_string(destination.join("SKILL.md"))
        .unwrap()
        .contains("Fresh package payload"));

    let empty_dir = shelf.join("incoming/empty");
    fs::create_dir_all(&empty_dir).unwrap();
    let no_installable = registry
        .install_skills(empty_dir.to_str().unwrap(), None)
        .unwrap();
    assert!(no_installable.installed.is_empty());
    assert_eq!(no_installable.failed.len(), 1);
    assert!(no_installable.failed[0]
        .message
        .contains("no installable skill package or raw markdown candidate found"));
}

#[test]
fn validate_skills_reports_missing_invalid_duplicate_and_generic_group_issues() {
    let (_temp, shelf, mut registry) = make_shelf();

    write_file(
        shelf.join("packages/engineering/dup-a/SKILL.md"),
        r#"---
name: Shared Name
description: First duplicate record
---
"#,
    );
    write_file(
        shelf.join("packages/engineering/dup-b/SKILL.md"),
        r#"---
name: Shared Name
description: Second duplicate record
---
"#,
    );
    write_file(
        shelf.join("packages/general/general-skill/SKILL.md"),
        r#"---
name: General Skill
description: Generic group record
---
"#,
    );
    write_file(
        shelf.join("packages/engineering/missing-later/SKILL.md"),
        r#"---
name: Missing Later
description: Will be deleted before validation
---
"#,
    );
    write_file(
        shelf.join("packages/engineering/invalid-later/SKILL.md"),
        r#"---
name: Invalid Later
description: Will be corrupted before validation
---
"#,
    );
    write_file(
        shelf.join("packages/engineering/yaml-multiline/SKILL.md"),
        r#"---
name: YAML Multiline
description: >
  Valid multiline YAML
  description should pass governance.
---
"#,
    );

    registry.rebuild().unwrap();

    fs::remove_file(shelf.join("packages/engineering/missing-later/SKILL.md")).unwrap();
    write_file(
        shelf.join("packages/engineering/invalid-later/SKILL.md"),
        r#"---
name Invalid Later
description: broken
"#,
    );

    let validation: SkillValidationResult = registry.validate_skills(None).unwrap();
    assert_eq!(validation.blocked.len(), 2);
    assert_eq!(validation.review_required.len(), 3);
    assert!(validation
        .issues
        .iter()
        .any(|issue| issue.code == "missing_skill_file"));
    assert!(validation
        .issues
        .iter()
        .any(|issue| issue.code == "invalid_frontmatter"));
    assert_eq!(
        validation
            .issues
            .iter()
            .filter(|issue| issue.code == "duplicate_skill_name")
            .count(),
        2
    );
    assert!(validation
        .issues
        .iter()
        .any(|issue| issue.code == "generic_group"));
    assert!(!validation
        .issues
        .iter()
        .any(|issue| issue.skill_id.as_deref() == Some("yaml-multiline")));
    assert!(validation
        .passed
        .iter()
        .any(|entry| entry.skill_id == "yaml-multiline"));

    let by_id = registry.validate_skills(Some("dup-a")).unwrap();
    assert_eq!(by_id.passed.len(), 1);

    let by_name = registry.validate_skills(Some("General Skill")).unwrap();
    assert_eq!(by_name.review_required.len(), 1);
    assert_eq!(by_name.review_required[0].code, "generic_group");

    let error = registry
        .validate_skills(Some("does-not-exist"))
        .unwrap_err();
    assert!(error.to_string().contains("unknown skill: does-not-exist"));
}

#[test]
fn manage_group_supports_create_update_rename_and_delete_contracts() {
    let (_temp, shelf, mut registry) = make_shelf();
    registry.rebuild().unwrap();

    let created: GroupCreateResult = registry
        .create_group(
            "custom-tools",
            "Custom tools",
            vec!["custom".into()],
            vec!["tools".into()],
        )
        .unwrap();
    assert_eq!(created.action, "created");
    assert_eq!(created.group.group, "custom-tools");
    assert!(shelf.join("packages/custom-tools").exists());

    let updated: GroupUpdateResult = registry
        .update_group(
            "custom-tools",
            Some("renamed-tools"),
            Some("Renamed tools"),
            Some(vec!["renamed".into()]),
            Some(vec!["toolbox".into()]),
        )
        .unwrap();
    assert_eq!(updated.action, "updated");
    assert_eq!(updated.previous_group, "custom-tools");
    assert_eq!(updated.group.group, "renamed-tools");
    assert!(!shelf.join("packages/custom-tools").exists());
    assert!(shelf.join("packages/renamed-tools").exists());

    let groups_json = fs::read_to_string(shelf.join("config/groups.json")).unwrap();
    assert!(groups_json.contains("\"group\": \"renamed-tools\""));
    assert!(!groups_json.contains("\"group\": \"custom-tools\""));

    write_file(
        shelf.join("packages/renamed-tools/occupied-skill/SKILL.md"),
        r#"---
name: Occupied Skill
description: Keeps group non-empty
---
"#,
    );
    registry.rebuild().unwrap();

    let err = registry.delete_group("renamed-tools").unwrap_err();
    assert!(err
        .to_string()
        .contains("group is not empty: renamed-tools (1 skills)"));

    fs::remove_dir_all(shelf.join("packages/renamed-tools/occupied-skill")).unwrap();
    registry.rebuild().unwrap();

    let deleted: GroupDeleteResult = registry.delete_group("renamed-tools").unwrap();
    assert_eq!(deleted.action, "deleted");
    assert_eq!(deleted.group, "renamed-tools");
    assert!(!shelf.join("packages/renamed-tools").exists());
}

#[test]
fn registry_exposes_empty_managed_groups_and_rebuild_issues() {
    let (_temp, shelf, mut registry) = make_shelf();

    write_file(
        shelf.join("packages/engineering/invalid-helper/SKILL.md"),
        r#"---
name Invalid Helper
description: broken frontmatter
"#,
    );

    registry.rebuild().unwrap();

    let groups = registry.list_groups();
    assert_eq!(
        groups,
        vec![
            GroupListItem {
                group: "engineering".into(),
                group_description: "Engineering skills".into(),
            },
            GroupListItem {
                group: "general".into(),
                group_description: "General skills".into(),
            },
        ]
    );

    let issues: Vec<RegistryIssue> = registry.list_issues();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].path.contains("invalid-helper"));
}
