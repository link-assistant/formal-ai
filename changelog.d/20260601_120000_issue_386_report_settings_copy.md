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
  categories, …), so following `defined_by` from any of the 163 meanings reaches
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
