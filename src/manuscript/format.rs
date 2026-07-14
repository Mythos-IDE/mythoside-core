use super::models::{Book, Chapter, Character, Location, Note, NoteType, Scene, Series};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

fn parse_yaml<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    serde_norway::from_str(text).map_err(|e| e.to_string())
}

fn serialize_yaml<T: Serialize>(value: &T) -> Result<String, String> {
    serde_norway::to_string(value).map_err(|e| e.to_string())
}

/// Splits a Markdown-with-YAML-frontmatter file into its `(frontmatter,
/// body)` halves. Frontmatter starts with a `---` line and ends at the next
/// line that is exactly `---`; everything after is the (trimmed) body.
fn split_frontmatter(file_text: &str) -> Result<(&str, &str), String> {
    let rest = file_text
        .strip_prefix("---\r\n")
        .or_else(|| file_text.strip_prefix("---\n"))
        .ok_or_else(|| "file does not start with a --- frontmatter block".to_string())?;

    let end = rest
        .find("\n---")
        .ok_or_else(|| "frontmatter block is not closed with ---".to_string())?;

    let frontmatter = &rest[..end];
    let after_delimiter = &rest[end + "\n---".len()..];
    let body = after_delimiter
        .strip_prefix("\r\n")
        .or_else(|| after_delimiter.strip_prefix('\n'))
        .unwrap_or(after_delimiter);

    Ok((frontmatter, body.trim()))
}

fn join_frontmatter(frontmatter_yaml: &str, body: &str) -> String {
    format!("---\n{frontmatter_yaml}---\n{body}\n")
}

// --- Series: <project-root>/series.yaml ---
pub fn parse_series(text: &str) -> Result<Series, String> {
    parse_yaml(text)
}

pub fn serialize_series(series: &Series) -> Result<String, String> {
    serialize_yaml(series)
}

// --- Book: <project-root>/<book-slug>/book.yaml ---
pub fn parse_book(text: &str) -> Result<Book, String> {
    parse_yaml(text)
}

pub fn serialize_book(book: &Book) -> Result<String, String> {
    serialize_yaml(book)
}

// --- Chapter: .../chapters/<order>-<chapter-slug>/chapter.yaml ---
pub fn parse_chapter(text: &str) -> Result<Chapter, String> {
    parse_yaml(text)
}

pub fn serialize_chapter(chapter: &Chapter) -> Result<String, String> {
    serialize_yaml(chapter)
}

// --- Scene: .../chapters/<...>/<order>-<scene-slug>.md ---
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneMeta {
    id: String,
    chapter_id: String,
    title: String,
    order: u32,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    characters: Vec<String>,
    created_at: String,
}

pub fn parse_scene(file_text: &str) -> Result<Scene, String> {
    let (frontmatter, body) = split_frontmatter(file_text)?;
    let meta: SceneMeta = parse_yaml(frontmatter)?;
    Ok(Scene {
        id: meta.id,
        chapter_id: meta.chapter_id,
        title: meta.title,
        order: meta.order,
        tags: meta.tags,
        characters: meta.characters,
        created_at: meta.created_at,
        content: body.to_string(),
    })
}

pub fn serialize_scene(scene: &Scene) -> Result<String, String> {
    let meta = SceneMeta {
        id: scene.id.clone(),
        chapter_id: scene.chapter_id.clone(),
        title: scene.title.clone(),
        order: scene.order,
        tags: scene.tags.clone(),
        characters: scene.characters.clone(),
        created_at: scene.created_at.clone(),
    };
    let yaml = serialize_yaml(&meta)?;
    Ok(join_frontmatter(&yaml, &scene.content))
}

// --- Character: .../characters/<slug>.md ---
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CharacterMeta {
    id: String,
    book_id: String,
    name: String,
    role: String,
    #[serde(default)]
    attributes: HashMap<String, String>,
    created_at: String,
}

