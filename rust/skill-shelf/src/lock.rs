use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use fs2::FileExt;

#[derive(Debug)]
pub enum LockError {
    AlreadyRunning,
    Io(std::io::Error),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "daemon is already running"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for LockError {}

impl From<std::io::Error> for LockError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug)]
pub struct DaemonLock {
    file: File,
    path: PathBuf,
}

impl Drop for DaemonLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        if let Ok(mut held) = held_paths().lock() {
            held.remove(&self.path);
        }
    }
}

pub fn acquire_daemon_lock(path: &Path) -> std::result::Result<DaemonLock, LockError> {
    let canonical_path = normalize_lock_path(path)?;
    {
        let mut held = held_paths().lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "lock registry poisoned")
        })?;
        if held.contains(&canonical_path) {
            return Err(LockError::AlreadyRunning);
        }
        held.insert(canonical_path.clone());
    }

    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            release_held_path(&canonical_path);
            return Err(LockError::Io(err));
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|err| {
            release_held_path(&canonical_path);
            LockError::Io(err)
        })?;

    if let Err(err) = file.try_lock_exclusive() {
        release_held_path(&canonical_path);
        let _ = err;
        return Err(LockError::AlreadyRunning);
    }

    if let Err(err) = file.set_len(0) {
        release_held_path(&canonical_path);
        return Err(LockError::Io(err));
    }
    if let Err(err) = writeln!(file, "pid={}", std::process::id()) {
        release_held_path(&canonical_path);
        return Err(LockError::Io(err));
    }
    if let Err(err) = file.flush() {
        release_held_path(&canonical_path);
        return Err(LockError::Io(err));
    }

    Ok(DaemonLock {
        file,
        path: canonical_path,
    })
}

fn held_paths() -> &'static Mutex<std::collections::HashSet<PathBuf>> {
    static HELD_PATHS: OnceLock<Mutex<std::collections::HashSet<PathBuf>>> = OnceLock::new();
    HELD_PATHS.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

fn normalize_lock_path(path: &Path) -> std::io::Result<PathBuf> {
    match path.canonicalize() {
        Ok(canonical) => Ok(canonical),
        Err(_) => {
            let base = std::env::current_dir()?;
            Ok(if path.is_absolute() {
                path.to_path_buf()
            } else {
                base.join(path)
            })
        }
    }
}

fn release_held_path(path: &Path) {
    if let Ok(mut held) = held_paths().lock() {
        held.remove(path);
    }
}
