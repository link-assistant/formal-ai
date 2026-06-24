# Issue 440 Case Study: List-Files Program Output

## Problem

Issue #440 reported that generated "list files" program answers were confusing
for Python users:

- The sample output showed Rust project files (`Cargo.toml`, `README.md`,
  `main.rs`) even when the answer asked the user to save Python as `main.py`.
- The "not run" status and "Copy the snippet..." instruction were rendered as
  one paragraph in the browser response.
- Light-theme markdown code blocks still used a dark code surface.

The original screenshots are stored in
[`screenshots/reported-output-1.png`](screenshots/reported-output-1.png) and
[`screenshots/reported-output-2.png`](screenshots/reported-output-2.png). Raw
issue and PR metadata is preserved under [`raw-data/`](raw-data/).

## Reproduction

The failing Rust-engine reproduction was added first in
`tests/unit/specification/code_generation/single_turn.rs`:

```text
Write me a Python program that lists files in the current directory in reverse-sorted order
```

Before the fix, the answer ended with:

```text
main.rs
README.md
Cargo.toml
```

The failing log is
[`raw-data/repro-unit-before.log`](raw-data/repro-unit-before.log). The fixed
log is [`raw-data/repro-unit-after.log`](raw-data/repro-unit-after.log).

## Root Cause

The catalog stored one static `ProgramTask.output` per task. The list-files
tasks used a Rust-shaped sample directory, and both the Rust answer renderer and
the browser worker reused that task output for every language. The browser
worker also encoded the "Copy the snippet..." sentence inside the same localized
`notRun` string, so Markdown had no paragraph break between status and action.

The CSS problem was separate: `.markdown-body pre`, `.code-block`, and the
syntax token palette were hard-coded to a dark code surface regardless of the
active app theme.

## Fix

The Rust catalog now resolves displayed output through
`ProgramSpec::expected_output()`. For list-files tasks, it builds a deterministic
sample directory from `README.md`, `data.txt`, and the selected language's
`save_as` filename, then sorts ascending or descending to match the requested
task. Other tasks keep their static fallback output.

The browser worker mirrors that behavior with `writeProgramExpectedOutput()`.
It also separates `notRun` and `copyInstruction` into distinct Markdown
paragraphs and renders the sample-directory sentence with the same
language-aware file set.

The markdown code theme now uses CSS variables:

- Light theme: pale code surface with dark text.
- Dark theme or system-dark fallback: the previous dark code palette.

The after screenshot is
[`screenshots/after-light-code-block.png`](screenshots/after-light-code-block.png).
It shows the browser response with `main.py`, `data.txt`, `README.md`, separate
status/copy paragraphs, and a light code block.

## Regression Coverage

Automated coverage added:

- Rust unit regression for Python reverse list-files output.
- Rust modifier regression updated to assert the new language-aware sample
  directory and reject the old Cargo fixture.
- Playwright regression for the browser response, including:
  - no `Cargo.toml` / `main.rs` in the Python answer,
  - `main.py`, `data.txt`, `README.md` in output,
  - separate status and copy-instruction paragraphs,
  - light theme code block colors.

## Verification

Local checks run on June 14, 2026:

- `cargo test python_list_files_reverse_sort_sample_output_matches_saved_file_name`
  ([after log](raw-data/repro-unit-after.log))
- `cargo test specification::code_generation`
  ([log](raw-data/cargo-test-code-generation.log))
- `cargo test` ([log](raw-data/cargo-test-full.log))
- `cargo fmt --all -- --check` ([log](raw-data/cargo-fmt-check.log))
- `cargo clippy --all-targets --all-features` ([log](raw-data/cargo-clippy.log))
- `rust-script scripts/check-file-size.rs`
  ([log](raw-data/check-file-size.log))
- `rust-script scripts/check-changelog-fragment.rs`
  ([log](raw-data/check-changelog-fragment.log))
- `git diff --check` ([log](raw-data/git-diff-check.log))
- `npm ci --prefix tests/e2e` ([log](raw-data/npm-ci-e2e.log))
- `npm run check:i18n --prefix tests/e2e`
  ([log](raw-data/npm-check-i18n.log))
- `npm run check:web-tdz --prefix tests/e2e`
  ([log](raw-data/npm-check-web-tdz.log))
- `cd tests/e2e && npx playwright test tests/issue-440.spec.js --config=playwright.local.config.js`
  ([log](raw-data/playwright-issue-440.log))

During the first clippy attempt, the filesystem reached 100% usage while linking
the `lindera-jieba` build script. After the transient target artifacts were
cleared, the same documented clippy command completed successfully; the final
passing log is the one linked above.