pub fn parse_character(file_text: &str) -> Result<Character, String> {
    let (frontmatter, body) = split_frontmatter(file_text)?;
    let meta: CharacterMeta = parse_yaml(frontmatter)?;
    Ok(Character {
        id: meta.id,
        book_id: meta.book_id,
        name: meta.name,
        role: meta.role,
        attributes: meta.attributes,
        created_at: meta.created_at,
        bio: body.to_string(),
    })
}

pub fn serialize_character(character: &Character) -> Result<String, String> {
    let meta = CharacterMeta {
        id: character.id.clone(),
        book_id: character.book_id.clone(),
        name: character.name.clone(),
        role: character.role.clone(),
        attributes: character.attributes.clone(),
        created_at: character.created_at.clone(),
    };
    let yaml = serialize_yaml(&meta)?;
    Ok(join_frontmatter(&yaml, &character.bio))
}

// --- Location: .../locations/<slug>.md ---
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocationMeta {
    id: String,
    book_id: String,
    name: String,
    created_at: String,
}

pub fn parse_location(file_text: &str) -> Result<Location, String> {
    let (frontmatter, body) = split_frontmatter(file_text)?;
    let meta: LocationMeta = parse_yaml(frontmatter)?;
    Ok(Location {
        id: meta.id,
        book_id: meta.book_id,
        name: meta.name,
        created_at: meta.created_at,
        description: body.to_string(),
    })
}

pub fn serialize_location(location: &Location) -> Result<String, String> {
    let meta = LocationMeta {
        id: location.id.clone(),
        book_id: location.book_id.clone(),
        name: location.name.clone(),
        created_at: location.created_at.clone(),
    };
    let yaml = serialize_yaml(&meta)?;
    Ok(join_frontmatter(&yaml, &location.description))
}

// --- Note: .../notes/<slug>.md ---
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteMeta {
    id: String,
    book_id: String,
    title: String,
    #[serde(rename = "type")]
    note_type: NoteType,
    created_at: String,
}

pub fn parse_note(file_text: &str) -> Result<Note, String> {
    let (frontmatter, body) = split_frontmatter(file_text)?;
    let meta: NoteMeta = parse_yaml(frontmatter)?;
    Ok(Note {
        id: meta.id,
        book_id: meta.book_id,
        title: meta.title,
        note_type: meta.note_type,
        created_at: meta.created_at,
        content: body.to_string(),
    })
}

