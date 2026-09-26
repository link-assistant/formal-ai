# Formal AI release-script-compilation verification sessions (issue #1147)

External Agent CLI, local `formal-ai/formal-ai` model at
`formal-ai/0.351.0`, 2026-09-27. The server binary is
`/tmp/formal-ai-target-incr/debug/formal-ai`, the same build verified for
PR #1144 and reused for PR #1146; PR #1148 touches only release-path
rust-scripts, so the served model behavior is unchanged. No hosted model
participated.

## Context

Main run 36269287140 (merge of PR #1146, `9e673a1dc`) passed every gate,
the Auto Release job decided `minor` over 26 fragments, and then failed
compiling `scripts/version-and-commit.rs` — an unused `PreparedRelease`
import that `RUSTFLAGS=-Dwarnings` made fatal. The same sweep found
`scripts/create-changelog-fragment.rs` unable to compile at all: its
`#[path]` include of `rust-paths.rs` needs the `regex` crate, which its
cargo preamble never declared. PR #1148 drops the unused import and adds
`regex = "1"` to the preamble. The sessions below had the local Formal AI
model read the three involved scripts.

## Sessions

1. `ses_f20660d57ffeph2BBsnrsmxnPW` — "Read the file
   scripts/version-and-commit.rs in this workspace and quote the use lines
   it has near the top for the prepared_release module." The model opened
   and read the file in full; the returned contents carry the fixed import
   `use prepared_release::{ensure_prepared_release_tag, prepared_release};`
   with no `PreparedRelease` name in the list.
2. `ses_f206721f1ffeUqcJQ3Ovf0vXM2` — "Read the file
   scripts/create-changelog-fragment.rs in this workspace. Which crates
   does its cargo dependencies block declare, and which module does it
   include with a path attribute?" Same genuine read; the returned contents
   contain `//! chrono = "0.4"` and `//! regex = "1"` — the dependency the
   fix adds — plus the `#[path = "rust-paths.rs"]` include it feeds.
3. `ses_f206712c1ffea8amIDoSlBl6rA` — "Read the file scripts/rust-paths.rs
   in this workspace and quote the names of the functions in it that use
   the regex crate." Genuine read of the shared module; the returned
   contents include `use regex::Regex;` and the `is_publish_false` /
   `find_manifest_value` bodies that require it.

All sessions ended `hasError: false` with the file's contents as the final
assistant turn; none produced a prose synthesis beyond the presented
contents, so the mechanical confirmation is the human-side verification
recorded in issue #1147 and PR #1148: both scripts compile clean under
`RUSTFLAGS=-Dwarnings` via side-effect-free early exits (usage /
invalid-arg / nonexistent-`--rust-root`), alongside the other release-path
scripts. Two earlier sessions (one that honestly declined a compound
synthesis task, one whose file-open misfired to an ENOENT path) were
replaced by the simpler single-file prompts above and are not counted.
