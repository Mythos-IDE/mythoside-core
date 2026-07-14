use super::format;
use super::models::{Book, Character, Location, Note, NoteType, Series};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// ISO-8601/RFC-3339 timestamp — used instead of a raw epoch number so the
/// on-disk YAML frontmatter stays human-readable (a core "local-first, it's
/// just plain files" selling point) and so `created_at` doesn't need the
/// `number | null` TS type specta gives f64/u64 (null covers NaN/Infinity,
/// which a timestamp never actually is, but the type can't express that).
fn now_iso8601() -> String {
    Utc::now().to_rfc3339()
}

/// Lowercases and replaces runs of non-alphanumeric characters with a single
/// `-`, trimming leading/trailing dashes. E.g. "Lyra Vance" -> "lyra-vance".
fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for c in input.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateCharacterInput {
    /// Filesystem path to the book's folder — the character file is written
    /// to `<book_dir>/characters/<slug>.md`.
    pub book_dir: String,
    pub book_id: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

pub fn create_character(input: CreateCharacterInput) -> Result<Character, String> {
    let id = Uuid::new_v4().to_string();
    let character = Character {
        id: id.clone(),
        book_id: input.book_id,
        name: input.name.clone(),
        role: input.role,
        attributes: input.attributes,
        created_at: now_iso8601(),
        bio: input.bio,
    };

    let characters_dir = Path::new(&input.book_dir).join("characters");
    fs::create_dir_all(&characters_dir).map_err(|e| e.to_string())?;

    // Short id suffix keeps filenames unique even if two characters share a name.
    let slug = format!("{}-{}", slugify(&input.name), &id[..8]);
    let file_path = characters_dir.join(format!("{slug}.md"));
    let contents = format::serialize_character(&character)?;
    fs::write(&file_path, contents).map_err(|e| e.to_string())?;

    Ok(character)
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ListCharactersOutput {
    pub characters: Vec<Character>,
    pub warnings: Vec<String>,
}

/// Scans `dir` for files with the given extension (no leading dot, e.g.
/// `"md"`), parsing each with `parse`. A file that fails to parse becomes a
/// `"<path>: <error>"` warning instead of failing the whole scan — one
/// corrupt/hand-edited file shouldn't hide every other entity in a list
/// view. A missing directory (nothing created yet) yields an empty result,
/// not a warning. Shared by list_characters/list_locations/list_notes;
/// list_books scans subdirectories for a fixed filename instead, a
/// different enough shape to not fit this helper.
fn scan_entities<T>(
    dir: &Path,
    extension: &str,
    parse: impl Fn(&str) -> Result<T, String>,
) -> (Vec<T>, Vec<String>) {
    let mut items = Vec::new();
    let mut warnings = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return (items, warnings);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some(extension) {
            continue;
        }
        match fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| parse(&text))
        {
            Ok(item) => items.push(item),
            Err(e) => warnings.push(format!("{}: {e}", path.display())),
        }
    }
    (items, warnings)
}

