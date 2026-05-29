use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tokio::sync::Mutex;

use crate::model::{SkillRecord, WatcherStatus};

#[derive(Clone, Debug, Eq)]
pub struct WorkspaceKey {
    pub shelf_root: PathBuf,
    pub config_hash: String,
}

impl WorkspaceKey {
    pub fn new(shelf_root: PathBuf, config_hash: String) -> Self {
        Self {
            shelf_root,
            config_hash,
        }
    }
}

impl PartialEq for WorkspaceKey {
    fn eq(&self, other: &Self) -> bool {
        self.shelf_root == other.shelf_root && self.config_hash == other.config_hash
    }
}

impl Hash for WorkspaceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.shelf_root.hash(state);
        self.config_hash.hash(state);
    }
}

#[derive(Debug)]
pub struct WorkspaceHandle {
    id: u64,
    key: WorkspaceKey,
    snapshot: RwLock<Arc<WorkspaceSnapshot>>,
    write_gate: Mutex<()>,
    last_touched: StdMutex<Instant>,
    busy_count: AtomicUsize,
    session_count: AtomicUsize,
    watcher: WorkspaceWatcher,
}

#[derive(Clone, Debug, Default)]
pub struct WorkspaceSnapshot {
    pub loaded: bool,
    pub records: Vec<SkillRecord>,
    pub read_models: HashMap<String, Value>,
    pub groups_count: usize,
    pub skills_count: usize,
    pub import_count: usize,
    pub index_updated_at: Option<u64>,
    pub issue_count: usize,
}

#[derive(Debug)]
struct WorkspaceBusyGuard<'a> {
    handle: &'a WorkspaceHandle,
}

impl<'a> WorkspaceBusyGuard<'a> {
    fn new(handle: &'a WorkspaceHandle) -> Self {
        handle.busy_count.fetch_add(1, Ordering::SeqCst);
        Self { handle }
    }
}

impl Drop for WorkspaceBusyGuard<'_> {
    fn drop(&mut self) {
        self.handle.busy_count.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct WorkspaceSessionGuard<'a> {
    handle: &'a WorkspaceHandle,
}

impl Drop for WorkspaceSessionGuard<'_> {
    fn drop(&mut self) {
        self.handle.session_count.fetch_sub(1, Ordering::SeqCst);
    }
}

impl WorkspaceHandle {
    fn new(id: u64, key: WorkspaceKey) -> Self {
        let packages_root = key.shelf_root.join("packages");
        Self {
            id,
            key,
            snapshot: RwLock::new(Arc::new(WorkspaceSnapshot::default())),
            write_gate: Mutex::new(()),
            last_touched: StdMutex::new(Instant::now()),
            busy_count: AtomicUsize::new(0),
            session_count: AtomicUsize::new(0),
            watcher: WorkspaceWatcher::new(packages_root),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn key(&self) -> &WorkspaceKey {
        &self.key
    }

    pub fn read_snapshot(&self) -> Arc<Vec<SkillRecord>> {
        self.snapshot
            .read()
            .expect("workspace snapshot poisoned")
            .records
            .clone()
            .into()
    }

    pub fn replace_snapshot(&self, records: Vec<SkillRecord>) {
        let mut snapshot = self.snapshot.write().expect("workspace snapshot poisoned");
        let mut next = (**snapshot).clone();
        next.loaded = true;
        next.skills_count = records.len();
        next.records = records;
        *snapshot = Arc::new(next);
    }

    pub fn read_state(&self) -> Arc<WorkspaceSnapshot> {
        self.snapshot
            .read()
            .expect("workspace snapshot poisoned")
            .clone()
    }

    pub fn replace_state(&self, state: WorkspaceSnapshot) {
        *self.snapshot.write().expect("workspace snapshot poisoned") = Arc::new(state);
    }

    pub async fn with_write<F, Fut, T>(&self, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let _guard = self.write_gate.lock().await;
        let _busy = WorkspaceBusyGuard::new(self);
        self.touch();
        let result = operation().await;
        self.touch();
        result
    }

    pub fn attach_session(&self) -> WorkspaceSessionGuard<'_> {
        self.session_count.fetch_add(1, Ordering::SeqCst);
        self.touch();
        WorkspaceSessionGuard { handle: self }
    }

    pub fn mark_used(&self) {
        self.touch();
    }

    pub fn poll_watcher_dirty(&self) -> bool {
        self.watcher.poll_dirty()
    }

    pub fn mark_watcher_clean(&self) {
        self.watcher.mark_clean();
    }

    pub fn watcher_status(&self) -> WatcherStatus {
        self.watcher.status()
    }

    fn idle_for(&self) -> Duration {
        Instant::now().saturating_duration_since(
            *self
                .last_touched
                .lock()
                .expect("workspace last_touched poisoned"),
        )
    }

    fn is_removable(&self, max_idle: Duration) -> bool {
        self.busy_count.load(Ordering::SeqCst) == 0
            && self.session_count.load(Ordering::SeqCst) == 0
            && self.idle_for() >= max_idle
    }

    fn touch(&self) {
        *self
            .last_touched
            .lock()
            .expect("workspace last_touched poisoned") = Instant::now();
    }
}

#[derive(Debug)]
struct WorkspaceWatcher {
    packages_root: PathBuf,
    state: StdMutex<WorkspaceWatcherState>,
}

#[derive(Debug, Default)]
struct WorkspaceWatcherState {
    running: bool,
    last_signature: Option<SkillTreeSignature>,
    dirty: bool,
    last_event_at_ms: Option<u64>,
    last_error: Option<String>,
}

type SkillTreeSignature = BTreeMap<String, SkillFileSignature>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SkillFileSignature {
    len: u64,
    modified_at_ms: Option<u64>,
}

impl WorkspaceWatcher {
    fn new(packages_root: PathBuf) -> Self {
        Self {
            packages_root,
            state: StdMutex::new(WorkspaceWatcherState {
                running: true,
                ..WorkspaceWatcherState::default()
            }),
        }
    }

