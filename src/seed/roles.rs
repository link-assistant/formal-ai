//! Semantic-role identifiers for the meaning lexicon (issue #386).
//!
//! Recognition code never matches a hardcoded natural-language word; it asks
//! the [lexicon](super::lexicon) "which surface forms evidence role *X*?" and
//! names only the language-independent role. Those role identifiers live here,
//! in one registry, so the loader in [`super::meanings`] stays focused on
//! parsing and querying and keeps clear of the seed file-size guard.
//!
//! Each constant is the exact `role` string a meaning declares in
//! `data/seed/meanings*.lino`. A role only needs a constant when *code* queries
//! it; roles that exist purely to group data (for example
//! `web_navigation_concept`) stay in the seed without a mirror here.

/// Semantic role: a thing a program produces that a later turn can refer back
/// to (a result, an output, the program/script/code itself, an ordering).
pub const ROLE_PROGRAM_ARTIFACT: &str = "program_artifact";
/// Semantic role: an operation a follow-up turn can request against the active
/// program (sort, reverse, cancel, change, …) — additive or subtractive.
pub const ROLE_PROGRAM_MODIFICATION: &str = "program_modification";
/// Semantic role: a kind of program artifact a user can ask to be authored
/// (a program, a script, code, a function). The noun side of "write a <kind>".
pub const ROLE_PROGRAM_KIND: &str = "program_kind";
/// Semantic role: a verb that requests a program artifact be produced (write,
/// create, show, generate, make, build). The verb side of "write a <kind>".
pub const ROLE_PROGRAM_REQUEST: &str = "program_request";
/// Semantic role: the program *genus* itself — the broad "program" noun
/// (program / программа / प्रोग्राम / 程序).
///
/// The script-authoring recognizer defers to the parametric write-program
/// route whenever this appears, so a full "write a program" request keeps its
/// richer formalization rather than collapsing to a bare script.
pub const ROLE_PROGRAM_GENUS: &str = "program_genus";
/// Semantic role: the author verb specifically used to request a script or
/// code be composed (write / напиши / написать / लिखो / 编写).
///
/// The verb side of "write a script" — a strict subset of
/// [`ROLE_PROGRAM_REQUEST`] that omits the show/create/generate verbs, which
/// must not on their own trigger synthesis.
pub const ROLE_SCRIPT_AUTHORING_VERB: &str = "script_authoring_verb";
/// Semantic role: a script-or-code artifact noun (script / code / скрипт /
/// код / स्क्रिप्ट / कोड / 脚本 / 代码).
///
/// The noun side of "write a script" — a strict subset of
/// [`ROLE_PROGRAM_KIND`] that excludes the broad program genus and the
/// function noun.
pub const ROLE_SCRIPT_OR_CODE_ARTIFACT: &str = "script_or_code_artifact";
/// Semantic role: a surface reference to the canonical hello-world archetype
/// — the codebase's minimal first program.
///
/// The script-authoring recognizer defers to the program-synthesis route for
/// these so the hello-world example keeps its dedicated handling.
pub const ROLE_HELLO_WORLD_REFERENCE: &str = "hello_world_reference";
/// Semantic role: a concrete unit of measurement (metre, byte, kilogram, …).
/// Each such meaning is `defined_by` the [`ROLE_PHYSICAL_DIMENSION`] it measures.
pub const ROLE_MEASUREMENT_UNIT: &str = "measurement_unit";
/// Semantic role: a physical dimension (length, mass, time, …). Units that
/// belong to different dimensions cannot be converted into one another.
pub const ROLE_PHYSICAL_DIMENSION: &str = "physical_dimension";
/// Semantic role: a named day of the week (Monday … Sunday). The meaning slug
/// is the English weekday name so a handler can resolve a matched lexeme back
/// to a position in the seven-day cycle.
pub const ROLE_CALENDAR_WEEKDAY: &str = "calendar_weekday";
/// Semantic role: the "comes after" relation between weekdays — a +1 step in
/// the seven-day cycle (after, next day, после, के बाद, 之后, …).
pub const ROLE_CALENDAR_DIRECTION_NEXT: &str = "calendar_direction_next";
/// Semantic role: the "comes before" relation between weekdays — a -1 step in
/// the seven-day cycle (before, previous day, перед, से पहले, 之前, …).
pub const ROLE_CALENDAR_DIRECTION_PREVIOUS: &str = "calendar_direction_previous";
/// Semantic role: the present day relative to the system clock (today,
/// сегодня, आज, 今天). Distinguishes a "what day is it now?" query.
pub const ROLE_CALENDAR_TODAY: &str = "calendar_today";
/// Semantic role: a reference to a day, date, or week — the noun a calendar
/// question is about (day, weekday, date, week, день, неделя, 星期, …).
pub const ROLE_CALENDAR_DAY_REFERENCE: &str = "calendar_day_reference";
/// Semantic role: an interrogative or imperative asking which day (what,
/// which, какой, कौन, 什么, …). The question side of a calendar query.
pub const ROLE_CALENDAR_QUESTION: &str = "calendar_question";
/// Semantic role: a relation in the knowledge base that maps a subject to a
/// value (capital, population, author, …).
///
/// A fact query detects which relation a prompt asks about by walking every
/// meaning carrying this role, in declaration order; each is `defined_by` the
/// `knowledge_relation` concept.
pub const ROLE_FACT_RELATION: &str = "fact_relation";
/// Semantic role: a follow-up that verifies an already-designed software
/// artifact behaves correctly (test it, run the tests, протестируй, 测试, …).
pub const ROLE_SOFTWARE_FOLLOWUP_VERIFICATION: &str = "software_followup_verification";
/// Semantic role: a follow-up that runs or executes the designed artifact
/// (run it, execute it, запусти, 运行, चलाओ, …).
pub const ROLE_SOFTWARE_FOLLOWUP_EXECUTION: &str = "software_followup_execution";
/// Semantic role: a follow-up that demonstrates the artifact's output
/// (show me, demo it, покажи, 显示, दिखाओ, …).
pub const ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION: &str = "software_followup_demonstration";
/// Semantic role: a verb that requests a software artifact be authored.
///
/// Surfaces include write, build, create, implement, develop, design, scaffold,
/// … — the verb side of "build me a <artifact>". Distinct from
/// `program_request`, which gates the narrower "write a <program>" synthesis
/// path; the two overlap on the shared verbs, but a software-authoring verb
/// need not trip program synthesis.
pub const ROLE_SOFTWARE_AUTHORING_ACTION: &str = "software_authoring_action";
/// Semantic role: a kind of software artifact an authoring request can ask for.
///
/// Examples are a web app, a CLI tool, a browser extension, a library, …. Each
/// is `defined_by` the `software_artifact` genus; a handler resolves a matched
/// lexeme back to its slug and maps the slug to a canonical English label.
pub const ROLE_SOFTWARE_ARTIFACT_KIND: &str = "software_artifact_kind";
/// Semantic role: a category a software feature requirement falls into.
///
/// Examples are state tracking, data exchange, automation, validation,
/// integration, user interface, and a catch-all project behavior. The union of
/// these meanings' words detects that a clause states a feature requirement;
/// the first category (in declaration order) whose word appears classifies it,
/// so the code knows only the concept "a requirement has a category".
pub const ROLE_SOFTWARE_REQUIREMENT_CATEGORY: &str = "software_requirement_category";
/// Semantic role: the software-feature genus (feature, requirement, …). A
/// prompt that mentions a feature/requirement earns the "requirements"
/// approval gate.
pub const ROLE_SOFTWARE_FEATURE: &str = "software_feature";
/// Semantic role: how the assistant should deliver a software solution.
///
/// The non-default modes — manual instructions, immediate execution, script
/// generation — each carry this role. A handler walks them in declaration
/// order (so the order encodes priority) and selects the first evidenced in
/// the prompt, falling back to code generation when none is.
pub const ROLE_SOFTWARE_DELIVERY_MODE: &str = "software_delivery_mode";
/// Semantic role: the programming language a software implementation targets.
///
/// python, rust, javascript, …. Walked in declaration order; the first
/// evidenced language wins, else the default (typescript) is used.
pub const ROLE_SOFTWARE_IMPLEMENTATION_LANGUAGE: &str = "software_implementation_language";
/// Semantic role: a tabletop/RPG game domain.
///
/// A D&D unit, token, wargame piece, Owlbear scene, …. A request is a
/// game-unit tracker only when it pairs a domain with a mechanic (see
/// [`ROLE_GAME_TRACKER_MECHANIC`]).
pub const ROLE_GAME_TRACKER_DOMAIN: &str = "game_tracker_domain";
/// Semantic role: a combat mechanic a tabletop tracker follows — hit points,
/// damage, protection, resistance, cooldowns. Pairs with the domain above.
pub const ROLE_GAME_TRACKER_MECHANIC: &str = "game_tracker_mechanic";
/// Semantic role: a request to approve the work step by step (each step, step
/// by step, …) — adds the `each_step` approval gate.
pub const ROLE_SOFTWARE_STEP_GRANULARITY: &str = "software_step_granularity";
/// Semantic role: a shell or command-line surface (shell, bash, a command,
/// docker, `WebVM`, …) — adds the `bash_command` approval gate.
pub const ROLE_SOFTWARE_BASH_COMMAND: &str = "software_bash_command";
/// Semantic role: a whole-prompt approval trigger (approve, yes, proceed, …).
///
/// Unlike the other roles this matches the *entire* compacted prompt, not a
/// passing mention: a go-ahead like "approve plan" moves the dialogue from
/// plan to implementation, while "approve the email validation step" does not.
pub const ROLE_SOFTWARE_APPROVAL_TRIGGER: &str = "software_approval_trigger";
/// Semantic role: the subject of a program-synthesis request — the *function*
/// it asks to be written (the noun side of "implement a function …").
pub const ROLE_PROGRAM_SYNTHESIS_SUBJECT: &str = "program_synthesis_subject";
/// Semantic role: a domain signal of a program-synthesis request — the target
/// language (Python) or a data kind it works over (tuple, numbers, vowels).
pub const ROLE_PROGRAM_SYNTHESIS_DOMAIN: &str = "program_synthesis_domain";
/// Semantic role: the request/specification verb of a program-synthesis
/// request (implement, write, return). The verb side of "implement a function".
pub const ROLE_PROGRAM_SYNTHESIS_ACTION: &str = "program_synthesis_action";
/// Semantic role: a surface signal that distinguishes one synthesis task.
///
/// The "distinct numbers"/"differ"/"threshold"/"similar elements"/"count
/// vowels" phrases. A task is `defined_by` the signals that evidence it.
pub const ROLE_PROGRAM_SYNTHESIS_SIGNAL: &str = "program_synthesis_signal";
/// Semantic role: a concrete synthesis task.
///
/// Its slug is the canonical Python function name (`has_close_elements`,
/// `similar_elements`, `count_vowels`). Walked in declaration order; a task is
/// selected when its name is declared or when every `program_synthesis_signal`
/// it is `defined_by` is evidenced in the prompt.
pub const ROLE_PROGRAM_SYNTHESIS_TASK: &str = "program_synthesis_task";
/// Semantic role: the user signalling they did not understand the assistant.
///
/// Asks it to make a prior answer clear ("I don't understand", "не понял",
/// "समझ नहीं आया", "我不明白", …). A meaning carrying this role is `defined_by`
/// the `clarification` and `understanding` concepts.
pub const ROLE_CLARIFICATION_REQUEST: &str = "clarification_request";
/// Semantic role: the user asking what the assistant is able to do.
///
/// A request to enumerate its capabilities ("what can you do", "что ты умеешь",
/// "你能做什么", …). Distinct from [`ROLE_CAPABILITY_QUERY_MORE`], the follow-up.
pub const ROLE_CAPABILITY_QUERY: &str = "capability_query";
/// Semantic role: the user asking what *else* the assistant can do.
///
/// A follow-up that requests capabilities beyond those already named ("what
/// else can you do", "что ещё ты умеешь", …) — a superset signal layered over
/// the base [`ROLE_CAPABILITY_QUERY`].
pub const ROLE_CAPABILITY_QUERY_MORE: &str = "capability_query_more";
/// Semantic role: the user asking the assistant to list facts about itself.
///
/// "facts about yourself", "факты о себе", "自我事实", …. Checked before the
/// broader self-introduction and known-facts queries so it wins the overlap.
pub const ROLE_SELF_FACT_QUERY: &str = "self_fact_query";
/// Semantic role: the user asking the assistant to introduce itself.
///
/// A get-acquainted request ("tell me about yourself", "расскажи о себе",
/// "介绍一下你自己", …). Suppressed when a [`ROLE_SELF_FACT_QUERY`] surface
/// also matches.
pub const ROLE_SELF_INTRODUCTION_REQUEST: &str = "self_introduction_request";
/// Semantic role: the noun naming the items a known-facts inventory asks about.
///
/// The "facts" surface inside a known-facts question ("what *facts* do you
/// know", "какие *факты* ты знаешь", "你知道什么*事实*", …). Carried by the shared
/// `fact` meaning, which is `defined_by` the `knowledge` concept, so the noun is
/// reused rather than duplicated. Composed with the interrogative and possession
/// roles to recognise a known-facts query.
pub const ROLE_KNOWLEDGE_INVENTORY_NOUN: &str = "knowledge_inventory_noun";
/// Semantic role: the interrogative or enumerating cue of a known-facts query.
///
/// The "what / which / list / show" surface that asks the assistant to surface
/// the items it holds ("какие", "перечисли", "哪些", …). A meaning carrying this
/// role is `defined_by` the `inquiry` concept.
pub const ROLE_KNOWLEDGE_INVENTORY_INTERROGATIVE: &str = "knowledge_inventory_interrogative";
/// Semantic role: second-person attribution of knowing to the assistant.
///
/// The "you know / you have / known to you" surface that pins the knowledge to
/// the assistant ("ты знаешь", "тебе известно", "你知道", …). A meaning carrying
/// this role is `defined_by` the `knowledge` and `assistant` concepts.
pub const ROLE_KNOWLEDGE_POSSESSION: &str = "knowledge_possession";
/// Semantic role: a complete standalone phrasing of a known-facts query.
///
/// A full surface that asks what the assistant knows about the world even when
/// the noun "facts" is absent ("what do you know about the world", "что тебе
/// вообще известно", "你知道什么", …). A meaning carrying this role is `defined_by`
/// the `knowledge` and `fact` concepts, and matches on its own without the
/// noun/interrogative/possession conjunction.
pub const ROLE_KNOWLEDGE_INVENTORY_PHRASE: &str = "knowledge_inventory_phrase";
/// Semantic role: the verb or noun that asks for a condensed summary.
///
/// The "summarize / summary / резюмируй / резюме / 总结" surface that directs the
/// assistant to condense discourse into its essence. A meaning carrying this
/// role is `defined_by` the `inquiry` and `answer` concepts. Composed with
/// [`ROLE_CONVERSATION_REFERENCE`] (or matched as a leading directive) to
/// recognise a request to summarize the running conversation.
pub const ROLE_CONVERSATION_SUMMARY_DIRECTIVE: &str = "conversation_summary_directive";
/// Semantic role: the noun naming the running dialogue between user and assistant.
///
/// The object a summary request points at ("conversation", "беседа",
/// "разговор", "对话", …). A meaning carrying this role is `defined_by` the
/// `inquiry` and `answer` concepts. Conjoined with
/// [`ROLE_CONVERSATION_SUMMARY_DIRECTIVE`] so "summarize the conversation"
/// triggers while a bare "summarize X" leaves other objects to other handlers.
pub const ROLE_CONVERSATION_REFERENCE: &str = "conversation_reference";
/// Semantic role: a complete standalone phrasing asking what the dialogue covered.
///
/// A full surface that asks the assistant to recount the conversation even when
/// no separate directive verb is present ("what have we talked about", "о чём мы
/// разговаривали", "我们聊了什么", …). A meaning carrying this role is `defined_by`
/// the `inquiry` and `conversation_reference` concepts, and matches on its own
/// without the directive/reference conjunction.
pub const ROLE_CONVERSATION_SUMMARY_PHRASE: &str = "conversation_summary_phrase";
/// Semantic role: a polite or elliptical frame requesting a summary.
///
/// An objectless courtesy surface that asks for a summary without naming the
/// conversation directly ("give me a summary", "can you summarize", "подведи
/// итог", "总结一下", …). A meaning carrying this role is `defined_by` the
/// `inquiry` and `conversation_summary_directive` concepts, and matches on its
/// own without the directive/reference conjunction.
pub const ROLE_CONVERSATION_SUMMARY_COURTESY: &str = "conversation_summary_courtesy";
/// Semantic role: a prompt asking how something works.
///
/// An inquiry into a mechanism or operating principle ("how does X work",
/// "как устроен X", "X कैसे काम करता है", "X 如何工作", …). Each surface marks the
/// subject position with the ellipsis (U+2026) slot marker (see
/// [`crate::seed::Slot`]); a meaning carrying this role is `defined_by` the
/// `inquiry` and `action` concepts.
pub const ROLE_MECHANISM_INQUIRY: &str = "mechanism_inquiry";
/// Semantic role: a prompt requesting the ordered steps to accomplish a task.
///
/// The how-to-X procedure question ("how to X", "как сделать X", "कैसे करें X",
/// "如何做 X", …). Every surface is a [`crate::seed::Slot::Prefix`] carrying the
/// task after the slot; a surface may name the canonical operation in an
/// `action` child.
pub const ROLE_PROCEDURAL_REQUEST: &str = "procedural_request";
/// Semantic role: a prompt asking to fetch a web resource over HTTP.
///
/// The retrieve-this-URL request ("fetch X", "сделай запрос к X", "अनुरोध भेजें",
/// "获取", …). Surfaces split into [`crate::seed::Slot::Prefix`] forms (the
/// literal precedes the URL — "fetch …") and [`crate::seed::Slot::Bare`]
/// markers matched anywhere in the prompt; a separate URL gate means a surface
/// only routes here when the prompt also carries a real URL. A meaning carrying
/// this role is `defined_by` the `inquiry`, `action`, and `web_resource`
/// concepts.
pub const ROLE_HTTP_FETCH: &str = "http_fetch";
/// Semantic role: a prompt asking to open or show a web resource.
///
/// The navigate-to-this-URL request ("open X", "перейди на X", "पर जाएं",
/// "打开", …) — open the page rather than fetch its bytes. Surfaces split into
/// [`crate::seed::Slot::Prefix`] forms (the literal precedes the URL — "open …")
/// and [`crate::seed::Slot::Bare`] markers matched anywhere in the prompt; a
/// bare URL on its own also counts. Like [`ROLE_HTTP_FETCH`] it is URL-gated and
/// `defined_by` the `inquiry`, `action`, and `web_resource` concepts.
pub const ROLE_URL_NAVIGATE: &str = "url_navigate";
/// Semantic role: an explicit "search the web for …" lead-in.
///
/// A [`crate::seed::Slot::Prefix`] surface whose literal, once stripped, leaves
/// the search query verbatim ("search the web for …", "найди в интернете …", …).
/// Checked first by the web-search recogniser because the query is whatever
/// follows the lead-in.
pub const ROLE_WEB_SEARCH_EXPLICIT_PREFIX: &str = "web_search_explicit_prefix";
/// Semantic role: a verb that asks to search/find/research something.
///
/// The union of every search verb across languages (" search ", " find ",
/// " поищи ", "搜索", …). A semantic web search needs an action marker present;
/// the [`ROLE_WEB_SEARCH_STRONG_ACTION`] subset is decisive on its own, while
/// the weaker verbs additionally require a [`ROLE_WEB_SEARCH_SIGNAL`].
pub const ROLE_WEB_SEARCH_ACTION: &str = "web_search_action";
/// Semantic role: a search verb decisive enough to imply web search alone.
///
/// The subset of [`ROLE_WEB_SEARCH_ACTION`] that does not need a co-occurring
/// reference-source signal (" search ", " research ", " поищи ", "搜索", …). The
/// generic "find/locate/learn" verbs (" find ", " найди ", …) are deliberately
/// *not* strong: they route to web search only alongside a signal word.
pub const ROLE_WEB_SEARCH_STRONG_ACTION: &str = "web_search_strong_action";
/// Semantic role: a reference-source signal noun.
///
/// Marks that a prompt is about looking something up on the web or in a
/// reference work (" web ", " internet ", " wikipedia ", " information ",
/// "信息", …). Pairs with a weak action verb to confirm web-search intent.
pub const ROLE_WEB_SEARCH_SIGNAL: &str = "web_search_signal";
/// Semantic role: a connective that delimits the search topic.
///
/// Carried by a single meaning whose slot encodes the direction: a
/// [`crate::seed::Slot::Prefix`] surface ("about …", "on …", "о …", "关于…")
/// introduces the topic *after* the marker, while a [`crate::seed::Slot::Suffix`]
/// surface ("… के बारे में", "… की जानकारी") closes the topic *before* the
/// marker in head-final languages. Reading the slot off each word form lets one
/// concept serve both head-initial and head-final word orders, so the recogniser
/// peels the query off whichever side the connective sits on.
pub const ROLE_WEB_SEARCH_TOPIC_MARKER: &str = "web_search_topic_marker";
/// Semantic role: an imperative search verb that leads straight into the query.
///
/// "search for X", "найди X", "खोजो X", "搜索X" — a [`crate::seed::Slot::Prefix`]
/// style lead where the query is whatever follows the imperative. Distinct from
/// [`ROLE_WEB_SEARCH_EXPLICIT_PREFIX`], which carries an explicit web/source
/// reference; these are the bare imperatives.
pub const ROLE_WEB_SEARCH_IMPERATIVE_LEAD: &str = "web_search_imperative_lead";
/// Semantic role: filler that precedes the real query and is stripped off it.
///
/// Politeness, articles, and "information about …" lead-ins ("please ", "the ",
/// "information about ", "информацию о ", "关于", …) that are not part of the
/// search topic and are trimmed from the front of an extracted query.
pub const ROLE_WEB_SEARCH_QUERY_LEADING_NOISE: &str = "web_search_query_leading_noise";
/// Semantic role: filler that follows the real query and is stripped off it.
///
/// Trailing source/qualifier phrases (" online", " on the internet",
/// " в интернете", " के बारे में", "的信息", …) trimmed from the end of an
/// extracted query so only the topic remains.
pub const ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE: &str = "web_search_query_trailing_noise";
/// Semantic role: a bare reference-source word that is not itself a query.
///
/// "web", "internet", "wikipedia", "интернет", "网上", … standing alone. When a
/// cleaned query reduces to just a source word it carries no topic, so the
/// recogniser rejects it.
pub const ROLE_WEB_SEARCH_SOURCE_ONLY: &str = "web_search_source_only";
/// Semantic role: a sign that an earlier conversation turn performed a web search.
///
/// "duckduckgo", "web search", "search the internet", "веб-поиск", "интернет",
/// "वेब खोज", "网络搜索", … matched as raw lowercased substrings against the text of
/// a *prior* turn (not the normalised current prompt). Lets a terse follow-up
/// ("search it") be read as referring back to a web search the assistant already
/// offered. Lexicalised in every supported language. Carried by `web_search_mention`.
pub const ROLE_WEB_SEARCH_HISTORY_SIGNAL: &str = "web_search_history_signal";
/// Semantic role: the predicate verb of a follow-up instruction clause.
///
/// "search X **and then compare** …", "search X**. summarize** …" — the verb
/// (" compare ", " summarize ", " explain ", " сравни ", "比较", …) that opens a
/// directive about what to do with the results. It is treated as a follow-up
/// boundary (and triggers query truncation) only when it is immediately preceded
/// by a boundary: sentence punctuation or a [`ROLE_CLAUSE_CONTINUATION_MARKER`].
/// A bare verb with no preceding boundary stays part of the topic.
pub const ROLE_FOLLOWUP_INSTRUCTION_VERB: &str = "followup_instruction_verb";
/// Semantic role: a conjunction/sequencer that can open a new clause.
///
/// "and", "then", "и", "затем", "并", "然后", … — together with sentence
/// punctuation these form the universal set of boundaries before which a
/// [`ROLE_FOLLOWUP_INSTRUCTION_VERB`] counts as a follow-up directive. Modelling
/// "and" and "then" separately lets the recogniser recognise the compound
/// "and then" by walking back over consecutive markers, so no compound surface
/// needs to be stored.
pub const ROLE_CLAUSE_CONTINUATION_MARKER: &str = "clause_continuation_marker";
/// Semantic role: an interrogative that opens an implicit research question.
///
/// "what is the …", "which …", "who …", "how …" and their translations. A
/// research question is recognised when an opener combines with a
/// [`ROLE_RESEARCH_SUPERLATIVE_MODIFIER`] or with both a
/// [`ROLE_RESEARCH_EVIDENCE_DOMAIN`] and a [`ROLE_RESEARCH_EVALUATION_DOMAIN`].
pub const ROLE_RESEARCH_QUESTION_OPENER: &str = "research_question_opener";
/// Semantic role: a superlative or recommendation modifier marking research.
///
/// "most", "best", "leading", "recommended", "state of the art", … — the
/// signal that a question seeks an externally-verifiable best/standard answer
/// rather than a local computation.
pub const ROLE_RESEARCH_SUPERLATIVE_MODIFIER: &str = "research_superlative_modifier";
/// Semantic role: a research-evidence noun.
///
/// "dataset", "benchmark", "corpus", "metric", "framework", "paper", "study", …
/// — the kind of artifact a research question asks the web to find.
pub const ROLE_RESEARCH_EVIDENCE_DOMAIN: &str = "research_evidence_domain";
/// Semantic role: an evaluation noun that pairs with evidence to mark research.
///
/// "evaluation", "validation", "quality", "translation", "comparison", … — the
/// assessment a research question is framed around.
pub const ROLE_RESEARCH_EVALUATION_DOMAIN: &str = "research_evaluation_domain";
/// Semantic role: an opener that asks to list every member of a set.
///
/// "list all …", "show all …", "перечисли всех …", "列出所有 …", … — the lead-in
/// of an enumeration research request, stripped to leave the set description.
pub const ROLE_ENUMERATION_REQUEST_OPENER: &str = "enumeration_request_opener";
/// Semantic role: a constraint connective that qualifies an enumeration.
///
/// "with", "that", "who", "having", "for", "с", "которые", "具有", … — the marker
/// that an enumeration request carries a filter (so it is a real research
/// request, not a bare noun phrase).
pub const ROLE_ENUMERATION_CONSTRAINT: &str = "enumeration_constraint";
/// Semantic role: a marker that names the language a translation reads *from*.
///
/// "from english", "с русского", "हिंदी से", "从中文", …. Each such meaning is
/// `defined_by` one `language_*` meaning and the source-direction relation, so a
/// handler reads the source language by walking the marker's `defined_by` edges —
/// never by matching a glued from-language phrase baked into the code.
pub const ROLE_TRANSLATION_SOURCE_MARKER: &str = "translation_source_marker";
/// Semantic role: a marker that names the language a translation renders *into*.
///
/// "to russian", "на английский", "अंग्रेजी में", "翻译成中文" → "成中文", …. Each
/// such meaning is `defined_by` one `language_*` meaning and the target-direction
/// relation; the handler resolves the target language the same way it resolves a
/// source: by following `defined_by` to the language meaning.
pub const ROLE_TRANSLATION_TARGET_MARKER: &str = "translation_target_marker";
/// Semantic role: the target-direction relation of a translation (the "into" side).
///
/// Its surfaces are the bare directional markers ("to", "на", "में", and the
/// Chinese resultatives 成/为/為/到). In Chinese these bare markers are scanned
/// directly: after a 翻译 verb the extractor stops the surface at the first of
/// them, so the boundary comes from this relation rather than a hardcoded list.
pub const ROLE_TRANSLATION_TARGET_DIRECTION: &str = "translation_target_direction";
/// Semantic role: the verb frame that brackets the surface to translate.
///
/// In head-initial English/Russian the form is a [`crate::seed::Slot::Circumfix`]
/// ("translate … to ", "переведи … на ") whose before-slot prefix is stripped and
/// after-slot marker bounds the surface; in head-final Hindi/Chinese the form is a
/// [`crate::seed::Slot::Bare`] verb stem (अनुवाद, 翻译/翻譯) that gates the
/// language-specific unquoted extractor. The extractor reads the slot to decide
/// which strategy applies, so one role serves both word orders.
pub const ROLE_TRANSLATION_UNQUOTED_FRAME: &str = "translation_unquoted_frame";
/// Semantic role: the verb-and-target compound introducing the target right after
/// the surface ("translate-into").
///
/// Head-final Hindi postposes the target onto the verb noun (" में अनुवाद"), so
/// the extractor keeps the text before it; Chinese prefixes the direction onto the
/// verb (翻译成, 翻译为, 翻译到, …), so the extractor stops the surface at the first
/// such compound. The English/Russian compounds are recorded for completeness and
/// are not separately scanned — those languages run through the circumfix frame.
pub const ROLE_TRANSLATION_INTO_MARKER: &str = "translation_into_marker";
/// Semantic role: the particle that flags the noun phrase to be translated.
///
/// Head-final Hindi postposes the marker after the object (का, को), used as a
/// right boundary; Chinese fronts a disposal particle before the object (把, 将),
/// stripped from the front. English/Russian mark the object positionally, so their
/// nearest realisations are recorded for completeness and not scanned — only the
/// Devanagari and Han forms are.
pub const ROLE_TRANSLATION_OBJECT_MARKER: &str = "translation_object_marker";
/// Semantic role: the translation/description command verb — the action a request
/// realises ("translate", "переведи"/"перевести"/"опиши", "अनुवाद", "翻译"/"翻譯").
///
/// Three handlers read this role instead of hardcoding the verbs. The
/// request-gate (`try_translation`) recognises a command by a *clause-initial*
/// English/Russian stem (`starts_with`) or, in head-final Hindi/Chinese where the
/// verb is not clause-initial, by the stem appearing anywhere together with a
/// target marker. The source-inferencer (`infer_source_from_prompt`) reads which
/// language's stem appears as the language the user issued the command in. The
/// formalization object-parser anchors its surface extraction on the same stems.
/// The per-language stems live once in `data/seed/meanings-translation.lino`; the
/// head-initial/head-final split is the linguistic typology the `translate`
/// meaning's gloss documents.
pub const ROLE_TRANSLATION_ACTION: &str = "translation_action";
/// Semantic role: the single root of the merged ontology — the `link` meaning.
///
/// Every other meaning descends from it through `defined_by` edges, so the whole
/// lexicon is one connected graph rooted at `link` (the relative-meta-logic
/// "everything is a link" stance). Exactly one meaning carries this role.
pub const ROLE_ONTOLOGY_ROOT: &str = "ontology_root";
/// Semantic role: the root of the type-system sub-ontology — the `type` meaning.
///
/// A distinguished node directly under `link`; the broadest classifications
/// (`entity`, `concept`) are `defined_by` it, giving a merged multi-root
/// ontology whose roots all reduce to `link`.
pub const ROLE_ONTOLOGY_TYPE: &str = "ontology_type";
/// Semantic role: a top-level ontological category each domain genus roots in.
///
/// `entity`, `concept`, `relation`, `action`, `property` — the bridge meanings
/// that connect every domain cluster (programs, calendars, facts, software, …)
/// up to the `link` root.
pub const ROLE_ONTOLOGY_CATEGORY: &str = "ontology_category";
/// Semantic role: the rule noun a behavior-rules-list request enumerates
/// ("rules"/"rule list", "правил"/"правила", "नियम"/"नियमों", "规则"/"規則").
///
/// One of three compositional dimensions the behavior-rules-list recogniser ANDs
/// together within a single language; carried by the `behavior_rule` meaning.
pub const ROLE_RULE_LISTING_SUBJECT: &str = "rule_listing_subject";
/// Semantic role: the enumerate request that asks the assistant to reveal a
/// set's members — the list/show imperative or the which/what interrogative.
///
/// Surface cues "list"/"show"/"what", "покажи"/"какие", "दिखाओ"/"कौन",
/// "列出"/"哪些"; the second compositional dimension, carried by
/// `rule_enumeration_request`.
pub const ROLE_RULE_LISTING_REQUEST: &str = "rule_listing_request";
/// Semantic role: the cue scoping a rules-listing request to the assistant's
/// own behavior.
///
/// The behaviour domain word, the second-person/own possessive, the existence
/// deixis, and the bare rule-list compound. The third compositional dimension,
/// carried by two meanings, `behavior_domain` and `assistant_own_ruleset`, whose
/// union is the original scope vocabulary.
pub const ROLE_RULE_LISTING_SCOPE: &str = "rule_listing_scope";
/// Semantic role: a fixed phrase that names the behavior-rule set outright and is
/// a standing list request without a separate verb ("existing behavior rules",
/// "行为规则", "व्यवहार के नियम").
///
/// Matched as a raw substring, independent of the compositional dimensions;
/// carried by `behavior_rule_set_phrase`.
pub const ROLE_RULE_LISTING_PHRASE: &str = "rule_listing_phrase";
/// Semantic role: a bare imperative verb that, clause-initially, requests a proof.
///
/// "prove", "proof", "докажи", "доказать", … — detected at the very start of the
/// prompt with a verb boundary (so "prover"/"proven" never match). Carried by the
/// `prove` meaning; queried as bare literals. Hindi and Chinese carry no bare
/// directive (their proof is caught by [`ROLE_PROOF_MARKER`]).
pub const ROLE_PROOF_DIRECTIVE: &str = "proof_directive";
/// Semantic role: a broad request frame asking for a proof, in any language.
///
/// "can you prove", "please prove", "give me a proof", "show that ", "demonstrate
/// that ", and their Russian/Hindi/Chinese counterparts — detected with a plain
/// prefix match (no verb boundary, no claim extraction), so a proof request is
/// recognised even without a following "that". The non-English leads each embed a
/// [`ROLE_PROOF_MARKER`] surface (so they also match mid-prompt); the English
/// markers cover only "prove that"/"proof of", so the English leads are the sole
/// surface for a polite English request. Carried by `proof_request_frame`; queried
/// as prefix literals.
pub const ROLE_PROOF_REQUEST_LEAD: &str = "proof_request_lead";
/// Semantic role: a proof verb or noun appearing anywhere inside the prompt.
///
/// " prove that ", " proof of ", " докажи ", "साबित कर", "证明", … — matched as
/// raw substrings (English and Russian space-wrapped for a word boundary;
/// Devanagari and Han bare). Carried by `proof_assertion`; queried as a raw
/// substring role.
pub const ROLE_PROOF_MARKER: &str = "proof_marker";
/// Semantic role: a prefix whose lead-in is stripped to extract the proof claim.
///
/// "prove that …", "докажи что …", "साबित करो कि …", "证明…", … — ordered most-
/// specific first so the extractor takes the first match and keeps "that"/"что"
/// out of the claim. Carried by the `prove` meaning; queried as prefix literals.
pub const ROLE_PROOF_CLAIM_SCAFFOLD: &str = "proof_claim_scaffold";
/// Semantic role: the surname Gödel naming the incompleteness interpretation.
///
/// "godel", "gödel", "гёдел", "哥德尔", "गोडेल", … matched as raw substrings to
/// steer the proof engine toward incompleteness. Carried by `godel`; read by the
/// Rust solver only.
pub const ROLE_PROOF_CONCEPT_GODEL: &str = "proof_concept_godel";
/// Semantic role: the concept of determinism naming that proof interpretation.
///
/// "determinism", "deterministic", "детерминизм", "决定论", "निर्धारणवाद", …
/// matched as raw substrings to steer the proof engine toward determinism.
/// Carried by `determinism`; read by the Rust solver only.
pub const ROLE_PROOF_CONCEPT_DETERMINISM: &str = "proof_concept_determinism";
/// Semantic role: a fronted interrogative opening a who-is question.
///
/// "who is ", "who was ", "кто такой ", "кто ", … — head-initial languages put
/// the interrogative first, detected with a prefix match. Carried by
/// `who_is_question`; queried as prefix literals.
pub const ROLE_WHO_QUESTION_LEAD: &str = "who_question_lead";
/// Semantic role: a postposed interrogative closing a who-is question.
///
/// " कौन है", " कौन हैं", "是谁", "是誰", … — head-final languages put the
/// interrogative last, detected with a suffix match. Carried by
/// `who_is_question`; queried as suffix literals.
pub const ROLE_WHO_QUESTION_TAIL: &str = "who_question_tail";
/// Semantic role: a crude taunt asking whether the assistant performed a bodily
/// action it cannot perform.
///
/// Russian inflections of сосать, the English interrogative, and the Hindi and
/// Chinese equivalents are matched as raw substrings. Content-policy screening
/// refuses any surface that is also vulgar before this role is read; the rest
/// receive a factual no-physical-body reply. Carried by `physical_action_query`;
/// read by the Rust solver and the JS worker.
pub const ROLE_PHYSICAL_ACTION_TRIGGER: &str = "physical_action_trigger";
/// Semantic role: the opening line of the Russian circular-joke idiom.
///
/// The calque buy an elephant in every supported language is matched as a raw
/// substring so the assistant recognises the idiom instead of returning an
/// unknown prompt. Carried by `circular_joke_idiom`; read by the Rust solver and
/// the JS worker.
pub const ROLE_CIRCULAR_JOKE_PHRASE: &str = "circular_joke_phrase";
/// Semantic role: a profanity or slur that flags a message as vulgar content.
///
/// The English and Russian forms are the original hardcoded refusal lists,
/// migrated verbatim; Hindi and Chinese carry equivalent obscenities so the
/// concept is lexicalized in every supported language. All forms are matched as
/// raw substrings, so the screen is language-independent and tolerant of
/// inflection. Carried by `vulgar_content`; read by the Rust solver only (the JS
/// worker has no content-policy handler, so the data is mirrored but unused
/// there).
pub const ROLE_VULGAR_CONTENT_MARKER: &str = "vulgar_content_marker";
/// Semantic role: a surface form that signals a prompt is talking about the
/// exchange rate between two currencies.
///
/// "exchange rate", "currency rate", "курс", "विनिमय दर", "汇率" — matched as
/// raw substrings so inflected and compound forms are caught. Carried by
/// `exchange_rate`; the calculator rate-basis handler requires it together with
/// [`ROLE_CURRENCY_USD_REFERENCE`] and [`ROLE_CALCULATION_BASIS_REFERENCE`].
/// Read by the Rust solver and the JS worker.
pub const ROLE_EXCHANGE_RATE_REFERENCE: &str = "exchange_rate_reference";
/// Semantic role: a surface form that signals a prompt mentions US dollars.
///
/// "usd", "dollar", "доллар" (and the misspellings "долар"/"долор"), "डॉलर",
/// "美元" — matched as raw substrings. Carried by `us_dollar`; the calculator
/// rate-basis handler requires it together with [`ROLE_EXCHANGE_RATE_REFERENCE`]
/// and [`ROLE_CALCULATION_BASIS_REFERENCE`]. Read by the Rust solver and the JS
/// worker.
pub const ROLE_CURRENCY_USD_REFERENCE: &str = "currency_usd_reference";
/// Semantic role: a phrase asking which value, rate, or method the assistant
/// uses or applies as the basis when it calculates.
///
/// Inflectable stems ("при расчёт", "использ", "примен", "calculation", …) and
/// fixed phrases ("do you use", "у тебя", "गणना", "计算", …) matched as raw
/// substrings. Carried by `calculation_basis`; the calculator rate-basis handler
/// requires it together with [`ROLE_EXCHANGE_RATE_REFERENCE`] and
/// [`ROLE_CURRENCY_USD_REFERENCE`]. Read by the Rust solver and the JS worker.
pub const ROLE_CALCULATION_BASIS_REFERENCE: &str = "calculation_basis_reference";
/// Semantic role: the interrogative word that asks for a cause or reason.
///
/// "why", "почему", "क्यों", "为什么" — the bare cause-asking word, with no
/// answer reference of its own. Carried by `causal_interrogative`; the
/// meta-explanation why-recogniser reads only the Hindi and Chinese surfaces,
/// pairing each with [`ROLE_PRIOR_ANSWER_REFERENCE`] in the same language to
/// detect a head-final why-question (the English and Russian why-questions front
/// the interrogative and are matched through [`ROLE_ANSWER_RATIONALE_LEAD`]).
/// Read by the Rust solver only (the JS worker has no meta-explanation handler).
pub const ROLE_CAUSAL_INTERROGATIVE: &str = "causal_interrogative";
/// Semantic role: a reference to the answer the assistant previously gave.
///
/// "answer", "ответ", "जवाब"/"उत्तर", "回答" — the object a why-question points
/// back at. Carried by `prior_answer_reference`; the meta-explanation
/// why-recogniser reads only the Hindi and Chinese surfaces, pairing each with
/// [`ROLE_CAUSAL_INTERROGATIVE`] in the same language. A dedicated reference (not
/// the broader `answer` meaning) so its Chinese surface stays exactly 回答, as the
/// original recogniser required. Read by the Rust solver only.
pub const ROLE_PRIOR_ANSWER_REFERENCE: &str = "prior_answer_reference";
/// Semantic role: the leading surface of a why-did-you-answer question.
///
/// The English and Russian why-questions front the interrogative, so they are
/// matched directly: a prefix surface ("why …", "почему …") fires when the
/// prompt opens with the literal, and a bare surface ("why did you answer",
/// "почему ты ответил", …) matches anywhere. Carried by
/// `answer_rationale_inquiry`; the meta-explanation why-recogniser iterates only
/// the English and Russian forms (the Hindi and Chinese forms are inert
/// completeness surfaces, handled instead by the per-language pairing of
/// [`ROLE_CAUSAL_INTERROGATIVE`] and [`ROLE_PRIOR_ANSWER_REFERENCE`]). Read by the
/// Rust solver only.
pub const ROLE_ANSWER_RATIONALE_LEAD: &str = "answer_rationale_lead";
/// Semantic role: a second-person reference to the assistant itself.
///
/// "you", "your", "formal ai", "ты", "вы", "आप", "तुम", "你", "您" and the
/// Russian stems "теб"/"тво" — matched as raw substrings, marking that a prompt
/// is addressed to the assistant. Carried by `assistant_self_reference`; the
/// architecture recogniser requires it together with
/// [`ROLE_ARCHITECTURE_CONCEPT`], and the how-you-work recogniser requires its
/// Russian forms together with [`ROLE_OPERATING_PRINCIPLE`]. Read by the Rust
/// solver and the JS worker.
pub const ROLE_ASSISTANT_SELF_REFERENCE: &str = "assistant_self_reference";
/// Semantic role: a complete how-do-you-work clause addressed to the assistant.
///
/// "how do you work", "как ты работаешь", "तुम कैसे काम करते हो",
/// "你是怎么工作的" and their variants — each a full clause matched as a raw
/// substring; the how-you-work recogniser fires when any one appears. Carried by
/// `assistant_mechanism_inquiry`; the Russian principle-of-operation phrasing is
/// handled separately by composing [`ROLE_OPERATING_PRINCIPLE`] with
/// [`ROLE_ASSISTANT_SELF_REFERENCE`]. Read by the Rust solver only.
pub const ROLE_ASSISTANT_MECHANISM_INQUIRY: &str = "assistant_mechanism_inquiry";
/// Semantic role: the concept of a thing's operating principle.
///
/// "operating principle", "принцип работы", "कार्य सिद्धांत", "工作原理" — the
/// how-you-work recogniser reads only the Russian surface, composing it with
/// [`ROLE_ASSISTANT_SELF_REFERENCE`] to catch "принцип работы … тебя". Carried by
/// `operating_principle`; the other languages are inert completeness forms. Read
/// by the Rust solver only.
pub const ROLE_OPERATING_PRINCIPLE: &str = "operating_principle";
/// Semantic role: a term naming part of an AI system's architecture or internals.
///
/// "language model", "neural network", "openai api", "world model", "links
/// notation rules", "бям", "нейросет", "ссылк", "神经", "语言模型" and the like —
/// matched as raw substrings (several Russian forms are inflectable stems).
/// Carried by `architecture_concept`; the architecture recogniser fires when one
/// appears together with [`ROLE_ASSISTANT_SELF_REFERENCE`], marking a question
/// about how the assistant is built rather than a task request. Read by the Rust
/// solver and the JS worker.
pub const ROLE_ARCHITECTURE_CONCEPT: &str = "architecture_concept";
/// Semantic role: the lead-in of a prompt asking for something to be explained.
///
/// Every interrogative or imperative that opens an explanation request lives here
/// rather than in the documentation handler. Each surface marks the subject
/// position with the ellipsis … (U+2026): a [`crate::seed::Slot::Prefix`] form
/// ("how …", "explain …", "как …", "क्या है …", "解释…") is matched by the literal
/// before the slot against the start of the prompt, while a bare form with no
/// ellipsis ("how", "कैसे काम", "如何工作", …) is matched as a raw substring
/// anywhere. A space-wrapped bare form (" how ", " как ") matches only on
/// whole-word boundaries. Carried by `explanation_request`; read by the Rust
/// solver and the JS worker so neither names an interrogative word in code.
pub const ROLE_EXPLANATION_REQUEST_LEAD: &str = "explanation_request_lead";
/// Semantic role: a noun naming the internet as the medium to search.
///
/// The same internet-naming surfaces that fill [`ROLE_WEB_SEARCH_SIGNAL`] and
/// [`ROLE_WEB_SEARCH_SOURCE_ONLY`] (" web ", " internet ", " online ",
/// " интернете ", "इंटरनेट", "网上", …), shared here so the documentation handler
/// can confirm that a prompt paired with an imperative search verb explicitly
/// asks to search the web — and screen such a prompt out of its method-question
/// gate. The English/Russian surfaces are space-wrapped, so they are matched
/// through the [`crate::seed::Lexicon::mentions_role_raw`] sibling convention used
/// by the web-search recogniser: a `format!(" {normalized} ")` pad plus
/// `contains`, giving a whole-token match that also catches a medium word at the
/// very end of the prompt ("search the web"). Carried by `reference_internet`;
/// read by the Rust solver and the JS worker.
pub const ROLE_WEB_MEDIUM: &str = "web_medium";
/// Semantic role: the noun "method" in the programming sense, in any language.
///
/// "method", "метод", "विधि", "方法" — the word a prompt uses to refer to a named
/// operation defined on a type or object (such as the join method of a
/// `DataFrame`). The documentation handler pairs this concept with the method's
/// own API identifier — which is written the same in every language — to
/// recognise a question about a specific method without naming the word "method"
/// in code. The space-delimited surfaces are matched on whole-token boundaries
/// through [`crate::seed::Lexicon::mentions_role`] (`surface_present`), while the
/// Han surface matches as a substring. Carried by `code_method`; read by the Rust
/// solver and the JS worker.
pub const ROLE_CODE_METHOD_NOUN: &str = "code_method_noun";
