# Formal AI release-version-path verification sessions (PR #1141)

External Agent CLI 0.26.0, local `formal-ai/formal-ai` model at
`formal-ai/0.351.0`, 2026-09-26. The server binary is the same build used
for the translation-dogfood evidence (PR #1139 merge content, `9c84a3af7`);
PR #1141 changes only workflow YAML and one shell script, so that build is
current for this verification. No hosted model participated.

## Context

PR #1141 fixes the release version reads that broke when plan 16 L1
(`70df98cf6`) moved `Cargo.toml` to `rust/Cargo.toml`: release.yml's two
"Resolve Pages deploy ref" steps and the Pages deploy "Read formal-ai
version" step now route through `rust-script scripts/get-version.rs`
(rust-paths layout discovery), and `scripts/stamp-pages-artifact.sh`
mirrors the fallback. The sessions below had the local Formal AI model
verify the mechanism this fix relies on.

## Sessions

1. `ses_f21f36eadffePzMssFucp0SsPc` — "run `rust-script
   scripts/get-version.rs` and report the version and the Cargo.toml it
   read from". The planner picked exactly the right artifacts (`read` on
   `scripts/get-version.rs` and `Cargo.toml`), but the session workspace
   was empty, so both reads returned file-not-found and the model reported
   those errors as its final answer. Client and harness exited zero; the
   resolution limit is the session workspace, not the transport.
2. `ses_f21f1d075ffeJDDfZRsaKXB1aa` — the same task with the resolver
   script, `scripts/rust-paths.rs`, and `rust/Cargo.toml` staged into the
   session workspace. The model read the full `get-version.rs` source —
   including the rust-paths discovery that supports both the root and the
   `rust/` layout — and then demonstrated the layout fact behind the
   regression: reading the repository-root `Cargo.toml` returned
   `Error: File not found ... /Cargo.toml`. It stopped after the two
   planned reads and did not proceed to `rust/Cargo.toml`.
3. `ses_f21f143b6ffe3RYepKX30kvGbF` — the direct positive leg: "read
   `rust/Cargo.toml` and report the declared formal-ai package version and
   the path". The model read the file and reported
   `name = "formal-ai"`, `version = "0.351.0"` — the exact manifest the
   fixed release steps read through `get-version.rs`.

Together: the resolver source was read (session 2), the absent root
manifest was demonstrated (session 2's ENOENT), and the `rust/Cargo.toml`
version the resolver reports was read back (session 3). The command never
executed in-session (the planner chose `read` over `bash` both times) —
recorded as-is; the command leg was verified outside the sessions by the
same invocation CI now uses (`rust-script scripts/get-version.rs` →
`Output: version=0.351.0`).

Session transcripts and full logs remain private at
`/tmp/formal-ai-1141-evidence.lT0wAM`,
`/tmp/formal-ai-1141-evidence2.PWuvmE` and
`/tmp/formal-ai-1141-evidence3.F5k8it` on the authoring machine.

This evidence rides in pull request
[#1141](https://github.com/link-assistant/formal-ai/pull/1141), whose
attributed commit names it.
