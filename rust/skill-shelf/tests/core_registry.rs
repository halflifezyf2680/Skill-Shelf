use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use skill_shelf::config::resolve_storage_layout;
use skill_shelf::model::{
    GroupListItem, GroupSkillsResult, RegistryIssue, ShelfStatus, SkillMeta, SkillRecord,
    WatcherStatus,
};
use skill_shelf::parser::parse_skill_file;
use skill_shelf::registry::SkillRegistry;
use skill_shelf::search::search_skills;

fn copy_dir_all(src: &Path, dst: &Path) {
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

fn write_file(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn parse_skill_file_uses_yaml_frontmatter_and_ts_keyword_rules() {
    let temp = tempfile::tempdir().unwrap();
    let skill_path = temp.path().join("SKILL.md");
    write_file(
        &skill_path,
        r#"---
name: "  Quoted Name  "
description: "  Alpha,  beta，gamma、delta / epsilon   zeta  "
---
Body text
"#,
    );

    let parsed = parse_skill_file(&skill_path, &[], 8, Some("engineering")).unwrap();

    assert_eq!(parsed.skill_name, "Quoted Name");
    assert_eq!(
        parsed.description,
        "Alpha, beta，gamma、delta / epsilon zeta"
    );
    assert_eq!(
        parsed.keywords,
        vec![
            "alpha".to_string(),
            "beta，gamma、delta".to_string(),
            "epsilon".to_string(),
            "zeta".to_string(),
        ]
    );
}

#[test]
fn rebuild_preserves_ts_style_meta_when_semantically_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let skill_path = shelf.join("packages/engineering/rust-helper/SKILL.md");
    let meta_path = shelf.join("packages/engineering/rust-helper/meta.json");
    let mtime_ms = fs::metadata(&skill_path)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        * 1000.0;
    let mtime_text = mtime_ms.to_string();
    let ts_style_mtime = if let Some((whole, fractional)) = mtime_text.split_once('.') {
        format!("{whole}.{fractional}000000")
    } else {
        format!("{mtime_text}.000000")
    };
    write_file(
        &meta_path,
        &format!(
            r#"{{
  "skillId": "rust-helper",
  "skillName": "Rust Helper",
  "description": "Helps with Rust code",
  "group": "engineering",
  "groupDescription": "Engineering skills",
  "keywords": [
    "helps",
    "with",
    "rust",
    "code"
  ],
  "updatedAtMs": {},
  "status": "ready"
}}
"#,
            ts_style_mtime
        ),
    );
    let before = fs::read_to_string(&meta_path).unwrap();

    let layout = resolve_storage_layout(&shelf);
    let mut registry = SkillRegistry::new(layout);
    registry.rebuild().unwrap();

    let after = fs::read_to_string(&meta_path).unwrap();
    assert_eq!(after, before);
}

#[test]
fn rebuild_loads_grouped_skills_and_persists_indexes() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let layout = resolve_storage_layout(&shelf);
    let mut registry = SkillRegistry::new(layout);
    registry.rebuild().unwrap();

    assert_eq!(registry.size(), 1);
    assert!(shelf.join("index/group-list.json").exists());
    assert!(shelf.join("index/groups/engineering.json").exists());
    assert!(shelf
        .join("index/skills/engineering--rust-helper.json")
        .exists());
    assert!(shelf
        .join("packages/engineering/rust-helper/meta.json")
        .exists());
}

