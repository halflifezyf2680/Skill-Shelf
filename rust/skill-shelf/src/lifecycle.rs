use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use sysinfo::{Pid, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownReason {
    StdinEof,
    Signal,
    ParentDeath,
}

pub fn process_exists(pid: u32) -> bool {
    process_start_time(pid).is_some()
}

pub fn current_parent_pid() -> Option<u32> {
    let system = System::new_all();
    let current_pid = Pid::from_u32(std::process::id());
    system
        .process(current_pid)
        .and_then(|process| process.parent())
        .map(Pid::as_u32)
}

fn process_start_time(pid: u32) -> Option<u64> {
    let system = System::new_all();
    system
        .process(Pid::from_u32(pid))
        .map(|process| process.start_time())
}

#[derive(Clone)]
pub struct EofNotifier {
    tx: mpsc::Sender<ShutdownReason>,
}

impl EofNotifier {
    pub fn notify_read(&self, bytes_read: usize) {
        if bytes_read == 0 {
            self.notify_eof();
        }
    }

    pub fn notify_eof(&self) {
        let _ = self.tx.send(ShutdownReason::StdinEof);
    }
}

pub struct StdinEofWatcher {
    rx: mpsc::Receiver<ShutdownReason>,
}

impl StdinEofWatcher {
    pub fn new() -> (Self, EofNotifier) {
        let (tx, rx) = mpsc::channel();
        (Self { rx }, EofNotifier { tx })
    }

    pub fn poll(&self) -> Option<ShutdownReason> {
        self.rx.try_recv().ok()
    }
}

pub struct SignalWatcher {
    flag: Arc<AtomicBool>,
}

impl SignalWatcher {
    pub fn new(flag: Arc<AtomicBool>) -> Self {
        Self { flag }
    }

    pub fn trigger(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn poll(&self) -> Option<ShutdownReason> {
        if self.flag.swap(false, Ordering::SeqCst) {
            Some(ShutdownReason::Signal)
        } else {
            None
        }
    }
}

pub struct ParentDeathWatcher {
    parent_pid: u32,
    expected_start_time: Option<u64>,
}

impl ParentDeathWatcher {
    pub fn new(parent_pid: u32) -> Self {
        Self {
            parent_pid,
            expected_start_time: process_start_time(parent_pid),
        }
    }

    pub fn with_expected_start_time(parent_pid: u32, expected_start_time: u64) -> Self {
        Self {
            parent_pid,
            expected_start_time: Some(expected_start_time),
        }
    }

    pub fn is_parent_alive(&self) -> bool {
        match (
            self.expected_start_time,
            process_start_time(self.parent_pid),
        ) {
            (Some(expected_start_time), Some(actual_start_time)) => {
                actual_start_time == expected_start_time
            }
            _ => false,
        }
    }

    pub fn poll(&self) -> Option<ShutdownReason> {
        if self.is_parent_alive() {
            None
        } else {
            Some(ShutdownReason::ParentDeath)
        }
    }
}

pub struct ShutdownPoller<'a> {
    stdin_eof: Option<&'a StdinEofWatcher>,
    parent_death: Option<&'a ParentDeathWatcher>,
}

impl<'a> ShutdownPoller<'a> {
    pub fn new(
        stdin_eof: Option<&'a StdinEofWatcher>,
        parent_death: Option<&'a ParentDeathWatcher>,
    ) -> Self {
        Self {
            stdin_eof,
            parent_death,
        }
    }

    pub fn poll(&self) -> Option<ShutdownReason> {
        self.stdin_eof
            .and_then(StdinEofWatcher::poll)
            .or_else(|| self.parent_death.and_then(ParentDeathWatcher::poll))
    }
}
