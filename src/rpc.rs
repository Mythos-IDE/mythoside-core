use crate::manuscript::commands::{
    self, CreateBookInput, CreateChapterInput, CreateCharacterInput, CreateLocationInput,
    CreateNoteInput, CreateSeriesInput, MoveDirection, UpdateChapterContentInput,
    UpdateChapterInput, UpdateSeriesInput,
};
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

/// Shared by every method that takes nothing but a bare `project_dir`
/// ("get_series", "list_books").
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDirParams {
    project_dir: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteBookParams {
    book_dir: String,
}

/// For "list_chapters" — genuinely book-scoped (unlike Character/Location/
/// Note, which moved to the series level and now share `ProjectDirParams`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BookDirParams {
    book_dir: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteChapterParams {
    chapter_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteCharacterParams {
    project_dir: String,
    character_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteLocationParams {
    project_dir: String,
    location_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteNoteParams {
    project_dir: String,
    note_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveChapterParams {
    book_dir: String,
    chapter_id: String,
    direction: MoveDirection,
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
            let ProjectDirParams { project_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let series = commands::get_series(&project_dir)?;
            serde_json::to_value(series).map_err(|e| e.to_string())
        }
        "list_series" => {
            // No params — resolves the same real Documents dir create_series
            // writes into, so (like "create_series") this isn't covered by a
            // dispatch-level test; see list_series_in's own unit tests.
            let series = commands::list_series()?;
            serde_json::to_value(series).map_err(|e| e.to_string())
        }
        "update_series" => {
            let input: UpdateSeriesInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let series = commands::update_series(input)?;
            serde_json::to_value(series).map_err(|e| e.to_string())
        }
        "create_book" => {
            let input: CreateBookInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let book = commands::create_book(input)?;
            serde_json::to_value(book).map_err(|e| e.to_string())
        }
        "list_books" => {
            let ProjectDirParams { project_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let books = commands::list_books(&project_dir)?;
            serde_json::to_value(books).map_err(|e| e.to_string())
        }
        "create_location" => {
            let input: CreateLocationInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let location = commands::create_location(input)?;
            serde_json::to_value(location).map_err(|e| e.to_string())
        }
        "list_locations" => {
            let ProjectDirParams { project_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let locations = commands::list_locations(&project_dir)?;
            serde_json::to_value(locations).map_err(|e| e.to_string())
        }
        "list_characters" => {
            let ProjectDirParams { project_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let characters = commands::list_characters(&project_dir)?;
            serde_json::to_value(characters).map_err(|e| e.to_string())
        }
        "create_note" => {
            let input: CreateNoteInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let note = commands::create_note(input)?;
            serde_json::to_value(note).map_err(|e| e.to_string())
        }
        "list_notes" => {
            let ProjectDirParams { project_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let notes = commands::list_notes(&project_dir)?;
            serde_json::to_value(notes).map_err(|e| e.to_string())
        }
        "delete_series" => {
            let ProjectDirParams { project_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            commands::delete_series(&project_dir)?;
            Ok(Value::Null)
        }
        "delete_book" => {
            let DeleteBookParams { book_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            commands::delete_book(&book_dir)?;
            Ok(Value::Null)
        }
        "delete_character" => {
            let DeleteCharacterParams {
                project_dir,
                character_id,
            } = serde_json::from_value(params).map_err(|e| e.to_string())?;
            commands::delete_character(&project_dir, &character_id)?;
            Ok(Value::Null)
        }
        "delete_location" => {
            let DeleteLocationParams {
                project_dir,
                location_id,
            } = serde_json::from_value(params).map_err(|e| e.to_string())?;
            commands::delete_location(&project_dir, &location_id)?;
            Ok(Value::Null)
        }
        "delete_note" => {
            let DeleteNoteParams {
                project_dir,
                note_id,
            } = serde_json::from_value(params).map_err(|e| e.to_string())?;
            commands::delete_note(&project_dir, &note_id)?;
            Ok(Value::Null)
        }
        "create_chapter" => {
            let input: CreateChapterInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let chapter = commands::create_chapter(input)?;
            serde_json::to_value(chapter).map_err(|e| e.to_string())
        }
        "list_chapters" => {
            let BookDirParams { book_dir } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let chapters = commands::list_chapters(&book_dir)?;
            serde_json::to_value(chapters).map_err(|e| e.to_string())
        }
        "delete_chapter" => {
            let DeleteChapterParams { chapter_path } =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            commands::delete_chapter(&chapter_path)?;
            Ok(Value::Null)
        }
        "update_chapter_content" => {
            let input: UpdateChapterContentInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let chapter = commands::update_chapter_content(input)?;
            serde_json::to_value(chapter).map_err(|e| e.to_string())
        }
        "update_chapter" => {
            let input: UpdateChapterInput =
                serde_json::from_value(params).map_err(|e| e.to_string())?;
            let chapter = commands::update_chapter(input)?;
            serde_json::to_value(chapter).map_err(|e| e.to_string())
        }
        "move_chapter" => {
            let MoveChapterParams {
                book_dir,
                chapter_id,
                direction,
            } = serde_json::from_value(params).map_err(|e| e.to_string())?;
            commands::move_chapter(&book_dir, &chapter_id, direction)?;
            Ok(Value::Null)
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
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
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
    fn dispatches_update_series() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let created = commands::create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Aethelgard Chronicles".into(),
                description: "Original.".into(),
            },
        )
        .unwrap();

        let update_params = json!({
            "projectDir": created.project_dir,
            "title": "Revised Title",
            "description": "Updated.",
        });
        let updated = dispatch("update_series", update_params, &watcher_state, &notifier)
            .expect("update_series should succeed");
        assert_eq!(updated["title"], "Revised Title");
        assert_eq!(updated["id"], created.series.id);
    }

    #[test]
    fn dispatches_create_book_then_list_books() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "title": "Shadow of the Void",
        });
        let created = dispatch("create_book", create_params, &watcher_state, &notifier)
            .expect("create_book should succeed");
        assert_eq!(created["book"]["title"], "Shadow of the Void");
        assert_eq!(created["book"]["order"], 1);

        let list_params = json!({ "projectDir": dir.path().to_string_lossy() });
        let listed = dispatch("list_books", list_params, &watcher_state, &notifier)
            .expect("list_books should succeed");
        assert_eq!(listed["books"].as_array().unwrap().len(), 1);
        assert!(listed["warnings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn dispatches_create_location_then_list_locations() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "name": "Aethelgard",
        });
        let created = dispatch("create_location", create_params, &watcher_state, &notifier)
            .expect("create_location should succeed");
        assert_eq!(created["name"], "Aethelgard");

        let list_params = json!({ "projectDir": dir.path().to_string_lossy() });
        let listed = dispatch("list_locations", list_params, &watcher_state, &notifier)
            .expect("list_locations should succeed");
        assert_eq!(listed["locations"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn dispatches_create_note_then_list_notes() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "title": "The Sealing",
            "type": "timeline",
        });
        let created = dispatch("create_note", create_params, &watcher_state, &notifier)
            .expect("create_note should succeed");
        assert_eq!(created["type"], "timeline");

        let list_params = json!({ "projectDir": dir.path().to_string_lossy() });
        let listed = dispatch("list_notes", list_params, &watcher_state, &notifier)
            .expect("list_notes should succeed");
        assert_eq!(listed["notes"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn dispatches_list_characters() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "name": "Lyra Vance",
            "role": "Protagonist",
        });
        dispatch("create_character", create_params, &watcher_state, &notifier)
            .expect("create_character should succeed");

        let list_params = json!({ "projectDir": dir.path().to_string_lossy() });
        let listed = dispatch("list_characters", list_params, &watcher_state, &notifier)
            .expect("list_characters should succeed");
        assert_eq!(listed["characters"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn dispatches_delete_book() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "title": "Shadow of the Void",
        });
        let created = dispatch("create_book", create_params, &watcher_state, &notifier).unwrap();
        let book_dir = created["bookDir"].as_str().unwrap();

        let delete_params = json!({ "bookDir": book_dir });
        assert!(dispatch("delete_book", delete_params, &watcher_state, &notifier).is_ok());
        assert!(!std::path::Path::new(book_dir).exists());
    }

    #[test]
    fn dispatches_delete_character() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "name": "Lyra Vance",
            "role": "Protagonist",
        });
        let created =
            dispatch("create_character", create_params, &watcher_state, &notifier).unwrap();
        let character_id = created["id"].as_str().unwrap();

        let delete_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "characterId": character_id,
        });
        assert!(dispatch("delete_character", delete_params, &watcher_state, &notifier).is_ok());

        let list_params = json!({ "projectDir": dir.path().to_string_lossy() });
        let listed = dispatch("list_characters", list_params, &watcher_state, &notifier).unwrap();
        assert!(listed["characters"].as_array().unwrap().is_empty());
    }

    #[test]
    fn dispatches_delete_location() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "name": "Aethelgard",
        });
        let created =
            dispatch("create_location", create_params, &watcher_state, &notifier).unwrap();
        let location_id = created["id"].as_str().unwrap();

        let delete_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "locationId": location_id,
        });
        assert!(dispatch("delete_location", delete_params, &watcher_state, &notifier).is_ok());
    }

    #[test]
    fn dispatches_delete_note() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "seriesId": "series-1",
            "title": "The Sealing",
            "type": "timeline",
        });
        let created = dispatch("create_note", create_params, &watcher_state, &notifier).unwrap();
        let note_id = created["id"].as_str().unwrap();

        let delete_params = json!({
            "projectDir": dir.path().to_string_lossy(),
            "noteId": note_id,
        });
        assert!(dispatch("delete_note", delete_params, &watcher_state, &notifier).is_ok());
    }

    #[test]
    fn dispatches_delete_series() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let created = commands::create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Aethelgard Chronicles".into(),
                description: "".into(),
            },
        )
        .unwrap();

        let delete_params = json!({ "projectDir": created.project_dir });
        assert!(dispatch("delete_series", delete_params, &watcher_state, &notifier).is_ok());
        assert!(!std::path::Path::new(&created.project_dir).exists());
    }

    #[test]
    fn dispatches_create_chapter_then_list_chapters_then_delete_chapter() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "bookDir": dir.path().to_string_lossy(),
            "bookId": "book-1",
            "title": "The Obsidian Gate",
            "content": "Original prose.",
        });
        let created = dispatch("create_chapter", create_params, &watcher_state, &notifier)
            .expect("create_chapter should succeed");
        assert_eq!(created["chapter"]["title"], "The Obsidian Gate");
        let chapter_path = created["chapterPath"].as_str().unwrap().to_string();

        let list_params = json!({ "bookDir": dir.path().to_string_lossy() });
        let listed = dispatch("list_chapters", list_params, &watcher_state, &notifier)
            .expect("list_chapters should succeed");
        assert_eq!(listed["chapters"].as_array().unwrap().len(), 1);

        let delete_params = json!({ "chapterPath": chapter_path });
        assert!(dispatch("delete_chapter", delete_params, &watcher_state, &notifier).is_ok());
        assert!(!std::path::Path::new(&chapter_path).exists());
    }

    #[test]
    fn dispatches_update_chapter_content() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "bookDir": dir.path().to_string_lossy(),
            "bookId": "book-1",
            "title": "The Obsidian Gate",
            "content": "Original prose.",
        });
        let created = dispatch("create_chapter", create_params, &watcher_state, &notifier)
            .expect("create_chapter should succeed");
        let chapter_path = created["chapterPath"].as_str().unwrap().to_string();

        let update_params = json!({ "chapterPath": chapter_path, "content": "Revised prose." });
        let updated = dispatch(
            "update_chapter_content",
            update_params,
            &watcher_state,
            &notifier,
        )
        .expect("update_chapter_content should succeed");
        assert_eq!(updated["content"], "Revised prose.");
    }

    #[test]
    fn dispatches_update_chapter() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();

        let create_params = json!({
            "bookDir": dir.path().to_string_lossy(),
            "bookId": "book-1",
            "title": "The Obsidian Gate",
        });
        let created = dispatch("create_chapter", create_params, &watcher_state, &notifier).unwrap();
        let chapter_path = created["chapterPath"].as_str().unwrap().to_string();

        let update_params = json!({ "chapterPath": chapter_path, "title": "Revised Title" });
        let updated = dispatch("update_chapter", update_params, &watcher_state, &notifier)
            .expect("update_chapter should succeed");
        assert_eq!(updated["title"], "Revised Title");
    }

    #[test]
    fn dispatches_move_chapter() {
        let dir = tempfile::tempdir().unwrap();
        let watcher_state = WatcherState::default();
        let notifier = Notifier::default();
        let book_dir = dir.path().to_string_lossy().to_string();

        let create =
            |title: &str| json!({ "bookDir": book_dir, "bookId": "book-1", "title": title });
        dispatch("create_chapter", create("First"), &watcher_state, &notifier).unwrap();
        let second = dispatch(
            "create_chapter",
            create("Second"),
            &watcher_state,
            &notifier,
        )
        .unwrap();
        let second_id = second["chapter"]["id"].as_str().unwrap().to_string();

        let move_params = json!({ "bookDir": book_dir, "chapterId": second_id, "direction": "up" });
        assert!(dispatch("move_chapter", move_params, &watcher_state, &notifier).is_ok());

        let listed = dispatch(
            "list_chapters",
            json!({ "bookDir": book_dir }),
            &watcher_state,
            &notifier,
        )
        .unwrap();
        assert_eq!(listed["chapters"][0]["chapter"]["title"], "Second");
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
