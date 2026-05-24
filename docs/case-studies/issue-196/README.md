# Issue 196 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/196>

Pull request: <https://github.com/link-assistant/formal-ai/pull/241>

Branch: `issue-196-baa4327fc44a`

Issue 196 asks for physical deletion of already deleted conversations, a full
memory reset, irreversible warnings and confirmation, an export-before-delete
path, support in every supported language, and a repository case study that
captures the requirements, research, and solution plan.

## Collected Data

Raw artifacts are preserved under `raw-data/`:

- `issue-196.json` and `issue-196-comments.json`: issue body and comments.
- `pr-241.json`, `pr-241-review-comments.json`,
  `pr-241-conversation-comments.json`, and `pr-241-reviews.json`: PR state and
  review surfaces.
- `branch-log.txt`: recent local branch history at collection time.
- `ci-runs-branch.json`: recent branch workflow runs at collection time.
- `online-research.md`: IndexedDB, erasure, and privacy-framework sources.
- `after-memory-controls.png`: browser screenshot showing the Reset memory
  control and deleted-conversation purge controls after the fix.
- `after-cargo-clippy.log`, `after-cargo-test-all-features.log`,
  `after-cargo-test-doc.log`, `after-check-file-size.log`,
  `after-check-i18n.log`, `after-check-language-test-coverage.log`,
  `after-check-intent-coverage.log`, `after-playwright-issue-196.log`, and
  `after-playwright-local.log`: local verification logs after the fix.

Local `logs/*.log` files are generated during development and ignored by git;
the tracked `after-*` copies above preserve the final verification evidence.

## Requirements

| ID | Requirement | Source | Solution |
| --- | --- | --- | --- |
| R196-1 | A conversation that is already soft-deleted must be physically removable from memory. | Issue body | Add selected conversation purging to Rust `MemoryStore`, CLI, browser IndexedDB storage, and browser deleted-conversations UI. |
| R196-2 | There must be a complete memory reset so tests can start from the standard pre-seed package. | Issue body | Add full store reset to Rust, CLI, browser storage, topbar, drawer, and natural-language reset actions. |
| R196-3 | Destructive actions must warn that the operation is irreversible and require confirmation. | Issue body | Browser actions show a backup prompt followed by an irreversible prompt; CLI actions refuse to run without `--confirm`. |
| R196-4 | Before confirmation, the user should be allowed to export memory if they have not done so. | Issue body | Browser prompts can run the existing full-memory export and cancel deletion; CLI destructive commands accept `--backup` and write a full `formal_ai_bundle` before deleting. |
| R196-5 | The flow must support all supported languages. | Issue body | Add reset phrases and UI/status/confirmation strings for English, Russian, Hindi, and Chinese; update i18n catalog checks and multilingual e2e coverage. |
| R196-6 | The repository must include a deep case study with online research, requirements, and solution plans. | Issue body | Add this folder, GitHub raw data snapshots, and online research notes. |

## Online Facts

MDN documents that IndexedDB `IDBObjectStore.delete()` deletes records by key
or key range, while `IDBObjectStore.clear()` removes every current record from
an object store. That maps directly to the two browser storage operations this
issue needs: cursor-based deletion for one or more conversations and store
clear for complete memory reset.

The ICO right-to-erasure guidance and NIST Privacy Framework both point toward
explicit erasure processes rather than hidden destructive side effects. The
important product requirements for this local assistant are therefore: make the
operation visible, warn about irreversibility, provide a backup/export path,
and document what was deleted.

## Existing Components

- `src/web/memory.js` already owns IndexedDB access and import/export shape.
- `src/web/app.js` already owns export/import actions, conversation
  projections, soft-delete markers, and localized UI labels.
- `src/memory.rs` already owns the portable `demo_memory` event schema and
  full-memory bundle import/export.
- `src/main.rs` already provides `formal-ai memory export|import|show`, making
  `memory purge-deleted` and `memory reset` natural additions.
- Issue 112 introduced append-only conversation deletion markers; issue 196
  deliberately adds explicit maintenance operations that can physically erase
  those marked conversations.

## Solution Plan

1. Extend the memory schema with optional `conversationId` and
   `conversationTitle` fields so Rust can identify the same conversation
   groups that the browser UI already tracks.
2. Add `MemoryStore::purge_deleted_conversations`,
   `MemoryStore::purge_conversation`, and `MemoryStore::reset` for embedders.
3. Add CLI commands that require `--confirm` and optionally write a full bundle
   backup through `--backup` before modifying the memory file.
4. Add browser storage helpers for selected conversation deletion, purging all
   soft-deleted conversations, and clearing the event store.
5. Add browser controls and natural-language reset routing with two-step
   confirmation: export-first prompt, then irreversible confirmation.
6. Localize the new labels, prompts, statuses, and reset phrases in all
   supported languages.
7. Add tests that first reproduce the missing destructive operations and then
   verify the implemented behavior across Rust, CLI, and browser surfaces.

## Fixes

- Added Rust memory purge/reset APIs and portable parsing/formatting for
  conversation metadata.
- Added `formal-ai memory purge-deleted` and `formal-ai memory reset`, both
  protected by `--confirm` and optional full-bundle `--backup`.
- Added IndexedDB delete/clear helpers while keeping generic delete APIs out of
  the public browser memory surface.
- Added Reset memory controls to the topbar and mobile drawer.
- Added permanent-delete controls for the deleted-conversations view.
- Added export-first and irreversible confirmation prompts before browser
  destructive operations.
- Added multilingual reset phrases and localized labels/statuses for English,
  Russian, Hindi, and Chinese.

## Verification Plan

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features --verbose`
- `cargo test --doc --verbose`
- `rust-script scripts/check-file-size.rs`
- `npm run check:i18n` in `tests/e2e`
- `npm run check:language-test-coverage` in `tests/e2e`
- `npm run check:intent-coverage` in `tests/e2e`
- `npx playwright test --config=playwright.local.config.js --grep "Issue #196"`

Focused checks added by this work:

- `library_memory_can_purge_soft_deleted_conversations`
- `library_memory_reset_clears_all_events`
- `cli_memory_purge_deleted_requires_confirmation_and_can_backup`
- `cli_memory_reset_requires_confirmation_and_can_backup`
- `Issue #196: reset memory phrases are recognised in every supported language`
- `Issue #196: deleted conversations can be permanently removed after export warning and confirmation`
- `Issue #196: reset memory clears all browser events after export warning and confirmation`