#[test]
fn rebuild_twice_overwrites_existing_indexes_and_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let layout = resolve_storage_layout(&shelf);
    let mut registry = SkillRegistry::new(layout.clone());
    registry.rebuild().unwrap();

    fs::write(&layout.group_list_path, b"stale group list").unwrap();
    fs::remove_file(layout.groups_root.join("engineering.json")).unwrap();
    fs::create_dir(layout.groups_root.join("engineering.json")).unwrap();
    fs::write(
        layout
            .groups_root
            .join("engineering.json")
            .join("leftover.json"),
        b"stale group index",
    )
    .unwrap();
    fs::remove_file(layout.skills_root.join("engineering--rust-helper.json")).unwrap();
    fs::create_dir(layout.skills_root.join("engineering--rust-helper.json")).unwrap();
    fs::write(
        layout
            .skills_root
            .join("engineering--rust-helper.json")
            .join("leftover.json"),
        b"stale skill index",
    )
    .unwrap();
    fs::write(
        shelf.join("packages/engineering/rust-helper/meta.json"),
        b"stale metadata",
    )
    .unwrap();

    registry.rebuild().unwrap();

    let groups: Vec<GroupListItem> =
        serde_json::from_slice(&fs::read(&layout.group_list_path).unwrap()).unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group, "engineering");

    let group_index: GroupSkillsResult =
        serde_json::from_slice(&fs::read(layout.groups_root.join("engineering.json")).unwrap())
            .unwrap();
    assert_eq!(group_index.group, "engineering");
    assert_eq!(group_index.skills.len(), 1);
    assert_eq!(group_index.skills[0].skill_id, "rust-helper");

    let skill_index: SkillRecord = serde_json::from_slice(
        &fs::read(layout.skills_root.join("engineering--rust-helper.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(skill_index.skill_id, "rust-helper");

    let meta: SkillMeta = serde_json::from_slice(
        &fs::read(shelf.join("packages/engineering/rust-helper/meta.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(meta.skill_id, "rust-helper");
}

#[test]
fn rebuild_replaces_index_contents_and_drops_obsolete_json() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let layout = resolve_storage_layout(&shelf);
    fs::create_dir_all(&layout.groups_root).unwrap();
    fs::create_dir_all(&layout.skills_root).unwrap();
    fs::write(
        &layout.group_list_path,
        serde_json::to_vec_pretty(&vec![GroupListItem {
            group: "obsolete".into(),
            group_description: "Obsolete".into(),
        }])
        .unwrap(),
    )
    .unwrap();
    fs::write(
        layout.groups_root.join("obsolete.json"),
        b"{\"group\":\"obsolete\"}",
    )
    .unwrap();
    fs::write(
        layout.skills_root.join("obsolete--old.json"),
        b"{\"skill_id\":\"old\"}",
    )
    .unwrap();

    let mut registry = SkillRegistry::new(layout.clone());
    registry.rebuild().unwrap();

    assert!(!layout.groups_root.join("obsolete.json").exists());
    assert!(!layout.skills_root.join("obsolete--old.json").exists());

    let groups: Vec<GroupListItem> =
        serde_json::from_slice(&fs::read(&layout.group_list_path).unwrap()).unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group, "engineering");

    let group_files: Vec<String> = fs::read_dir(&layout.groups_root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(group_files, vec!["engineering.json"]);

    let skill_files: Vec<String> = fs::read_dir(&layout.skills_root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(skill_files, vec!["engineering--rust-helper.json"]);
}

#[test]
fn registry_resolves_by_composite_id_bare_id_and_exact_name() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let layout = resolve_storage_layout(&shelf);
    let mut registry = SkillRegistry::new(layout);
    registry.rebuild().unwrap();

    let by_composite = registry.get_by_id("engineering/rust-helper").unwrap();
    let by_bare = registry.get_by_id("rust-helper").unwrap();
    let by_name = registry.get_by_name("Rust Helper").unwrap();

    assert_eq!(by_composite.skill_id, "rust-helper");
    assert_eq!(by_bare.skill_id, "rust-helper");
    assert_eq!(by_name.skill_id, "rust-helper");
}

#[test]
fn registry_lists_related_skills_in_name_order_with_limit() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    write_file(
        shelf.join("packages/engineering/alpha-helper/SKILL.md"),
        r#"---
name: Alpha Helper
description: First related skill
---
"#,
    );
    write_file(
        shelf.join("packages/engineering/zulu-helper/SKILL.md"),
        r#"---
name: Zulu Helper
description: Last related skill
---
"#,
    );

    let layout = resolve_storage_layout(&shelf);
    let mut registry = SkillRegistry::new(layout);
    registry.rebuild().unwrap();

    let related = registry.list_related_skills("rust-helper", 1);

    assert_eq!(related.len(), 1);
    assert_eq!(related[0].skill_name, "Alpha Helper");
}

#[test]
fn registry_exposes_rebuild_issues_and_managed_groups() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_shelf");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    write_file(
        shelf.join("packages/engineering/invalid-helper/SKILL.md"),
        r#"---
name Invalid Helper
description: broken frontmatter
"#,
    );

    let layout = resolve_storage_layout(&shelf);
    let mut registry = SkillRegistry::new(layout);
    registry.rebuild().unwrap();

    let issues: Vec<RegistryIssue> = registry.list_issues();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].path.contains("invalid-helper"));

    let groups = registry.list_groups();
    assert_eq!(
        groups,
        vec![GroupListItem {
            group: "engineering".into(),
            group_description: "Engineering skills".into(),
        }]
    );
}

