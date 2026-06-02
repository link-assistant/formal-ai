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
/// Semantic role: a connective that introduces the search topic *after* it.
///
/// "search X **about** Y" — the topic follows the marker (" about ", " on ",
/// " о ", " про ", "关于", …). Used to peel the query off the tail of a prompt.
pub const ROLE_WEB_SEARCH_TOPIC_AFTER: &str = "web_search_topic_after";
/// Semantic role: a postposition that closes the search topic *before* it.
///
/// The mirror of [`ROLE_WEB_SEARCH_TOPIC_AFTER`] for head-final languages: the
/// topic precedes the marker (Hindi " के बारे में", " की जानकारी", …). Used to
/// peel the query off the head of a prompt.
pub const ROLE_WEB_SEARCH_TOPIC_BEFORE: &str = "web_search_topic_before";
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
/// Semantic role: a trailing instruction clause that is not part of the query.
///
/// "search X **and then compare** …", "search X**. summarize** …" — a follow-up
/// directive appended after the topic. The recogniser truncates the query at
/// the earliest such marker so the instruction does not pollute the search
/// terms.
pub const ROLE_WEB_SEARCH_FOLLOWUP_INSTRUCTION: &str = "web_search_followup_instruction";
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
