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
- Name every sqlx migration `NNNN_migration.sql`, where `NNNN` starts at `0001` and increments by one with zero padding.

### Planning

- When planning UI changes, run `scripts/plan`, then populate the new preview with the UI under review. The command
  creates `plans/plan-TIMESTAMP.html` using the current Unix timestamp and creates or updates the `plan.html` symlink.
- In `plan.html`, show only the UI under review. Omit commentary unless requested.

### Code Reviews

- Use subagents for code reviews. When a review prompt names `Claude` or `Codex`, use the following model and effort:
    - `Claude`: Claude Code CLI with Fable 5 at max effort.
    - `Codex` (subagent): Codex CLI with GPT-5.6 Sol at max effort.
- Give each new subagent just enough context so review-fix loops do not drag on by rediscovering the same issues,
  over-engineering, or going too far down rabbit holes.
- Subagents must review the entire change thoroughly and not stop after finding the first few issues.
- Reviews may extend beyond the uncommitted changes when useful. Report and address other issues noticed along the way.
- Assign each issue a severity of high, medium, or low.
- Have every issue confirmed by at least one other agent, either the parent agent or another subagent.

### Style

- Name functions imperatively (`do_this`, `find_that`). Predicates read as questions (`is_live`, `has_item`).
- Name Rust test functions `test_<behavior>` in the present tense.
- Keep code comments, documentation comments, and Markdown prose within 120 columns; do not wrap them earlier.
- Prefer clarity to brevity. Accept added verbosity when it prevents ambiguity.

### Documentation

- Do NOT edit AGENTS.md, CLAUDE.md or README.md unless explicitly told to do so.
- Use plain English, ELI5 style without replacing canonical, context-appropriate terms with less precise words.
- Use terms consistently. Avoid synonyms for variety.

### Git

- Before drafting a commit message, review recent messages with `git log` and match their writing style.
- Use a single-line, imperative commit subject without a trailing period.
- After a blank line, use a flat bullet-point list of short sentences describing the change.
- Never commit or push unless explicitly asked.
- Never add emojis, em-dashes or AI attribution to commits or pull requests.

### Occam's Razor

- Prefer deleting, consolidating, or reusing existing code before adding code.
- Choose the simplest design that fully satisfies the current requirements; add abstractions, extensions, dependencies
  etc. only when concrete requirements or repeated patterns justify them.
