# Reporting an issue from any Formal AI surface

Formal AI can file a GitHub issue about its own behavior from the web app, the
CLI, the desktop app, Telegram, and any coding harness that gives it a shell
tool. Whichever surface you use, the issue that arrives is the same document.

This page describes that document, the commands that build it, and the choices
the report makes on your behalf — most importantly, what it uploads and who can
read it.

## The document

Every report body has the same six sections, in this order:

| Section | What it carries |
| --- | --- |
| `## Environment` | Version, surface, URL or session, timestamp |
| `## User Context` | Only the settings that differ from their shipped defaults (omitted entirely when nothing differs) |
| `## Reproduction of dialog` | The whole conversation as one fenced block, `U:` / `A:` / `T:` per turn |
| `## Reasoning Trace` | The focused assistant turn's intent, evidence, and tool calls |
| `## Description` | An empty placeholder for you to fill in |
| `## Attach full memory (optional)` | A pointer to [`upload-memory.md`](upload-memory.md) |

The agentic path appends one more block after those six: the complete Links
Notation context of the session (see [Oversize contexts](#oversize-contexts)).

The format lives in exactly one place, [`src/issue_report.rs`](../src/issue_report.rs).
The browser cannot link the Rust core — the wasm worker is a standalone `rustc`
build — so [`src/web/app/issue-report.js`](../src/web/app/issue-report.js) is a
hand-written mirror of it, and
[`tests/integration/issue_839_report_parity.rs`](../tests/integration/issue_839_report_parity.rs)
renders the same fixture through both and fails the moment they drift. Every
phrase in the document comes from `data/seed/agent-info.lino`, so a surface
cannot invent its own wording either.

### Title

The title quotes what the conversation was about, never the turn that asked for
the report:

1. The trailing report request (`report issue`, `Сообщи об ошибке`) is dropped
   before anything is chosen — it is not the subject.
2. If two or more distinct user turns remain and `` `first` + `last` `` fits in
   120 characters, that is the title.
3. Otherwise the first turn alone, backticked and truncated on a word boundary.
4. The bare default (`Formal AI agentic session report`) appears only when the
   conversation contains no user turn at all.

## Building a report from the CLI

```bash
# Which session is this shell inside?
formal-ai context session

# The complete conversation, as Links Notation
formal-ai context export --session ses_06ac01b87ffeW5XnFmtYE8Amil --source both

# The full issue body, context attached
formal-ai report body --session latest --source both --output report.md
```

`formal-ai report body` renders the six sections and attaches the context. It is
the whole of the report pipeline: the generated shell script only runs it and
hands the file to `gh issue create --body-file`.

Useful flags:

| Flag | Default | Meaning |
| --- | --- | --- |
| `--session` | `latest` | `latest` resolves the session this shell is in |
| `--source` | `both` | `harness`, `server`, `both`, `opencode`, or `auto` |
| `--surface` | `agentic-cli` | Recorded in `## Environment` |
| `--context-output` | — | Also keep the exported context at this path |
| `--max-inline-bytes` | `50000` | Largest context attached inline |
| `--max-excerpt-bytes` | `12000` | Largest excerpt kept once the full context moved out |
| `--oversize` | `gist` | `gist` uploads the full context; `excerpt` keeps it local |
| `--gist-visibility` | `secret` | `secret` or `public` |

### Sources

- `harness` is the conversation the coding harness stored (for OpenCode, its
  SQLite database).
- `server` is the conversation Formal AI's own server recorded — a conversation
  record, not the HTTP proxy trace. The proxy trace is still written, and is
  still useful, but it is a different artifact and is never what a report
  exports.
- `both` merges the two and marks which turns came from where.
- `auto` prefers Formal AI's own capture and falls back to the harness.

A named source that cannot be exported is an error. `--source harness` on a
session the harness has never heard of fails and prints why; it does not quietly
fall back to another capture and file an issue whose body describes something
else. An export that resolves but yields no messages fails the same way. This is
deliberate: issue #838 was filed with 271 KB of base64 proxy traffic in place of
the conversation, and the run reported success.

### The session identifier

The exported session is the caller's real session. The HTTP server reads the
`x-formal-ai-dialog-id` header on every request and records it, so when the
planner writes a report command it names the session the harness is actually in.
Reports used to name a hash of the conversation's first message, which no
harness had ever heard of and which collided between any two conversations that
opened with the same sentence.

If the client sends no session header, the generated script resolves the id
itself with `formal-ai context session` rather than guessing.

## Oversize contexts

A conversation is usually larger than a GitHub issue body. When the exported
context does not fit inline:

- **`--oversize gist` (default)** uploads the complete context to a gist and
  links it from the body, keeping a record-safe excerpt inline.
- **`--oversize excerpt`** keeps only the excerpt; the complete context stays on
  your machine.

Truncation never cuts inside a record. The excerpt keeps whole Links Notation
records from the start and the end of the conversation and replaces the middle
with an explicit `... omitted N records ...` marker, so what remains still
parses and still says how much is missing. (Issue #838's report was cut with
`tail -c 12000`, which lands mid-record by construction.)

### Gist visibility

**A gist carries the whole conversation.** The default is `secret`: unlisted,
reachable only through the link in the issue, and therefore readable by anyone
that link reaches. It is not private, and it is not redacted. Pass
`--gist-visibility public` only if you mean it, and `--oversize excerpt` if the
conversation should not leave your machine at all.

## Failure is loud

The generated script checks every program it will call with `command -v` before
it does anything, and stops with a message naming the missing program rather
than filing half a report. If the export fails or is empty, the script stops
before `gh` is reached. A report either carries the conversation or does not get
filed.

The tests in
[`tests/integration/issue_839_report_script.rs`](../tests/integration/issue_839_report_script.rs)
run the generated script end to end against a fixture session — with a stubbed
`gh` on an otherwise empty `PATH` — and assert on the file `gh issue create`
actually received.
