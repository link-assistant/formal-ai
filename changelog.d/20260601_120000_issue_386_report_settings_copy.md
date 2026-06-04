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
- Write-a-script recognition (`is_write_script_request` in
  `src/solver_helpers.rs`, its single call site in
  `src/solver_handlers/mod.rs`, and the `formal_ai_worker.js` mirror) is
  data-driven too. Four new semantic roles in `src/seed/roles.rs` name the
  concepts the recogniser reasons over — `program_genus` (the broad "program"
  noun), `script_authoring_verb` (the write / напиши / написать / लिखो / 编写
  author verb, a strict subset of `program_request` that omits
  show/create/generate), `script_or_code_artifact` (the script / code / скрипт /
  код / स्क्रिप्ट / कोड / 脚本 / 代码 noun, a strict subset of `program_kind`
  that excludes the program genus and the function noun), and
  `hello_world_reference` (the canonical hello-world archetype) — carried by the
  `program`, `write`, `script`/`code` meanings and a new self-describing
  `hello_world` meaning in `data/seed/meanings.lino`, each lexicalised in every
  supported language. The recogniser steps aside for the broad program genus and
  the hello-world archetype (which the parametric write-program and
  program-synthesis routes own) and otherwise fires when a `script_authoring_verb`
  meets a `script_or_code_artifact`, so the former hardcoded per-language
  verb/noun substring lists are gone — the code knows only the concept "author a
  script" and the two routes it defers to. Adding the `написать` infinitive to the
  `write` meaning also makes it evidence `program_request` like its imperative
  sibling `напиши`, so "написать код" now routes to the program path consistently
  rather than falling through to `unknown`. The implementation file's unit tests
  moved to a sibling `src/solver_helpers_tests.rs` mounted with `#[path]` (the
  `blueprint_tests.rs` precedent) so the recogniser file stays under the
  1000-line file-size guard, and the worker's embedded `MEANINGS_LINO`
  regenerates byte-identically (issue #386).
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
- The "how does X work" / "how to X" handler (`src/solver_handler_how.rs`) is
  data-driven too. Two self-describing meanings in `data/seed/meanings-how.lino`
  carry every surface the handler used to hardcode: `mechanism_inquiry`
  ("how does X work", "как устроен X", "X कैसे काम करता है", "X 如何工作") and
  `procedural_request` ("how to X", "как сделать X", "कैसे करें X", "如何做 X"),
  each `defined_by` the `inquiry` and `action` concepts and lexicalised in every
  supported language. Rather than carry per-language prefix/circumfix/suffix
  arrays, each surface word encodes the position of the subject (or task) slot
  with an ellipsis marker `…` (U+2026): no marker is a bare phrase, a trailing
  `…` is a prefix surface, a leading `…` is a suffix surface, and a `…` in the
  middle is a circumfix surface. The handler derives its affix-matching strategy
  by bucketing the forms by `WordForm::slot()` (a `Slot` computed from the
  marker) and matching each against the prompt — so the code knows only the
  concepts "an inquiry into a mechanism" and "a request for a procedure", never a
  surface word. A procedural surface may name its canonical operation in an
  `action` child (do/perform/implement/create/write); when it does not, the
  operation is taken from the task's first word. Declaration and bucket order are
  preserved so behaviour is identical to the former inline arrays, and the
  existing multilingual reasoning-path tests still pin "how it works", "как
  устроен AUR", "AUR कैसे काम करता है", "AUR 如何工作", and the procedural
  "how to" cases. The `formal_ai_worker.js` mirror drives its
  `extractHowItWorksSubject` / `extractProceduralHowToTask` recognisers from the
  same embedded meanings — bucketing the slot-marked surfaces by position with a
  shared `makeWordForm` helper exactly as the Rust handler does — instead of the
  inline per-language prefix/circumfix/suffix arrays it carried before. A parity
  harness (`experiments/issue-386-js-how-cluster.mjs`) proves the worker
  reproduces the canonical surface set with the expected per-slot bucket counts
  and returns byte-identical results to the pre-conversion logic across a
  multilingual prompt battery (issue #386).
- The web-intent handlers (`src/solver_handlers/web_requests.rs` and their
  `formal_ai_worker.js` mirror) are data-driven too. Three self-describing
  meanings in `data/seed/meanings-web-navigation.lino` carry every surface the
  two handlers used to hardcode in four inline arrays: `web_resource` (the
  URL-identified thing both intents act on — url/site/page, `defined_by`
  `entity`), `http_fetch` ("fetch …", "сделай запрос к …", "अनुरोध भेजें",
  "发送请求"), and `url_navigate` ("go to …", "открой …", "पर जाएं", "打开"), the
  two verbs each `defined_by` `inquiry` + `action` + `web_resource` and
  lexicalised in every supported language. As in the how-cluster, each surface
  marks its URL slot with the ellipsis marker `…` (U+2026): a trailing `…` is a
  prefix surface ("fetch …" begins "fetch google.com") and no marker is a bare
  phrase matched anywhere ("запрос к" appears inside "сделать запрос к
  google.com"). A shared `role_evidences_web_intent` helper buckets a role's
  forms by `WordForm::slot()` and matches each against the prompt, so
  `is_http_fetch_prompt`/`is_url_navigate_prompt` ask the lexicon for the
  `http_fetch`/`url_navigate` roles instead of carrying
  `HTTP_FETCH_PREFIXES`/`HTTP_FETCH_MARKERS`/`URL_NAVIGATE_PREFIXES`/
  `URL_NAVIGATE_MARKERS` — the code knows only the concepts "fetch a web
  resource" and "navigate to a web resource". The protective URL gate
  (`first_url_candidate`, which rejects `@`-bearing tokens so emails never
  trigger) and the bare-URL navigation early-return are unchanged. Because the
  verbs now exist in every language, Hindi and Chinese fetch/navigate requests
  ("打开 https://…", "获取 https://…", "पर जाएं …") route correctly where the
  former English/Russian-only arrays recognised nothing, with the fetch and
  navigate verb sets staying disjoint. A parity harness
  (`experiments/issue-386-js-web-navigation.mjs`) proves the worker reproduces
  the canonical surface set (16 prefix + 25 bare http_fetch forms, 45 prefix + 27
  bare url_navigate forms), routes 83 English/Russian probes byte-identically to
  the pre-conversion logic through the real URL gate and fetch-before-navigate
  precedence, and adds the Hindi/Chinese coverage the old arrays lacked (issue
  #386).
- Web-search request recognition (`src/solver_handlers/web_search_intent.rs` and
  its `formal_ai_worker.js` mirror) is data-driven too — the deepest of the web
  clusters. Four self-describing seed files carry every surface the recogniser
  used to hardcode in seventeen inline arrays: `meanings-web-search.lino` (the
  `web_search_concept` backbone plus the `web_search_action`/`_strong_action`/
  `_signal`/`_source_only`/`_imperative_lead` roles),
  `meanings-web-search-query.lino` (`web_search_explicit_prefix`, the
  `web_search_topic_marker` whose prefix and suffix forms split into the
  before/after topic markers, and the leading/trailing query-noise roles),
  `meanings-web-research.lino` (the `research_question_opener`/
  `_superlative_modifier`/`research_evidence_domain`/`research_evaluation_domain`
  and `enumeration_request_opener`/`enumeration_constraint` roles), and
  `meanings-web-followup.lino` (`followup_instruction_verb`,
  `clause_continuation_marker`) — each `defined_by` the concepts it builds on and
  lexicalised in every supported language. As in the how- and navigation-clusters
  each surface marks its query slot with the ellipsis marker `…` (U+2026), so the
  recogniser buckets a role's forms by `WordForm::slot()` and matches prefixes,
  suffixes, and bare phrases by position. A single `WebSearchMarkers` projection
  (an 18-field struct on the Rust side, `webSearchMarkers()` memoised on the
  worker) gathers the seventeen roles once — `web_search_topic_marker` feeding two
  fields — and every detector (explicit-prefix stripping, semantic-action
  extraction, enumeration-research and implicit-research-question gating,
  source-only removal, and follow-up-clause truncation) reads from it instead of a
  hardcoded array, so the code knows only the concepts "a web search", "its
  query", "a research question", and "a follow-up instruction". Follow-up
  truncation is now a single universal-boundary routine
  (`truncate_search_instruction_tail`) that cuts the query at the first
  `followup_instruction_verb` lying on a token or sentence boundary in any
  language, replacing the per-language tail heuristics. A new
  `is_personal_fact_filter_request` guard suppresses web search when the prompt
  asks about the user's own contributed facts ("facts I have contributed", "my
  facts"), fixing a leak where the pre-conversion worker returned a bogus
  `{query:"my"}` search for "search my facts". Because the markers now exist in
  every language, source-marker queries ("Find apple on the internet" / "Найди
  яблоко в интернете" / "सेब के बारे में इंटरनेट पर खोजो" / "查找苹果网上信息"),
  enumeration-research, and implicit research questions resolve in Hindi and
  Chinese as well as English and Russian. A parity harness
  (`experiments/issue-386-js-web-search.mjs`) proves the worker reproduces all
  seventeen role word-sets from the seed, exposes the eighteen-field marker
  projection memoised, reproduces a frozen 33-prompt golden of pre-conversion
  behaviour byte-identically, and matches the Rust handler's multilingual
  source-marker, enumeration, implicit-research, and follow-up-drop cases — 78
  assertions, all green (issue #386).
- Every meaning now descends from a single ontology root, so the lexicon is one
  connected graph rather than disjoint clusters. A new backbone
  (`data/seed/meanings-ontology.lino`) defines `link` as the self-rooted root of
  the merged ontology (the relative-meta-logic "everything is a link" stance),
  `type` as a type-system sub-root directly under it, and
  `entity`/`concept`/`relation`/`action`/`property` as the top-level categories
  every domain genus roots in. Each existing cluster gains a `defined_by` edge up
  into one of these categories (`program` → `entity`, `sort`/`modify` →
  `action`, `quantity` → `property`, `calendar_day` → `concept`,
  `knowledge_relation` → `relation`, the software-project genera → their
  categories, …), so following `defined_by` from any meaning reaches
  `link`. A public ontology-reasoning API (`Lexicon::ontology_root`,
  `Lexicon::reaches_root`) and two invariants
  (`the_ontology_has_a_single_link_root`, `every_meaning_reaches_the_link_root`)
  enforce it; the `formal_ai_worker.js` mirror carries the same backbone and the
  parity harness proves the worker forms one connected ontology under the single
  `link` root (issue #386).
- Self-awareness known-facts recognition (`src/solver_handlers/self_awareness.rs`
  and its `formal_ai_worker.js` mirror) is data-driven too. The "facts" noun, the
  enumerating interrogatives (what/which/list/show), the second-person attribution
  of knowing (you know / you have / тебе известно / 你知道 / …), and the complete
  standalone phrasings that ask what the assistant knows now live as
  self-describing meanings in `data/seed/meanings-intent.lino` — the shared `fact`
  noun (reused through its `knowledge` definition rather than duplicated) plus the
  new `knowledge_inventory_probe`, `assistant_knowing`, and
  `knowledge_inventory_query` meanings, each `defined_by` the
  `knowledge`/`inquiry`/`fact` concepts and lexicalised in every supported
  language. `is_known_fact_query` now composes four semantic roles —
  `knowledge_inventory_noun` ∧ `knowledge_inventory_interrogative` ∧
  `knowledge_possession`, or the standalone `knowledge_inventory_phrase` — with one
  universal algorithm for every language instead of four per-language word
  conjunctions. Two deliberate consistency refinements follow: Chinese now also
  requires an explicit second-person marker (你知道/您知道/你有/您有), so a bare
  noun-only "哪些事实" falls through exactly as the English "which facts" does; and
  the Russian noun matches clean citation forms (факт/факты) at token boundaries
  like every other lexicon noun, rather than the former stem-fragment
  `.contains("факт")`. `self_awareness_language` now detects the language purely by
  Unicode script range (the Cyrillic range subsumes the former hardcoded
  second-person pronoun list), and the now-unused `contains_any` helper was
  removed (issue #386).
- Conversation-summary recognition (`try_summarize_conversation` in
  `src/solver_handlers/mod.rs` and its `formal_ai_worker.js` mirror) is
  data-driven too. Four self-describing meanings in
  `data/seed/meanings-intent.lino` carry every surface the recogniser used to
  hardcode in an English exact-set, a fifteen-entry prefix set, and three
  per-language anchored regexes: `conversation_summary_directive` (the summarize
  / суммируй / резюме / सारांश / 总结 verb), `conversation_reference` (the
  conversation / беседа / बातचीत / 对话 noun the directive can take as object),
  `conversation_summary_phrase` (complete standalone phrasings such as "summarize
  so far", "what have we talked about", "о чём мы разговаривали"), and
  `conversation_summary_courtesy` (objectless courtesy frames such as "can you
  summarize", "подведи итог", "सार दो"), each `defined_by` the `inquiry` concept
  and the summary concepts it builds on, and lexicalised in every supported
  language. `asks_for_conversation_summary` now composes those roles with one
  universal algorithm for every language — a standalone phrase, a courtesy frame,
  a directive together with a conversation reference, or a bare directive (the
  whole prompt for whitespace-delimited scripts, a leading directive for CJK) —
  instead of the former English exact-set / prefix lists and the
  Russian/Hindi/Chinese anchored regexes. Two refinements follow from reasoning
  over the concept rather than the raw words: the CJK bare directive now anchors
  at the start (`总结…`) so a compound like "工作总结" (a *work* summary) no longer
  mis-triggers — fixing a Rust `.contains("总结")` bug that the worker's `^总结`
  regex never had — and the directive-plus-reference conjunction recognises any
  conversation reference ("summarize our discussion", "резюме разговора"), not
  only the handful of "the/this/our conversation/chat" prefixes the worker
  enumerated. The generic `words_for_role` accessor the bare-directive check uses
  is now named identically on both sides (the worker's misnamed-but-generic
  `calendarWordsForRole`, already used for non-calendar roles, was renamed
  `wordsForRole`). The `formal_ai_worker.js` mirror queries the same embedded
  meanings, and a parity harness
  (`experiments/issue-386-worker-summarize-parity.mjs`) proves the recogniser
  fires on nineteen multilingual phrasings across all four composition arms,
  rejects content-summary and unrelated prompts, routes the four pinned
  with-history cases to `summarize_conversation`, and honours the empty-history
  turn gate (issue #386).
- The remaining user-intent recognisers (`src/solver_handlers/user_intent.rs`
  and its `formal_ai_worker.js` mirror) are data-driven too — proof requests,
  who-is questions, and the prior-turn web-search signal. A new self-describing
  seed file (`data/seed/meanings-proof.lino`) defines five meanings: `prove`
  (carrying both the clause-initial `proof_directive` bare verbs — prove / proof
  / докажи / доказать / … — and the `proof_claim_scaffold` prefixes that strip
  the claim out of "prove that …" / "докажи что …" / "साबित करो कि …" / "证明…",
  separated by slot within the one meaning), `proof_request_frame` (the English
  `proof_request_lead` frames that need no *that* clause — "can you prove …",
  "give me a proof of …"), `proof_assertion` (the mid-prompt `proof_marker`
  substrings in every language), and the `godel` / `determinism` proof concepts
  (`proof_concept_godel` / `proof_concept_determinism`); the who-is surfaces move
  into a `who_is_question` meaning in `data/seed/meanings-intent.lino` (the
  head-initial `who_question_lead` prefix — "who is …", "кто такой …" — and the
  head-final `who_question_tail` suffix — "… कौन है", "…是谁"); and the
  prior-turn signal becomes a `web_search_mention` meaning in
  `data/seed/meanings-web-search.lino` carrying the raw `web_search_history_signal`
  substrings. `is_proof_request`, `extract_claim_from_prompt`, `is_who_question`,
  the Goedel/determinism guards, and `prior_history_mentions_web_search` now ask
  the lexicon for those roles — bucketing each role's forms by `WordForm::slot()`
  so the clause-initial verb-boundary check, the first-matching-prefix claim
  extraction, and the head-initial/head-final who-is split are all derived from
  the data — instead of the former hardcoded per-language word arrays; the four
  generic affix helpers shared with the web-search cluster
  (`search{Prefix,Suffix,Bare,Source}Literals`) are renamed to the
  universal `{prefix,suffix,bare,source}Literals` now that proof and who-is reuse
  them. Reasoning over the concept also unified the Rust proof-marker behaviour
  with the worker's (it gained three Russian mid-sentence markers it had lacked),
  with no test regressing. A parity harness
  (`experiments/issue-386-worker-user-intent-parity.mjs`) loads the committed
  baseline and the working-tree worker into separate sandboxes and proves the
  four recognisers return byte-identical results across a 50-prompt multilingual
  matrix — including the prover/proven/improve/approve boundary negatives and
  claim extraction with leading noise — 221 assertions, all green (issue #386).
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
- Every meaning in the lexicon now lexicalises *all* supported languages
  (en/ru/hi/zh), enforced unconditionally by the
  `every_meaning_covers_all_supported_languages` invariant. The two remaining
  English-/Russian-only meanings were backfilled with genuine surfaces: the
  broad proof request-frame (`proof_request_frame`, role `proof_request_lead`)
  gained Russian, Hindi and Chinese leads — each embedding an existing
  `proof_marker` substring (доказать / साबित / 证明 …) so recognition stays
  behaviour-neutral while the request-frame concept is complete in every
  language — and the prior-turn web-search signal (`web_search_mention`, role
  `web_search_history_signal`) gained Hindi and Chinese surfaces. A
  language-coverage audit (`experiments/issue-386-audit-language-coverage.mjs`)
  and the 221-assertion parity harness confirm the backfill leaves every
  recogniser byte-identical to its pre-backfill behaviour (issue #386).
- The policy and edge-case handlers (`src/solver_handlers_policy.rs`, the
  `is_inappropriate_content` screen in `src/solver_helpers.rs`, and the
  `formal_ai_worker.js` mirror) are data-driven too. A new seed file
  (`data/seed/meanings-policy.lino`) defines three self-describing meanings, each
  rooted in the `link` ontology and lexicalised in every supported language:
  `physical_action_query` (role `physical_action_trigger` — the crude "did you
  …" taunt the assistant answers factually because it has no physical body),
  `circular_joke_idiom` (role `circular_joke_phrase` — «купи слона» and its
  buy-an-elephant calque), and `vulgar_content` (role `vulgar_content_marker` —
  the English profanity and Russian mat migrated verbatim from the old hardcoded
  refusal lists, plus Hindi and Chinese equivalents). `try_physical_action_question`,
  `try_kupi_slona`, and `is_inappropriate_content` now ask the lexicon for those
  roles as raw substrings instead of carrying inline word arrays, so the code
  knows only the concepts while the surfaces live once in data; the physical-
  action and buy-elephant replies localise through `seed::response_for`. Because
  the idiom is now lexicalised everywhere, the buy-an-elephant calque routes to
  the same handler in every language, and the content screen generalises to
  Hindi and Chinese obscenities it never covered before. A vm parity harness
  (`experiments/issue-386-js-policy.mjs`) proves the worker's buy-elephant
  recogniser and its embedded policy lexicon — including the
  Rust-only `vulgar_content_marker` and `physical_action_trigger` roles — stay on
  par across all four languages (issue #386).
- The currency rate-basis handler (`src/solver_handlers/calculator_rate.rs` and
  its `formal_ai_worker.js` mirror) is data-driven too. A new seed file
  (`data/seed/meanings-calculator.lino`) defines four self-describing meanings,
  each rooted in the `link` ontology and lexicalised in every supported language:
  a `money` genus (`defined_by` `concept`, role `monetary_concept` — structural
  only, no handler queries it) that groups the currency meanings so they build
  from a shared concept, the `exchange_rate` between currencies (`defined_by`
  `money` + `relation`, role `exchange_rate_reference`), the `us_dollar` currency
  (`defined_by` `money`, role `currency_usd_reference` — including the two common
  Russian misspellings долар/долор), and the `calculation_basis` question frame
  (`defined_by` `action` + `inquiry`, role `calculation_basis_reference` — the
  "do you use … for calculations" / "у тебя … при расчётах" side of the prompt).
  `asks_for_usd_rate_basis` now composes the three queried roles as raw substrings
  via `Lexicon::mentions_role_raw` — an `exchange_rate_reference` *and* a
  `currency_usd_reference` *and* a `calculation_basis_reference` — instead of the
  former three hardcoded per-language `contains` disjunctions, so the code knows
  only the concepts while every surface lives once in data. The migration is
  byte-faithful: the role surface sets equal the original recognizer lists exactly
  (the worker even gains the "calculations" plural the Rust list always carried), so
  the USD/RUB delegation is behaviour-neutral. A vm parity harness
  (`experiments/issue-386-js-calculator-rate.mjs`) proves the worker routes the
  five spec prompts to the calculator in all four languages, falls through on
  currency prompts that miss one of the three concepts, and reproduces every role's
  surface set byte-for-byte across en/ru/hi/zh (issue #386).
- Natural-language skill recognition (`src/skill_compiler.rs`, its
  `structured.rs` permission/determinism screens, and the
  `formal_ai_worker.js` mirror) is data-driven too. The trigger leads
  ("when i say", "when the user says/asks", "if i ask"), the response verbs
  (answer/reply/respond, the Russian stem "ответ", …), the standalone
  behaviour-rule edit directives, and the conditional when-then frames now live
  as self-describing meanings in `data/seed/meanings-skill-compiler.lino` — each
  `defined_by` the concepts it builds on (a trigger lead `defined_by` `relation`
  + `inquiry`, a when-then frame `defined_by` `relation` + `concept`, …) and
  lexicalised in every supported language. The when-then frames are stored as
  `Slot::Circumfix` word forms whose literal before the ellipsis … (U+2026) is
  the head clause and whose literal after it is the link clause, so the head/link
  keyword pairs that were hardcoded in both runtimes now live once in data.
  `looks_like_skill_description` and `explicit_teaching_form` query the
  `skill_teaching_trigger_lead`, `skill_teaching_response_verb`,
  `behavior_rule_edit_directive`, and `skill_when_then_pair` roles via
  `Lexicon::mentions_role_raw`/`role_word_forms` instead of the former inline
  string lists and the `WHEN_THEN_KEYWORD_PAIRS` table. The structured-skill
  determinism screen and the implicit-capability inference likewise read the
  `nondeterministic_marker`, `shell_capability_cue`, and `network_capability_cue`
  roles (shell checked before network so a step touching both is attributed to
  the shell), while the formal `tool:local_shell`/`tool:web_fetch` identifiers
  stay in code as a tool-namespace bridge. Because the surfaces now cover every
  language uniformly, the browser worker gains the trigger leads it used to miss
  ("when the user says"/"when the user asks") and the "respond" verb. A 28-case
  truth table is shared, case for case, between the Rust inline test
  (`skill_description_recogniser_reads_every_language_from_the_lexicon`) and a vm
  parity harness (`experiments/issue-386-worker-skill-trigger-parity.mjs`) so the
  two runtimes are proven to agree across en/ru/hi/zh (issue #386).
- Natural-language tool/API recognition
  (`src/solver_handlers/natural_language_tools.rs`) is data-driven too. A new seed
  file (`data/seed/meanings-tool-access.lino`) defines five self-describing
  meanings, each rooted in the `link` ontology and lexicalised in every supported
  language: `tool_invocation_cue` (role `tool_invocation_cue` — the call / invoke /
  run / api / tool surfaces, `defined_by` `action`), `calculator_tool` (role
  `calculator_tool_name`, `defined_by` `entity`), `web_search_tool` (role
  `web_search_tool_name`, including the `web_search`/`web search`/`web-search`
  spellings, `defined_by` `entity`), `local_shell_tool` (role
  `local_shell_request_cue` — the whole request phrases such as "local shell tool"
  and "invoke the shell tool", which bundle verb and tool name so the cue is
  decisive on its own, `defined_by` `entity`), and `tool_argument_marker` (role
  `tool_argument_marker` — the "with query" / "query" / "with" / "for" argument
  introducers, `defined_by` `relation`). `is_explicit_tool_api_request` now asks
  the lexicon whether a prompt evidences a named tool together with a
  `tool_invocation_cue`, `is_explicit_local_shell_request` asks for the
  `local_shell_request_cue` alone, and the fallback argument extractor walks the
  English `tool_argument_marker` forms in declaration (priority) order, so the
  former hardcoded alias slices, the space-padded cue substrings (`" api"`,
  `"call "`, …), the local-shell phrase list, and the four `after_marker` calls
  are gone — the code knows only the concepts "an explicit tool call", "the named
  calculator/web-search/shell tool", and "the phrase that introduces a tool
  argument". Matching is token-bounded (the CJK-substring / whole-token contract),
  so a cue like "tool" no longer matches inside a larger word; the English forms
  drive the argument heuristic while the other languages stay in the seed for
  self-description. The handler is Rust-only — the browser worker has no
  natural-language-tool route — but its embedded `MEANINGS_LINO` mirrors the new
  file byte-identically so the shared knowledge base stays complete (issue #386).
- Feature-capability recognition (`src/solver_handlers/feature_capability.rs` and
  its `formal_ai_worker.js` mirror) is data-driven too. A new seed file
  (`data/seed/meanings-feature-capability.lino`) defines nineteen self-describing
  meanings, each rooted in the `link` ontology and lexicalised in every supported
  language: sixteen `feature_capability_*` alias meanings (role
  `feature_capability_alias`, `defined_by` `concept` — the surface words that name
  each of the sixteen advertised features: web search, diagnostics, agent mode,
  definition fusion, configuration, memory actions, greeting, write program,
  concept lookup, arithmetic, translation, memory, demo mode, http url, javascript
  execution, planning), the `feature_capability_question` frame (role
  `feature_capability_question`, `defined_by` `action`, grounded in *can* — the
  "can you …" / "do you support …" / "умеешь ли …" / "你能…" availability question
  the recogniser keys on), and the two action-request gates
  `feature_action_arithmetic` and `feature_action_planning` (roles
  `feature_action_arithmetic`/`feature_action_planning`, `defined_by` `action` —
  the imperative "can you calculate …" / "can you summarize …" frames that must
  route to the live arithmetic and planning handlers, not the capability answer).
  `detect_feature_capability`, `is_feature_capability_question`, and
  `is_feature_action_request` now ask the lexicon for those roles — resolving a
  matched `feature_capability_alias` meaning back to its stable slug, gating on the
  `feature_capability_question` frame (with the pre-existing English
  "is/are … enabled/available" availability shape kept as a structural fallback),
  and stepping aside when an arithmetic/planning *action* frame leads the prompt —
  instead of the former per-feature alias arrays and the hardcoded
  `WEB_SEARCH_CAPABILITY_PHRASES` / `featureAliases` lists, so the code knows only
  the concepts "a feature", "a question about whether a feature is available", and
  "an imperative that should run the feature instead". The migration is
  byte-faithful to the alias/action split that origin/main relied on: the
  arithmetic *alias* set stays `arithmetic` / `calculate` / `math` / `2 + 2`
  (Russian `арифмет` / `считать` / `посчитать`, …) while `compute` lives only in
  the `feature_action_arithmetic` *action* role, so a bare "Can you compute 7 * 6?"
  detects no capability and falls through to the calculation handler exactly as
  before. Because the alias words now exist in every language, feature-availability
  questions resolve uniformly across en/ru/hi/zh. A vm parity harness
  (`experiments/issue-386-js-feature-capability.mjs`) replays the sixty
  feature×language rows and twenty web-search probes from the Rust battery in
  `tests/unit/specification/capabilities.rs`, the arithmetic/planning action gates,
  the alias/action role separation, and the availability-frame fallback — 95
  assertions, all green — proving the worker's recogniser agrees with the Rust
  solver in every language (issue #386).
- The Playwright starter-script recogniser
  (`src/solver_handlers/playwright_script.rs` and its `formal_ai_worker.js`
  mirror) is data-driven too. A new seed file (`data/seed/meanings-playwright.lino`)
  defines two self-describing meanings rooted in the `link` ontology and
  lexicalised in every supported language: `playwright` (role
  `playwright_tool_name`, `defined_by` `entity` — the tool name plus its common
  misspelling, whose `playright` word form carries `action "playwright"` naming
  the canonical spelling) and `playwright_script_request_cue` (role
  `playwright_script_cue`, `defined_by` `concept` — the script-authoring cues
  *script* / *test* / *spec* / *code* / *write* / *create* / *generate* / *make* /
  *build* / "can you" / "could you" and their ru/hi/zh equivalents).
  `is_playwright_script_request` now gates on both roles via `mentions_role_raw`
  (raw substring, byte-faithful to the former two `contains` pairs), and
  `mentions_playwright_misspelling` resolves the misspelled form by its `action`,
  so the handler still reports the `Playright -> Playwright` correction without
  naming either spelling in code — the code knows only "the Playwright tool" and
  "a request to author a script for it".
- Research comparison-table recognition (`src/solver_handlers/research_table.rs`
  and its `formal_ai_worker.js` mirror) is data-driven too. A new seed file
  (`data/seed/meanings-research-table.lino`) defines eight self-describing
  meanings rooted in the `link` ontology and lexicalised in every supported
  language: the comparison trigger `compare` (role `comparison_table_trigger`),
  the weak pair `table` (role `comparison_table_noun`) and `differences` (role
  `comparison_difference_cue`), the `research_prompt_signal` meaning (role
  `research_prompt_signal` — bare markers like 'web search' / 'research' plus
  prefix surfaces 'search …' / 'find information …' whose `…` slot the code reads
  as a `before_slot` opener), and four `research_criterion` meanings declared in
  column order (`key_differences` / `use_cases` / `advantages` / `disadvantages`).
  `is_comparison_table_request`, `looks_like_research_prompt`, and
  `append_criteria_from_text` now ask the lexicon for those roles, and a new
  `Criterion::from_slug` keys each column off the matched meaning's slug — so the
  code names only the language-independent slug, never a surface word. Declaration
  order fixes the K/U/A/D column order, and the space-guarded criterion stems
  `pro ` and ` con ` keep their surrounding spaces (the criterion match stays a
  raw `contains`) so they never match inside *process* / *control*. Because the
  comparison gate is now token-bounded across en/ru/hi/zh, the apple-on-the-internet
  web-search prompts (which name no comparison/table/difference surface in any
  language) still route to web search rather than tripping the table follow-up.
  The Rust lib and unit suites stay green — including the Playwright
  clarification/starter and research comparison-table specifications — and the
  worker's inline `MEANINGS_LINO` was regenerated so the mirror verifier
  (`experiments/issue-386-meanings-mirror.mjs`) reports byte-identical parity
  across all twenty-eight meaning files (issue #386).
- Conversational-opener topic extraction (`conversation_topic` in
  `src/solver_handlers/benchmark_prompts.rs` and its `formal_ai_worker.js`
  mirror) is data-driven too. A new seed file
  (`data/seed/meanings-conversation.lino`) defines the
  `conversation_topic_opener` meaning (role `conversation_topic_opener`,
  `defined_by` the `inquiry` and `action` concepts) rooted in the `link`
  ontology and lexicalised in every supported language — the let-us-talk-about-X
  phrasings ("let's talk about …", "давай поговорим о …", "चलो बात करें …",
  "聊聊…"). Each surface marks the topic position with the ellipsis `…` (U+2026)
  slot marker, so the recogniser walks the role's prefix forms in declaration
  order and strips the opener via `before_slot()` instead of the former
  fifteen-entry per-language prefix array. The single surface whose `action` is
  `scan` ("поговорим о …") is additionally matched anywhere in the prompt,
  preserving the old `split_once` fallback that catches an opener following a
  greeting — so the code knows only the concept "an opener that proposes a
  topic" while the surfaces live once in data. The `formal_ai_worker.js` mirror
  queries the same embedded meanings and the mirror verifier
  (`experiments/issue-386-meanings-mirror.mjs`) reports byte-identical parity
  across all twenty-nine meaning files (issue #386).
- Software-project follow-up output extraction (`extract_expected_output` in
  `src/solver_handlers/software_project_followup.rs`) is data-driven too. A new
  `output_display_request` meaning in `data/seed/meanings-software-project.lino`
  (role `output_display_request`, `defined_by` the `software_followup` and
  `action` concepts) carries the show-me/print/display openers that name what
  the user wants surfaced ("show me …", "show …", "print …", "display …", plus
  ru/hi/zh surfaces such as "покажи мне …" / "मुझे दिखाओ …" / "给我看…"). Each
  surface marks the output position with the ellipsis `…` (U+2026) slot marker,
  so the handler walks the role's forms in declaration order — the longer "show
  me " tried before the bare "show " — strips the opener via `before_slot()`,
  and reads the clause that follows from the original-case prompt (stopped at
  the first sentence-ending punctuation, capped at twelve words) instead of the
  former hardcoded four-marker array. The opener is still matched anywhere in
  the prompt, so "test it and show me the result" keeps capturing "the result".
  Because the openers now exist in every language, a Hindi or Chinese follow-up
  records its expected output where the English-only array recognised nothing.
  The extractor is Rust-only — the browser worker has no follow-up output route
  — but its embedded `MEANINGS_LINO` mirrors the new meaning byte-identically so
  the shared knowledge base stays complete, with the mirror verifier reporting
  parity across all twenty-nine meaning files (issue #386).
- The mechanism-inquiry subject cleanup (`strip_mechanism_tail` /
  `clean_mechanism_subject` in `src/solver_handler_how.rs` and the
  `formal_ai_worker.js` mirror) is data-driven too — completing the how-cluster
  conversion. Three new self-describing meanings in `data/seed/meanings-how.lino`
  carry the surfaces these helpers used to hardcode in three inline arrays:
  `mechanism_predicate` (role `mechanism_predicate`, `defined_by` `action` +
  `mechanism_inquiry` — the "… work" / "… works" / "… structured" / … predicate
  tails a prefix match leaves behind), `detail_modifier` (role `detail_modifier`,
  `defined_by` `property` + `mechanism_inquiry` — the "… in detail" / "…
  internally" / "… please" / … thoroughness-or-politeness tails), and
  `non_referential_subject` (role `non_referential_subject`, `defined_by`
  `entity` + `mechanism_inquiry` — the pronouns and dangling function words "it" /
  "this" / "does …" / "to …" / … that name no real topic). As in the rest of the
  cluster each surface marks its slot with the ellipsis `…` (U+2026): the predicate
  and detail tails are suffix surfaces whose text after the slot is the literal to
  strip (tried in declaration order — the predicate by return-on-first-match, the
  modifiers stripped together in one re-trimming pass), while the reject set mixes
  bare surfaces matched against the whole candidate with prefix surfaces matched
  against its start. `strip_mechanism_tail` walks the `mechanism_predicate` role
  and `clean_mechanism_subject` walks `detail_modifier` then
  `non_referential_subject`, so the former `[" work", …]` / `[" in detail", …]`
  arrays and the nineteen-entry `PRONOUN_SUBJECTS` set with its four `starts_with`
  checks are gone — the code knows only the concepts "the predicate that completes
  a how-it-works clause", "an optional detail modifier", and "a subject that names
  no real topic". Because the surfaces now cover every language, the ru/hi/zh
  predicate tails and the hi/zh detail modifiers are stripped where the
  English-/Russian-only arrays left them intact, while every English and Russian
  case stays byte-identical. The `formal_ai_worker.js` mirror drives
  `cleanMechanismSubject` / `stripMechanismTail` from the same embedded meanings,
  and a differential parity harness
  (`experiments/issue-386-js-mechanism-subject.mjs`) reconstructs the
  pre-conversion arrays and proves the two functions are byte-identical to them
  across forty-nine English/Russian probes, documents the ten intended
  all-language generalizations, and confirms the issue-#386 reasoning paths — the
  mirror verifier reporting parity across all twenty-nine meaning files (issue
  #386).
- The procedural-task cleanup (`clean_procedural_fragment` /
  `correct_common_procedural_typos` in `src/solver_handler_how.rs` and the
  `formal_ai_worker.js` mirror) is data-driven too — finishing the how-cluster
  conversion. Two new self-describing meanings in `data/seed/meanings-how.lino`
  carry the surfaces these helpers used to hardcode in a seventeen-entry suffix
  array and a single-entry typo table: `procedural_task_modifier` (role
  `procedural_task_modifier`, `defined_by` `property` + `procedural_request` —
  the trailing "step by step" / "in steps" / "for me" / "please" / … and their
  ru/hi/zh equivalents that a procedural extractor strips from the end of the
  task) and `common_typo` (role `common_typo`, `defined_by` `relation` — a
  misspelling paired with its correction, the canonical case being the
  transposed "dirven" -> "driven"). Each modifier surface is a `Slot::Suffix`
  whose text after the ellipsis `…` (U+2026) is the literal tail to strip, walked
  in declaration order with the first match winning, so the longer Russian
  "напиши по шагам" is still tried before its "по шагам" tail; each typo surface
  is a `Slot::Bare` whose `action` child names the correct spelling, so a task
  token is repaired by data rather than a hardcoded `token == "dirven"` check.
  The former inline suffix array and the one-branch typo table are gone — the
  code knows only the concepts "a trailing step-by-step or politeness modifier"
  and "a misspelling and its correction". Because the surfaces now cover every
  language, the genuine ru/hi/zh typos (руский -> русский, वेबसाईट -> वेबसाइट,
  登陆 -> 登录) are repaired where the English-only table did nothing, while every
  pre-existing case stays byte-identical. A differential parity harness
  (`experiments/issue-386-js-procedural-cluster.mjs`) reconstructs the
  pre-conversion suffix array and typo table, proves the two functions are
  byte-identical to them across thirty-two cleanup probes and six typo probes
  (all four languages, order-sensitivity, punctuation and whitespace controls),
  documents the three intended all-language typo generalizations, and confirms
  the four issue-#343 spec-driven prompts still reduce to "spec driven
  development" with a recorded dirven->driven fix — the mirror verifier reporting
  parity across all twenty-nine meaning files (issue #386).
- The prior-reply topic scan (`extract_topic_from_prior_reply` in
  `src/solver_handler_how.rs`) is data-driven too — closing out the how-cluster
  conversion. When a "how does it work?" follow-up names no subject and the prior
  assistant reply has no "Term (category):" header, the handler falls back to the
  first capitalised token that is not a function word. That skip list was a
  hardcoded English title-case array
  (`["I", "The", "A", "An", "In", "To", "For", "Of", "And", "Or", "Source"]`); it
  now lives as a self-describing `topic_scan_stop_word` meaning in
  `data/seed/meanings-how.lino` (role `topic_scan_stop_word`, `defined_by` the
  `concept` category — closed-class articles, prepositions, conjunctions and
  pronouns plus the 'source' citation heading, lexicalised in every supported
  language). The handler walks the role's `Slot::Bare` forms and compares them
  case-insensitively, so the code knows only the concept "a function word that
  names no topic". The case-insensitive match is a strict superset of the former
  case-sensitive comparison: it reproduces the old behaviour for ordinary
  title-case prose and additionally skips all-caps English function words and
  capitalised Cyrillic ones that the English-only array left to be mis-read as the
  topic. The extractor is Rust-only — the browser worker has no prior-reply topic
  route — but its embedded `MEANINGS_LINO` mirrors the new meaning byte-identically
  so the shared knowledge base stays complete, and a regression test
  (`how_it_works_prior_reply_fallback_skips_function_words_case_insensitively`)
  pins the generalization while the mirror verifier reports parity across all
  twenty-nine meaning files (issue #386).
- Counting numbers are a self-describing ontology too, so the code reads counts
  from data instead of hardcoded number words. A new cardinal-number sub-graph
  in `data/seed/meanings-units.lino` defines the `cardinal_number` genus
  (`defined_by` the `quantity` property) and the leaves `zero`…`ten` (role
  `cardinal_number_word`, each `defined_by` `cardinal_number`), lexicalised in
  every supported language; each leaf's English lexeme carries both the spelled
  word ("ten") and the script-independent numeral surface ("10"), from which the
  cardinal's integer value is read. `contains_spelled_arithmetic`
  (`src/calculation.rs`) now asks the lexicon for the `cardinal_number_word`
  forms — skipping the pure-numeral surfaces the numeric parser already handles —
  instead of the former twenty-six-entry English/Russian number-word table, and
  the brainstorm-count recogniser (`requested_brainstorm_count` in
  `src/solver_handlers/benchmark_prompts.rs` and the worker's
  `requestedBrainstormCount`) derives the requested count from the `ten`
  cardinal's own numeral surface via a new `cardinal_value` / `cardinalValue`
  helper, replacing the hardcoded `TEN_HINTS` / `tenHints` literal. Matching is
  the boundary-aware lexicon contract, so the spelled "ten" no longer false-
  matches inside "often" and the Chinese 十 matches as a substring; because the
  cardinals now exist in every language, "придумай десять идей" / "दस नाम सुझाओ" /
  "给我十个想法" all resolve to ten where the former English-leaning table missed
  them. A new role `ROLE_CARDINAL_NUMBER_WORD` (`src/seed/roles.rs`) names the
  concept, and a vm parity harness
  (`experiments/issue-386-worker-brainstorm-count-parity.mjs`) proves the worker
  reads the count from the seed across the pinned English prompts, the
  multilingual cardinal cases, and the "often" substring negative (issue #386).
- The browser worker's `currencyCodeFromWord` no longer carries hand-written
  Russian declension tables for the dollar and ruble. It now walks the
  `currency_usd_reference`, `currency_eur_reference` and `currency_rub_reference`
  roles (`data/seed/meanings-calculator.lino`) and returns the ISO 4217 code of
  the first role a surface matches, with the canonical code mapped in a tiny
  `currencyCodeForRole` resolver (the output stays in code; only the recognition
  vocabulary lives in the seed). Matching follows each surface's script the same
  way `surfacePresent` already splits CJK from the rest: Latin and CJK/Devanagari
  surfaces match the whole token exactly — so unrelated words such as "rubbish"
  or "european" are rejected just as the original exact-match list rejected them
  — while Cyrillic surfaces are treated as stems and matched by prefix, so every
  Russian declension (доллар…, руб…) is recognised from the доллар / руб stems
  without enumerating each inflected form. A vm parity harness
  (`experiments/issue-386-worker-currency-code-parity.mjs`) proves the seed-driven
  walk returns byte-identical codes to the former tables across all 13 USD, 4 EUR
  and 14 RUB inputs, rejects the unrelated words, and still resolves the
  "1000 рублей в долларах" → USD capture pinned by the calculator delegation and
  multilingual e2e tests (issue #386).
- The calculator's spelled-operator detector (`contains_word_operator` in
  `src/calculation.rs`) no longer carries a 14-element array of operator words.
  The spelled operators now live as their own ontology: an `arithmetic_operation`
  genus (`defined_by "action"`) with five operations beneath it — `addition`,
  `subtraction`, `multiplication`, `division`, `modulo` — each `defined_by` that
  genus and carrying its operator surfaces in English, Russian, Hindi and Chinese
  (`data/seed/meanings-calculator.lino`). A new role
  `ROLE_ARITHMETIC_OPERATOR_WORD` (`src/seed/roles.rs`) marks those surfaces, and
  the detector reads them through `Lexicon::mentions_role`, which matches each
  operator as a whole token (and CJK surfaces as a substring) — the same boundary
  contract the former space-padded `.contains` checks enforced for the English and
  Russian operators, now extended to every language the meanings lexicalise. The
  pinned spelled-operator delegations ("two plus two", "nine multiplied by nine",
  "шесть умножить на семь", "the fifth Fibonacci number multiplied by 10", …) keep
  resolving through the existing Rust unit suite (issue #386).
- The calculator's request-cue stripper (`strip_calculation_wrappers` in
  `src/calculation.rs`) no longer carries a 28-element array of leading prompt
  cues. Those cues — imperatives like "calculate" / "посчитай" / "गणना करें" and
  question openers like "what is" / "сколько будет" / "请计算" — now live in a new
  `calculation_request` meaning (`defined_by "action"` and `defined_by "inquiry"`)
  with their surfaces in English, Russian, Hindi and Chinese
  (`data/seed/meanings-calculator.lino`). A new role
  `ROLE_CALCULATION_REQUEST_CUE` (`src/seed/roles.rs`) marks them, and the
  stripper reads them through `Lexicon::words_for_role`, rebuilding each surface
  into a strip prefix that follows its script: space-delimited scripts gain a
  trailing space so a cue strips only on a word boundary ("calculate" never eats
  the start of "calculated"), while CJK surfaces strip as-is because those scripts
  have no inter-word spaces. The Chinese cues are stored longest first, and
  `words_for_role` preserves declaration order, so a more specific cue strips
  before a shorter one it contains. The pinned prompt-stripping delegations keep
  resolving through the existing Rust unit suite (issue #386).
- The browser worker's calculation-signal recognizers (`src/web/formal_ai_worker.js`)
  now read the same meanings as the Rust solver instead of carrying their own
  literal arrays. `hasArithmeticWordOperator` reads `ROLE_ARITHMETIC_OPERATOR_WORD`
  through `lexiconMentionsRole` (dropping the 14-element `ARITHMETIC_WORD_OPERATORS`),
  `hasSpelledArithmetic` reads `ROLE_CARDINAL_NUMBER_WORD` through `roleWordForms`,
  skipping pure-numeral surfaces (dropping the 26-element `ARITHMETIC_NUMBER_WORDS`),
  and `extractArithmeticExpression` rebuilds its leading-cue prefixes from
  `ROLE_CALCULATION_REQUEST_CUE` via `wordsForRole` (dropping the 28-element prefix
  array). Each conversion is byte-faithful to the former arrays for every English
  and Russian case and additionally recognises the Hindi and Chinese surfaces the
  seed lexicalises, exactly mirroring the Rust `contains_word_operator` /
  `contains_spelled_arithmetic` / `strip_calculation_wrappers` changes. A new
  parity harness `experiments/issue-386-worker-calc-signal-parity.mjs` pins the
  equivalence (issue #386).
- The calculator's *trailing*-cue stripper no longer carries a hardcoded suffix
  list either — completing `strip_calculation_wrappers` (`src/calculation.rs`)
  and its worker twin `extractArithmeticExpression`
  (`src/web/formal_ai_worker.js`). The trailing cues a calculation prompt may
  carry split into two self-describing meanings in
  `data/seed/meanings-calculator.lino`: `calculation_result_query`
  (`defined_by` `action` + `inquiry`, role `calculation_result_query_cue` — the
  equals word or sign, the how-much-is-it question, and the head-final
  do-the-calculation imperative: equal / equals / = / равно / 是多少 / 等于多少 /
  等于几 / कितना है / क्या है / की गणना करें) and `politeness` (`defined_by`
  `property`, role `politeness_cue` — the courtesy tail that carries no task
  content: please / for me / пожалуйста / कृपया / 请). A new
  `calculation_wrapper_suffixes` helper walks the two roles via
  `Lexicon::words_for_role` and rebuilds each surface into a strip suffix
  following its script — CJK surfaces strip as-is (no inter-word spaces), a
  pure-symbol surface like the equals sign strips both bare and on a word
  boundary (so a compact `2*2+2=` is recognised), and every other surface gains
  a leading space so the cue strips only on a word boundary — replacing the
  former thirteen-element Rust array and the eleven-regex worker array (two new
  roles `ROLE_CALCULATION_RESULT_QUERY_CUE` / `ROLE_POLITENESS_CUE` in
  `src/seed/roles.rs` name the concepts). The conversion is byte-faithful to the
  former arrays for every English and Russian case and adds the bare-`=` strip
  to the worker so the two engines now agree on `2*2+2=` (the old worker left the
  sign in place); because the cues now exist in every language, the new ru
  `равно`, hi `कृपया` and zh `请` surfaces — needed for the every-language
  invariant — strip where the old arrays left them, while the Hindi cues gain a
  required leading space so they too strip only on a boundary. A vm parity
  harness (`experiments/issue-386-worker-calc-suffix-parity.mjs`) reconstructs
  the pre-conversion regexes and proves the worker is byte-identical to them
  across the English/Russian/Chinese cases, applies the bare-`=` consistency fix
  and the three multilingual generalizations, and composes prefix-plus-suffix
  stripping — all green (issue #386).
- The calculator router's currency-conversion exemption
  (`has_calculation_signal` in `src/calculation.rs`) no longer hardcodes a
  to/into/convert/exchange list. A prompt that pairs a currency symbol with
  letters but is not an explicit `calculate` command is otherwise treated as
  prose and rejected; a conversion is itself a calculation, so a conversion cue
  must exempt it. Those cues now live in a new self-describing
  `quantity_conversion` meaning (`defined_by` `action` + `relation`, role
  `quantity_conversion_cue`) in `data/seed/meanings-calculator.lino`,
  lexicalised in every supported language — the bare target markers to / into,
  the verbs convert / exchange, and their ru/hi/zh equivalents (конвертировать /
  обмен, बदलें / परिवर्तित, 转换 / 兑换). The guard reads them through
  `Lexicon::mentions_role` (a new role `ROLE_QUANTITY_CONVERSION_CUE` in
  `src/seed/roles.rs` names the concept), which matches each surface
  whole-token in space-delimited scripts — byte-faithful to the former
  `lower.contains(" to ")` so the markers to/into still count only on a word
  boundary, never inside another word — and as a substring in Chinese. This is a
  dedicated meaning rather than a reuse of `conversion_action` (the
  money-specific verb the compound-interest handler matches as a raw substring):
  adding the bare markers to/into there would match them everywhere, whereas the
  router's general conversion signal must stay whole-token. The exemption is
  strictly more permissive (it can only flip the prose-rejection guard from
  reject to accept), so adding the multilingual surfaces leaves every existing
  case byte-identical. The browser worker has no twin — its conversion rescue is
  the separate `evaluateCurrencyConversionExpression` — so only its embedded
  `MEANINGS_LINO` was re-synced byte-identically; a regression test
  (`calculator_currency_conversion_is_exempt_from_prose_rejection`) pins the
  English behaviour and the no-cue contrast (issue #386).
- The calculator router's known-domain-word gate (`has_calculation_signal` in
  `src/calculation.rs` and the `extractArithmeticExpression` gate in
  `src/web/formal_ai_worker.js`) no longer carries a hardcoded signal array — the
  62-entry Rust list and the worker's 39-entry list are both gone. The surfaces
  whose presence beside a number marks a prompt as a calculation now live as
  self-describing meanings read through three roles: `math_function_name` (a new
  `mathematical_function` genus in `data/seed/meanings-calculator.lino` with
  `square_root` / `sine` / `cosine` / `tangent` / `logarithm` /
  `natural_logarithm` beneath it, each lexicalised in every supported language),
  `calculation_domain_term` (carried by the currency meanings `us_dollar` /
  `euro` / `ruble` and the calculator-relevant measurement units `kilobyte` /
  `megabyte` / `kilogram` / `gram` / `ton` / `second` / `minute` / `hour` /
  `millisecond` / `day` / `month` in `data/seed/meanings-units.lino`, each still
  `defined_by` the dimension it measures), and the CJK members of the existing
  `quantity_conversion_cue`. A shared `calculator_domain_signals` helper (its
  worker twin `calculatorDomainSignals`, using a new `isAsciiText` mirror of
  Rust's `str::is_ascii`) shapes each surface by script: a `math_function_name`
  gains only a leading space so it still fires when glued to a parenthesis
  ("sqrt(16)"); a `calculation_domain_term` is matched whole-token (leading and
  trailing space) for ASCII so a short code never fires inside a longer word;
  non-ASCII surfaces in both roles match as raw substrings so every inflected
  form is caught; and only the CJK conversion verbs (转换 / 兑换 / 换成) become
  signals — the Latin to/into are far too common to mark a calculation on their
  own. Two new roles `ROLE_MATH_FUNCTION_NAME` / `ROLE_CALCULATION_DOMAIN_TERM`
  (`src/seed/roles.rs`) name the concepts, so the code knows only "a mathematical
  function" and "a calculator-domain term" while the words live once in data. The
  conversion is byte-faithful to the former arrays for the whole-token currency
  codes and units, and tightens the latent substring false positives the old
  leading-space-only word forms allowed (for example "euro" inside "european" and
  "dollar" inside "dollarized") now that ASCII domain terms match whole-token.
  Because the surfaces now cover every language uniformly, the worker gains the
  sin/cos/tan/log/ln math functions and the CJK/Devanagari unit surfaces it
  lacked, and both engines drop the Russian and Hindi month *names* (феврал /
  январ, फरवरी / जनवरी) the old arrays carried: those name calendar months, not
  durations, and a genuine date-difference calculation still carries a duration
  unit (месяцев / दिन / months) the domain-term role covers — while the Chinese
  month names stay recognised because 二月 / 一月 embed the 月 month-unit ideograph
  the role matches as a substring. A vm parity harness
  (`experiments/issue-386-worker-calc-signal-parity.mjs`) reconstructs the
  pre-conversion worker array and proves the new `calculatorDomainSignals` gate is
  byte-identical across the agreement cases, documents the eight intended
  differences (the month-name drops, the whole-token tightening, the
  math-function adds, and the Chinese unit adds), and confirms
  `extractArithmeticExpression` still routes "convert 10 tons to kg" and
  "300000 ms in seconds" to the calculator while rejecting digit-free CJK prose
  (issue #386).
- The arithmetic evaluator's spelled→symbolic rewrite (`normalize_expression` in
  `src/arithmetic.rs` and its worker twin `normalizeArithmeticWords` in
  `src/web/formal_ai_worker.js`) no longer carries a hardcoded word→value map —
  the Rust `ARITHMETIC_WORD_TOKENS`/phrase pairs and the worker's five phrase
  regexes plus its `ARITHMETIC_WORD_TOKENS` map are all gone. A spelled
  expression ("two plus three", "пять умножить на два", "पाँच गुणा दो") is
  rewritten into its symbolic form ("2 + 3", "5 * 2") from the seed: every
  `cardinal_number_word` and `arithmetic_operator_word` meaning carries its
  script-independent *value surface* — the word form with no alphabetic
  character, the numeral "2" for the cardinal two, the symbol "+" for addition —
  and each spelled surface maps onto it. The five operator meanings
  (`data/seed/meanings-calculator.lino`) gained that symbol word form so an
  operator is value-carrying exactly as a cardinal already was. A new
  `Lexicon::arithmetic_normalization_tables` (`src/seed/meanings.rs`) derives the
  `(tokens, phrases)` mapping — single words applied after tokenization, longest-
  first multi-word phrases applied before it so "разделить на" rewrites before
  the shorter "делить на" it contains — and `normalize_expression` folds the
  phrases then maps the tokens. Because `arithmetic.rs` is compiled into the wasm
  worker (`#![no_std]`, no `build.rs`) and cannot reach the seed at runtime, the
  table is materialized at author time into the `no_std` static
  `src/arithmetic_word_tables.rs` by `examples/issue_386_gen_arith_table.rs` and
  pinned to the live seed by the `arithmetic_word_tables_match_seed` test
  (`src/calculation.rs`), so a stale table fails CI. The worker's
  `arithmeticNormalizationTables()` derives the same mapping from its inline
  `MEANINGS_LINO`. The new symbol word forms are detection-neutral: a new
  `Lexicon::mentions_role_spelled` (worker `lexiconMentionsRoleSpelled`) skips
  value surfaces, so the spelled-operator gate (`contains_word_operator` /
  `hasArithmeticWordOperator`) still keys only on the alphabetic operator words
  and never treats a bare "+" as one. Because the mapping now spans every
  language the meanings lexicalise, the worker gains the Hindi space-separated
  ("पाँच गुणा दो" → "5 * 2") and the CJK/"по модулю" arithmetic the former
  English/Russian-only map lacked. A vm parity harness
  (`experiments/issue-386-worker-arith-normalize-parity.mjs`) proves the worker's
  derived tables equal the Rust-generated static entry-for-entry (67 tokens, 6
  phrases, order included) and that `normalizeArithmeticWords`/`evaluateArithmetic`
  agree on en/ru/hi golden cases — so all three representations (the two language
  builders and the materialized table) are proven identical (issue #386).
- The unknown-implementation-language extractor (`requested_program_language`
  in `src/intent_formalization.rs` and the `programLanguageFromPrompt` mirror in
  `formal_ai_worker.js`) no longer hardcodes the function words that introduce a
  language name. Two new self-describing meanings —
  `implementation_language_preposition` ("in"/"на") and
  `implementation_language_noun` ("language"/"языке"), in
  `data/seed/meanings-software-project.lino` — carry those surfaces in every
  supported language; the positional scan reads the head-initial English/Russian
  markers from the lexicon and returns the bare language name trailing them, so
  an unknown target such as "write a program in Brainfuck" still resolves with no
  literals in the parser. The catalog-driven resolution of *known* languages is
  unchanged; worker parity with the Rust extractor is proven entry-for-entry by
  `experiments/issue-386-worker-program-language-parity.mjs` (issue #386).
- The translation handler's define-in-Links-Notation gate (`try_translation`
  in `src/solver_handlers/mod.rs`) no longer keys on the literal verb `define `
  or the format phrases ` links notation` / ` в links`. Two new meanings —
  `definition_command` (the imperative verb) and `links_notation_format` (the
  target-format name), in `data/seed/meanings-translation.lino` — carry those
  surfaces in every supported language; the gate composes the head-initial
  English verb with a quoted or backticked phrase and an English/Russian format
  marker sourced from the lexicon, preserving the original recogniser exactly.
  The scanned surface set is locked by the lib test
  `define_in_links_roles_expose_the_scanned_surfaces` (`src/seed/meanings.rs`),
  and a dispatch-level test documents that `concept_lookup` answers these prompts
  first so the refactor changes nothing observable (issue #386).
- The worker's `N% of M <currency>` recognizer (`formal_ai_worker.js`) no
  longer hardcodes the `usd|eur|rub|dollars?|euros?|rubles?` alternation. A new
  cached `percentOfExpressionRegex()` builds the trailing-currency alternation
  from the same three currency-reference roles `currencyCodeFromWord` resolves,
  so the recognizer captures exactly the ISO codes and the English/Cyrillic/CJK
  /Devanagari names the resolver already understands — longest-first and
  regex-escaped. Parity with the resolver is proven by
  `experiments/issue-386-worker-percent-of-currency-parity.mjs` (issue #386).

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
- "Можешь написать мне Playwright скрипт?" (and its English counterpart) again
  route to the Playwright starter-script handler instead of the generic
  write-program clarification. The issue #386 generalisation of
  `writeProgramParameters` made "написать … скрипт" look like a bare
  write-program request, and the browser worker dispatched `tryWriteProgram`
  ahead of `tryPlaywrightScript` — the reverse of the canonical Rust order where
  `try_playwright_script` runs before the specialized-handler group. The worker
  dispatch was reordered to mirror `src/solver.rs`, with a vm regression harness
  (`experiments/issue-386-worker-playwright-dispatch.mjs`) asserting the
  Playwright handler wins for both languages while a bare "напиши программу"
  still reaches write-program (issue #386).
- CJK prose no longer triggers a phantom unit-incompatibility refusal. The
  unit-word boundary check (`contains_unit_word` in `src/solver_handler_units.rs`)
  previously took the permissive substring path for every non-ASCII unit, so the
  day unit "天" matched inside "天气" (weather) and the gram unit "克" inside the
  transliteration "弗拉克斯", turning a units-free Chinese prompt into a bogus
  time-vs-mass incompatibility answer. Because CJK ideographs are alphabetic to
  `char::is_alphabetic` and the scripts have no inter-word spaces, the same
  word-boundary rule already used for ASCII units now also applies to CJK units —
  a unit glued inside a larger compound is rejected, while one next to a digit
  ("7天", "5千克") or at a token edge still matches. Inflected alphabetic scripts
  (Russian "килобайт" → "килобайте", Hindi "किलोबाइट") keep the permissive
  substring path, since they attach suffixes directly to the unit (issue #386).
