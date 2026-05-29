use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillShelfStorageLayout {
    pub shelf_root: PathBuf,
    pub config_root: PathBuf,
    pub group_catalog_path: PathBuf,
    pub packages_root: PathBuf,
    pub index_root: PathBuf,
    pub group_list_path: PathBuf,
    pub groups_root: PathBuf,
    pub skills_root: PathBuf,
    pub staging_root: PathBuf,
    pub staging_imports_root: PathBuf,
    pub staging_repaired_root: PathBuf,
    pub logs_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPolicy {
    pub accept_package_directories: bool,
    pub accept_raw_markdown: bool,
    pub package_precedence: String,
    pub raw_markdown_requires_frontmatter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexPolicy {
    pub default_search_result_limit: usize,
    pub max_keywords_per_skill: usize,
    pub max_related_skills: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchPolicy {
    pub enabled: bool,
    pub use_polling: bool,
    pub polling_interval_ms: u64,
    pub await_write_stability_ms: u64,
    pub await_write_poll_ms: u64,
    pub sync_delete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillShelfRuntimeConfig {
    pub storage: SkillShelfStorageLayout,
    pub install_policy: InstallPolicy,
    pub index_policy: IndexPolicy,
    pub watch_policy: WatchPolicy,
}

pub fn load_config() -> SkillShelfRuntimeConfig {
    load_config_from(|name| std::env::var_os(name))
}

pub fn load_config_from<F>(mut lookup: F) -> SkillShelfRuntimeConfig
where
    F: FnMut(&str) -> Option<OsString>,
{
    let shelf_root =
        lookup_path(&mut lookup, "SKILL_SHELF_ROOT").unwrap_or_else(default_shelf_root);
    let storage = resolve_storage_layout(&shelf_root);
    SkillShelfRuntimeConfig {
        storage,
        install_policy: InstallPolicy {
            accept_package_directories: lookup_bool(
                &mut lookup,
                "SKILL_SHELF_ACCEPT_PACKAGES",
                true,
            ),
            accept_raw_markdown: lookup_bool(&mut lookup, "SKILL_SHELF_ACCEPT_RAW_MARKDOWN", true),
            package_precedence: "package-first".to_string(),
            raw_markdown_requires_frontmatter: lookup_bool(
                &mut lookup,
                "SKILL_SHELF_RAW_REQUIRES_FRONTMATTER",
                true,
            ),
        },
        index_policy: IndexPolicy {
            default_search_result_limit: lookup_usize(&mut lookup, "SKILL_SHELF_SEARCH_LIMIT", 8),
            max_keywords_per_skill: lookup_usize(&mut lookup, "SKILL_SHELF_MAX_KEYWORDS", 12),
            max_related_skills: lookup_usize(&mut lookup, "SKILL_SHELF_MAX_RELATED_SKILLS", 5),
        },
        watch_policy: WatchPolicy {
            enabled: lookup_bool(&mut lookup, "SKILL_SHELF_WATCH", true),
            use_polling: lookup_bool(&mut lookup, "SKILL_SHELF_WATCH_USE_POLLING", true),
            polling_interval_ms: lookup_u64(&mut lookup, "SKILL_SHELF_WATCH_INTERVAL_MS", 100),
            await_write_stability_ms: lookup_u64(
                &mut lookup,
                "SKILL_SHELF_WATCH_STABILITY_MS",
                300,
            ),
            await_write_poll_ms: lookup_u64(&mut lookup, "SKILL_SHELF_WATCH_POLL_MS", 50),
            sync_delete: lookup_bool(&mut lookup, "SKILL_SHELF_WATCH_SYNC_DELETE", true),
        },
    }
}

pub fn default_install_policy() -> InstallPolicy {
    InstallPolicy {
        accept_package_directories: true,
        accept_raw_markdown: true,
        package_precedence: "package-first".to_string(),
        raw_markdown_requires_frontmatter: true,
    }
}

pub fn default_index_policy() -> IndexPolicy {
    IndexPolicy {
        default_search_result_limit: 8,
        max_keywords_per_skill: 12,
        max_related_skills: 5,
    }
}

pub fn default_watch_policy() -> WatchPolicy {
    WatchPolicy {
        enabled: true,
        use_polling: true,
        polling_interval_ms: 100,
        await_write_stability_ms: 300,
        await_write_poll_ms: 50,
        sync_delete: true,
    }
}

pub fn resolve_storage_layout(shelf_root: impl AsRef<Path>) -> SkillShelfStorageLayout {
    let shelf_root = shelf_root.as_ref();
    let shelf_root = shelf_root.canonicalize().unwrap_or_else(|_| {
        if shelf_root.is_absolute() {
            shelf_root.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(shelf_root)
        }
    });
    let config_root = shelf_root.join("config");
    let index_root = shelf_root.join("index");
    let staging_root = shelf_root.join("staging");

    SkillShelfStorageLayout {
        shelf_root: shelf_root.clone(),
        config_root: config_root.clone(),
        group_catalog_path: config_root.join("groups.json"),
        packages_root: shelf_root.join("packages"),
        index_root: index_root.clone(),
        group_list_path: index_root.join("group-list.json"),
        groups_root: index_root.join("groups"),
        skills_root: index_root.join("skills"),
        staging_root: staging_root.clone(),
        staging_imports_root: staging_root.join("imports"),
        staging_repaired_root: staging_root.join("repaired"),
        logs_root: shelf_root.join("logs"),
    }
}

fn default_shelf_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
        .join("hub")
}

fn lookup_path<F>(lookup: &mut F, name: &str) -> Option<PathBuf>
where
    F: FnMut(&str) -> Option<OsString>,
{
    lookup(name).map(PathBuf::from)
}

fn lookup_bool<F>(lookup: &mut F, name: &str, default: bool) -> bool
where
    F: FnMut(&str) -> Option<OsString>,
{
    lookup(name)
        .map(|value| value.to_string_lossy() != "0")
        .unwrap_or(default)
}

fn lookup_usize<F>(lookup: &mut F, name: &str, default: usize) -> usize
where
    F: FnMut(&str) -> Option<OsString>,
{
    lookup(name)
        .and_then(|value| value.to_string_lossy().parse::<usize>().ok())
        .unwrap_or(default)
}

fn lookup_u64<F>(lookup: &mut F, name: &str, default: u64) -> u64
where
    F: FnMut(&str) -> Option<OsString>,
{
    lookup(name)
        .and_then(|value| value.to_string_lossy().parse::<u64>().ok())
        .unwrap_or(default)
}
