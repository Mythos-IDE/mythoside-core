# Workspace Crates Specification

This document provides a detailed specification for every crate in the `mythoside-core` Cargo Workspace.

---

## Crate 1: `mythoside-common`

### Purpose
Provides shared utility functions and stateless helpers used across the core backend.

### Responsibilities
- String slugification (`slugify`).
- ISO-8601 / RFC-3339 timestamp generation (`now_iso8601`).

### What DOES Belong Here
- Pure, stateless helper functions required by multiple crates.
- Shared primitive formatting or calculation routines.

### What MUST NOT Belong Here
- Domain models (`Series`, `Book`, `Chapter`).
- File I/O or filesystem operations.
- Parsing or serialization logic.

### Dependencies
- `chrono` (for UTC timestamp generation).

### Public API
- `pub fn now_iso8601() -> String`
- `pub fn slugify(input: &str) -> String`

### Internal API
- Standard unit tests in `lib.rs`.

### Future Evolution
Will remain a small utility crate. If shared primitive types or error wrappers emerge, they may reside here if they have no domain dependencies.

---

## Crate 2: `mythoside-narrative`

### Purpose
Defines the pure domain model for manuscript entities.

### Responsibilities
- Define struct and enum definitions for manuscript entities: `Series`, `Book`, `Chapter`, `Character`, `Location`, `Note`, `NoteType`.
- Expose attributes with `serde` and `specta::Type` derivations for type-safe RPC serialization.

### What DOES Belong Here
- Data structure fields, type definitions, and enum variants representing the narrative domain.
- Serde and Specta attribute annotations on domain types.

### What MUST NOT Belong Here
- Markdown/YAML frontmatter parsing or formatting routines.
- Filesystem reading/writing code.
- JSON-RPC handling logic.

### Dependencies
- `serde`
- `specta`

### Public API
- `pub struct Series`
- `pub struct Book`
- `pub struct Chapter`
- `pub struct Character`
- `pub struct Location`
- `pub enum NoteType`
- `pub struct Note`

### Internal API
- None (pure type definitions).

### Future Evolution
Will expand as new domain entities (e.g., Factions, Items, Relationships, Timelines) are added to the Mythos domain specification.

---

## Crate 3: `mythoside-parser`

### Purpose
Handles parsing and serialization between raw string documents (Markdown + YAML frontmatter) and `mythoside-narrative` domain structures.

### Responsibilities
- Split frontmatter delimiters (`---`) from Markdown prose body (`split_frontmatter`, `join_frontmatter`).
- Parse and serialize YAML documents (`parse_series`, `serialize_series`, `parse_book`, `serialize_book`).
- Parse and serialize combined frontmatter + Markdown files (`parse_chapter`, `serialize_chapter`, `parse_character`, `serialize_character`, `parse_location`, `serialize_location`, `parse_note`, `serialize_note`).

### What DOES Belong Here
- Document formatting, string parsing, and YAML serialization.
- Metadata extraction from frontmatter headers.

### What MUST NOT Belong Here
- File I/O operations (`std::fs::read_to_string`, `std::fs::write`).
- Directory traversal or path manipulation.
- RPC request handling.

### Dependencies
- `mythoside-narrative`
- `serde`
- `serde_norway`

### Public API
- `pub fn parse_series(text: &str) -> Result<Series, String>`
- `pub fn serialize_series(series: &Series) -> Result<String, String>`
- `pub fn parse_book(text: &str) -> Result<Book, String>`
- `pub fn serialize_book(book: &Book) -> Result<String, String>`
- `pub fn parse_chapter(file_text: &str) -> Result<Chapter, String>`
- `pub fn serialize_chapter(chapter: &Chapter) -> Result<String, String>`
- `pub fn parse_character(file_text: &str) -> Result<Character, String>`
- `pub fn serialize_character(character: &Character) -> Result<String, String>`
- `pub fn parse_location(file_text: &str) -> Result<Location, String>`
- `pub fn serialize_location(location: &Location) -> Result<String, String>`
- `pub fn parse_note(file_text: &str) -> Result<Note, String>`
- `pub fn serialize_note(note: &Note) -> Result<String, String>`

### Internal API
- `fn parse_yaml<T: DeserializeOwned>(text: &str) -> Result<T, String>`
- `fn serialize_yaml<T: Serialize>(value: &T) -> Result<String, String>`
- `fn split_frontmatter(file_text: &str) -> Result<(&str, &str), String>`
- `fn join_frontmatter(frontmatter_yaml: &str, body: &str) -> String`

### Future Evolution
Will evolve to support advanced manuscript document formats, block AST parsing, or export formats without requiring changes to filesystem or RPC crates.

---

## Crate 4: `mythoside-workspace`

### Purpose
Manages project/workspace loading, directory layout hierarchy, file system operations, chapter ordering, and native filesystem watching.

### Responsibilities
- Create, list, update, and delete series, books, chapters, characters, locations, and notes on disk.
- Automatically compute sequential order numbers for books and chapters (`next_book_order`).
- Manage background file watching (`WatcherState`, `start_watching`, `stop_watching`).
- Convert filesystem change events into `FileChangeEvent` notifications.

### What DOES Belong Here
- Directory scanning (`scan_entities`), entity mutation on disk (`update_entity_by_id`), and recoverable deletes (`trash::delete`).
- Native filesystem event monitoring via `notify`.

