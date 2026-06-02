---
bump: minor
---

### Added
- Settings panel can reset each setting to its default individually, or all of
  them at once (issue #386).
- Conversations list can copy the whole dialog as Markdown; with diagnostics
  mode on, reasoning steps are folded in after each AI message (issue #386).

### Changed
- Prompt recognition references *meanings*, not hardcoded word lists. A new
  canonical lexicon (`data/seed/meanings.lino`) defines language-independent,
  self-describing meanings — each `defined_by` other meanings (a closed graph in
  the spirit of relative-meta-logic), grounded in real lexical data
  (`wiktionary`), tagged with the semantic `role`s it plays, and lexicalised in
  every supported language. The program-artifact follow-up gate
  (`src/program_coreference.rs` and its `formal_ai_worker.js` mirror) no longer
  enumerates ~100 per-language words; it asks the lexicon which surface words
  evidence a `program_artifact` and a `program_modification`, so the words live
  once in data while the code understands the concepts (issue #386).
- Unit-incompatibility detection (`src/solver_handler_units.rs`) is now
  data-driven too. The units, the physical dimensions they measure, and their
  surface words in every supported language live in the lexicon
  (`data/seed/meanings-units.lino`), where each unit meaning is `defined_by` the
  dimension it measures. The handler walks every `measurement_unit` meaning and
  resolves its dimension label through the `defined_by` graph, so the code knows
  only the concepts "measurement unit" and "physical dimension" — no hardcoded
  unit arrays remain. The lexicon is split across `meanings*.lino` files (listed
  by `MEANING_FILES`) so no single seed file breaches the file-size guard; the
  Rust loader and the `formal_ai_worker.js` mirror both walk every `meanings`
  container (issue #386).
- Calendar weekday reasoning (`src/solver_handlers/calendar.rs` and its
  `formal_ai_worker.js` mirror) is data-driven too. The seven weekdays, the
  "day after"/"day before" relations, "today", the day/date/week references, and
  the interrogatives that ask "which day" now live as self-describing meanings
  in `data/seed/meanings-calendar.lino` — each `defined_by` the calendar
  concepts it builds on and lexicalised in every supported language. The handler
  detects the operation and weekday by querying the lexicon for the
  `calendar_direction_next`/`calendar_direction_previous`/`calendar_weekday`/…
  roles instead of matching hardcoded alias and marker arrays. Because the words
  now exist in every language, weekday-relation answers work in Hindi and
  Chinese as well as English and Russian — not only the originally supported
  cases (issue #386).
- Knowledge-base fact-relation detection
  (`src/solver_handlers/benchmark_prompts.rs`) is data-driven too. The nine
  relations a fact query can ask about (capital, population, currency, official
  language, continent, book author, painting painter, build year, physical
  constant) and the surface words that evidence each one in every supported
  language now live as self-describing meanings in
  `data/seed/meanings-facts.lino` — each `defined_by` a `knowledge_relation`
  concept that is in turn `defined_by` `knowledge_subject` and `knowledge_value`
  (a closed cycle in the spirit of relative-meta-logic). `detect_relation` walks
  every meaning carrying the `fact_relation` role in declaration order instead
  of the former hardcoded per-language keyword table, so the code knows only the
  concept "a relation maps a subject to a value" while the words live once in
  data. Declaration order is preserved so the shared "написал" verb still
  resolves to the book author before the painting painter, and the relation
  slugs (hence the `fact_query:relation:*` reasoning trace) stay identical to the
  browser worker (issue #386).
- Software-project request recognition (`src/solver_handlers/software_project.rs`
  and its `formal_ai_worker.js` mirror) is data-driven too. The authoring verbs
  (write/build/create/implement/develop/design/scaffold) and the 19 artifact
  kinds a request can ask for (web app, CLI tool, browser extension, library, …)
  now live as self-describing meanings in
  `data/seed/meanings-software-project.lino` — each artifact kind `defined_by`
  the `software_artifact` genus and lexicalised in every supported language. The
  handler builds its recognition tables by querying the lexicon for the
  `software_authoring_action` and `software_artifact_kind` roles, resolving a
  matched lexeme back to its stable slug; a small in-code resolver maps the slug
  to its canonical English label (the calendar `from_slug` precedent), so
  recognition vocabulary lives in data while the canonical output stays in code.
  The word-boundary scan is now CJK-aware: CJK surfaces match as substrings while
  Latin/Cyrillic/Devanagari keep whole-token boundaries, so a short surface like
  `апи` (API) never matches inside the Cyrillic verb `напиши` ("write") — fixing
  a regression that mislabelled a plain "write a program" request as a software
  project. Because the artifact words now exist in every language, "create a
  library"/"создай библиотеку"/"एक डैशबोर्ड बनाओ"/"开发一个网站" all resolve to
  the same canonical artifact. Feature-requirement detection and subtask
  categorization are data-driven the same way: the seven requirement categories
  (state tracking, data exchange, automation, validation, integration, user
  interface, and a catch-all project behavior) are self-describing meanings
  `defined_by` the `software_feature` genus and lexicalised in every supported
  language. A clause is a requirement when it contains any
  `software_requirement_category` word, and the first category (in declaration
  order) whose word it contains classifies the resulting subtask, so the former
  hardcoded `FEATURE_MARKERS` list and the seven-branch classifier are gone — the
  code knows only the concept "a requirement has a category" (issue #386).
- The remaining software-project request signals are lexicon-driven too. The
  delivery mode (manual instructions, immediate execution, script generation, or
  the default generated code), the implementation language (python, rust,
  javascript, or the default typescript), the game-unit tracker (a request is one
  only when it pairs a `game_tracker_domain` with a `game_tracker_mechanic`), the
  step-granularity and shell/command approval gates, and the whole-prompt approval
  trigger (approve/yes/proceed/…) are now self-describing meanings in
  `data/seed/meanings-software-project.lino`, each `defined_by` the concepts it
  builds on and lexicalised in every supported language. The detectors walk the
  matching `software_delivery_mode`/`software_implementation_language`/
  `game_tracker_*`/`software_step_granularity`/`software_bash_command`/
  `software_approval_trigger` roles (delivery modes and languages in declaration
  order, so the order encodes priority) and resolve a matched slug back to its
  stable label, so the former hardcoded `contains_any`/`contains_word` keyword
  lists are gone — the code knows only the concepts and the words live once in
  data. The boundary-aware, now phrase-capable matcher (`surface_present`)
  replaces raw substring scans, so a short surface like `hp` never matches inside
  `php` and a multi-word go-ahead matches only on word boundaries; the
  `formal_ai_worker.js` mirror walks the same embedded meanings, and all 22
  dialogue examples classify identically in the Rust solver and the browser
  worker (issue #386).
- Python program-synthesis recognition
  (`src/solver_handlers/program_synthesis.rs`, the
  `looks_like_program_synthesis` router in `src/intent_formalization.rs`, and the
  `formal_ai_worker.js` mirror) is data-driven too. The request *subject* (the
  function asked for), its *domain* signals (Python, or a data kind it works over
  — tuple/numbers/vowels), the request *action* verbs (implement/write/return),
  the per-task distinguishing *signals* (distinct numbers/differ/threshold/
  similar elements/count vowels), and the synthesis *tasks* themselves
  (`has_close_elements`, `similar_elements`, `count_vowels` — each slug *is* the
  canonical Python function name) now live as self-describing meanings in
  `data/seed/meanings-program-synthesis.lino`, each `defined_by` the concepts it
  builds on and lexicalised in every supported language. The gate asks the
  lexicon for the `program_synthesis_subject`/`_domain`/`_action` roles; task
  selection walks every `program_synthesis_task` meaning in declaration order and
  picks the first whose `defined_by` `program_synthesis_signal`s are all
  evidenced, using its slug directly as the function name — so the former
  hardcoded English-substring gate and the per-task phrase checks
  (`"similar elements"`, `"count vowels"`, `"distinct numbers" && "differ" &&
  "threshold"`) are gone. The browser worker additionally embeds the full
  multilingual operation vocabulary inline (byte-identical to
  `data/seed/operation-vocabulary.lino`) and runs `canonicalizedPrompt()` — the
  JS mirror of `OperationVocabulary::canonicalized_prompt` — before gating,
  replacing the hand-maintained three-operation `PROGRAM_MODIFIER_OPERATIONS`
  subset it carried before; native operation verbs glued to sentence punctuation
  (a Hindi `लिखें।`, a Chinese `编写`, a Russian `напиши`) now canonicalize to
  their English tokens so the boundary-aware gate accepts them. Because the
  vocabulary now exists in every language, multilingual `count_vowels` and
  `similar_elements` requests in Russian, Hindi, and Chinese synthesize correctly
  in both the Rust solver and the browser worker — not only English (issue #386).
- Conversational-intent recognition is data-driven too. A closed sub-graph of
  conversational meanings (`data/seed/meanings-intent.lino`) defines the
  assistant, user, inquiry, and answer plus the concepts they build on —
  capability, knowledge, fact, introduction, clarification, understanding — each
  `defined_by` the others (a closed graph in the spirit of relative-meta-logic)
  and lexicalised in every supported language. Five role-bearing meanings carry
  the surface words the handlers used to hardcode: `clarification_request`
  ("I don't understand", "не понял", "समझ नहीं आया", "我不明白"),
  `capability_query` ("what can you do", "что ты умеешь", the "что за дичь"
  slang, "你能做什么"), its follow-up `capability_query_more` ("what else can you
  do", "что ещё ты умеешь", "और क्या कर सकते", "你还能做什么"), `self_fact_query`,
  and `self_introduction_request`. The clarification and capability gates
  (`src/solver_handlers/user_intent.rs`) and the self-fact / self-introduction
  gates (`src/solver_handlers/self_awareness.rs`) now ask the lexicon which role
  a prompt evidences instead of matching per-language phrase arrays; each
  re-normalises the prompt first so trailing punctuation ("what can you do?") and
  apostrophes ("I don't understand") collapse to the canonical spacing the seed
  stores. Recognition is language-agnostic — the surface words are
  script-specific — while the per-language response bodies stay in code, so the
  Chinese/Hindi "what else can you do" follow-ups ("你还能做什么", "और क्या कर
  सकते") now reach the capabilities answer even though the former
  Russian/English-only "more" check missed them. The `formal_ai_worker.js` mirror
  queries the same embedded meanings, and a parity harness
  (`experiments/issue-386-js-intent-lexicon.mjs`) proves the worker's role →
  word-sets and its recognizers agree with the seed and the Rust handlers across
  all four languages (issue #386).
- The prefilled "Report issue" body omits settings already at their shipped
  default (Mode, Status, Diagnostics, Theme, Guess/Follow-up probability,
  Temperature, inference-only Location), folds the worker into the version line
  (`<version> (wasm)`), shortens the attach-memory section to a docs pointer, and
  drops the Reasoning Trace when the dialog was trimmed to fit GitHub's URL cap
  (issue #386).
- Documented the issue #386 case study (`docs/case-studies/issue-386/`) with raw
  data, a reconstructed timeline, the full requirements list, a corrected
  root-cause analysis of the "Отмени сортировку" refusal, and the implemented
  inverse-derivation fix.

### Fixed
- The follow-up "Отмени сортировку" ("cancel the sorting") no longer returns
  `intent: unknown`. Operations now declare their inverse in the seed
  (`cancel_reverse_sort` carries `inverse "reverse_sort"`), and the subtractive
  substitution rules are *derived at runtime* by mirroring the additive ones, so
  a "cancel X" follow-up lowers the accumulated program back through "X" —
  restoring the ascending sort while keeping earlier edits such as the path
  argument. Adding a new cancellable operation is now pure seed data with no new
  control flow, and the behavior is covered across English, Russian, Hindi, and
  Chinese in both the Rust solver and the web worker (issue #386).
