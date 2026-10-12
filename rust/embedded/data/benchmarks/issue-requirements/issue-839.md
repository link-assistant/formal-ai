Reported from an agentic session via natural language (no web UI available).

Issue reporting is the feature every other bug reaches us through. When it degrades, every other issue arrives degraded. #838 is the proof: it was filed **by** the report flow, and the artifact it produced is unusable — no conversation, no server log, a title that echoes a prompt the user never typed, and a body containing a 12 KB slice from the middle of one base64 HTTP record.

This issue supersedes the reporting half of #819, #824, #832. Issues #832 and #833 fixed the *previous* generation of this bug (hard-coded `curl :3000`, unverified success claims, non-multiselect questions). Those fixes hold. What #838 shows is that the layer underneath them was never right.

## 1. What #838 actually produced vs. what was expected

The user selected **Harness log + Server log + GitHub issue**. Expected: a complete export of both the local opencode session and the Formal AI server context, in the shape of the well-formed markdown session export that already exists.

What landed in the issue body:

```lino
      request_id request_f117b38f39e4d063
      request_model formal-ai
      request_tools (bash edit glob grep question read skill task todowrite webfetch websearch write)
      response_body "b64:ZGF0YTogeyJjaG9pY2VzIjpb…"
```

That is one raw HTTP proxy exchange, sliced mid-record. There is no conversation, no user turn, no assistant turn, no tool result. The gist referenced in the body is a **secret gist** whose filename is the literal unexpanded `formal-ai-report.XXXXXX.lino` — `mktemp`'s template was never substituted into `gh gist create --filename`.

## 2. Root cause — three defects, in order

**All three were reproduced empirically against `main` @ `1873e873` (0.303.0) using the real #838 session.** The export machinery itself is sound — given the *correct* session id it produces a perfect result. The defect is that the report flow never passes the correct id:

```console
$ formal-ai context export --session 'ses_06ac01b87ffeW5XnFmtYE8Amil' --source harness --output -
conversation ses_06ac01b87ffeW5XnFmtYE8Amil
  message_count 31
  messages
    message
      … text "Search hive-mind on desktop"          # ← the real conversation, all 31 turns
```

```console
$ formal-ai context export --session 'dialog_a57762f1eb61e809' --source harness --output /tmp/x.lino
$ wc -c /tmp/x.lino          #  271581   ← 271 KB
$ grep -c 'b64:' /tmp/x.lino #       13   ← base64 HTTP bodies
```

`dialog_a57762f1eb61e809` is the id the report flow actually generated in #838. `--source harness` fails to resolve it as an opencode session and **silently falls back to the server HTTP log** (`report_issue.rs` → `cli_context.rs:128-137`), yielding 271 KB of base64 proxy exchanges. That is >50 KB, so §2.3's gist + `tail -c 12000` branch fires and slices it mid-record. This is the complete, verified causal chain from id to the garbage body in #838.

### 2.1 `dialog_id` is a hash of the first user message, not a session identifier

`src/agentic_coding/report_issue.rs:405-412`:

```rust
fn dialog_id(messages: &[ChatMessage]) -> String {
    let basis = messages.iter()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.user_request_text())
        .unwrap_or_default();
    stable_id("dialog", &basis)
}
```

`stable_id` is FNV-1a over the text (`src/web_engine_core.rs:91`). So the exported session id is a content hash of the *first user message*, which:

