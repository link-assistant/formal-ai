# Contributing to rust-ai-driven-development-pipeline-template

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/YOUR-USERNAME/rust-ai-driven-development-pipeline-template.git
   cd rust-ai-driven-development-pipeline-template
   ```

2. **Install Rust**

   Install Rust using rustup (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Install development tools**

   ```bash
   rustup component add rustfmt clippy
   cargo install rust-script
   ```

4. **Install pre-commit hooks** (optional but recommended)

   ```bash
   pip install pre-commit
   pre-commit install
   ```

5. **Build the project**

   ```bash
   cargo build
   ```

## Development Workflow

1. **Create a feature branch**

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes**

   - Write code following the project's style guidelines
   - Add tests for any new functionality
   - Update documentation as needed

3. **Run quality checks**

   ```bash
   # Format code
   cargo fmt

   # Run Clippy lints
   cargo clippy --all-targets --all-features

   # Check file sizes (requires rust-script)
   rust-script scripts/check-file-size.rs

   # Run all checks together
   cargo fmt --check && cargo clippy --all-targets --all-features && rust-script scripts/check-file-size.rs
   ```

4. **Run tests**

   ```bash
   # Run all tests
   cargo test

   # Run tests with verbose output
   cargo test --verbose

   # Run doc tests
   cargo test --doc

   # Run a specific test
   cargo test test_name
   ```

   CI caps each test-matrix job at 10 minutes. Rust's built-in `cargo test` runner does not provide a portable global per-test timeout, so wrap long-running network, IO, or async tests with explicit test-level deadlines. If a repository adopts `cargo nextest`, configure runner deadlines with options such as `--slow-timeout` and `--leak-timeout`.

5. **Add a changelog fragment**

   For any user-facing changes, create a changelog fragment:

   ```bash
   # Create a new file in changelog.d/
   # Format: YYYYMMDD_HHMMSS_description.md
   touch changelog.d/$(date +%Y%m%d_%H%M%S)_my_change.md
   ```

   Edit the file to document your changes:

   ```markdown
   ### Added
   - Description of new feature

   ### Fixed
   - Description of bug fix
   ```

   **Why fragments?** This prevents merge conflicts in CHANGELOG.md when multiple PRs are open simultaneously.

6. **Commit your changes**

   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

   Pre-commit hooks will automatically run and check your code.

7. **Push and create a Pull Request**

   ```bash
   git push origin feature/my-feature
   ```

   Then create a Pull Request on GitHub.

## Code Style Guidelines

This project uses:

- **rustfmt** for code formatting
- **Clippy** for linting with pedantic and nursery lints enabled
- **cargo test** for testing

### Code Standards

- Follow Rust idioms and best practices
- Use documentation comments (`///`) for all public APIs
- Write tests for all new functionality
- Keep functions focused and reasonably sized
- Keep files under 1000 lines
- Use meaningful variable and function names

### Documentation Format

Use Rust documentation comments:

