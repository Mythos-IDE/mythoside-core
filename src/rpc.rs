use crate::manuscript::commands::{self, CreateCharacterInput, CreateSeriesInput};
use crate::watcher::{self, FileChangeEvent, WatcherState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// One JSON-RPC-ish request per line on stdin: `{"id": 1, "method": "...",
/// "params": {...}}`. `id` is echoed back on the matching `Response` so a
/// client that pipelines multiple in-flight requests can match them up.
/// Both derives are used: the server (this crate's `main.rs`) only
/// deserializes it; the client (src-tauri) only serializes it.
#[derive(Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// One line on stdout per `Request` received, `result` xor `error` set.
/// The server serializes it; the client deserializes it.
#[derive(Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Server-initiated line on stdout with no `id` — distinguishes it from a
/// `Response` on the client's read side. Used for watcher events, which
/// aren't triggered by any single request.
#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

/// Thread-safe writer for JSON-RPC lines on stdout — shared between the main
/// request/response loop and the watcher's background callback thread
/// (`notify` delivers filesystem events on its own thread), both of which
/// write lines to the same stdout.
#[derive(Clone)]
pub struct Notifier(Arc<Mutex<io::Stdout>>);

impl Default for Notifier {
    fn default() -> Self {
        Notifier(Arc::new(Mutex::new(io::stdout())))
    }
}

impl Notifier {
    pub fn send_response(&self, response: &Response) {
        self.write_line(response);
    }

    pub fn send_file_changed(&self, event: FileChangeEvent) {
        let notification = Notification {
            method: "file-changed".to_string(),
            params: serde_json::to_value(event).unwrap_or(Value::Null),
        };
        self.write_line(&notification);
    }

    fn write_line<T: Serialize>(&self, value: &T) {
        let Ok(line) = serde_json::to_string(value) else {
            return;
        };
        let mut out = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartWatchingParams {
    path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSeriesParams {
    project_dir: String,
}

/// Routes one request's `(method, params)` to the matching handler. Kept
/// Notifier-generic-free of any actual stdout/process concerns beyond that
/// type, so this is unit-testable without spawning the real binary — see
/// tests below.
pub fn dispatch(
    method: &str,
    params: Value,
    watcher_state: &WatcherState,
    notifier: &Notifier,
) -> Result<Value, String> {
    match method {
        "create_character" => {
            let input: CreateCharacterInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let character = commands::create_character(input)?;
            serde_json::to_value(character).map_err(|e| e.to_string())
        }
        "create_series" => {
            let input: CreateSeriesInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let series = commands::create_series(input)?;
            serde_json::to_value(series).map_err(|e| e.to_string())
        }
        "get_series" => {
            let GetSeriesParams { project_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let series = commands::get_series(&project_dir)?;
            serde_json::to_value(series).map_err(|e| e.to_string())
        }
        "start_watching" => {
            let StartWatchingParams { path } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let notifier = notifier.clone();
            watcher::start_watching(watcher_state, &path, move |event| {
                notifier.send_file_changed(event);
            })?;
            Ok(Value::Null)
        }
        "stop_watching" => {
            watcher::stop_watching(watcher_state)?;
            Ok(Value::Null)
        }
        _ => Err(format!("unknown method: {method}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dispatches_create_character_and_returns_the_created_entity() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let params = json!({
            "bookDir": dir.path().to_string_lossy(),
            "bookId": "book-1",
            "name": "Lyra Vance",
            "role": "Protagonist",
        });

        let result = dispatch("create_character", params, &watcher_state, &notifier)
            .expect("create_character should succeed");
        assert_eq!(result["name"], "Lyra Vance");
        assert!(result["id"].as_str().is_some_and(|s| !s.is_empty()));
    }

    #[test]
    fn dispatches_get_series_for_a_previously_created_series() {
        // Seeds the fixture via `create_series_in` directly rather than
        // dispatching "create_series" — that method resolves the real OS
        // Documents directory (see its doc comment), which a test must
        // never write into.
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let created = commands::create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Aethelgard Chronicles".into(),
                description: "An epic fantasy series.".into(),
            },
        )
        .expect("should create the series fixture");

        let get_params = json!({ "projectDir": created.project_dir });
        let fetched = dispatch("get_series", get_params, &watcher_state, &notifier)
            .expect("get_series should succeed");
        assert_eq!(fetched["title"], "The Aethelgard Chronicles");
        assert_eq!(fetched["id"], created.series.id);
    }

    #[test]
    fn dispatches_start_and_stop_watching() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let params = json!({ "path": dir.path().to_string_lossy() });
        assert!(dispatch("start_watching", params, &watcher_state, &notifier).is_ok());
        assert!(dispatch("stop_watching", Value::Null, &watcher_state, &notifier).is_ok());
    }

    #[test]
    fn rejects_an_unknown_method() {
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();
        assert!(dispatch("delete_everything", Value::Null, &watcher_state, &notifier).is_err());
    }
}