- has no relationship to the opencode harness session (`ses_06ac01b87ffeW5XnFmtYE8Amil` in #838), so `--source harness` can never resolve the real transcript;
- collides across conversations that open with the same sentence — every "Find hive-mind on my desktop" session maps to one `dialog_*.jsonl`, appending unrelated dialogs into one file;
- changes if the first message is edited.

In #838 it produced `dialog_a57762f1eb61e809`, while the actual session was `ses_06ac01b87ffe…`.

### 2.2 The dialog log stores raw HTTP exchanges, not conversations

`src/dialog_log.rs:119-164` writes `DialogExchangeLog { timestamp_unix_ms, dialog_id, request_id, exchange }` where `exchange` is `summarize_proxy_exchange(...)` — method, path, headers, base64 request/response bodies. That is a proxy trace, not a dialog.

So the *server* side has no conversation-shaped record — only HTTP plumbing. The `request_id`/`response_body "b64:…"` block in #838 is this structure faithfully rendered.

This matters because of the fallback in `src/cli_context.rs:128-137`: when `--source harness` cannot resolve a session, it does not fail — it returns the server HTTP log instead. Combined with §2.1 that fallback fires on every agentic report, so the flow reliably exports proxy traces while reporting success.

Two fixes are needed and they are independent: pass the real session id (§2.1), **and** make `--source harness` fail loudly instead of silently degrading to a different, wrong source. A conversation-shaped server record is additionally required for `--source server|both` to mean anything.

### 2.3 Silent truncation at a byte offset, mid-record

`src/agentic_coding/report_issue.rs:383-402` — the generated shell:

```sh
if [ "$(wc -c < "$context_file")" -le 50000 ]; then … cat … ;
else context_url=$(gh gist create --filename formal-ai-context.lino "$context_file");
     tail -c 12000 "$context_file" | sed '1d' >> "$body_file"; fi
```

`tail -c 12000 | sed '1d'` cuts at a byte offset and drops the first surviving line. Applied to Links Notation this is guaranteed to land mid-record and destroy the tree structure — which is precisely what the reader of #838 sees. Compare the web path, which backfills whole turns and emits `... omitted N earlier messages ...` (`src/web/app/main.jsx:5314-5416`).

## 3. The web reporter is the quality bar, and the agentic path implements none of it

`createIssueReportBody` (`src/web/app/main.jsx:5474-5531`) emits six sections. The agentic path emits an intro line plus a byte-slice.

| Section | Web (`main.jsx`) | Agentic (#838) |
| --- | --- | --- |
| Title | `Unknown prompt: …` / `Issue with dialog: …` (`:5452-5462`) | `Formal AI: ` + last user message, truncated to 72 (`report_issue.rs:414-431`) |
| `## Environment` | version, URL, mode, diagnostics, timestamp (`:5489-5502`) | absent |
| `## User Context` | languages, theme, viewport, locale; defaults elided per #386 (`:2171-2216`) | absent |
| `## Reproduction of dialog` | `U:`/`A:` transcript with legend (`:5129-5175`) | absent |
| `## Reasoning Trace` | intent, evidence, tool_calls, capped (`:5251-5301`) | absent |
| `## Description` | `<!-- Please describe what looked wrong … -->` | absent |
| `## Attach full memory` | link to `docs/upload-memory.md` | absent |
| Size handling | whole-turn backfill + explicit `omitted N` markers | blind `tail -c 12000` |

**Definition of done: one shared report-body builder produces identical sections for web, CLI, desktop, Telegram, and VS Code.** The web implementation is the reference; it must move out of JSX into the core so every surface renders the same document.

## 4. Title quality

Current: `config("issue_report_title_prefix") + truncate(last_user_message, 72)` — `report_issue.rs:414-431`.

For #838 this yielded `Formal AI: Find hive-mind on my desktop`, but the session's *first* request was `Search hive-mind on desktop` and the last was `report issue`. The title described neither the problem nor the session.

Adopt the convention the maintainer applied by hand when retitling #826 and #827: **when the first and last user messages both fit, use both.**

```
Formal AI: `Что такое фуфломицин?` + `Так что это такое то?`
Formal AI: `ФБС vs ФБО` + `Зарепорти баг`
```

Rules:
1. Drop the report-invoking turn itself (`report issue`, `Зарепорти баг`, `Report`) before selecting — it is never the subject.
2. If ≥2 distinct user turns remain and `` `first` + `last` `` fits the limit, use that form, each backticked.
3. Otherwise use the first turn alone, backticked, truncated on a word boundary.
4. Never emit the bare default `Formal AI agentic session report` when any user turn exists.

## 5. Truthfulness of the completion message

`report_finished` (`report_issue.rs:433-455`) scans stdout for a URL containing `/issues/` — that fix from #832 is correct and must stay. Extend it: the export steps must be verified too. If `--source harness` yields 0 bytes or no `message` records, the run must **fail loudly** rather than filing an issue with an empty body. #838 filed successfully while carrying nothing.

Add a pre-flight assertion that every command in the generated script exists (`command -v`) — a hallucinated or renamed subcommand under `set -eu` must never be reported as a successful filing.

## 6. Why CI stayed green

`tests/integration/issue_819_report_flow.rs:186-193` asserts only on the *command string*:

```rust
assert!(command.contains("gh issue create"), "{command}");
assert!(command.contains("--source harness"), "{command}");
assert!(command.contains("--source server"), "{command}");
assert!(command.contains("--source both"), "{command}");
```

Every one of these passes for a script that produces an empty issue. No test executes the script, and no test inspects the resulting body. `tests/unit/issue_832.rs` similarly asserts `!command.contains("curl")`.

The web path, by contrast, *does* assert body content (`tests/e2e/tests/demo.spec.js:185-209` checks `## Environment`, `## Reproduction of dialog`, the `U:`/`A:` legend). That asymmetry is the whole story.

## 7. Definition of done

**Capability**
- [ ] The report flow passes the **real harness session id** (`ses_…`), not an FNV hash of the first message. `x-formal-ai-dialog-id` already exists at `dialog_log.rs:167-175` — propagate it end to end. Verified: with the real id, export already works perfectly (31/31 messages).
- [ ] `--source harness` **fails loudly** when it cannot resolve a harness session, instead of silently returning the server HTTP log (`cli_context.rs:128-137`). Silent source substitution is the bug that made #838 look successful.
- [ ] A conversation-shaped server-side record exists, distinct from the HTTP proxy trace in `src/dialog_log.rs`, so `--source server|both` carries turns rather than base64 bodies.
- [ ] `--source both` merges harness and server records into one document.

**Format**
- [ ] One shared report-body builder, used by web, CLI, desktop, Telegram, VS Code, emitting all six sections.
- [ ] Titles follow the first+last convention in §4.
- [ ] Truncation never cuts inside a record; oversize content attaches in full as a gist and the body carries a structured summary plus explicit `omitted N` markers.
- [ ] `gh gist create --filename` receives a real filename, never an unexpanded `mktemp` template. Gist visibility (secret vs public) is an explicit, documented choice.

**Honesty**
- [ ] Empty or unresolvable export fails loudly; no issue is filed claiming attached context it does not have.
- [ ] Every command in a generated script is verified to exist before execution.

**Tests** — each must fail against today's `main`:
- [ ] Execute the generated script end to end against a fixture session and assert the resulting body contains all six sections and the full transcript.
- [ ] Assert a multi-turn conversation round-trips: every user turn, assistant turn, and tool result present in the exported `.lino`.
- [ ] Assert two different conversations that share a first message do not collide into one export.
- [ ] Assert the title convention across the #826/#827/#838 fixtures.
- [ ] Assert a truncated export never splits a Links Notation record.
- [ ] Assert a failed/empty export produces a failure message, not a filed issue.
- [ ] Parity test: web and agentic bodies for the same fixture conversation are section-for-section identical.

## 8. Refactoring in scope

- `src/agentic_coding/report_issue.rs:383-402` — replace the generated shell heredoc with a real, testable command; the current template hard-codes formatting decisions inside a shell string where nothing can assert on them.
- `src/dialog_log.rs` — separate "HTTP proxy trace" from "conversation record"; today one structure is asked to serve both and serves neither.
- `src/web/app/main.jsx:5474-5531` — extract the body builder to the shared core; keep the JSX as a thin caller.
- `tests/integration/issue_819_report_flow.rs` — replace string-containment assertions with artifact assertions.
- Remove the "single combined command" expectation encoded at `tests/integration/issue_819_report_flow.rs:186` ("Every selected destination is fulfilled in a single executable step"); see the companion answer-quality issue — selected destinations should run as separate, individually verifiable steps.

## Environment

- Version: 0.303.0 (`main` @ `1873e873`); #838 was filed from 0.302.2
- Evidence: [session log for #838](https://gist.githubusercontent.com/konard/6194a81163cb43cb2cdebf58ccdf7838/raw/aa56371bb64b3e401e7221fdd8c8238590fdcb9e/session-ses_06ac.md.log.txt)
- Related: #819, #822, #824, #832, #833

## Description

<!-- Add anything the analysis above missed. -->

