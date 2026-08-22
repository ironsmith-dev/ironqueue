# AGENTS.md

See [README.md](README.md).

## Rules

### Development

- This project has no users yet. Break API and database schemas freely without worrying about backward compatibility.
- Never use `unwrap()` or `expect()` outside tests.
- Use `thiserror` for library errors and `anyhow` for application errors.
- After a unit of work, run the `get_file_problems` tool and investigate reported errors and warnings. Fix confirmed
  problems and suppress false positives with the narrowest possible scope. Then run the `reformat_file` tool on every
  Markdown file edited if any. Then run the `prek run --all-files` command.
- Start dependencies with `docker compose up -d --wait` before developing or testing.
- Every SQL query must have a matching integration test against the Postgres instance in Docker Compose.
- `compose.yaml` runs with `synchronous_commit=off` for speed, so it is not durable: a crash-recovery test must set
  `synchronous_commit = on` for its own database first, or it asserts against PostgreSQL's configuration rather than
  against this crate.
- Use runtime `sqlx` query functions, not the compile-time macro variants.
- Name every sqlx migration `NNNN_migration.sql`, where `NNNN` is the zero-padded version, consecutive from `0001`.

### Style

- Name functions imperatively (`do_this`, `find_that`). Predicates read as questions (`is_live`, `has_item`).
- Name Rust test functions `test_<behavior>` in the present tense.
- Keep code comments, documentation comments, and Markdown prose within 120 columns; do not wrap them earlier.
- Prefer clarity to brevity. Accept added verbosity when it prevents ambiguity.

### Documentation

- Do NOT edit AGENTS.md, CLAUDE.md or README.md unless explicitly told to do so.
- Use plain English ELI5 style

### Git

- Use a single-line, imperative commit subject without a trailing period.
- After a blank line, use a flat bullet-point list of short sentences describing the change.
- Never commit or push unless explicitly asked.
- Never add emojis, em-dashes or AI attribution to commits or pull requests.

### Occam's Razor

- Prefer deleting, consolidating, or reusing existing code before adding code.
- Choose the simplest design that fully satisfies the current requirements; add abstractions, extensions, or
  dependencies only when concrete requirements or repeated patterns justify them.
