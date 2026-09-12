//! Native recursive workspace file watching.

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::{
    io,
    path::Path,
    sync::{Mutex, mpsc},
    time::Duration,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEvent {
    pub kind: String,
    pub paths: Vec<String>,
}

pub struct WorkspaceWatcher {
    _watcher: RecommendedWatcher,
    receiver: Mutex<mpsc::Receiver<notify::Result<Event>>>,
}

impl WorkspaceWatcher {
    /// Starts the operating system's recursive watcher for a workspace.
    ///
    /// # Errors
    /// Returns an error when the native watcher cannot be created or attached.
    pub fn start(root: &Path) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = sender.send(event);
        })
        .map_err(io::Error::other)?;
        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(io::Error::other)?;
        Ok(Self {
            _watcher: watcher,
            receiver: Mutex::new(receiver),
        })
    }

    /// Waits briefly for native changes and drains the currently queued batch.
    ///
    /// # Errors
    /// Returns an error when the receiver is unavailable or the native backend reports failure.
    pub fn changes(&self, timeout: Duration) -> io::Result<Vec<WorkspaceEvent>> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|_| io::Error::other("watcher lock unavailable"))?;
        let mut events = Vec::new();
        match receiver.recv_timeout(timeout) {
            Ok(event) => events.push(event.map_err(io::Error::other)?),
            Err(mpsc::RecvTimeoutError::Timeout) => return Ok(vec![]),
            Err(error) => return Err(io::Error::other(error)),
        }
        while let Ok(event) = receiver.try_recv() {
            events.push(event.map_err(io::Error::other)?);
        }
        Ok(events
            .into_iter()
            .map(|event| WorkspaceEvent {
                kind: format!("{:?}", event.kind),
                paths: event
                    .paths
                    .into_iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn starts_and_polls_native_backend() {
        let d = tempfile::tempdir().unwrap();
        let watcher = WorkspaceWatcher::start(d.path()).unwrap();
        fs::write(d.path().join("changed.txt"), "value").unwrap();
        let _events = watcher.changes(Duration::from_millis(100)).unwrap();
    }
}