```rust
/// Brief description of the function.
///
/// Longer description if needed.
///
/// # Arguments
///
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of when errors are returned
///
/// # Examples
///
/// ```
/// use my_package::example_function;
/// let result = example_function(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn example_function(arg1: i32, arg2: i32) -> i32 {
    arg1 + arg2
}
```

## Testing Guidelines

- Write tests for all new features
- Maintain or improve test coverage
- Use descriptive test names
- Organize tests in modules when appropriate
- Use `#[cfg(test)]` for test-only code

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod my_feature_tests {
        use super::*;

        #[test]
        fn test_basic_functionality() {
            assert_eq!(my_function(), expected_result);
        }

        #[test]
        fn test_edge_case() {
            assert_eq!(my_function(edge_case_input), expected_result);
        }
    }
}
```

## Project Conventions (recurring maintainer recommendations)

These conventions recur in almost every issue review. They are collected here so
contributors — human and AI — can apply them up front instead of rediscovering
them in review. They reflect the project's vision: a deterministic, symbolic
agent whose every answer is a projection of an append-only event log, with no
hardcoded prompt→answer tables.

1. **Mirror parity (Rust ↔ JS worker).** Every reasoning path in the Rust engine
   (`src/*.rs`) has a twin in the browser worker `src/web/formal_ai_worker.js`,
   so the CLI, library, HTTP server, Telegram bot, and website all answer the
   same prompt identically. A behavioural change in one **must** be mirrored in
   the other in the same PR. Name and comment the twin so the parity is obvious
   (e.g. "Mirrors `try_x` in `src/solver_handler_x.rs`").

2. **Data-driven seed, no hardcoded natural language in code (issues #386,
   #513).** Natural language is *data*, never a string literal in the engine.
   This applies to **both directions** of every reasoning path:
   - **Triggers / detection.** All multilingual phrases, surfaces, run verbs,
     shell tokens, concept summaries, and the tool registry live in
     `data/seed/*.lino`. Recognisers ask the lexicon for a *meaning* by role
     (`lexicon().meanings_with_role(ROLE_…)`) or load a named vocabulary
     (e.g. `seed::terminal_command_vocabulary()`); they never hardcode
     per-language phrase arrays or branch on literal user phrasings.
   - **Responses / output.** Every user-facing answer string is a template in
     `data/seed/multilingual-responses.lino` looked up by intent
     (`seed::response_for(intent, lang)` in Rust, `answerFor(...)` in the
     worker). Code fills placeholders like `{command}`; it does not embed the
     surrounding prose.
   - **Web front-end (React).** Every string the user sees in `src/web/app.js`
     — permission-panel titles, button labels, status words, onboarding copy,
     system messages — is a catalog entry in `src/web/i18n-catalog.lino`, looked
     up at render time via `t("<key>", params)` (the `window.FormalAiI18n`
     engine). Never pass a prose string literal as a child of `h(...)`; route it
     through `t(...)` so it follows the active UI language and fills placeholders
     like `{granted}/{total}`.

   The principle is **meanings ↔ naturalization**: a *meaning* (a slug grounded
   in seed data) can be *naturalized* into a natural-language surface, and any
   natural-language word can be *formalized* back into a meaning. Code only ever
   moves meanings around; the words live in the seed. Add a new cue or answer by
   editing the `.lino` file and declaring the role/intent — not by typing a
   phrase into `src/*.rs` or `formal_ai_worker.js`.

   This is enforced by CI, not just convention:
   - **Total reference-closure gate** (`tests/unit/total_closure.rs` →
     `scripts/audit-total-closure.py`, run by `cargo test --tests`). Every bare
     value token in any `data/seed/*.lino` must resolve to a defined meaning, a
     declared role, a cached dictionary lemma, or a Wikidata id. New vocabulary
     that resolves to nothing fails the build. Ground new tokens by running
     `python3 scripts/close-total.py` (idempotent; emits each unresolved token
     as a first-class meaning under `data/seed/closure-generated-*.lino`) until
     `python3 scripts/audit-total-closure.py` reports `unresolved_distinct: 0`.
   - **Worker seed parity checks.** Where the JS worker consumes a generated web
     seed copy, a `--check` guard fails the build on loader regressions and on
     drift in a present mirror (e.g. the “Check terminal vocabulary worker seed
     wiring” CI step runs
     `node experiments/issue-513-sync-worker-terminal.mjs --check`). Refresh
     the web seed copy with `scripts/sync-seed.sh` or by running the same script
     without `--check`.
   - **Web-UI hardcoded-string guard (#511).** `npm --prefix tests/e2e run
     check:web-hardcoded-ui` parses every `h(...)` call in `src/web/app.js` and
     fails the build when a child argument is a bare prose string literal, so new
     English text cannot leak into the UI. `npm --prefix tests/e2e run check:i18n`
     asserts every required key exists in all four locales and that sample
     interpolations render. When you add a web-UI string: add the key + all four
     translations in `src/web/i18n-catalog.lino`, register it in `REQUIRED_KEYS`
     in `tests/e2e/scripts/check-i18n-catalog.mjs`, and render it with `t(...)`.

   See `docs/design/no-hardcoded-natural-language.md` for the full rationale,
   the meanings ↔ naturalization model, and a worked example.

3. **Roles are declared, then generated.** When you add a meaning with a new
   `role`, declare it as a `ROLE_*` constant in `src/seed/roles/*.rs`, re-export
   it from `src/seed.rs`, and regenerate the registry with
   `python3 scripts/generate-role-registry.py` (keeps `data/seed/roles.lino` in
   lockstep; enforced by `reference_closure` tests).

4. **Supported-language coverage.** New conversational cues should cover the
   project's supported languages (currently en, ru, hi, zh). The
   `tests/e2e/scripts/check-*.mjs` guards fail a one-language change.

5. **Fix everywhere, not just the reported spot.** If a defect has more than one
   site (most do, because of mirror parity), fix all of them in the one PR.

6. **Reproduce first, then fix.** Add a failing test that reproduces the issue
   before implementing the fix; a bug fix without a reproducing regression test
   is treated as incomplete.

7. **When data is insufficient, add tracing.** If there is not enough signal to
   find a root cause, add debug output / a verbose mode (default **off**) and
   keep it in the code so the next iteration has the data.

8. **Case study per issue.** Download the issue's logs and data into
   `docs/case-studies/issue-{id}/` (raw JSON under `raw-data/`) and write a
   `README.md` that reconstructs the timeline, enumerates every requirement,
   finds the root cause(s), surveys prior art / existing libraries, and records
   the implemented fix and its verification.

9. **Report upstream when relevant.** If an issue is rooted in another
   repository we can file against, open an issue there with a reproducible
   example, a workaround, and a suggested fix.

10. **One PR per issue.** Plan and execute everything for an issue in a single
    pull request; commit atomic, individually useful steps so interrupted work
    stays preserved.

11. **Prefer the meta algorithm; drive Formal AI to solve its own tasks
    (direction set by issue #538).** The long-term way we develop this project is
    to treat every task as a message formalized into the meta language and to let
    Formal AI — driven through its own Agent CLI
    (<https://github.com/link-assistant/agent>) — reason about and solve the task
    by editing its own data (memory) and meta algorithm (reasoning), rather than
    a human hand-coding each answer. When Formal AI cannot yet perform a step,
    the goal is to improve the meta algorithm just enough that it can, verifying
    generality by phrasing the same request different ways and by reproducing the
    change in a clean repository copy driven by the Agent CLI.

    **Honest current status.** This is the stated direction, **not yet the
    enforced default.** The self-hosting loop (spawn a Formal AI server, hand it
    the issue, capture the Agent-CLI session JSON that reproduces the change) is a
    tracked programme — see `docs/case-studies/issue-538` (requirements R378–R386)
    and `ROADMAP.md`. Until it is wired up, contributors still make the change
    directly, but should: (a) express new knowledge as grounded seed *data* and
    reuse the existing meta-algorithm mechanisms rather than adding bespoke
    branches (conventions 2–3 above); (b) record any capability the Agent-CLI loop
    could not yet perform as a follow-up so the meta algorithm's gaps stay
    visible; and (c) when a task's requirements conflict (as issue #538's small
    concrete ask conflicts with its sweeping vision), surface the contradiction
    and separate the verifiable core from the tracked programme instead of
    silently doing part of everything.

## Pull Request Process

1. Ensure all tests pass locally
2. Update documentation if needed
3. Add a changelog fragment (see step 5 in Development Workflow)
4. Ensure the PR description clearly describes the changes
5. Link any related issues in the PR description
6. Wait for CI checks to pass
7. Address any review feedback

## Changelog Management

This project uses a fragment-based changelog system similar to [Scriv](https://scriv.readthedocs.io/) (Python) and [Changesets](https://github.com/changesets/changesets) (JavaScript).

### Creating a Fragment

```bash
# Create a new fragment with timestamp
touch changelog.d/$(date +%Y%m%d_%H%M%S)_description.md
```

### Fragment Categories

Use these categories in your fragments:

- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Features that will be removed in future
- **Removed**: Features that were removed
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### During Release

Fragments are automatically collected into CHANGELOG.md during the release process. The release workflow:

1. Collects all fragments
2. Updates CHANGELOG.md with the new version entry
3. Removes processed fragment files
4. Bumps the version in Cargo.toml
5. Creates a git tag and GitHub release

## Project Structure

```
.
├── .github/workflows/    # GitHub Actions CI/CD
├── changelog.d/          # Changelog fragments
│   ├── README.md         # Fragment instructions
│   └── *.md              # Individual changelog fragments
├── examples/             # Usage examples
├── scripts/              # Rust scripts (via rust-script)
├── src/
│   ├── lib.rs            # Library entry point
│   └── main.rs           # Binary entry point
├── tests/                # Integration tests
├── .gitignore            # Git ignore patterns
├── .pre-commit-config.yaml  # Pre-commit hooks
├── Cargo.toml            # Project configuration
├── CHANGELOG.md          # Project changelog
├── CONTRIBUTING.md       # This file
├── LICENSE               # Unlicense (public domain)
└── README.md             # Project README
```

## Release Process

This project uses semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Releases are managed through GitHub releases. To trigger a release:

1. Manually trigger the release workflow with a version bump type
2. Or: Update the version in Cargo.toml and push to main

## Getting Help

- Open an issue for bugs or feature requests
- Use discussions for questions and general help
- Check existing issues and PRs before creating new ones

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

Thank you for contributing!