### What MUST NOT Belong Here
- Stdio line reading/writing or JSON-RPC protocol parsing.
- Direct YAML text parsing (delegated to `mythoside-parser`).

### Dependencies
- `mythoside-common`
- `mythoside-narrative`
- `mythoside-parser`
- `serde`
- `specta`
- `uuid`
- `chrono`
- `notify`
- `dirs`
- `trash`
- `tempfile` (dev dependency)

### Public API
- `pub mod watcher`:
  - `pub struct FileChangeEvent`
  - `pub struct WatcherState`
  - `pub fn start_watching(...)`
  - `pub fn stop_watching(...)`
- `pub mod commands`:
  - `pub struct CreateSeriesInput`, `pub struct CreateSeriesOutput`, `pub struct ListSeriesOutput`, `pub struct UpdateSeriesInput`
  - `pub fn create_series`, `pub fn create_series_in`, `pub fn list_series`, `pub fn list_series_in`, `pub fn get_series`, `pub fn update_series`, `pub fn delete_series`
  - `pub struct CreateBookInput`, `pub struct BookHandle`, `pub struct ListBooksOutput`
  - `pub fn create_book`, `pub fn list_books`, `pub fn delete_book`
  - `pub struct CreateChapterInput`, `pub struct ChapterHandle`, `pub struct ListChaptersOutput`, `pub struct UpdateChapterInput`, `pub struct UpdateChapterContentInput`, `pub enum MoveDirection`
  - `pub fn create_chapter`, `pub fn list_chapters`, `pub fn update_chapter`, `pub fn update_chapter_content`, `pub fn move_chapter`, `pub fn delete_chapter`
  - `pub struct CreateCharacterInput`, `pub struct ListCharactersOutput`, `pub struct UpdateCharacterInput`
  - `pub fn create_character`, `pub fn list_characters`, `pub fn update_character`, `pub fn delete_character`
  - `pub struct CreateLocationInput`, `pub struct ListLocationsOutput`, `pub struct UpdateLocationInput`
  - `pub fn create_location`, `pub fn list_locations`, `pub fn update_location`, `pub fn delete_location`
  - `pub struct CreateNoteInput`, `pub struct ListNotesOutput`, `pub struct UpdateNoteInput`
  - `pub fn create_note`, `pub fn list_notes`, `pub fn update_note`, `pub fn delete_note`

### Internal API
- `fn scan_entities<T>(...)`
- `fn delete_entity_by_id<T>(...)`
- `fn update_entity_by_id<T>(...)`
- `fn update_series_yaml(...)`
- `fn resolve_documents_dir()`

### Future Evolution
Will act as the workspace document coordinator. If local DB caching or metadata indexing is introduced in the future, `workspace` will orchestrate calls between disk storage and index layers.

---

## Crate 5: `mythoside-rpc`

### Purpose
Implements the JSON-RPC stdio protocol, request deserialization, thread-safe stdout notification writing, and method routing.

### Responsibilities
- Deserialize line-delimited JSON-RPC requests (`Request`).
- Serialize line-delimited responses (`Response`) and unsolicited file-watch notifications (`Notification`).
- Provide thread-safe stdout writing (`Notifier`).
- Route method strings (`"create_character"`, `"list_chapters"`, etc.) to `mythoside-workspace::commands` and `mythoside-workspace::watcher`.

### What DOES Belong Here
- RPC wire format definition.
- Stdio line serialization and deserialization.
- Request-to-command dispatch routing (`dispatch`).

### What MUST NOT Belong Here
- Direct file I/O or directory scanning.
- Manuscript format parsing.

### Dependencies
- `mythoside-narrative`
- `mythoside-workspace`
- `serde`
- `serde_json`

### Public API
- `pub struct Request`
- `pub struct Response`
- `pub struct Notification`
- `pub struct Notifier`
- `pub fn dispatch(method: &str, params: Value, watcher_state: &WatcherState, notifier: &Notifier) -> Result<Value, String>`

### Internal API
- Parameter deserialization structs (`ProjectDirParams`, `BookDirParams`, `DeleteBookParams`, etc.).

### Future Evolution
Will remain transport-focused. If alternative transports (e.g. WebSocket or IPC) are introduced, `dispatch` will remain the transport-agnostic entry point.

---

## Crate 6: `mythoside-core` (Root Facade)

### Purpose
Executable binary target and top-level backward-compatible facade re-exporting internal crate modules.

### Responsibilities
- Provide main binary executable loop in `src/main.rs`.
- Re-export internal modules (`manuscript::models`, `manuscript::format`, `manuscript::commands`, `watcher`, `rpc`) to preserve public API stability for external callers.

### Dependencies
- `mythoside-common`
- `mythoside-narrative`
- `mythoside-parser`
- `mythoside-workspace`
- `mythoside-rpc`

### Public API
- `pub mod manuscript`:
  - `pub use mythoside_workspace::commands;`
  - `pub use mythoside_parser as format;`
  - `pub use mythoside_narrative as models;`
- `pub use mythoside_workspace::watcher;`
- `pub use mythoside_rpc as rpc;`

### Future Evolution
Will remain the workspace root facade and entrypoint executable.
