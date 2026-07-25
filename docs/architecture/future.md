# Future Architecture Strategy

> [!IMPORTANT]
> **Do NOT implement these crates today.**
> This document describes how the workspace architecture may evolve in the future when real product requirements demand it. No code or empty placeholder crates should be created until the specific preconditions outlined below are met.

---

## Extraction Decision Framework

A new crate must **NEVER** be created for any of the following non-reasons:
- ❌ To lower the line count of an existing file.
- ❌ To satisfy aesthetic preference for "many small crates".
- ❌ In anticipation of a feature that has not yet been requested or scheduled.
- ❌ Because another project uses a similar crate structure.

A new crate **MAY ONLY** be extracted when:
1. Concrete implementation code exists and is actively being written.
2. The code represents an independent domain capability with clear input/output types.
3. Combining it in an existing crate creates build friction, excessive dependency bloat, or circular dependency risks.

---

## Potential Future Crates

### 1. `mythoside-analysis` (Analysis Engine)

#### When It Should Exist
When MythosIDE introduces passive or active manuscript rule checks—such as character consistency verification, scene timeline chronology validation, word count tracking, or prose style analysis.

#### Code That Would Move Here
- Static manuscript consistency check logic.
- Rule execution routines evaluating `Series`, `Book`, and `Chapter` data.

#### Problem That Must Appear Before Extraction
Complex validation rules begin cluttering document management logic in `mythoside-workspace` or require heavy third-party natural language / linting dependencies.

#### What Should NEVER Trigger Extraction
Simple field validation (e.g. checking that a title is non-empty) or basic slugification.

---

### 2. `mythoside-diagnostics` (Diagnostic Types & Reporting)

#### When It Should Exist
When manuscript analysis errors, frontmatter parsing warnings, and prose linting results require a unified, structured reporting format (diagnostic severity, error codes, source code spans, line/column ranges, and inline fix suggestions).

#### Code That Would Move Here
- Unparseable file warning strings currently accumulated in `ListBooksOutput::warnings`, `ListSeriesOutput::warnings`, etc.

#### Problem That Must Appear Before Extraction
The simple `warnings: Vec<String>` pattern currently returned by `scan_entities` becomes insufficient for rich UI error reporting (e.g., highlighting specific lines in an editor).

#### What Should NEVER Trigger Extraction
Having raw string error messages in `Result<T, String>`.

---

### 3. `mythoside-storage` (Storage & Indexing Engine)

#### When It Should Exist
When direct disk directory scanning (`std::fs::read_dir` on every query) becomes a performance bottleneck for large manuscripts (e.g., thousands of chapters), requiring a persistent local database (SQLite/DuckDB) or cached binary metadata index.

#### Code That Would Move Here
- Best-effort series index file maintenance (`update_series_yaml`).
- Directory scanning helpers (`scan_entities`, `delete_entity_by_id`, `update_entity_by_id`).

#### Problem That Must Appear Before Extraction
Manuscript loading latency measured on real large projects exceeds acceptable UI thresholds, requiring local index caching.

#### What Should NEVER Trigger Extraction
Handling standard plain-file reads and writes for small-to-medium manuscripts.

---

### 4. `mythoside-knowledge-graph` (Entity Relationship Graph)

#### When It Should Exist
When MythosIDE introduces cross-entity relationship visualization (e.g., Character A is related to Character B, Character X appears in Chapter Y at Location Z, Lore Note N is linked to Event E).

#### Code That Would Move Here
- Entity link parsing from `Chapter::characters`, `Chapter::tags`, and `Note::note_type`.

#### Problem That Must Appear Before Extraction
The frontend or backend needs to execute graph queries (e.g., "Find all chapters where Character A and Character B interact") that cannot be efficiently answered by linear vector filtering.

#### What Should NEVER Trigger Extraction
Storing basic string ID lists (`character_ids`, `location_ids`) on `Series`.

---

### 5. `mythoside-search` (Full-Text & Vector Search)

#### When It Should Exist
When MythosIDE implements local full-text search, fuzzy search, or semantic vector retrieval over chapter prose and lore notes.

#### Code That Would Move Here
- Full-text token indexing and search query execution.

#### Problem That Must Appear Before Extraction
In-memory string matching (`contains`) is no longer sufficient for multi-book manuscript search.

#### What Should NEVER Trigger Extraction
Simple substring filtering in memory.

---

### 6. `mythoside-ai` (Local AI & LLM Assistants)

#### When It Should Exist
When MythosIDE integrates local LLM inference, embedding generation, or AI-assisted prose generation features.

#### Code That Would Move Here
- Prompt framing, local LLM bindings, and embedding vector pipelines.

#### Problem That Must Appear Before Extraction
Heavy ML runtime dependencies (e.g., ONNX, llama.cpp bindings) would slow down compilation of core workspace code if included in standard builds.

#### What Should NEVER Trigger Extraction
Basic static text generation or template expansion.

---

## Summary Checklist for Future Contributors

Before creating a new crate, ask yourself:
1. *Is there working code for this feature right now?*
2. *Does this code belong to a distinct domain capability?*
3. *Can this code live inside an existing crate as a `mod` without causing circular dependencies or build bloat?*

If the answer to #3 is **Yes**, keep it as a module in an existing crate.
Only create a new crate when the answer to #3 is **No**.