pub fn list_characters(book_dir: &str) -> Result<ListCharactersOutput, String> {
    let characters_dir = Path::new(book_dir).join("characters");
    let (characters, warnings) = scan_entities(&characters_dir, "md", format::parse_character);
    Ok(ListCharactersOutput {
        characters,
        warnings,
    })
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationInput {
    pub book_dir: String,
    pub book_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

pub fn create_location(input: CreateLocationInput) -> Result<Location, String> {
    let id = Uuid::new_v4().to_string();
    let location = Location {
        id: id.clone(),
        book_id: input.book_id,
        name: input.name.clone(),
        created_at: now_iso8601(),
        description: input.description,
    };

    let locations_dir = Path::new(&input.book_dir).join("locations");
    fs::create_dir_all(&locations_dir).map_err(|e| e.to_string())?;

    // Short id suffix keeps filenames unique even if two locations share a name.
    let slug = format!("{}-{}", slugify(&input.name), &id[..8]);
    let file_path = locations_dir.join(format!("{slug}.md"));
    let contents = format::serialize_location(&location)?;
    fs::write(&file_path, contents).map_err(|e| e.to_string())?;

    Ok(location)
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ListLocationsOutput {
    pub locations: Vec<Location>,
    pub warnings: Vec<String>,
}

pub fn list_locations(book_dir: &str) -> Result<ListLocationsOutput, String> {
    let locations_dir = Path::new(book_dir).join("locations");
    let (locations, warnings) = scan_entities(&locations_dir, "md", format::parse_location);
    Ok(ListLocationsOutput {
        locations,
        warnings,
    })
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteInput {
    pub book_dir: String,
    pub book_id: String,
    pub title: String,
    // `Note` itself wire-renames this field to `type` (models.rs) — matched
    // here on purpose so TS sees `input.type`/`note.type` consistently
    // instead of `input.noteType`/`note.type`.
    #[serde(rename = "type")]
    pub note_type: NoteType,
    #[serde(default)]
    pub content: String,
}

pub fn create_note(input: CreateNoteInput) -> Result<Note, String> {
    let id = Uuid::new_v4().to_string();
    let note = Note {
        id: id.clone(),
        book_id: input.book_id,
        title: input.title.clone(),
        note_type: input.note_type,
        created_at: now_iso8601(),
        content: input.content,
    };

    let notes_dir = Path::new(&input.book_dir).join("notes");
    fs::create_dir_all(&notes_dir).map_err(|e| e.to_string())?;

    // Short id suffix keeps filenames unique even if two notes share a title.
    let slug = format!("{}-{}", slugify(&input.title), &id[..8]);
    let file_path = notes_dir.join(format!("{slug}.md"));
    let contents = format::serialize_note(&note)?;
    fs::write(&file_path, contents).map_err(|e| e.to_string())?;

    Ok(note)
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ListNotesOutput {
    pub notes: Vec<Note>,
    pub warnings: Vec<String>,
}

pub fn list_notes(book_dir: &str) -> Result<ListNotesOutput, String> {
    let notes_dir = Path::new(book_dir).join("notes");
    let (notes, warnings) = scan_entities(&notes_dir, "md", format::parse_note);
    Ok(ListNotesOutput { notes, warnings })
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateSeriesInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateSeriesOutput {
    pub series: Series,
    /// Where `series.yaml` actually landed — the caller (a UI, eventually a
    /// "recent projects" list) needs this to read the series back later via
    /// `get_series`. Nothing asked the user to type or pick this; see
    /// `create_series`.
    pub project_dir: String,
}

/// Cross-platform "Documents" directory (`~/Documents` on macOS/Windows,
/// XDG-aware on Linux via the `dirs` crate — not Tauri-specific, this crate
/// has no Tauri dependency). New series live in a `MythosIDE/` subfolder
/// there by default, named from a slug of the title, so nobody has to type
/// or pick a filesystem path just to start writing — a relative path typed
/// into a UI resolves against whatever directory the *process* happens to
/// be running from, which is a footgun (hit this exact confusion once: a
/// dev-mode test run put a series inside `src-tauri/` because that's where
/// `cargo run` executes from).
fn resolve_documents_dir() -> Result<PathBuf, String> {
    dirs::document_dir().ok_or_else(|| "could not determine the Documents directory".to_string())
}

/// Crate-visible (not private) so `rpc.rs`'s tests can seed a fixture
/// series without going through the public `create_series`, which resolves
/// the *real* OS Documents directory — a dispatch-level test calling that
/// would write throwaway folders into the developer's actual `~/Documents`
/// on every `cargo test` run.
pub(crate) fn create_series_in(
    base_dir: &Path,
    input: CreateSeriesInput,
) -> Result<CreateSeriesOutput, String> {
    let id = Uuid::new_v4().to_string();
    // Short id suffix keeps folder names unique even if two series share a title.
    let slug = format!("{}-{}", slugify(&input.title), &id[..8]);
    let project_dir = base_dir.join("MythosIDE").join(&slug);

    let series = Series {
        id,
        title: input.title,
        description: input.description,
        created_at: now_iso8601(),
    };

    fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;
    let contents = format::serialize_series(&series)?;
    fs::write(project_dir.join("series.yaml"), contents).map_err(|e| e.to_string())?;

    Ok(CreateSeriesOutput {
        series,
        project_dir: project_dir.to_string_lossy().into_owned(),
    })
}

pub fn create_series(input: CreateSeriesInput) -> Result<CreateSeriesOutput, String> {
    let documents_dir = resolve_documents_dir()?;
    create_series_in(&documents_dir, input)
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ListSeriesOutput {
    /// Reuses `CreateSeriesOutput`'s `{series, project_dir}` pairing rather
    /// than inventing an identically-shaped struct — every caller that
    /// needs to open a series already needs both fields together.
    pub series: Vec<CreateSeriesOutput>,
    pub warnings: Vec<String>,
}

/// Crate-visible for the same reason as `create_series_in`: a test must
/// never scan (or write into) the developer's real `~/Documents`.
pub(crate) fn list_series_in(base_dir: &Path) -> ListSeriesOutput {
    let mythoside_dir = base_dir.join("MythosIDE");
    let mut series = Vec::new();
    let mut warnings = Vec::new();

    let Ok(entries) = fs::read_dir(&mythoside_dir) else {
        return ListSeriesOutput { series, warnings }; // nothing created yet, not an error
    };
    for entry in entries.flatten() {
        let project_dir = entry.path();
        let series_yaml = project_dir.join("series.yaml");
        if !series_yaml.is_file() {
            continue;
        }
        match fs::read_to_string(&series_yaml)
            .map_err(|e| e.to_string())
            .and_then(|text| format::parse_series(&text))
        {
            Ok(parsed) => series.push(CreateSeriesOutput {
                series: parsed,
                project_dir: project_dir.to_string_lossy().into_owned(),
            }),
            Err(e) => warnings.push(format!("{}: {e}", series_yaml.display())),
        }
    }

    ListSeriesOutput { series, warnings }
}

/// Every series a user has created, so the app can offer "open an existing
/// series" instead of only ever starting a new one. Takes no path — same
/// reasoning as `create_series`, this just re-scans the same Documents
/// folder `create_series` writes into.
pub fn list_series() -> Result<ListSeriesOutput, String> {
    let documents_dir = resolve_documents_dir()?;
    Ok(list_series_in(&documents_dir))
}

pub fn get_series(project_dir: &str) -> Result<Series, String> {
    let file_path = Path::new(project_dir).join("series.yaml");
    let contents = fs::read_to_string(&file_path)
        .map_err(|e| format!("could not read {}: {e}", file_path.display()))?;
    format::parse_series(&contents)
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSeriesInput {
    pub project_dir: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
}

/// Full replace, not a patch — `id`/`created_at` are read back from the
/// existing `series.yaml` and preserved, everything else is overwritten.
pub fn update_series(input: UpdateSeriesInput) -> Result<Series, String> {
    let mut series = get_series(&input.project_dir)?;
    series.title = input.title;
    series.description = input.description;

    let contents = format::serialize_series(&series)?;
    fs::write(Path::new(&input.project_dir).join("series.yaml"), contents)
        .map_err(|e| e.to_string())?;

    Ok(series)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BookHandle {
    pub book: Book,
    /// Where `book.yaml` actually landed — mirrors `CreateSeriesOutput`'s
    /// `project_dir` pairing. Character/Location/Note all key off a book,
    /// not a series (see their `book_id`/`book_dir` fields), so callers need
    /// this to create any of those under the returned book.
    pub book_dir: String,
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookInput {
    pub project_dir: String,
    pub series_id: String,
    pub title: String,
    #[serde(default)]
    pub synopsis: String,
}

/// Counts existing `<project_dir>/*/book.yaml` folders to assign the next
/// `order` automatically, the same "don't make the user supply what the
/// system can compute" call as `create_series`'s path resolution.
///
/// NOTE: if a `delete_book` command is ever added, two books could collide
/// on `order` after a delete-then-recreate. Not solved here — there's no
/// delete in scope yet, and it's cheap to revisit once there is.
fn next_book_order(project_dir: &Path) -> u32 {
    let Ok(entries) = fs::read_dir(project_dir) else {
        return 1;
    };
    let existing = entries
        .flatten()
        .filter(|entry| entry.path().join("book.yaml").is_file())
        .count();
    existing as u32 + 1
}

pub fn create_book(input: CreateBookInput) -> Result<BookHandle, String> {
    let id = Uuid::new_v4().to_string();
    let project_dir = Path::new(&input.project_dir);
    let order = next_book_order(project_dir);

    // Short id suffix keeps folder names unique even if two books share a title.
    let slug = format!("{}-{}", slugify(&input.title), &id[..8]);
    let book_dir = project_dir.join(&slug);

    let book = Book {
        id,
        series_id: input.series_id,
        title: input.title,
        synopsis: input.synopsis,
        order,
        created_at: now_iso8601(),
    };

    fs::create_dir_all(&book_dir).map_err(|e| e.to_string())?;
    let contents = format::serialize_book(&book)?;
    fs::write(book_dir.join("book.yaml"), contents).map_err(|e| e.to_string())?;

    Ok(BookHandle {
        book,
        book_dir: book_dir.to_string_lossy().into_owned(),
    })
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ListBooksOutput {
    pub books: Vec<BookHandle>,
    /// `"<path>: <parse error>"` per book.yaml that failed to parse — one
    /// corrupt/hand-edited book shouldn't hide every other book from the
    /// dashboard, but it also shouldn't silently vanish with no trace.
    pub warnings: Vec<String>,
}

pub fn list_books(project_dir: &str) -> Result<ListBooksOutput, String> {
    let project_dir = Path::new(project_dir);
    let entries = fs::read_dir(project_dir).map_err(|e| e.to_string())?;

    let mut books = Vec::new();
    let mut warnings = Vec::new();
    for entry in entries.flatten() {
        let book_dir = entry.path();
        let book_yaml = book_dir.join("book.yaml");
        if !book_yaml.is_file() {
            continue;
        }
        match fs::read_to_string(&book_yaml)
            .map_err(|e| e.to_string())
            .and_then(|text| format::parse_book(&text))
        {
            Ok(book) => books.push(BookHandle {
                book,
                book_dir: book_dir.to_string_lossy().into_owned(),
            }),
            Err(e) => warnings.push(format!("{}: {e}", book_yaml.display())),
        }
    }
    books.sort_by_key(|handle| handle.book.order);

    Ok(ListBooksOutput { books, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_lowercases_and_dashes_non_alphanumeric_runs() {
        assert_eq!(slugify("Lyra Vance"), "lyra-vance");
        assert_eq!(slugify("  Silas -- Thorne!!  "), "silas-thorne");
    }

    #[test]
    fn creates_a_character_file_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let input = CreateCharacterInput {
            book_dir: dir.path().to_string_lossy().into_owned(),
            book_id: "book-1".into(),
            name: "Lyra Vance".into(),
            role: "Protagonist".into(),
            bio: "Stealthy, skilled in alchemy.".into(),
            attributes: HashMap::from([("home".to_string(), "Aethelgard".to_string())]),
        };

        let character = create_character(input).expect("should create the character");
        assert_eq!(character.name, "Lyra Vance");
        assert!(!character.id.is_empty());

        let characters_dir = dir.path().join("characters");
        let entries: Vec<_> = fs::read_dir(&characters_dir).unwrap().collect();
        assert_eq!(entries.len(), 1, "expected exactly one character file");

        let written = fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
        let parsed = format::parse_character(&written).unwrap();
        assert_eq!(parsed, character);
    }

    #[test]
    fn creates_a_series_yaml_under_a_slugified_subfolder() {
        let dir = tempfile::tempdir().unwrap();
        let input = CreateSeriesInput {
            title: "The Aethelgard Chronicles".into(),
            description: "An epic fantasy series.".into(),
        };

        let output = create_series_in(dir.path(), input).expect("should create the series");
        assert_eq!(output.series.title, "The Aethelgard Chronicles");
        assert!(!output.series.id.is_empty());
        assert!(output.project_dir.contains("the-aethelgard-chronicles"));

        let written =
            fs::read_to_string(Path::new(&output.project_dir).join("series.yaml")).unwrap();
        assert_eq!(format::parse_series(&written).unwrap(), output.series);
    }

    #[test]
    fn gets_a_previously_created_series() {
        let dir = tempfile::tempdir().unwrap();
        let created = create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Aethelgard Chronicles".into(),
                description: "".into(),
            },
        )
        .unwrap();

        let fetched = get_series(&created.project_dir).expect("should read the series");
        assert_eq!(fetched, created.series);
    }

    #[test]
    fn get_series_fails_when_series_yaml_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(get_series(&dir.path().to_string_lossy()).is_err());
    }

    #[test]
    fn list_series_is_empty_when_none_created_yet() {
        let dir = tempfile::tempdir().unwrap();
        let result = list_series_in(dir.path());
        assert!(result.series.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn lists_every_series_created_under_the_base_dir() {
        let dir = tempfile::tempdir().unwrap();
        create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Aethelgard Chronicles".into(),
                description: "".into(),
            },
        )
        .unwrap();
        create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Silverwood Saga".into(),
                description: "".into(),
            },
        )
        .unwrap();

        let result = list_series_in(dir.path());
        assert!(result.warnings.is_empty());
        assert_eq!(result.series.len(), 2);
        let titles: Vec<_> = result
            .series
            .iter()
            .map(|s| s.series.title.clone())
            .collect();
        assert!(titles.contains(&"The Aethelgard Chronicles".to_string()));
        assert!(titles.contains(&"The Silverwood Saga".to_string()));
    }

    #[test]
    fn list_series_collects_a_warning_for_an_unparseable_series_yaml_instead_of_failing() {
        let dir = tempfile::tempdir().unwrap();
        create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Aethelgard Chronicles".into(),
                description: "".into(),
            },
        )
        .unwrap();

        let broken_dir = dir.path().join("MythosIDE").join("broken-series");
        fs::create_dir_all(&broken_dir).unwrap();
        fs::write(
            broken_dir.join("series.yaml"),
            "not: valid: series: yaml: at: all",
        )
        .unwrap();

        let result = list_series_in(dir.path());
        assert_eq!(
            result.series.len(),
            1,
            "the good series should still show up"
        );
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("broken-series"));
    }

    #[test]
    fn updates_a_series_title_and_description_but_keeps_id_and_created_at() {
        let dir = tempfile::tempdir().unwrap();
        let created = create_series_in(
            dir.path(),
            CreateSeriesInput {
                title: "The Aethelgard Chronicles".into(),
                description: "Original description.".into(),
            },
        )
        .unwrap();

        let updated = update_series(UpdateSeriesInput {
            project_dir: created.project_dir.clone(),
            title: "The Aethelgard Chronicles: Revised".into(),
            description: "Updated description.".into(),
        })
        .expect("should update the series");

        assert_eq!(updated.id, created.series.id);
        assert_eq!(updated.created_at, created.series.created_at);
        assert_eq!(updated.title, "The Aethelgard Chronicles: Revised");
        assert_eq!(updated.description, "Updated description.");

        let fetched = get_series(&created.project_dir).unwrap();
        assert_eq!(fetched, updated);
    }

    #[test]
    fn creates_a_book_with_auto_assigned_order_and_slugified_subfolder() {
        let dir = tempfile::tempdir().unwrap();
        let handle = create_book(CreateBookInput {
            project_dir: dir.path().to_string_lossy().into_owned(),
            series_id: "series-1".into(),
            title: "Shadow of the Void".into(),
            synopsis: "The first book.".into(),
        })
        .expect("should create the book");

        assert_eq!(handle.book.order, 1);
        assert!(handle.book_dir.contains("shadow-of-the-void"));

        let written = fs::read_to_string(Path::new(&handle.book_dir).join("book.yaml")).unwrap();
        assert_eq!(format::parse_book(&written).unwrap(), handle.book);
    }

    #[test]
    fn create_book_increments_order_for_each_new_book() {
        let dir = tempfile::tempdir().unwrap();
        let input = |title: &str| CreateBookInput {
            project_dir: dir.path().to_string_lossy().into_owned(),
            series_id: "series-1".into(),
            title: title.into(),
            synopsis: "".into(),
        };

        let first = create_book(input("Shadow of the Void")).unwrap();
        let second = create_book(input("The Obsidian Gate")).unwrap();

        assert_eq!(first.book.order, 1);
        assert_eq!(second.book.order, 2);
    }

    #[test]
    fn lists_books_sorted_by_order() {
        let dir = tempfile::tempdir().unwrap();
        let input = |title: &str| CreateBookInput {
            project_dir: dir.path().to_string_lossy().into_owned(),
            series_id: "series-1".into(),
            title: title.into(),
            synopsis: "".into(),
        };
        create_book(input("Shadow of the Void")).unwrap();
        create_book(input("The Obsidian Gate")).unwrap();

        let result = list_books(&dir.path().to_string_lossy()).expect("should list books");

        assert!(result.warnings.is_empty());
        assert_eq!(result.books.len(), 2);
        assert_eq!(result.books[0].book.title, "Shadow of the Void");
        assert_eq!(result.books[1].book.title, "The Obsidian Gate");
    }

    #[test]
    fn creates_a_location_file_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let location = create_location(CreateLocationInput {
            book_dir: dir.path().to_string_lossy().into_owned(),
            book_id: "book-1".into(),
            name: "Aethelgard".into(),
            description: "The last free city.".into(),
        })
        .expect("should create the location");

        assert_eq!(location.name, "Aethelgard");

        let locations_dir = dir.path().join("locations");
        let entries: Vec<_> = fs::read_dir(&locations_dir).unwrap().collect();
        assert_eq!(entries.len(), 1, "expected exactly one location file");

        let written = fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
        assert_eq!(format::parse_location(&written).unwrap(), location);
    }

    #[test]
    fn lists_locations_for_a_book() {
        let dir = tempfile::tempdir().unwrap();
        let book_dir = dir.path().to_string_lossy().into_owned();
        create_location(CreateLocationInput {
            book_dir: book_dir.clone(),
            book_id: "book-1".into(),
            name: "Aethelgard".into(),
            description: "The last free city.".into(),
        })
        .unwrap();

        let result = list_locations(&book_dir).expect("should list locations");
        assert!(result.warnings.is_empty());
        assert_eq!(result.locations.len(), 1);
        assert_eq!(result.locations[0].name, "Aethelgard");
    }

    #[test]
    fn list_locations_is_empty_when_the_book_has_none_yet() {
        let dir = tempfile::tempdir().unwrap();
        let result = list_locations(&dir.path().to_string_lossy()).expect("should not fail");
        assert!(result.locations.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn lists_characters_for_a_book() {
        let dir = tempfile::tempdir().unwrap();
        let book_dir = dir.path().to_string_lossy().into_owned();
        create_character(CreateCharacterInput {
            book_dir: book_dir.clone(),
            book_id: "book-1".into(),
            name: "Lyra Vance".into(),
            role: "Protagonist".into(),
            bio: "".into(),
            attributes: HashMap::new(),
        })
        .unwrap();

        let result = list_characters(&book_dir).expect("should list characters");
        assert!(result.warnings.is_empty());
        assert_eq!(result.characters.len(), 1);
        assert_eq!(result.characters[0].name, "Lyra Vance");
    }

    #[test]
    fn creates_a_timeline_note_file_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let note = create_note(CreateNoteInput {
            book_dir: dir.path().to_string_lossy().into_owned(),
            book_id: "book-1".into(),
            title: "The Sealing".into(),
            note_type: NoteType::Timeline,
            content: "Year 0 of the Third Age.".into(),
        })
        .expect("should create the note");

        assert_eq!(note.note_type, NoteType::Timeline);

        let notes_dir = dir.path().join("notes");
        let entries: Vec<_> = fs::read_dir(&notes_dir).unwrap().collect();
        assert_eq!(entries.len(), 1, "expected exactly one note file");

        let written = fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
        assert_eq!(format::parse_note(&written).unwrap(), note);
    }

    #[test]
    fn lists_notes_of_both_types_for_a_book() {
        let dir = tempfile::tempdir().unwrap();
        let book_dir = dir.path().to_string_lossy().into_owned();
        create_note(CreateNoteInput {
            book_dir: book_dir.clone(),
            book_id: "book-1".into(),
            title: "The Sealing".into(),
            note_type: NoteType::Timeline,
            content: "Year 0.".into(),
        })
        .unwrap();
        create_note(CreateNoteInput {
            book_dir: book_dir.clone(),
            book_id: "book-1".into(),
            title: "The Void Walker Prophecy".into(),
            note_type: NoteType::Lore,
            content: "Long ago...".into(),
        })
        .unwrap();

        let result = list_notes(&book_dir).expect("should list notes");
        assert!(result.warnings.is_empty());
        assert_eq!(result.notes.len(), 2);
        assert!(result
            .notes
            .iter()
            .any(|n| n.note_type == NoteType::Timeline));
        assert!(result.notes.iter().any(|n| n.note_type == NoteType::Lore));
    }

    #[test]
    fn list_books_collects_a_warning_for_an_unparseable_book_yaml_instead_of_failing() {
        let dir = tempfile::tempdir().unwrap();
        create_book(CreateBookInput {
            project_dir: dir.path().to_string_lossy().into_owned(),
            series_id: "series-1".into(),
            title: "Shadow of the Void".into(),
            synopsis: "".into(),
        })
        .unwrap();

        let broken_dir = dir.path().join("broken-book");
        fs::create_dir_all(&broken_dir).unwrap();
        fs::write(
            broken_dir.join("book.yaml"),
            "not: valid: book: yaml: at: all",
        )
        .unwrap();

        let result = list_books(&dir.path().to_string_lossy()).expect("should not fail outright");

        assert_eq!(result.books.len(), 1, "the good book should still show up");
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("broken-book"));
    }
}
