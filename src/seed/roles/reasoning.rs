//! Role constants for reasoning and dialogue cues: interrogatives, jokes and
//! vulgar-content guards, currency/exchange-rate and calculation cues,
//! politeness, causal and answer-rationale leads, assistant self-description,
//! architecture and explanation, web medium, code, skills, behavior-rule
//! edits, finance, unit conversion and math-function names (issue #386).
//!
//! Re-exported flat through [`super`] so every constant stays reachable as
//! `crate::seed::roles::ROLE_*` and `crate::seed::ROLE_*` (issue #386).

/// Semantic role: a fronted question word that opens a content question.
///
/// "what ", "who ", "why ", "что ", "кто ", "как ", … — the wh-words. English
/// and Russian are head-initial, so the opener starts the prompt and is detected
/// with a prefix match (a trailing space follows the bare word). Hindi and
/// Chinese are head-final and carried for coverage but not matched positionally.
/// Carried by `interrogative_opener`; queried as prefix literals by the intent
/// classifier to tell a question from a statement.
pub const ROLE_INTERROGATIVE_OPENER: &str = "interrogative_opener";
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
/// Semantic role: a natural-language cue that requests an arithmetic calculation.
///
/// Imperatives ("calculate", "посчитай", "गणना करें") and question openers
/// ("what is", "сколько будет", "请计算") carried by `calculation_request`.
/// `strip_calculation_wrappers` rebuilds each surface into a strip prefix —
/// space-delimited scripts gain a trailing space so the cue strips only on a
/// word boundary, CJK surfaces strip as-is — and removes it from the front of a
/// prompt. Read by the Rust solver and the JS worker.
pub const ROLE_CALCULATION_REQUEST_CUE: &str = "calculation_request_cue";
/// Semantic role: a trailing cue that asks for the computed result of the
/// preceding arithmetic expression.
///
/// An equals/how-much-is-it query ("equals", "=", "是多少", "कितना है", …) or a
/// head-final "do the calculation" imperative ("की गणना करें") that trails the
/// expression. `strip_calculation_wrappers` rebuilds each surface into a strip
/// suffix — space-delimited scripts gain a leading space so the cue strips only
/// on a word boundary, CJK surfaces strip as-is, and the bare equals sign also
/// matches with no leading space — and removes it from the end of a prompt so the
/// bare expression remains. Carried by `calculation_result_query`; read by the
/// Rust solver and the JS worker.
pub const ROLE_CALCULATION_RESULT_QUERY_CUE: &str = "calculation_result_query_cue";
/// Semantic role: a natural-language cue asking for elapsed time between two
/// clock times.
///
/// Phrases such as "how long", "elapsed time", "сколько времени", "कितना समय",
/// and "多久" carry the cue. The calculator router composes this role with two
/// valid `HH:MM` clock-time mentions and delegates the generated time difference
/// to `link-calculator`; the cue alone never marks a prompt as arithmetic. Read
/// by the Rust solver and the JS worker.
pub const ROLE_TIME_DURATION_CUE: &str = "time_duration_cue";
/// Semantic role: a politeness or courtesy marker that softens a request.
///
/// A please/for-me style tail ("please", "for me", "пожалуйста", "कृपया", "请")
/// that carries no task content. `strip_calculation_wrappers` removes it from the
/// end of a calculation prompt so the bare expression remains. Carried by
/// `politeness`; read by the Rust solver and the JS worker.
pub const ROLE_POLITENESS_CUE: &str = "politeness_cue";
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
/// The same internet-naming surfaces that fill `ROLE_WEB_SEARCH_SIGNAL` and
/// `ROLE_WEB_SEARCH_SOURCE_ONLY` (" web ", " internet ", " online ",
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
/// Semantic role: the opening clause of a taught skill that names its trigger.
///
/// The clause-initial lead a natural-language skill uses to introduce the phrase
/// that should fire a taught behaviour ("when i say …", "when the user asks …",
/// "когда я скажу …", "当用户说 …"). Every lead is a bare surface matched as a raw
/// substring through [`crate::seed::Lexicon::mentions_role_raw`] after the
/// description is lower-cased; a lead only teaches a skill when it co-occurs with a
/// [`ROLE_SKILL_TEACHING_RESPONSE_VERB`] and the prose also quotes a trigger and a
/// reply in backticks. Carried by `skill_teaching_trigger`; read by the Rust skill
/// compiler and the JS worker so neither names a trigger word in code.
pub const ROLE_SKILL_TEACHING_TRIGGER_LEAD: &str = "skill_teaching_trigger_lead";
/// Semantic role: the verb introducing the reply a taught skill should emit.
///
/// The verb a natural-language skill uses to name its response side ("answer",
/// "reply", "respond", the Russian stem "ответ", "回答"). Matched as a raw substring
/// through [`crate::seed::Lexicon::mentions_role_raw`], so an inflectable stem folds
/// its endings; a response verb only teaches a skill when paired with a
/// [`ROLE_SKILL_TEACHING_TRIGGER_LEAD`]. Carried by `skill_teaching_response`; read
/// by the Rust skill compiler and the JS worker so neither names a reply verb in
/// code.
pub const ROLE_SKILL_TEACHING_RESPONSE_VERB: &str = "skill_teaching_response_verb";
/// Semantic role: a standalone imperative to add or update a behaviour rule.
///
/// A direct instruction to change runtime behaviour ("add behavior rule", "update
/// behavior rule", "добавь правило поведения", "添加行为规则") rather than a
/// trigger-and-reply teaching pair. Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] and recognised on its own — without
/// needing a separate response verb — because the imperative already names both the
/// edit and its object. Carried by `behavior_rule_edit`; read by the Rust skill
/// compiler and the JS worker.
pub const ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE: &str = "behavior_rule_edit_directive";
/// Semantic role: the when-then frame of a conditional skill instruction.
///
/// The head-and-link frame a conditional rule uses ("when … then ", "когда …
/// тогда ", "जब … तब ", "当 …时回答 "). Each surface is a
/// [`crate::seed::Slot::Circumfix`]: [`crate::seed::WordForm::before_slot`] is the
/// head the instruction must contain and [`crate::seed::WordForm::after_slot`] is
/// the link that must follow it. A skill is taught only when a backtick-quoted span
/// sits between the head and the link and another follows the link. Carried by
/// `skill_when_then`; read by the Rust skill compiler and the JS worker so neither
/// names a keyword pair in code.
pub const ROLE_SKILL_WHEN_THEN_PAIR: &str = "skill_when_then_pair";
/// Semantic role: the opening of a freely-phrased multi-step procedure (issue #674).
///
/// The lead a user says before describing a procedure they want compiled into a
/// skill ("when i ", "whenever i ", "когда я ", "जब मैं ", "当我"). Matched as a raw
/// substring after the prompt is lower-cased; its presence is one of the two guards
/// (the other being at least one recognised step) the procedure compiler requires
/// before it treats a prompt as a program at all. Carried by `skill_procedure_trigger`;
/// read by the Rust arbitrary-procedure compiler.
pub const ROLE_SKILL_PROCEDURE_TRIGGER_LEAD: &str = "skill_procedure_trigger_lead";
/// Semantic role: the word that ends one procedure clause and starts the next.
///
/// A conjunction or sequencing adverb ("and", "then", "и", "затем", "और", "然后").
/// The compiler splits a procedure sentence on punctuation and on these surfaces, so
/// widening the set of accepted connectives is a pure data edit. Carried by
/// `skill_procedure_separator`; read by the Rust arbitrary-procedure compiler.
pub const ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR: &str = "skill_procedure_clause_separator";
/// Semantic role: the verb naming what one compiled procedure step *does*.
///
/// Every meaning carrying this role is one entry of the step vocabulary, and the
/// meaning's own slug (`skill_procedure_fetch`, `skill_procedure_translate`, …) *is*
/// the canonical step kind the compiler emits and the executing host dispatches on.
/// A new step kind is therefore a new meaning in
/// `data/seed/meanings-skill-procedure.lino` plus a host capability — never a new
/// match arm in the compiler. Read by the Rust arbitrary-procedure compiler.
pub const ROLE_SKILL_PROCEDURE_STEP_VERB: &str = "skill_procedure_step_verb";
/// Semantic role: the noun naming what one compiled procedure step operates *on*.
///
/// The object of a step ("title", "translation", "both", "заголовок", "перевод",
/// "оба", …). The matched meaning's slug becomes the step's canonical argument, which
/// is why the same procedure phrased in English and in Russian canonicalises — and so
/// content-addresses — identically. Read by the Rust arbitrary-procedure compiler.
pub const ROLE_SKILL_PROCEDURE_STEP_OBJECT: &str = "skill_procedure_step_object";
/// Semantic role: a marker that a structured-skill step is non-deterministic.
///
/// A word flagging a step as non-deterministic or otherwise unreviewable ("random",
/// "nondeterministic", "non-deterministic", "arbitrary code", "случайный", "随机"),
/// which the compiler refuses because a compiled skill must be deterministic and
/// reviewable. Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the step text is lower-cased.
/// Carried by `nondeterministic_step`; read by the Rust skill compiler.
pub const ROLE_NONDETERMINISTIC_MARKER: &str = "nondeterministic_marker";
/// Semantic role: a cue that a structured-skill step needs the shell capability.
///
/// A word implying a step touches the local shell or filesystem (`local_shell`,
/// "shell", "filesystem", "list files", "оболочка", "文件系统"), so the compiler
/// requires an explicit `tool:local_shell` permission grant. Matched as a raw
/// substring through [`crate::seed::Lexicon::mentions_role_raw`] after the step text
/// is lower-cased, and checked before [`ROLE_NETWORK_CAPABILITY_CUE`] so a step that
/// touches both is attributed to the shell. The `tool:local_shell` identifier is a
/// tool-namespace bridge that stays in code; the cue surfaces live in the data.
/// Carried by `shell_capability_need`; read by the Rust skill compiler.
pub const ROLE_SHELL_CAPABILITY_CUE: &str = "shell_capability_cue";
/// Semantic role: a cue that a structured-skill step needs network access.
///
/// A word implying a step reaches the network or fetches a remote resource ("http",
/// "network", "fetch", "web request", "сеть", "网络请求"), so the compiler requires
/// an explicit `tool:web_fetch` permission grant. Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the step text is lower-cased,
/// and checked after [`ROLE_SHELL_CAPABILITY_CUE`]. The `tool:web_fetch` identifier
/// is a tool-namespace bridge that stays in code; the cue surfaces live in the data.
/// Carried by `network_capability_need`; read by the Rust skill compiler.
pub const ROLE_NETWORK_CAPABILITY_CUE: &str = "network_capability_cue";
/// Semantic role: a cue that a prompt is an investment word problem.
///
/// A word naming investing or an investment ("invest", "investment", "инвестиц",
/// "निवेश", "投资"). Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is lower-cased.
/// The compound-interest handler requires this together with
/// [`ROLE_INTEREST_CUE`] and [`ROLE_COMPOUNDING_ACTION_CUE`] before it answers.
/// Carried by `investment`; read by the Rust compound-interest handler and its
/// JS worker mirror.
pub const ROLE_INVESTMENT_CUE: &str = "investment_cue";
/// Semantic role: a cue that a prompt concerns financial interest.
///
/// A word naming interest in the money sense ("interest", "процент", "ब्याज",
/// "利息"). Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is lower-cased.
/// Required by the compound-interest handler together with
/// [`ROLE_INVESTMENT_CUE`] and [`ROLE_COMPOUNDING_ACTION_CUE`]. Carried by
/// `interest_finance`; read by the Rust compound-interest handler and its JS
/// worker mirror.
pub const ROLE_INTEREST_CUE: &str = "interest_cue";
/// Semantic role: a cue that interest is compounded.
///
/// A word naming compounding ("compound", "сложный процент", "चक्रवृद्धि",
/// "复利"). Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is lower-cased.
/// Required by the compound-interest handler together with
/// [`ROLE_INVESTMENT_CUE`] and [`ROLE_INTEREST_CUE`]. Carried by `compounding`;
/// read by the Rust compound-interest handler and its JS worker mirror.
pub const ROLE_COMPOUNDING_ACTION_CUE: &str = "compounding_action_cue";
/// Semantic role: how often interest compounds.
///
/// A word naming a compounding frequency ("monthly", "quarterly", "weekly",
/// "daily", "annually", "yearly"). Matched as a raw substring through
/// [`crate::seed::Lexicon::meanings_with_role`] in declaration order so the
/// finer frequencies are tried before the annual fallback. Each carrying meaning
/// maps to a fixed number of periods per year by slug (`compounding_monthly`,
/// `compounding_quarterly`, `compounding_weekly`, `compounding_daily`,
/// `compounding_annual`). Read by the Rust compound-interest handler and its JS
/// worker mirror.
pub const ROLE_COMPOUNDING_FREQUENCY_CUE: &str = "compounding_frequency_cue";
/// Semantic role: a request for a live, web-sourced exchange rate.
///
/// A phrase asking for a current or web exchange rate ("web", "current
/// exchange", "current rate", "exchange rate"). Matched as a raw substring
/// through [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is
/// lower-cased. The compound-interest handler reads it to add a caveat that web
/// freshness is not independently verified. Carried by `live_rate_freshness`;
/// read by the Rust compound-interest handler and its JS worker mirror.
pub const ROLE_LIVE_RATE_FRESHNESS_CUE: &str = "live_rate_freshness_cue";
/// Semantic role: the unit of a compound-interest term, the year.
///
/// A word naming the time unit a term is measured in ("year", "год", "वर्ष",
/// "年"). Located as a raw substring through
/// [`crate::seed::Lexicon::words_for_role`] so the handler can read the number
/// immediately before the earliest occurrence as the number of years. Carried by
/// `year_period`; read by the Rust compound-interest handler and its JS worker
/// mirror.
pub const ROLE_YEAR_UNIT_CUE: &str = "year_unit_cue";
/// Semantic role: a request to convert money between currencies.
///
/// A word naming a currency conversion ("convert", "конвертир", "परिवर्तित",
/// "转换"). Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is lower-cased.
/// The compound-interest handler requires it together with
/// [`ROLE_FINAL_AMOUNT_REFERENCE`] before it converts a previously computed final
/// amount. Carried by `conversion_action`; read by the Rust compound-interest
/// handler and its JS worker mirror.
pub const ROLE_CONVERSION_ACTION_CUE: &str = "conversion_action_cue";
/// Semantic role: a cue that a prompt converts one quantity into another.
///
/// A currency or unit conversion marker or verb (to, into, convert, exchange,
/// конвертировать, обмен, बदलें, परिवर्तित, 转换, 兑换). Matched whole-token through
/// [`crate::seed::Lexicon::mentions_role`] after the prompt is lower-cased, so the
/// bare target markers to/into signal a conversion only on a word boundary, never
/// as a substring of another word. `has_calculation_signal` reads it to exempt a
/// currency-plus-letters prompt from the prose-rejection guard, because a
/// conversion is itself a calculation. Distinct from
/// [`ROLE_CONVERSION_ACTION_CUE`], the money-specific verb the compound-interest
/// handler matches as a raw substring — this one is the calculator router's
/// general conversion signal and must stay whole-token. Carried by
/// `quantity_conversion`; read by the Rust calculation router (the JS worker
/// rescues currency conversions through its own currency-conversion evaluator).
pub const ROLE_QUANTITY_CONVERSION_CUE: &str = "quantity_conversion_cue";
/// Semantic role: a reference to a previously computed final amount.
///
/// A phrase naming the final amount of a calculation ("final amount", "итоговая
/// сумма", "अंतिम राशि", "最终金额"). Matched as a raw substring through
/// [`crate::seed::Lexicon::mentions_role_raw`] after the prompt is lower-cased.
/// The compound-interest handler requires it together with
/// [`ROLE_CONVERSION_ACTION_CUE`] before it converts the prior final amount.
/// Carried by `final_amount`; read by the Rust compound-interest handler and its
/// JS worker mirror.
pub const ROLE_FINAL_AMOUNT_REFERENCE: &str = "final_amount_reference";
/// Semantic role: a surface form naming the euro.
///
/// A word or code naming the euro ("eur", "euro", "euros", "евро", "यूरो",
/// "欧元"). Matched as a whole token through
/// [`crate::seed::Lexicon::mentions_role`] (and by prefix in `currency_after`) so
/// the compound-interest handler can decide whether to convert a final amount
/// into euros. The ISO code identifier "EUR" stays in code; the surfaces live in
/// the data. Carried by `euro`; read by the Rust compound-interest handler, the
/// calculation rate handlers, and the JS worker.
pub const ROLE_CURRENCY_EUR_REFERENCE: &str = "currency_eur_reference";
/// Semantic role: a surface form naming the Russian ruble.
///
/// A word or code naming the ruble ("rub", "ruble", "rubles", "рубль", "रूबल",
/// "卢布"). Matched as a whole token through
/// [`crate::seed::Lexicon::mentions_role`] so the calculation rate handlers and
/// the compound-interest worker can recognise a ruble currency code. The ISO code
/// identifier "RUB" stays in code; the surfaces live in the data. Carried by
/// `ruble`; read by the calculation rate handlers and the JS worker.
pub const ROLE_CURRENCY_RUB_REFERENCE: &str = "currency_rub_reference";
/// Semantic role: a calculator-domain term that signals a calculation.
///
/// A currency, measurement unit, or other quantity word whose presence beside a
/// number marks a prompt as arithmetic or a unit/currency conversion ("usd",
/// "kg", "ms", "доллар", "公斤", "месяцев"). `has_calculation_signal` reads it to
/// recognise a calculation when no operator symbol is present. ASCII surfaces
/// (the three-letter codes and English unit words) are matched whole-token so a
/// short code never fires inside a longer word; non-ASCII surfaces (Cyrillic
/// stems, CJK, Devanagari) are matched as raw substrings so every inflected form
/// is caught. Carried by the currency meanings (`us_dollar`, `euro`, `ruble`) and
/// the calculator-relevant measurement units (`kilogram`, `gram`, `kilobyte`,
/// `megabyte`, `second`, `minute`, `hour`, `millisecond`, `day`, `month`, `ton`);
/// read by the Rust calculation router and the JS worker.
pub const ROLE_CALCULATION_DOMAIN_TERM: &str = "calculation_domain_term";
/// Semantic role: the name of a mathematical function.
///
/// A function name such as "sqrt", "sin", "cos", "tan", "log", or "ln" (with its
/// translations). `has_calculation_signal` reads it so a prompt like "sqrt(16)"
/// is recognised as a calculation. ASCII names are matched on a leading word
/// boundary so they are caught even when they abut a parenthesis; non-ASCII names
/// are matched as raw substrings. Carried by the function meanings under the
/// `mathematical_function` genus (`square_root`, `sine`, `cosine`, `tangent`,
/// `logarithm`, `natural_logarithm`); read by the Rust calculation router and the
/// JS worker.
pub const ROLE_MATH_FUNCTION_NAME: &str = "math_function_name";
/// Semantic role: the "combine the numbers" framing of a reachability search.
///
/// The word that signals a prompt is about combining given *numbers* — "number"
/// (en), the Cyrillic stem "числ" (числа / чисел / числами), Devanagari "संख्या",
/// and CJK "数字". Matched as a raw substring so every inflected form is caught.
/// `crate::solver_search` requires it *together with* [`ROLE_REACHABILITY_SEARCH_CUE`]
/// to recognise a budget-driven reachability search, so a plain calculation never
/// reaches that path. Carried by `reachability_operand_framing`; read by the Rust
/// solver's budget-search stage.
pub const ROLE_REACHABILITY_OPERAND_FRAMING: &str = "reachability_operand_framing";
/// Semantic role: the search verb of a reachability problem.
///
/// "find" / "combine" / "reach" / "make" / "express" / "arrange" and their
/// translations, recorded as bare stems and matched as raw substrings so inflected
/// forms (найдите, संयोजित) still hit. Read together with
/// [`ROLE_REACHABILITY_OPERAND_FRAMING`] to gate the budget-search stage.
/// Carried by `reachability_search_action`; read by the Rust solver's budget-search
/// stage.
pub const ROLE_REACHABILITY_SEARCH_CUE: &str = "reachability_search_cue";
/// Semantic role: the target-value marker of a reachability problem.
///
/// "equals" / "equal to" / "results in" and their translations ("равно", "बराबर",
/// "等于", …), recorded as surfaces and matched as raw substrings. The byte
/// position nearest such a marker anchors which integer in the prompt is the
/// *target* the search must reach, so the operand-then-target and
/// target-then-marker orders both resolve without a language branch. Carried by
/// `reachability_target_marker`; read by the Rust solver's budget-search stage.
pub const ROLE_REACHABILITY_TARGET_MARKER: &str = "reachability_target_marker";
