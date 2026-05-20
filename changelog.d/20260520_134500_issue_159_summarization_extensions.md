---
bump: minor
---

### Added
- Extended the `summarization` module (`src/summarization.rs`) with a README
  ingestion path: `strip_markdown_noise` removes badges, fenced code blocks,
  HTML comments, heading markers, and blockquote chevrons; `formalize_markdown`
  feeds the cleaned prose through the existing classifier; `describe_readme`
  drives the whole `formalize → summarize → deformalize` pipeline with any
  `SummarizationConfig` (returns the repository slug in `Topic` mode).
- Added `DialogTurn`, `formalize_dialog`, `summarize_dialog`, and
  `generate_chat_title` so multi-turn conversations and chat titles are
  produced by the same pipeline (user turns weigh +20, assistant turns -10).
  The conversation-summary intent in `src/solver_handlers/mod.rs` now uses
  this pipeline and logs `summarization:mode`, `summarization:language`, and
  `chat_title` evidence in addition to the per-turn list.
- Wired `try_http_fetch` to recognise curated GitHub repositories
  (`match_curated_github_url`): when the requested URL points to one of the
  projects in `data/seed/projects.lino`, the response embeds a `Standard`-mode
  project description and the trace records `http_fetch:curated_project`,
  `summarization:mode`, and `summarization:language` so the URL → curated
  record → summary path is fully visible.
- Generalized project lookup so Hive Mind is treated as a promoted project
  record instead of a special handler. Project promotion defaults on for
  `link-assistant`, `link-foundation`, and `linksplatform`, can be switched
  off through solver/browser configuration, and explicit GitHub/GitLab/
  Bitbucket repository URLs route through the same `project_lookup` intent.
- Documented the default 30-statement cap as a `DEFAULT_MAX_STATEMENTS`
  constant in `src/summarization.rs` and re-exported it from the crate root.
- Re-exported the new helpers (`describe_readme`, `formalize_markdown`,
  `strip_markdown_noise`, `DialogTurn`, `formalize_dialog`, `summarize_dialog`,
  `generate_chat_title`, `DEFAULT_MAX_STATEMENTS`) from the crate root so
  downstream callers and the integration tests can use them directly.
- Added 11 specification tests in
  `tests/unit/specification/summarization_pipeline.rs` covering the new
  README, dialog, and chat-title flows plus the documented size targets and
  default cap; added two new tests to
  `tests/unit/specification/project_lookups.rs` pinning the HTTP-fetch
  curated-URL evidence.
- Documented the new surfaces in `ARCHITECTURE.md` § 7.1 and added rows
  R202-R207 to `REQUIREMENTS.md`.
