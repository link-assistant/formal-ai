---
bump: minor
---

### Added
- Added a default assistant thinking preview that shows a collapsed current step, a faded previous step, an expandable localized summary list, and a configurable thinking-detail setting while preserving raw reasoning diagnostics behind the diagnostics toggle.
- Added first-class solver thinking metadata derived from the append-only event log and exposed it through Links Notation, Chat Completions, Responses, and the desktop HTTP chat path.
- Made thinking steps concrete by default: a shared naturalizer turns each reasoning event into a human-readable sentence that names the real content (the prompt, the detected language, the chosen route, the computed `expr = result`, the looked-up entity, the composed answer) instead of a generic label, and surfaces the same concrete reasoning on the CLI `--thinking` output, the OpenAI-compatible and Anthropic APIs, the browser, and the Telegram bot via a native collapsed-by-default expandable blockquote.

### Changed
- Promoted thinking to a first-class concern in a dedicated `thinking` module (step model plus naturalizer) so it is shared across every surface rather than embedded in the engine internals.
