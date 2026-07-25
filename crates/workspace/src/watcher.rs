use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Clone, Serialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeEvent {
    pub kind: String, // "create" | "modify" | "remove" | "other" | "error"
    pub paths: Vec<String>,
}

impl From<Event> for FileChangeEvent {
    fn from(event: Event) -> Self {
        let kind = match event.kind {
            EventKind::Create(_) => "create",
            EventKind::Modify(_) => "modify",
            EventKind::Remove(_) => "remove",
            _ => "other",
        }
        .to_string();
        let paths = event
            .paths
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        FileChangeEvent { kind, paths }
    }
}

/// Starts a recursive native watch on `root`, invoking `on_event` for every
/// filesystem event `notify` reports. This is the part unit-tested below;
/// `start_watching` just plugs in a `FileChangeEvent` callback (the RPC
/// server wires that to a stdout notification — see `main.rs`).
fn watch<F>(root: &Path, on_event: F) -> notify::Result<RecommendedWatcher>
where
    F: Fn(notify::Result<Event>) + Send + 'static,
{
    let mut watcher = notify::recommended_watcher(on_event)?;
    watcher.watch(root, RecursiveMode::Recursive)?;
    Ok(watcher)
}

/// Holds the single active watcher, if any. Replacing or clearing this
/// (see `stop_watching`) drops the previous `RecommendedWatcher`, which is
/// how `notify` stops watching — there's no separate "unwatch" call needed.
#[derive(Default)]
pub struct WatcherState(Mutex<Option<RecommendedWatcher>>);

pub fn start_watching(
    state: &WatcherState,
    path: &str,
    on_event: impl Fn(FileChangeEvent) + Send + 'static,
) -> Result<(), String> {
    let root = PathBuf::from(path);
    if !root.exists() {
        return Err(format!("path does not exist: {path}"));
    }

    let watcher = watch(&root, move |res| {
        let payload = match res {
            Ok(event) => FileChangeEvent::from(event),
            Err(e) => FileChangeEvent {
                kind: "error".to_string(),
                paths: vec![e.to_string()],
            },
        };
        on_event(payload);
    })
    .map_err(|e| e.to_string())?;

    let mut guard = state
        .0
        .lock()
        .map_err(|_| "watcher state poisoned".to_string())?;
    *guard = Some(watcher);
    Ok(())
}

pub fn stop_watching(state: &WatcherState) -> Result<(), String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "watcher state poisoned".to_string())?;
    *guard = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, Receiver};
    use std::time::{Duration, Instant};

    /// Native file-watch backends (FSEvents/inotify/ReadDirectoryChangesW)
    /// don't guarantee the first event delivered is the one a test
    /// triggered — spurious directory-metadata events or coalescing can
    /// arrive first. Drain events until one matches or the deadline passes,
    /// rather than asserting on a single `recv`.
    fn wait_for_event_matching(
        rx: &Receiver<Event>,
        predicate: impl Fn(&Event) -> bool,
        timeout: Duration,
    ) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            match rx.recv_timeout(remaining) {
                Ok(event) if predicate(&event) => return true,
                Ok(_) => continue,
                Err(_) => return false,
            }
        }
    }

    fn mentions(event: &Event, filename: &str) -> bool {
        event.paths.iter().any(|p| p.ends_with(filename))
    }

    #[test]
    fn detects_a_file_created_in_the_watched_directory() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = channel();

        let _watcher = watch(dir.path(), move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .expect("watcher should start");

        // Give the native watcher a moment to register before triggering it.
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(dir.path().join("scene.md"), "hello").unwrap();

        assert!(
            wait_for_event_matching(&rx, |e| mentions(e, "scene.md"), Duration::from_secs(5)),
            "should receive a filesystem event for the new file"
        );
    }

    #[test]
    fn detects_a_file_modification() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("scene.md");
        std::fs::write(&file_path, "original").unwrap();

        let (tx, rx) = channel();
        let _watcher = watch(dir.path(), move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .expect("watcher should start");

        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(&file_path, "updated content").unwrap();

        assert!(
            wait_for_event_matching(&rx, |e| mentions(e, "scene.md"), Duration::from_secs(5)),
            "should receive a filesystem event for the modified file"
        );
    }

    #[test]
    fn file_change_event_maps_kinds_to_stable_strings() {
        let create = FileChangeEvent::from(Event::new(EventKind::Create(
            notify::event::CreateKind::File,
        )));
        assert_eq!(create.kind, "create");

        let remove = FileChangeEvent::from(Event::new(EventKind::Remove(
            notify::event::RemoveKind::File,
        )));
        assert_eq!(remove.kind, "remove");
    }
}
