---
bump: minor
---

### Fixed
- Thinking traces are now written in the language of the answer on every non-UI surface (issue #889, parent #710). The browser panel was already localized through the web i18n catalog, so a Russian, Hindi, Chinese or Spanish answer arrived with an English explanation of how it was produced everywhere else:
  - the sentences a trace is made of moved out of `src/thinking.rs` into seed data (`data/seed/multilingual-responses-thinking.lino` and `…-thinking-narrative.lino`), translated into every registered language, so adding a language means adding records rather than editing Rust (R379);
  - the CLI `--thinking` trace (including its heading), the OpenAI Chat Completions `reasoning`/`reasoning_content` fields, the OpenAI Responses reasoning item, the Anthropic extended-thinking block and the Telegram expandable blockquote all narrate in the resolved answer language, which is derived from the trace itself;
  - the language names inside the trace are localized too, so a Russian trace reads «Определить язык запроса: русский.» instead of naming the language in English;
  - the machine-readable `step`/`detail` trace keys and the step ids stay language-neutral, so downstream consumers never have to parse prose.
