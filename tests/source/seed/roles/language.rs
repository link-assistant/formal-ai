//! Role constants for language work: translation-direction markers,
//! definition-in-links, compositional lemmas, the ontology backbone,
//! behavior-rule listing, proof, and who-is questions (issue #386).
//!
//! Re-exported flat through [`super`] so every constant stays reachable as
//! `crate::seed::roles::ROLE_*` and `crate::seed::ROLE_*` (issue #386).

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
/// Semantic role: a marker that names the language a non-translation answer
/// should be rendered in.
///
/// Concept lookup uses this to distinguish `Tell me about Telegram Ads in
/// Russian` from a context query about a concept named `Telegram Ads in
/// Russian`: the marker meaning is `defined_by` one `language_*` meaning, so
/// the handler resolves the response language by walking the seed graph rather
/// than by hardcoding multilingual suffix phrases.
pub const ROLE_RESPONSE_LANGUAGE_MARKER: &str = "response_language_marker";
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
/// Semantic role: the imperative verb that asks for a phrase to be defined.
///
/// The `try_translation` request-gate reads this instead of a hardcoded verb to
/// recognise a define-in-Links-Notation request. Only the English surface is
/// scanned, as a clause-initial prefix with a trailing space (so `defined` and
/// `definition` never trigger it); the Russian, Hindi and Chinese imperatives are
/// carried for coverage but not consulted, mirroring the original recogniser which
/// gated on the English verb alone. Carried by `definition_command` in
/// `data/seed/meanings-translation.lino`.
pub const ROLE_DEFINITION_COMMAND: &str = "definition_command";
/// Semantic role: a phrase naming Links Notation as a render-target format.
///
/// The `try_translation` request-gate reads this instead of hardcoded format
/// strings: the English `links notation` and the Russian code-switched `в links`
/// are scanned as space-prefixed substrings, exactly the original recogniser's two
/// literals; the Hindi and Chinese renderings are carried for coverage but not
/// consulted. Carried by `links_notation_format` in
/// `data/seed/meanings-translation.lino`.
pub const ROLE_LINKS_NOTATION_FORMAT: &str = "links_notation_format";
/// Semantic role: a source-language lemma the compositional translator composes.
///
/// The ru→en compositional fallback (`compositional_candidates` in
/// `src/translation/pipeline.rs`) fires only when no Wiktionary/Wikidata entry
/// resolves a multi-word title. It walks the title word by word, resolving each
/// Russian surface to its English form through the meaning carrying this role
/// that lists the surface — so the per-word table lives in
/// `data/seed/meanings-translation.lino`, never in code. Head-initial English and
/// Russian are the consulted pair; the Hindi and Chinese forms are carried for
/// coverage. A form tagged `action "genitive"` is the inflected noun the
/// genitive-of construction reads (see [`ROLE_COMPOSITIONAL_GENITIVE_HEAD`]).
pub const ROLE_COMPOSITIONAL_LEMMA: &str = "compositional_lemma";
/// Semantic role: a fixed source-language phrase with a canned target rendering.
///
/// Some short Russian questions translate as wholes, not word by word (`кто ты
/// такой` → `Who are you?`). The compositional fallback looks the normalized title
/// up among the meanings carrying this role before attempting word-by-word
/// composition, returning the meaning's English form verbatim — terminal
/// punctuation and capitalization included. The phrases live in
/// `data/seed/meanings-translation.lino`; the code names only the role.
pub const ROLE_COMPOSITIONAL_PHRASE: &str = "compositional_phrase";
/// Semantic role: a relation noun that governs a Russian genitive complement.
///
/// In a phrase like `примеры согласования` (`examples of agreement`) the head noun
/// takes a genitive-inflected complement English renders with `of`. The
/// compositional translator treats a [`ROLE_COMPOSITIONAL_LEMMA`] word also
/// carrying this role as such a head: when the next word is a lemma form tagged
/// `action "genitive"`, it emits `head of complement`. The heads live in
/// `data/seed/meanings-translation.lino`; only the construction rule is code.
pub const ROLE_COMPOSITIONAL_GENITIVE_HEAD: &str = "compositional_genitive_head";
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
/// Semantic role: a quantity cue asking for the number of behavior rules.
///
/// Surface cues "how many"/"count"/"total", "сколько"/"всего", "कितने"/"कुल",
/// and "多少"/"一共"; combined with the rule subject and either explicit
/// behavior scope or a previous rule-list answer by the behavior-rules handler.
pub const ROLE_RULE_COUNT_REQUEST: &str = "rule_count_request";
/// Semantic role: a brevity cue asking to compress the previous behavior-rule
/// listing instead of repeating the full catalog.
///
/// Surface cues "brief"/"short", "кратко"/"коротко", "संक्षेप", and "简短";
/// used only when the preceding assistant message was the behavior-rule list.
pub const ROLE_RULE_BRIEF_REQUEST: &str = "rule_brief_request";
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
