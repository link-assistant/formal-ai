//! Role constants for conversational intent: clarification, capability and
//! self/knowledge inventory, conversation summary, how/mechanism procedure,
//! common typos, and the web-search, research and enumeration cluster
//! (issue #386).
//!
//! Re-exported flat through [`super`] so every constant stays reachable as
//! `crate::seed::roles::ROLE_*` and `crate::seed::ROLE_*` (issue #386).

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
/// Semantic role: a natural-language query that searches prior dialog turns.
///
/// The phrase asks for mentions of a subject in the conversation history
/// ("when did I mention X", "search conversations for X", "когда я спрашивал
/// про X", …). Every surface marks the search term with the ellipsis slot so
/// the handler can extract the term from seed data instead of hardcoding cue
/// phrases in Rust.
pub const ROLE_CONVERSATION_RECALL_QUERY: &str = "conversation_recall_query";
/// Semantic role: a recall query scoped to other conversations or chats.
///
/// This role mirrors the browser memory-search affordance for prompts such as
/// "find X in another conversation" or "найди X в других беседах". Surfaces
/// still expose the searched term with the ellipsis slot; runtimes that do not
/// receive conversation identifiers record the requested scope as metadata and
/// search the provided prior turns.
pub const ROLE_CONVERSATION_RECALL_OTHER_QUERY: &str = "conversation_recall_other_query";
/// Semantic role: a query asking for the content of the immediately preceding
/// message.
///
/// This role recognizes prompts such as "what was written in the previous
/// message", "что было написано в прошлом сообщении", "पिछले संदेश में क्या लिखा
/// था", or "上一条消息写了什么". Unlike [`ROLE_CONVERSATION_RECALL_QUERY`], it carries
/// no search term: the handler simply replays the last prior turn (any role)
/// that immediately precedes the current prompt. Surfaces are bare phrases
/// matched anywhere in the prompt via [`crate::seed::Lexicon::mentions_role`].
pub const ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE: &str = "conversation_recall_previous_message";
/// Semantic role: a natural-language directive that appends a statement to the
/// associative memory.
///
/// Recognizes the leading verb phrase of an append request ("remember …",
/// "запомни …", "याद रखो …", "记住…") as a [`crate::seed::Slot::Prefix`] whose
/// ellipsis slot carries the statement to store. This is the *write* half of the
/// natural-language memory primitive: a recall reads memory, an append extends
/// it. Surfaces live in `data/seed/meanings-conversation.lino`.
pub const ROLE_MEMORY_APPEND_DIRECTIVE: &str = "memory_append_directive";
/// Semantic role: a phrase that scopes an operation to the associative memory
/// ("in memory", "в памяти", "स्मृति में", "在记忆中").
///
/// A substitution request must name this scope so a bare "replace X with Y"
/// (which is a coding request) is never mistaken for a memory rewrite. Surfaces
/// are bare phrases matched anywhere via [`crate::seed::Lexicon::mentions_role`].
pub const ROLE_MEMORY_SCOPE: &str = "memory_scope";
/// Semantic role: the connector word that separates the old value from the new
/// value in a memory substitution ("with"/"by", "на", "की जगह", "换成").
///
/// The substitution parser splits the operand span on this connector to recover
/// `(old, new)`. Surfaces live in `data/seed/meanings-conversation.lino`.
pub const ROLE_MEMORY_SUBSTITUTION_CONNECTOR: &str = "memory_substitution_connector";
/// Semantic role: the verb that marks a memory substitution ("replace",
/// "замени", "बदलो"/"रखो", "把"/"替换").
///
/// Stripped (at either edge — SVO languages lead with it, Hindi trails) before
/// the operand span is split on the connector. Surfaces live in
/// `data/seed/meanings-conversation.lino`.
pub const ROLE_MEMORY_SUBSTITUTION_DIRECTIVE: &str = "memory_substitution_directive";
/// Semantic role: a conversational opener that proposes a topic to discuss.
///
/// The let-us-talk-about-X phrasing that introduces a subject for open
/// conversation ("let's talk about …", "давай поговорим о …", "चलो बात करें …",
/// "聊聊…", …). Every surface is a [`crate::seed::Slot::Prefix`] carrying the
/// topic after the ellipsis (U+2026) slot marker; a surface whose `action`
/// child is `scan` is also matched anywhere in the prompt, not only at the
/// start, so an opener that follows a greeting is still recognized. A meaning
/// carrying this role is `defined_by` the `inquiry` and `action` concepts.
pub const ROLE_CONVERSATION_TOPIC_OPENER: &str = "conversation_topic_opener";
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
/// Semantic role: a bare procedural "how …" lead-in that omits the expected
/// connector ("to", "do I", …).
///
/// The extractor only accepts this weak prefix when the first word after the
/// slot is listed under [`ROLE_PROCEDURAL_ACTION_VERB`], so telegraphic prompts
/// such as "how order X" recover as procedures while arbitrary "how <word>"
/// questions keep flowing to their more specific handlers or to unknown.
pub const ROLE_PROCEDURAL_REQUEST_ELIDED_LEAD: &str = "procedural_request_elided_lead";
/// Semantic role: a procedural action verb that may follow an elided
/// [`ROLE_PROCEDURAL_REQUEST_ELIDED_LEAD`] prefix.
///
/// These are bare action surfaces ("order", …) checked as exact lexeme entries,
/// not a broad verb detector, so adding a new accepted telegraphic action is a
/// seed change with reviewable blast radius.
pub const ROLE_PROCEDURAL_ACTION_VERB: &str = "procedural_action_verb";
/// Semantic role: the predicate that completes a how-it-works clause.
///
/// The verb or participle stating that a subject operates, is structured, or is
/// built ("work", "works", "structured", "built", …). Every surface is a
/// [`crate::seed::Slot::Suffix`]; the text after the `…` slot is the predicate
/// tail a mechanism-inquiry extractor strips so the bare subject remains. A
/// meaning carrying this role is `defined_by` the `action` and
/// `mechanism_inquiry` concepts.
pub const ROLE_MECHANISM_PREDICATE: &str = "mechanism_predicate";
/// Semantic role: an optional thoroughness or politeness modifier on a
/// how-it-works question ("in detail", "internally", "please", …).
///
/// Every surface is a [`crate::seed::Slot::Suffix`]; the text after the `…` slot
/// is the modifier tail a mechanism-inquiry extractor strips, in declaration
/// order, so the bare subject remains. A meaning carrying this role is
/// `defined_by` the `property` and `mechanism_inquiry` concepts.
pub const ROLE_DETAIL_MODIFIER: &str = "detail_modifier";
/// Semantic role: a subject candidate that names no real topic.
///
/// A pronoun or bare function word that points back at the surrounding context
/// instead of introducing a subject ("it", "this", "does …", "to …", …), so a
/// how-it-works extractor rejects it and falls back to the active topic.
/// [`crate::seed::Slot::Bare`] surfaces match the whole candidate exactly;
/// [`crate::seed::Slot::Prefix`] surfaces match when the candidate begins with
/// the literal before the `…` slot. A meaning carrying this role is `defined_by`
/// the `entity` and `mechanism_inquiry` concepts.
pub const ROLE_NON_REFERENTIAL_SUBJECT: &str = "non_referential_subject";
/// Semantic role: an optional step-by-step or politeness modifier trailing a
/// procedural "how to X" task ("step by step", "in steps", "please", …).
///
/// Every surface is a [`crate::seed::Slot::Suffix`]; the text after the `…` slot
/// is the modifier tail a procedural extractor strips from the end of the task,
/// in declaration order with the first match winning, so a longer modifier such
/// as the Russian "напиши по шагам" is tried before its "по шагам" tail. A
/// meaning carrying this role is `defined_by` the `property` and
/// `procedural_request` concepts.
pub const ROLE_PROCEDURAL_TASK_MODIFIER: &str = "procedural_task_modifier";
/// Semantic role: a follow-up that asks for the concrete steps of an active
/// procedure ("give me specific instructions", "the exact steps", "step by
/// step", "дай конкретные инструкции", "具体步骤", …).
///
/// Issue #444: after a "how to X" turn the assistant answered with a discovery
/// plan, and the user then asked "Can you give me specific instructions?" — a
/// prompt that carries no "how to" lead-in of its own. A meaning carrying this
/// role lets the solver recognise the elaboration request and rebind it to the
/// procedure recovered from the prior turn instead of falling to the unknown
/// opener. Surfaces are matched as raw substrings of the normalized prompt
/// because most are multi-word phrases. A meaning carrying this role is
/// `defined_by` the `inquiry` and `procedural_request` concepts.
pub const ROLE_PROCEDURAL_ELABORATION: &str = "procedural_elaboration";
/// Semantic role: a common misspelling paired with its correction.
///
/// A [`crate::seed::Slot::Bare`] surface whose `text` is the misspelled token and
/// whose `action` child names the correct spelling, so a procedural extractor can
/// repair a task token by data rather than a hardcoded typo table (the canonical
/// example is the transposed "dirven" -> "driven"). A meaning carrying this role
/// is `defined_by` the `relation` concept.
pub const ROLE_COMMON_TYPO: &str = "common_typo";
/// Semantic role: a closed-class function word or citation heading that names
/// no subject.
///
/// An article, preposition, conjunction or pronoun, or a citation heading such
/// as "source", that a scanner looking for the topic of a prior assistant reply
/// skips. Every surface is a [`crate::seed::Slot::Bare`] word compared
/// case-insensitively (after lowercasing) against a capitalised token from the
/// reply; the first capitalised token that is not one of these is taken as the
/// topic. A meaning carrying this role is `defined_by` the `concept` category.
pub const ROLE_TOPIC_SCAN_STOP_WORD: &str = "topic_scan_stop_word";
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
/// Semantic role: an action that requests an originality/plagiarism check.
///
/// "check", "verify", "проверь", "जांचें", "检查", ... name the operation.
/// The document-originality handler combines this role with a plagiarism or
/// originality subject role and a text/document/attachment role, so generic
/// "check this" prompts are not routed as web-grounded originality checks.
pub const ROLE_DOCUMENT_ORIGINALITY_CHECK_ACTION: &str = "document_originality_check_action";
/// Semantic role: a subject signal for originality/plagiarism checking.
///
/// Surfaces are intentionally read with the raw-substring sibling of the
/// lexicon matcher because supported languages inflect these stems heavily
/// ("уникальность", "уникальный", "plagiarism", "plagiarized", ...).
pub const ROLE_DOCUMENT_ORIGINALITY_SUBJECT: &str = "document_originality_subject";
/// Semantic role: the document/text/attachment being checked for originality.
///
/// This role marks the task as operating on supplied text, an attached file, or
/// a document-like artifact. Attachment metadata can also satisfy this gate
/// when the prompt was composed by a client surface.
pub const ROLE_DOCUMENT_ORIGINALITY_DOCUMENT: &str = "document_originality_document";
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
/// Semantic role: a news/headlines subject marker for web search.
///
/// "news", "headlines", "новост", "新闻", … name news as the information
/// subject. On its own this is just content; paired with
/// [`ROLE_WEB_SEARCH_NEWS_RECENCY`] it routes bare latest-news prompts to web
/// search without requiring an imperative verb.
pub const ROLE_WEB_SEARCH_NEWS_SUBJECT: &str = "web_search_news_subject";
/// Semantic role: a freshness marker that turns a news subject into a search.
///
/// "latest", "breaking", "последн", "свеж", "最新", … indicate that current
/// results are needed. The web-search recogniser requires this together with
/// [`ROLE_WEB_SEARCH_NEWS_SUBJECT`] for bare news prompts.
pub const ROLE_WEB_SEARCH_NEWS_RECENCY: &str = "web_search_news_recency";
/// Semantic role: a records/documents subject marker for web search.
///
/// "records", "filings", "statements", "statistics", "записи", "отчёт",
/// "रिकॉर्ड", "记录", … name retrievable records, filings, or figures as the
/// information subject. On its own this is just content; paired with a
/// [`ROLE_WEB_SEARCH_TOPIC_MARKER`] connective ("records *for* boeing", "записи
/// *о* …") it routes a verbless "records about a subject" request to web search
/// without requiring an imperative search verb.
pub const ROLE_WEB_SEARCH_RECORDS_SUBJECT: &str = "web_search_records_subject";
/// Semantic role: a public-event subject marker for web search.
///
/// "event", "hackathon", "conference", "хакатон", "黑客松", … name event
/// categories whose active/current instances are normally external and
/// time-sensitive. Paired with [`ROLE_RESEARCH_QUESTION_OPENER`] and
/// [`ROLE_WEB_SEARCH_NEWS_RECENCY`], this routes questions such as "Какие
/// хакатоны сейчас проходят?" to web search without requiring an imperative
/// search verb.
pub const ROLE_WEB_SEARCH_PUBLIC_EVENT_SUBJECT: &str = "web_search_public_event_subject";
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
/// Semantic role: a "tell me about" opener whose object is a public term.
///
/// "tell me about …", "расскажи мне об …", "बताओ …", "告诉我…" and their
/// translations. The web-search recogniser only uses this role after seeded
/// concept lookup cannot answer, so known local concepts keep their direct
/// explanation path while unknown public terms fall through to external search.
pub const ROLE_TERM_INFORMATION_REQUEST_OPENER: &str = "term_information_request_opener";
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
