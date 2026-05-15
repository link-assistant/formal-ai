# Issue 18 Case Study: Export/import should always contain full memory of AI agent

## Summary

Issue [#18](https://github.com/link-assistant/formal-ai/issues/18) reports that
the **Export memory** button on
[https://link-assistant.github.io/formal-ai](https://link-assistant.github.io/formal-ai)
saves only the in-session event log rather than the entire memory of the AI
agent. The reporter wants every export to be **full and self-contained by
default** so the resulting file is debuggable on its own, survives seed updates
in the upstream repository, survives user edits of the seed, and can be
attached to bug reports without manual steps.

Three concrete asks fall out of the issue body:

1. **Default to full memory on every export.** The output must capture the
   agent's full state (static seed + dynamic event log + UI preferences +
   environment metadata), not "only small changes or part of memory". Older
   files must still import without breaking.
2. **Surface known data migrations after import.** When the imported file's
   seed version differs from the running app's seed version, the agent should
   tell the user what migrations are recommended so the experience improves
   automatically.
3. **Make the prefilled "Report issue" link practical for `.lino` payloads.**
   GitHub issue attachments do not accept `.lino` files yet — the body needs
   to instruct users to wrap the export in a `.zip`, with sensitive content
   redacted, and the issue-report flow should make that obvious. (The issue
   body itself is too big to embed via the GitHub `?body=` query string.)

Everything has to stay in sync across the codebase, docs, requirements, and
tests after this PR is closed.

## Collected Data

Fresh GitHub evidence lives in
[`raw-data/`](./raw-data) so future analysts can replay the investigation
without re-querying the API:

- [`raw-data/issue-18.json`](./raw-data/issue-18.json) — full issue body and
  metadata captured with `gh issue view`.
- [`raw-data/issue-18-comments.json`](./raw-data/issue-18-comments.json) —
  issue conversation comments (empty at collection time; the request is in
  the issue body).
- [`raw-data/pr-19.json`](./raw-data/pr-19.json) — pull request metadata for
  the draft that implements this case study.
- [`raw-data/pr-19-conversation-comments.json`](./raw-data/pr-19-conversation-comments.json)
  — PR discussion comments.
- [`raw-data/pr-19-review-comments.json`](./raw-data/pr-19-review-comments.json)
  — inline code-review comments.
- [`raw-data/pr-19-reviews.json`](./raw-data/pr-19-reviews.json) — PR review
  records.
- [`raw-data/recent-merged-prs.json`](./raw-data/recent-merged-prs.json) —
  recent merged PRs used to match the repository's documentation style.

## Prior Case Studies

This work extends the export/import + universal-seed scaffolding established
by earlier issues:

- [`../issue-16/README.md`](../issue-16/README.md): introduced the IndexedDB
  append-only memory log, the **Export memory** / **Import memory** buttons,
  and the original "Download bundle" action.
- [`../issue-17/README.md`](../issue-17/README.md): made the seed canonical
  across every interface (browser worker, Rust solver, CLI, HTTP server,
  Telegram bot) and added `formal-ai memory|bundle` subcommands.

## Timeline of Events

| Timestamp (UTC) | Event |
| --- | --- |
| 2026-05-15 12:25 | Reporter reproduces the dialogue shown in the issue on https://link-assistant.github.io/formal-ai and clicks **Export memory**. |
| 2026-05-15 12:26 | Browser writes `formal-ai-memory.lino` containing only `demo_memory` (events 1-14). Seed, preferences, version, and environment metadata are missing. |
| 2026-05-15 12:28 | Reporter files issue #18 noting that the export is "not full memory, but only small changes or part of memory". |
| 2026-05-15 12:30 | AI issue solver claims branch `issue-18-65b2eb796b44` and opens draft PR #19. |
| 2026-05-15 12:36 | Investigation begins under `docs/case-studies/issue-18/`. |

## Reproducing the Bug

1. Visit [https://link-assistant.github.io/formal-ai](https://link-assistant.github.io/formal-ai).
2. Wait for the randomized demo to play a greeting + hello-world turn (or
   switch to manual mode and send any prompt).
3. Click **Export memory** in the top bar.
4. Open `formal-ai-memory.lino`.

Expected: the file describes the full agent state (seed + events +
preferences + version + environment metadata) so a maintainer can
reconstitute the agent.

Observed (before this PR): the file is a `demo_memory` document containing
**only the recent events**. Without the seed and preferences, the file is
not enough to debug a regression or replay the agent on a different machine.

## Root Cause Analysis

### Where the partial export comes from

The web demo defines two separate export actions in
[`src/web/app.js`](../../../src/web/app.js):

- `handleExportMemory` calls
  [`FormalAiMemory.exportLinksNotation(events)`](../../../src/web/memory.js) —
  which only emits the `demo_memory` event log.
- `handleExportBundle` calls
  [`FormalAiMemory.exportBundle({ seed, events, info })`](../../../src/web/memory.js)
  — which emits the full `formal_ai_bundle` with seed and metadata.

The default top-bar button users click is **Export memory**, but it routes
through the partial path. The "Download bundle" button is hidden behind less
prominent labelling and a longer tooltip, so most users never find it —
explaining why the reporter's export missed the seed.

### Why the bundle button alone is not enough

Even if users could be trained to click the right button, the partial export
silently throws away three useful inputs:

1. **Seed contents.** If the upstream repository ships a new
   `data/seed/concepts.lino`, an old `formal-ai-memory.lino` cannot recover
   the agent's prior responses because the seed has changed under it.
2. **UI preferences.** `demo_preferences` (demo mode, diagnostics mode) lives
   in `localStorage` and is required to fully reconstruct the user's view.
3. **Version + environment metadata.** Without `version`, `url`, `userAgent`,
   etc. it is impossible to tell whether a regression is a code change or a
   data change.

### Why imports do not advertise migrations

`handleImportMemory` calls `FormalAiMemory.importEvents(...)` and reports a
plain count. It never inspects the imported document for a `version` field
or a seed checksum, so the user is not told when their imported state was
authored against a different seed version. This is the root cause of "no
migration suggestions" — there is no comparison step at all.

### Why the prefilled issue body fails for `.lino`

The body created by `createIssueReportBody` instructs users to "drag
`formal-ai-bundle.lino` into this issue". GitHub's issue-attachment allow
list, however, does not currently include `.lino`; the file upload silently
fails or shows a "We don't support that file type" toast. The body also
underplays redaction and never mentions that the attached file is the
**full memory** of the agent (which may contain personal prompts or tool
outputs).

## Requirements

Distilled from the issue body, in priority order. New requirements (R109+)
extend the matrix tracked in [REQUIREMENTS.md](../../../REQUIREMENTS.md).

| ID | Requirement |
| --- | --- |
| R109 | The default **Export memory** action must always produce the full, self-contained agent state (seed + memory log + UI preferences + environment metadata + version). No second button click is required to get the full export. |
| R110 | Imports must accept both the legacy `demo_memory` document and the new `formal_ai_bundle` document so old exports continue to work. |
| R111 | After import, surface known data migrations (e.g. seed-version mismatch) directly in the UI so the user can take action without reading code. |
| R112 | The prefilled "Report issue" body must instruct users to wrap the `.lino` export in a `.zip` (GitHub does not yet accept `.lino` attachments), and must explicitly remind users to redact sensitive content before attaching. |
| R113 | The CLI surface (`formal-ai memory export`, `formal-ai bundle export`) and the Rust library API must default to full-memory output as well, so every interface stays consistent. |
| R114 | Sync VISION.md, REQUIREMENTS.md, README.md, and the e2e/unit tests with the new defaults so the documentation, the requirement matrix, and the regression coverage all describe the same behavior. |

## Solution Plan

### R109 — full memory on every export

- Add `FormalAiMemory.exportFullMemory({ seed, events, preferences, info })`
  in [`src/web/memory.js`](../../../src/web/memory.js): a single entry point
  that produces the `formal_ai_bundle` shape, now including a
  `preferences` subsection.
- Re-wire the **Export memory** button in
  [`src/web/app.js`](../../../src/web/app.js) to call `exportFullMemory`.
  Keep the `formal-ai-memory.lino` filename so users' bookmarks/scripts
  still work, but populate it with the full bundle.
- Keep `exportBundle` and `exportLinksNotation` exported for backwards
  compatibility — they are documented entry points used by the CLI and the
  e2e tests.

### R110 — backwards-compatible imports

- Add `FormalAiMemory.importFullMemory(text)` that:
  - If the document starts with `formal_ai_bundle`, parses both the seed
    section and the embedded `demo_memory` and returns
    `{ events, seedFiles, info, preferences, migrations }`.
  - Otherwise falls back to `parseLinksNotation` (legacy `demo_memory`).
- Update `handleImportMemory` in
  [`src/web/app.js`](../../../src/web/app.js) so both formats are accepted.

### R111 — migration suggestions on import

- Define `FormalAiMemory.suggestMigrations({ imported, current })`. The
  first migration check covers the seed version baked into
  `data/seed/agent-info.lino` (`field "version"`). When the version moved
  forward, return `["Seed version 0.21.0 → 0.22.0: review new tools or
  multilingual responses in data/seed/."]`.
- Surface the suggestion in the memory-status indicator after import so the
  user sees it without opening DevTools.

### R112 — `.zip`-of-`.lino` prefilled issue body

- Rewrite the **Attach full state** block in `createIssueReportBody` so it:
  - Calls out that GitHub issues do not accept `.lino` files yet and asks
    the user to wrap the export in a `.zip`.
  - Reminds the user to scan the file for sensitive content (names, secrets,
    pasted code) and redact before attaching.
  - Explains that the file is the **full memory** of the agent, not a
    diff — so the maintainer can replay the exact session.
- Mention this in the **Report issue** button tooltip too.

### R113 — CLI / Rust API parity

- `formal-ai memory export` should default to the full bundle (just like the
  browser button) while keeping a `--events-only` opt-in for backwards
  compatibility.
- Re-export `MemoryStore::export_full_memory` from `formal_ai`'s crate root.

### R114 — sync docs and tests

- Add new rows R109-R114 to `REQUIREMENTS.md`.
- Update `README.md` and `VISION.md` to describe the new defaults.
- Update the Playwright e2e suite to assert that **Export memory** now
  contains the seed and the preferences, and that **Import memory** accepts
  a bundle.
- Add Rust unit coverage for the new helpers in `src/memory.rs`.

## Existing Components and Prior Art

- The `formal_ai_bundle` document format is already established by
  `src/web/memory.js::exportBundle` and `src/memory.rs::export_bundle`. We
  reuse it rather than inventing a new format.
- Browser file downloads use the standard `Blob` + anchor pattern already
  in `src/web/app.js::downloadTextFile`; no new dependency needed.
- For the `.zip`-of-`.lino` story we deliberately do **not** add a
  JS/Rust zip dependency in this PR. GitHub already supports `.zip`
  attachments, so we ask the user to zip locally — the trade-off is that
  the user has to perform one extra step, but we avoid pulling in a new
  binary dependency to the WebAssembly worker.

## Online Research

GitHub's documentation on attachment types
([docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files))
lists the currently allowed file extensions for issues and pull request
comments. `.lino` is **not** on the list as of this writing, while `.zip` is
— which is why the recommended workflow is to compress the export. We file
this as a known limitation rather than a bug against GitHub.

If GitHub later adds `.lino` to the allow-list we will revisit the
recommendation; the case study captures the workaround so the change is
discoverable.

## Verification

- Manual: open `index.html` via a local web server, click **Export memory**,
  open the downloaded file and verify it contains a `formal_ai_bundle`
  header, every seed file, the `demo_preferences` snapshot, and the event
  log.
- Automated: Playwright `Export memory now produces a full formal_ai_bundle`
  test in `tests/e2e/tests/multilingual.spec.js`.
- Rust: `cargo test --all-features` — `memory::tests::full_memory_round_trip`.

## Status

Implemented in PR [#19](https://github.com/link-assistant/formal-ai/pull/19).
