use super::format;
use super::models::{Character, Series};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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
pub struct CreateSeriesInput {
    /// Filesystem path to the project root — `series.yaml` is written
    /// directly there (a project root *is* a series, per the folder
    /// convention: `<project_dir>/series.yaml`, `<project_dir>/<book-slug>/
    /// book.yaml`, etc.).
    pub project_dir: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
}

pub fn create_series(input: CreateSeriesInput) -> Result<Series, String> {
    let series = Series {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        description: input.description,
        created_at: now_iso8601(),
    };

    let project_dir = Path::new(&input.project_dir);
    fs::create_dir_all(project_dir).map_err(|e| e.to_string())?;

    let contents = format::serialize_series(&series)?;
    fs::write(project_dir.join("series.yaml"), contents).map_err(|e| e.to_string())?;

    Ok(series)
}

pub fn get_series(project_dir: &str) -> Result<Series, String> {
    let file_path = Path::new(project_dir).join("series.yaml");
    let contents = fs::read_to_string(&file_path)
        .map_err(|e| format!("could not read {}: {e}", file_path.display()))?;
    format::parse_series(&contents)
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
    fn creates_a_series_yaml_at_the_project_root() {
        let dir = tempfile::tempdir().unwrap();
        let input = CreateSeriesInput {
            project_dir: dir.path().to_string_lossy().into_owned(),
            title: "The Aethelgard Chronicles".into(),
            description: "An epic fantasy series.".into(),
        };

        let series = create_series(input).expect("should create the series");
        assert_eq!(series.title, "The Aethelgard Chronicles");
        assert!(!series.id.is_empty());

        let written = fs::read_to_string(dir.path().join("series.yaml")).unwrap();
        assert_eq!(format::parse_series(&written).unwrap(), series);
    }

    #[test]
    fn gets_a_previously_created_series() {
        let dir = tempfile::tempdir().unwrap();
        let created = create_series(CreateSeriesInput {
            project_dir: dir.path().to_string_lossy().into_owned(),
            title: "The Aethelgard Chronicles".into(),
            description: "".into(),
        })
        .unwrap();

        let fetched = get_series(&dir.path().to_string_lossy()).expect("should read the series");
        assert_eq!(fetched, created);
    }

    #[test]
    fn get_series_fails_when_series_yaml_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(get_series(&dir.path().to_string_lossy()).is_err());
    }
}
