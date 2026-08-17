## Issue #196 Permanent Memory Deletion And Reset

Issue [#196](https://github.com/link-assistant/formal-ai/issues/196)
requires physical deletion of already deleted conversations, a complete memory
reset, irreversible warnings, an export-before-delete path, support in every
supported language, and a dedicated repository case study.

| ID | Requirement | Status |
| --- | --- | --- |
| R226 | Already soft-deleted conversations must be physically removable from the memory log. | Implemented by `MemoryStore::purge_deleted_conversations`, `MemoryStore::purge_conversation`, `formal-ai memory purge-deleted`, and browser `FormalAiMemory.purgeDeletedConversations` / `deleteEventsByConversationId`. Covered by Rust, CLI, and Playwright tests. |
| R227 | Users must be able to reset the dynamic memory log completely. | Implemented by `MemoryStore::reset`, `formal-ai memory reset`, browser `FormalAiMemory.clearEvents`, topbar/drawer Reset memory controls, and natural-language reset actions. Covered by Rust, CLI, and Playwright tests. |
| R228 | Destructive memory actions must require irreversible confirmation and provide a backup/export path first. | Browser purge/reset/permanent-delete actions first offer the existing full-memory export and then ask for irreversible confirmation. CLI purge/reset commands fail without `--confirm` and can write a full `formal_ai_bundle` through `--backup` before modifying the memory file. |
| R229 | Reset and deletion UX must be localized for every supported language. | New labels, titles, confirmation prompts, statuses, and reset phrases are present for English, Russian, Hindi, and Chinese in `src/web/i18n-catalog.lino`; catalog and multilingual coverage scripts guard the keys and language matrix. |
| R230 | Issue research, requirements, and solution planning must be preserved under `docs/case-studies/issue-196`. | Implemented with issue/PR snapshots, branch/CI snapshots, online research notes, requirement table, existing-component analysis, solution plan, and verification plan. |
