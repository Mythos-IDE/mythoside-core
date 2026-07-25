# Mythos Core Architecture

This document describes the high-level architecture, design principles, and module boundaries of `mythoside-core`. It serves as the primary reference for contributors to understand why the workspace is structured this way and to guide future design decisions.

---

## 1. Executive Summary & Design Rationale

`mythoside-core` is the local core backend for MythosIDE. It is designed as a local-first service communicating over standard I/O (stdio) via a line-delimited JSON-RPC interface.

To support clear domain boundaries, maintain compilation speed, and ensure deterministic data flow, `mythoside-core` is structured as a **Cargo Workspace** composed of five internal domain crates and one top-level facade crate:

1. **`mythoside-common`**: Shared utilities (slugification, timestamp formatting).
2. **`mythoside-narrative`**: Pure domain data models (Series, Book, Chapter, Character, Location, Note).
3. **`mythoside-parser`**: Frontmatter splitting and YAML/Markdown serialization and deserialization.
4. **`mythoside-workspace`**: Project directory layout, document state management, filesystem operations, and file watching.
5. **`mythoside-rpc`**: Stdio transport framing, request deserialization, and method dispatching.
6. **`mythoside-core`**: The workspace root executable binary (`main.rs`) and backward-compatible public facade crate.

---

## 2. Architectural Principles

All code organization and refactoring in this repository must comply with the following core architectural rules:

1. **A crate represents a domain capability.** A crate is created only when it owns a distinct business capability or architectural layer with clean input/output boundaries.
2. **Do not crate-split to reduce file count.** File count alone is never a justification for a crate. Rust modules (`mod`) provide sufficient namespace encapsulation within a crate.
3. **No placeholder or speculative crates.** Never create empty crates, TODO crates, or speculative abstractions for hypothetical future features. Crates are extracted only when implementation code exists.
4. **Dependencies flow strictly downward (Acyclic).** Higher-level crates depend on lower-level crates. Lower-level crates never depend on, reference, or import higher-level crates.
5. **Prefer stable APIs over clever architectures.** Explicit, simple function signatures and standard `Result<T, String>` error types are preferred over complex generic traits or unnecessary dynamic dispatch.
6. **Behavior is more important than elegance.** Refactoring must preserve 100% behavioral equivalence and maintain zero-breaking-change compatibility for external callers.
7. **Extract boundaries only upon repeated friction.** Extract a module into a separate crate only when independent compilation, testing, or domain boundaries demonstrate concrete engineering benefit.

---

## 3. Dependency Graph

The workspace enforces a strict, acyclic, top-down dependency hierarchy:

```
                      +-------------------+
                      |  mythoside-core   | (Facade & Executable Binary)
                      +---------+---------+
                                |
                                v
                      +-------------------+
                      |   mythoside-rpc   | (Transport & Dispatching)
                      +---------+---------+
                                |
                                v
                      +-------------------+
                      | mythoside-workspace| (Document State & I/O)
                      +----+--------+-----+
                           |        |
             +-------------+        +------------+
             |                                   |
             v                                   v
+-------------------+                 +-------------------+
|  mythoside-parser |                 | mythoside-common  |
+---------+---------+                 +-------------------+
          |
          v
+-------------------+
|mythoside-narrative|
+-------------------+
```

### Text Hierarchy Representation

```
mythoside-core (Facade / Binary)
├── mythoside-rpc
│   ├── mythoside-workspace
│   │   ├── mythoside-parser
│   │   │   └── mythoside-narrative
│   │   ├── mythoside-narrative
│   │   └── mythoside-common
│   └── mythoside-narrative
```

### Graph Rules

- **`mythoside-narrative`** has **zero** internal workspace dependencies. It depends only on standard libraries and serialization derives (`serde`, `specta`).
- **`mythoside-common`** has **zero** internal workspace dependencies.
- **`mythoside-parser`** depends **only** on `mythoside-narrative`.
- **`mythoside-workspace`** depends on `mythoside-common`, `mythoside-narrative`, and `mythoside-parser`.
- **`mythoside-rpc`** depends on `mythoside-workspace` and `mythoside-narrative`.
- **`mythoside-core`** depends on all internal crates to act as the single facade entry point.

---

## 4. Why `mythoside-core` Exists as a Facade

External consumers—specifically the Tauri desktop application shell spawning the core sidecar process—interact with `mythoside-core`. 

`mythoside-core` serves as a facade to:
1. **Maintain Zero-Breakage Public API**: It re-exports all domain modules (`manuscript::models`, `manuscript::format`, `manuscript::commands`, `watcher`, `rpc`). External code importing `mythoside_core::manuscript::models::Book` or `mythoside_core::rpc::dispatch` continues to compile without modification.
2. **Isolate Entry Point**: `main.rs` contains the stdio main loop that initializes state, reads stdin lines, invokes `mythoside_rpc::dispatch`, and writes responses to stdout.

---

## 5. Current Absence of Additional Crates

You may notice that logical concepts such as *Analysis Engine*, *Diagnostics*, *Knowledge Graph*, *Storage Engine*, *Search*, or *AI* do not exist as crates.

This is intentional:
- No code for complex static analysis, vector search, or database indexing currently exists in the codebase.
- Manuscript storage currently uses direct, plain-file disk reads and writes (`std::fs`), which fulfills the product's local-first requirement ("your manuscript is plain Markdown/YAML files on disk").
- Creating empty or speculative crates would violate Principle #3 ("No placeholder or speculative crates").

For guidelines on when and how future crates should be extracted, see [`docs/architecture/future.md`](docs/architecture/future.md).
For a detailed specification of each current crate, see [`docs/architecture/workspace.md`](docs/architecture/workspace.md).
