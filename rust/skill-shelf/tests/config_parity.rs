use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use skill_shelf::config::load_config_from;

#[test]
fn load_config_reads_supported_env_overrides() {
    let root = PathBuf::from(r"C:\SkillShelf\custom-root");
    let mut env = HashMap::new();
    env.insert(
        "SKILL_SHELF_ROOT".to_string(),
        root.to_string_lossy().into_owned(),
    );
    env.insert("SKILL_SHELF_ACCEPT_PACKAGES".to_string(), "0".to_string());
    env.insert(
        "SKILL_SHELF_ACCEPT_RAW_MARKDOWN".to_string(),
        "0".to_string(),
    );
    env.insert(
        "SKILL_SHELF_RAW_REQUIRES_FRONTMATTER".to_string(),
        "0".to_string(),
    );
    env.insert("SKILL_SHELF_SEARCH_LIMIT".to_string(), "11".to_string());
    env.insert("SKILL_SHELF_MAX_KEYWORDS".to_string(), "17".to_string());
    env.insert(
        "SKILL_SHELF_MAX_RELATED_SKILLS".to_string(),
        "9".to_string(),
    );
    env.insert("SKILL_SHELF_WATCH".to_string(), "0".to_string());
    env.insert("SKILL_SHELF_WATCH_USE_POLLING".to_string(), "0".to_string());
    env.insert(
        "SKILL_SHELF_WATCH_INTERVAL_MS".to_string(),
        "250".to_string(),
    );
    env.insert(
        "SKILL_SHELF_WATCH_STABILITY_MS".to_string(),
        "600".to_string(),
    );
    env.insert("SKILL_SHELF_WATCH_POLL_MS".to_string(), "75".to_string());
    env.insert("SKILL_SHELF_WATCH_SYNC_DELETE".to_string(), "0".to_string());

    let config = load_config_from(|name| env.get(name).map(|value| OsString::from(value)));

    assert_eq!(config.storage.shelf_root, root);
    assert!(!config.install_policy.accept_package_directories);
    assert!(!config.install_policy.accept_raw_markdown);
    assert_eq!(config.install_policy.package_precedence, "package-first");
    assert!(!config.install_policy.raw_markdown_requires_frontmatter);
    assert_eq!(config.index_policy.default_search_result_limit, 11);
    assert_eq!(config.index_policy.max_keywords_per_skill, 17);
    assert_eq!(config.index_policy.max_related_skills, 9);
    assert!(!config.watch_policy.enabled);
    assert!(!config.watch_policy.use_polling);
    assert_eq!(config.watch_policy.polling_interval_ms, 250);
    assert_eq!(config.watch_policy.await_write_stability_ms, 600);
    assert_eq!(config.watch_policy.await_write_poll_ms, 75);
    assert!(!config.watch_policy.sync_delete);
}