    fn poll_dirty(&self) -> bool {
        match scan_skill_tree(&self.packages_root) {
            Ok(signature) => {
                let mut state = self.state.lock().expect("workspace watcher poisoned");
                state.last_error = None;
                match &state.last_signature {
                    Some(previous) if previous == &signature => state.dirty,
                    Some(_) => {
                        state.last_signature = Some(signature);
                        state.dirty = true;
                        state.last_event_at_ms = Some(unix_time_ms());
                        true
                    }
                    None => {
                        state.last_signature = Some(signature);
                        state.dirty = false;
                        false
                    }
                }
            }
            Err(error) => {
                let mut state = self.state.lock().expect("workspace watcher poisoned");
                state.last_error = Some(error);
                state.dirty
            }
        }
    }

    fn mark_clean(&self) {
        match scan_skill_tree(&self.packages_root) {
            Ok(signature) => {
                let mut state = self.state.lock().expect("workspace watcher poisoned");
                state.last_signature = Some(signature);
                state.dirty = false;
                state.last_error = None;
            }
            Err(error) => {
                let mut state = self.state.lock().expect("workspace watcher poisoned");
                state.last_error = Some(error);
            }
        }
    }

    fn status(&self) -> WatcherStatus {
        let state = self.state.lock().expect("workspace watcher poisoned");
        WatcherStatus {
            running: state.running,
            last_event_at_ms: state.last_event_at_ms,
            last_error: state.last_error.clone(),
        }
    }
}

impl Drop for WorkspaceWatcher {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.running = false;
        }
    }
}

fn scan_skill_tree(root: &Path) -> Result<SkillTreeSignature, String> {
    let mut signature = BTreeMap::new();
    if !root.exists() {
        return Ok(signature);
    }
    scan_skill_tree_inner(root, root, &mut signature)?;
    Ok(signature)
}

fn scan_skill_tree_inner(
    base: &Path,
    current: &Path,
    signature: &mut SkillTreeSignature,
) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|error| format!("failed to read {}: {error}", current.display()))?;
    for entry in entries {
        let entry = entry
            .map_err(|error| format!("failed to read {} entry: {error}", current.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to stat {}: {error}", path.display()))?;
        if file_type.is_dir() {
            scan_skill_tree_inner(base, &path, signature)?;
        } else if file_type.is_file() && entry.file_name() == "SKILL.md" {
            let metadata = entry
                .metadata()
                .map_err(|error| format!("failed to stat {}: {error}", path.display()))?;
            let relative_path = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            signature.insert(
                relative_path,
                SkillFileSignature {
                    len: metadata.len(),
                    modified_at_ms: metadata
                        .modified()
                        .ok()
                        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                        .map(|duration| duration.as_millis() as u64),
                },
            );
        }
    }
    Ok(())
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug)]
pub struct WorkspaceManager {
    state: Mutex<WorkspaceManagerState>,
}

#[derive(Debug)]
struct WorkspaceManagerState {
    next_workspace_id: u64,
    workspaces: HashMap<WorkspaceKey, Arc<WorkspaceHandle>>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(WorkspaceManagerState {
                next_workspace_id: 1,
                workspaces: HashMap::new(),
            }),
        }
    }

    pub async fn get_or_create(&self, key: WorkspaceKey) -> Arc<WorkspaceHandle> {
        let mut state = self.state.lock().await;
        if let Some(existing) = state.workspaces.get(&key) {
            existing.touch();
            return Arc::clone(existing);
        }

        let workspace = Arc::new(WorkspaceHandle::new(state.next_workspace_id, key.clone()));
        state.next_workspace_id += 1;
        state.workspaces.insert(key, Arc::clone(&workspace));
        workspace
    }

    pub async fn unload(&self, key: &WorkspaceKey) -> bool {
        self.state.lock().await.workspaces.remove(key).is_some()
    }

    pub async fn remove_idle(&self, max_idle: Duration) -> usize {
        let snapshot: Vec<(WorkspaceKey, Arc<WorkspaceHandle>)> = {
            let state = self.state.lock().await;
            state
                .workspaces
                .iter()
                .map(|(key, handle)| (key.clone(), Arc::clone(handle)))
                .collect()
        };

        let mut idle_keys = Vec::new();
        for (key, handle) in snapshot {
            if handle.is_removable(max_idle) {
                idle_keys.push(key);
            }
        }

        let mut state = self.state.lock().await;
        let mut removed = 0;
        for key in idle_keys {
            let should_remove = state
                .workspaces
                .get(&key)
                .is_some_and(|handle| handle.is_removable(max_idle));
            if should_remove && state.workspaces.remove(&key).is_some() {
                removed += 1;
            }
        }
        removed
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}
