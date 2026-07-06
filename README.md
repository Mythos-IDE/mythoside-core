# MythosIDE

English · [Türkçe](./README.TR.md)
[![License: FSL-1.1-ALv2](https://img.shields.io/badge/license-FSL--1.1--ALv2-blue)](./LICENSE.md)
[![GitHub Discussions](https://img.shields.io/github/discussions/Mythos-IDE/mythoside-core)](https://github.com/Mythos-IDE/mythoside-core/discussions)
[![GitHub issues](https://img.shields.io/github/issues/Mythos-IDE/mythoside-core)](https://github.com/Mythos-IDE/mythoside-core/issues)

**A writer's IDE for novelists building complex worlds.**

MythosIDE merges the structured, long-form writing approach of tools like
Scrivener with the intelligent, context-aware experience of a software IDE —
purpose-built for fantasy, sci-fi, and epic fiction writers who are tired of
stitching together five different apps to keep their world consistent.

> Status: early development. Expect rough edges. Contributions and feedback
> are very welcome.

## Why MythosIDE?

- **Structural hierarchy built for fiction** — Series → Book → Chapter →
  Scene, not a generic outliner you have to configure yourself.
- **Intelligent world-building** — type `@CharacterName` and get an instant
  contextual profile card without leaving your draft.
- **Local-first, always** — the source of truth is plain Markdown + YAML
  frontmatter on your disk. No lock-in, no "what happens to my novel if this
  company shuts down."
- **Fast under the hood** — a local SQLite (FTS5) index makes
  cross-referencing and relationship queries ("which clan appears in Chapter
  4?") instant, without ever becoming the source of truth itself.

## Tech stack

- [Tauri](https://tauri.app/) — lightweight, native-feeling cross-platform
  desktop shell
- TypeScript / Node.js
- SQLite + FTS5 for local indexing
- A customized web-based text editor (ProseMirror/Monaco-based)

## Getting started

Setup instructions will land here as the project stabilizes. Track progress
in [Issues](../../issues) and [Discussions](../../discussions).

## License

MythosIDE is source-available under the
[Functional Source License, v1.1 (ALv2 Future License)](./LICENSE.md). In
short: you're free to use, read, modify, and self-host it for your own
writing — you just can't repackage it as a competing commercial product or
service. Each release converts to Apache 2.0 automatically two years after
publication.

"MythosIDE" and its logo are trademarks of the project and are not covered by
the license above — see [LICENSE.md](./LICENSE.md) for details.

## Contributing

See [CONTRIBUTING.md](https://github.com/Mythos-IDE/.github/blob/main/CONTRIBUTING.md) before opening a pull request.

## Security

See [SECURITY.md](https://github.com/Mythos-IDE/.github/blob/main/SECURITY.md) for how to report vulnerabilities.
