## Issue #278 Native Doublets Store Default Requirements

| ID | Requirement | Status |
| --- | --- | --- |
| R231 | Default native builds must persist link records through the `link-cli` library without requiring an opt-in feature. | Implemented by enabling `doublets-native` in Cargo's default feature set and by `selected_link_store_backend()` returning `LinkStoreBackend::LinkCli` for default native builds. |
| R232 | Native callers must retain an explicit fallback path for the human-reviewable `.lino` projection. | Implemented by `default_native_link_store()`, whose `--no-default-features` build returns `MemoryStore`, and by the no-default regression test in `src/link_store.rs`. |
| R233 | Existing `demo_memory` and `formal_ai_bundle` `.lino` exports must import into the native store and export back to deterministic Links Notation. | Covered by `link_cli_default_imports_full_lino_bundle_and_exports_deterministically` in `tests/source/source_tests/link_store/tests.rs`. |
| R234 | Stable IDs, append-only native record history, and malformed import rejection must remain shared semantics across backends. | Covered by the `LinkStore` tests for stable IDs, native mirroring, strict malformed rejection, and no-mutation behavior in `src/link_store.rs`. |
| R235 | Browser storage must stay compatible with the doublets-web / IndexedDB projection while CLI, HTTP, library, and Telegram surfaces share the `.lino` migration contract. | Documented in `ARCHITECTURE.md` section 4.2 and `data/seed/environments.lino`; the browser keeps `selectedLinkStoreBackend()` in `src/web/memory.js`. |
| R236 | Architecture, vision, README, and environment docs must describe the native transactional store as the current default. | Implemented by updating `ARCHITECTURE.md`, `VISION.md`, `README.md`, `REQUIREMENTS.md`, and `data/seed/environments.lino` to describe link-cli as the native default and `.lino` as the portable source and recovery projection. |
