---
bump: minor
---

### Added
- Multilingual chat across Rust, CLI, HTTP server, Telegram bot, and the web demo: greetings, identity, unknown-answer, and concept-lookup handlers reply in Russian, Hindi, and Chinese in addition to English; the input language is detected from Unicode blocks (Cyrillic, Devanagari, CJK) (issue #16).
- Browser Wikipedia REST fallback for `What is X?` prompts that miss the offline `CONCEPTS` table; the offline corpus still wins and the network answer carries `source:` evidence so the citation can be audited.
- `data/seed/` as the canonical knowledge surface for every interface: `src/seed.rs` `include_str!`-embeds every `.lino` file and exposes typed accessors (`multilingual_responses`, `concepts`, `tools`, `language_rules`, `prompt_patterns`, `intent_routing`) that the Rust library, CLI binary, HTTP server, and Telegram webhook all read; the browser worker fetches the same files through `src/web/seed_loader.js`.
- Data-driven intent routing via `data/seed/intent-routing.lino` with explicit `keyword` / `phrase` / `token` / `combo` match semantics, replacing hardcoded match arms in the dispatcher.
- `seed::merged_bundle()` / `seed::parse_bundle()` plus their JS mirrors `FormalAiSeed.parseBundle` / `FormalAiSeed.loadFromBundle` give the seed a single-file Links Notation round-trip.
- Append-only memory log in the web demo (`src/web/memory.js`) with Export memory, Import memory, and Download bundle buttons; reasoning steps and tool invocations are recorded alongside chat turns, and no delete/forget/clear API is exposed.
- Tool registry panel surfacing seeded tools (`http_fetch`, `web_search`, `wikipedia_lookup`, `eval_js`, `read_local_file`, `append_memory`, `export_memory`) with a `thinking` vs `agent` mode badge.
- Playwright e2e suite (`tests/e2e/tests/multilingual.spec.js`) covering multilingual greetings/identity, offline + Wikipedia `What is X?` resolution, export/import round-trip, append-only enforcement, bundle download, tool registry, and reasoning-event logging; runs both against `npx serve src/web` (PR) and against `link-assistant.github.io/formal-ai` (post-deploy).
- `docs/case-studies/issue-16/` with raw GitHub data, online research, requirements-to-implementation mapping, and a Follow-Up section covering the universal seed loader and bundle round-trip.

### Changed
- Moved the deployable demo from `docs/demo/` to `src/web/` so it sits next to the other library/CLI/web sources; the GitHub Pages deploy job already targets `src/web`.
- Relocated `REQUIREMENTS.md` to the repository root next to `VISION.md` and expanded it with R90–R104 (move to `src/web`, multilingual surface, Wikipedia fallback, e2e coverage, export/import, append-only constraint, seed externalization, tool registry UI, reasoning/tool-call events, single-file bundle, universal seed loader, intent routing as data, bundle round-trip).
- Rewrote `src/web/formal_ai_worker.js` to initialise mutable `MULTILINGUAL_ANSWERS`, `CONCEPTS`, and `TOOLS` tables from the seed instead of carrying hardcoded literals; `solve()` now returns structured `steps[]` and `toolCalls[]` for the append-only log.
