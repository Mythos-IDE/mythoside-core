use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_at: String,
    /// Convenience index of this series' entities, kept in sync (best-effort)
    /// by the create_*/delete_* commands — NOT the source of truth. Listing
    /// (list_books/list_characters/etc.) still scans the actual directories;
    /// this is just a human-readable summary living in the same file, so a
    /// glance at series.yaml shows what belongs to it without walking the
    /// filesystem. `#[serde(default)]` so series.yaml files written before
    /// this field existed still parse.
    #[serde(default)]
    pub book_ids: Vec<String>,
    #[serde(default)]
    pub character_ids: Vec<String>,
    #[serde(default)]
    pub location_ids: Vec<String>,
    #[serde(default)]
    pub note_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: String,
    pub series_id: String,
    pub title: String,
    pub synopsis: String,
    pub order: u32,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: String,
    pub book_id: String,
    pub title: String,
    pub order: u32,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: String,
    pub chapter_id: String,
    pub title: String,
    pub order: u32,
    pub tags: Vec<String>,
    pub characters: Vec<String>,
    pub created_at: String,
    pub content: String, // Markdown body
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub id: String,
    /// Series-level, not book-level: the same character can appear across
    /// multiple books in the series (Book 1, Book 2, ...), so ownership
    /// can't be pinned to a single book.
    pub series_id: String,
    pub name: String,
    pub role: String,
    pub attributes: HashMap<String, String>,
    pub created_at: String,
    pub bio: String, // Markdown body
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: String,
    /// Series-level for the same reason as `Character::series_id` — a
    /// location can recur across multiple books.
    pub series_id: String,
    pub name: String,
    pub created_at: String,
    pub description: String, // Markdown body
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum NoteType {
    Lore,
    Timeline,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    /// Series-level for the same reason as `Character::series_id`.
    pub series_id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub note_type: NoteType,
    pub created_at: String,
    pub content: String, // Markdown body
}