#[test]
fn nonexistent_relative_shelf_root_resolves_absolute() {
    let layout = resolve_storage_layout("tests/fixtures/does-not-exist");

    assert!(layout.shelf_root.is_absolute());
    assert!(layout.config_root.is_absolute());
    assert_eq!(
        layout.shelf_root.file_name().and_then(|name| name.to_str()),
        Some("does-not-exist")
    );
}

#[test]
fn rebuild_ungrouped_weak_signal_skill_falls_back_to_specialized_domain() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/rebuild_ungrouped_default_group");
    let shelf = temp.path().join("hub");
    copy_dir_all(fixture, &shelf);

    let layout = resolve_storage_layout(&shelf);
    let mut registry = SkillRegistry::new(layout);
    registry.rebuild().unwrap();

    let records = registry.list_skill_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].group, "specialized-domain");
    assert_eq!(records[0].group_description, "Specialized domain skills");
    assert!(shelf.join("index/groups/specialized-domain.json").exists());
}

#[test]
fn search_prefers_name_and_description_matches() {
    let records = vec![
        skill_shelf::model::SkillRecord {
            skill_id: "rust-helper".into(),
            skill_name: "Rust Helper".into(),
            description: "Helps with Rust code".into(),
            group: "engineering".into(),
            group_description: "Engineering skills".into(),
            keywords: vec!["rust".into(), "code".into()],
            skill_path: "unused".into(),
            updated_at_ms: 1.0,
            status: "ready".into(),
        },
        skill_shelf::model::SkillRecord {
            skill_id: "visual-designer".into(),
            skill_name: "Visual Designer".into(),
            description: "Creates layouts".into(),
            group: "design".into(),
            group_description: "Design skills".into(),
            keywords: vec!["ui".into()],
            skill_path: "unused".into(),
            updated_at_ms: 1.0,
            status: "ready".into(),
        },
    ];

    let results = search_skills(&records, "rust code", 8);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].skill_id, "rust-helper");
    assert!(results[0].score > 0);
}

#[test]
fn no_match_query_returns_empty_results() {
    let records = vec![skill_shelf::model::SkillRecord {
        skill_id: "rust-helper".into(),
        skill_name: "Rust Helper".into(),
        description: "Helps with Rust code".into(),
        group: "engineering".into(),
        group_description: "Engineering skills".into(),
        keywords: vec!["rust".into(), "code".into()],
        skill_path: "unused".into(),
        updated_at_ms: 1.0,
        status: "ready".into(),
    }];

    let results = search_skills(&records, "totally unrelated astronomy", 8);

    assert!(results.is_empty());
}

#[test]
fn watcher_status_and_shelf_status_serde_roundtrip() {
    let status = ShelfStatus {
        groups_count: 3,
        skills_count: 12,
        import_count: 1,
        index_updated_at: Some(42),
        watcher_status: WatcherStatus {
            running: true,
            last_event_at_ms: Some(10),
            last_error: None,
        },
        issue_count: 2,
    };

    let json = serde_json::to_string(&status).unwrap();
    let decoded: ShelfStatus = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded, status);
}
