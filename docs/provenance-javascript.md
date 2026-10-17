# JavaScript provenance integrity prototype

The JavaScript runtime can retain a private hash chain for an actual dogfood drive session:

```sh
node experiments/js_dogfood/drive.mjs --dir /tmp/task --provenance-dir /tmp/task-evidence 'Create file hello.mjs containing console.log(42);'
node js/provenance/cli.mjs verify-drive /tmp/task-evidence/session-EXACT-ID /tmp/task
node js/provenance/cli.mjs snapshot /tmp/task
```

The task directory must already exist. Snapshot requires a Git repository root and includes every stage-zero tracked regular working file, including binary files. Untracked files are outside this declared inventory. Symlinks, gitlinks and unresolved index stages are refused. The record separately binds index paths, modes and blob IDs; no oversized file is silently excluded.

Default snapshot budgets are 50,000 files, 4 MiB per file and 32 MiB total. All file-count and size checks precede the first source payload read. Content hashing consumes a per-file iterator and retains leaf metadata rather than the source corpus. The repository API accepts explicit maxFiles, maxFileBytes and maxTotalBytes overrides. Large repositories can exceed the default budget and fail; this is not evidence of a complete snapshot. The implementation is one-shot CPU JavaScript per bounded file; no throughput or GPU claim is made.

Leaves are BLAKE3 of UTF-8 path, a zero byte, then exact file bytes. Files and languages sort by UTF-8 byte order. A binary tree duplicates the final node at odd levels; internal nodes and empty roots have versioned domain separators. The total root binds each language name, file count and root. Language extensions come from the maintained seed table, with the required JavaScript extensions and package/Cargo manifests explicitly grouped. Generic short Merkle proofs are available from snapshot.mjs.

Drive evidence lives outside the solved workspace. Each session directory and payload file is private. Exact original planner inputs, raw planner responses, dispatched plans and available reasoning/derivation fields are retained as typed Links Notation byte records. They can contain prompts and file contents: do not upload private sessions. Normal logs must use the shared logger's redacted summaries. The chain links each observed direct write/edit to the dispatched message_out and records pre/post BLAKE3 and Git SHA-1 blob digests at adapter execution time. Shell commands and external writers receive no edit attribution. Missing reasoning fields are explicitly unavailable. All normal stop reasons and exceptions terminate the chain; failed storage can leave incomplete evidence that must not be accepted.

The shared onEvent drive option reports exact chosen plans, actual tool receipts, one outer terminal result and exceptions. A failed event sink does not replace task execution; its failure count appears in eventSinkFailures. The logging integration consumes these events without putting original payloads in normal diagnostics.

The verifier recomputes the pinned chain and every retained sidecar hash, requires thinking-record linkage, and compares the latest recorded file postimages with current bytes. A changed or missing file is foreign and makes the command fail. It reports self-supplied anchor trust unless the API caller supplies a separately pinned anchor. A valid self-supplied chain is not a trusted identity or proof of authorship.

The session primitive also supports a snapshotSource reader that captures a source_snapshot before commit and terminal records. Snapshot failures prevent those boundary records. Actual native CLI routing, real engine seed/corpus and persistent-memory custody, installation signing keys, commit trailers and diff-line coverage, every real response's original thinking trace, deterministic computational replay, GPU hashing and hosted performance targets remain unfinished. Driver dialogue integrity is narrower than whole-engine memory integrity. This prototype does not establish autonomous authorship or full acceptance of issues #1213–#1219.