pub fn serialize_note(note: &Note) -> Result<String, String> {
    let meta = NoteMeta {
        id: note.id.clone(),
        book_id: note.book_id.clone(),
        title: note.title.clone(),
        note_type: note.note_type.clone(),
        created_at: note.created_at.clone(),
    };
    let yaml = serialize_yaml(&meta)?;
    Ok(join_frontmatter(&yaml, &note.content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_series_through_series_yaml() {
        let series = Series {
            id: "series-1".into(),
            title: "The Aethelgard Chronicles".into(),
            description: "An epic fantasy series.".into(),
            created_at: "2024-07-03T10:26:40Z".to_string(),
        };
        let parsed = parse_series(&serialize_series(&series).unwrap()).unwrap();
        assert_eq!(parsed, series);
    }

    #[test]
    fn round_trips_a_book_through_book_yaml() {
        let book = Book {
            id: "book-1".into(),
            series_id: "series-1".into(),
            title: "Shadow of the Void".into(),
            synopsis: "The first book.".into(),
            order: 1,
            created_at: "2024-07-03T10:26:40Z".to_string(),
        };
        let parsed = parse_book(&serialize_book(&book).unwrap()).unwrap();
        assert_eq!(parsed, book);
    }

    #[test]
    fn round_trips_a_chapter_through_chapter_yaml() {
        let chapter = Chapter {
            id: "chapter-1".into(),
            book_id: "book-1".into(),
            title: "The Obsidian Gate".into(),
            order: 1,
            created_at: "2024-07-03T10:26:40Z".to_string(),
        };
        let parsed = parse_chapter(&serialize_chapter(&chapter).unwrap()).unwrap();
        assert_eq!(parsed, chapter);
    }

    #[test]
    fn round_trips_a_scene_through_markdown_and_frontmatter() {
        let scene = Scene {
            id: "scene-1".into(),
            chapter_id: "chapter-1".into(),
            title: "The Void Begins".into(),
            order: 1,
            tags: vec!["action".into(), "reveal".into()],
            characters: vec!["lyra-vance".into(), "silas-thorne".into()],
            created_at: "2024-07-03T10:26:40Z".to_string(),
            content: "The air in the chamber was heavy with the scent of ancient dust and ozone."
                .into(),
        };
        let file = serialize_scene(&scene).unwrap();
        assert!(file.contains("---"));
        assert!(file.contains("The air in the chamber"));
        assert_eq!(parse_scene(&file).unwrap(), scene);
    }

    #[test]
    fn round_trips_a_character_through_markdown_and_frontmatter() {
        let mut attributes = HashMap::new();
        attributes.insert("age".to_string(), "24".to_string());
        attributes.insert("home".to_string(), "Aethelgard".to_string());
        let character = Character {
            id: "lyra-vance".into(),
            book_id: "book-1".into(),
            name: "Lyra Vance".into(),
            role: "Protagonist".into(),
            attributes,
            created_at: "2024-07-03T10:26:40Z".to_string(),
            bio: "Stealthy, skilled in alchemy.".into(),
        };
        let file = serialize_character(&character).unwrap();
        assert_eq!(parse_character(&file).unwrap(), character);
    }

    #[test]
    fn round_trips_a_location_through_markdown_and_frontmatter() {
        let location = Location {
            id: "aethelgard".into(),
            book_id: "book-1".into(),
            name: "Aethelgard".into(),
            created_at: "2024-07-03T10:26:40Z".to_string(),
            description: "The last free city.".into(),
        };
        let file = serialize_location(&location).unwrap();
        assert_eq!(parse_location(&file).unwrap(), location);
    }

    #[test]
    fn round_trips_a_lore_note_through_markdown_and_frontmatter() {
        let note = Note {
            id: "note-1".into(),
            book_id: "book-1".into(),
            title: "The Void Walker Prophecy".into(),
            note_type: NoteType::Lore,
            created_at: "2024-07-03T10:26:40Z".to_string(),
            content: "Long ago, the Void Walker was sealed beneath Aethelgard.".into(),
        };
        let file = serialize_note(&note).unwrap();
        assert!(file.contains("type: lore"));
        assert_eq!(parse_note(&file).unwrap(), note);
    }

    #[test]
    fn round_trips_a_timeline_note_through_markdown_and_frontmatter() {
        let note = Note {
            id: "note-2".into(),
            book_id: "book-1".into(),
            title: "The Sealing".into(),
            note_type: NoteType::Timeline,
            created_at: "2024-07-03T10:26:40Z".to_string(),
            content: "Year 0 of the Third Age.".into(),
        };
        let file = serialize_note(&note).unwrap();
        assert!(file.contains("type: timeline"));
        assert_eq!(parse_note(&file).unwrap(), note);
    }

    #[test]
    fn defaults_optional_scene_fields_when_frontmatter_omits_them() {
        let file = "---\nid: scene-2\nchapterId: chapter-1\ntitle: Bare Scene\norder: 2\ncreatedAt: \"2024-07-03T10:26:40Z\"\n---\nJust prose.";
        let scene = parse_scene(file).unwrap();
        assert!(scene.tags.is_empty());
        assert!(scene.characters.is_empty());
    }

    #[test]
    fn rejects_a_note_file_with_an_invalid_type() {
        let file = "---\nid: note-3\nbookId: book-1\ntitle: Bad\ntype: mythical\ncreatedAt: \"2024-07-03T10:26:40Z\"\n---\nContent.";
        assert!(parse_note(file).is_err());
    }

    #[test]
    fn rejects_metadata_missing_a_required_field() {
        assert!(parse_book("id: book-2\ntitle: Missing seriesId\norder: 1").is_err());
    }
}
