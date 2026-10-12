// Known-fact inventory queries, runtime rule updates, feature capability
// status, the architecture explanation and translation phrase normalization.
// Loaded by ../formal_ai_worker.js.
// Issue #386: a known-facts inventory query is recognised by composing meaning
// roles, not by matching raw words per language. The universal algorithm is
// identical for every language: the prompt either names the knowledge `fact`
// noun together with an enumerating interrogative and a second-person
// attribution of knowing, or it matches one of the complete standalone
// phrasings that ask what the assistant knows even without the noun. The
// prompt is re-normalised first so the boundary-aware matcher sees punctuation
// collapsed to spaces. Mirror of is_known_fact_query in
// src/solver_handlers/self_awareness.rs.
function isKnownFactQuery(normalized) {
  if (isSelfFactQuery(normalized)) return false;
  const cleaned = normalizePrompt(normalized);
  const composed =
    lexiconMentionsRole(ROLE_KNOWLEDGE_INVENTORY_NOUN, cleaned) &&
    lexiconMentionsRole(ROLE_KNOWLEDGE_INVENTORY_INTERROGATIVE, cleaned) &&
    lexiconMentionsRole(ROLE_KNOWLEDGE_POSSESSION, cleaned);
  return (
    composed || lexiconMentionsRole(ROLE_KNOWLEDGE_INVENTORY_PHRASE, cleaned)
  );
}

function cleanRuleQuery(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[\s`"':._,\-?!]+|[\s`"':._,\-?!]+$/g, "")
    .toLowerCase();
}

// The rule a prompt asks to read: the text after its longest seeded
// `rule_detail_request` opening. Mirrors `fn detail_query` in
// rust/src/solver_handlers/behavior_rules.rs.
const ROLE_RULE_DETAIL_REQUEST = "rule_detail_request";

function detailQuery(prompt) {
  const lower = String(prompt || "").toLowerCase();
  const opening = wordsForRole(ROLE_RULE_DETAIL_REQUEST)
    .map((surface) => String(surface).toLowerCase())
    .filter((surface) => surface && lower.startsWith(surface))
    .sort((left, right) => right.length - left.length)[0];
  if (opening) return cleanRuleQuery(String(prompt || "").slice(opening.length));
  if (lower.includes("rule_unknown")) return "unknown";
  return "";
}

function findBehaviorRule(query) {
  const cleaned = cleanRuleQuery(query);
  const withoutPrefix = cleaned.startsWith("rule_") ? cleaned.slice(5) : cleaned;
  return behaviorRuleRecords().find(
    (rule) =>
      rule.id === cleaned ||
      rule.id === `rule_${withoutPrefix}` ||
      rule.intent === cleaned ||
      rule.intent === withoutPrefix ||
      rule.label.toLowerCase().includes(withoutPrefix),
  );
}

const directRoleSurfacePresent = (role, lower) => roleWordForms(role).some((form) => form.text && lower.includes(form.text));

// All teaching, catalog and history replay consumers use the same package.
function runtimeRuleFromText(text) {
  const result = crateModule("crate/skill_compiler.mjs").compileNaturalLanguageSkill(String(text || ""));
  return result.status === "compiled" ? result : null;
}

function runtimeRuleForPrompt(prompt, history) {
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index] || {};
    if (String(turn.role || "").toLowerCase() !== "user") continue;
    const rule = runtimeRuleFromText(turn.content);
    if (rule && rule.replay(prompt)) {
      return rule;
    }
  }
  return null;
}

function collectRuntimeRules(history) {
  const turns = Array.isArray(history) ? history : [];
  const seen = new Set();
  const rules = [];
  for (const turn of turns) {
    const role = String((turn || {}).role || "").toLowerCase();
    if (role !== "user") continue;
    const rule = runtimeRuleFromText((turn || {}).content);
    if (rule && !seen.has(rule.id)) {
      seen.add(rule.id);
      rules.push(rule);
    }
  }
  return rules;
}

function tryBehaviorRules(prompt, normalized, history, preferences) {
  const language = behaviorRuleResponseLanguage(normalized, detectLanguage(prompt));
  const preferenceMatch = String(prompt || "").match(/`([^`]+)`/u);
  if (
    preferenceMatch &&
    directRoleSurfacePresent(ROLE_CONVERSATION_PREFERENCE_AVOID, normalized)
  ) {
    return {
      intent: "conversation_preference",
      content: answerFor("conversation_preference", language)
        .replace("{term}", preferenceMatch[1].trim()),
      confidence: 1.0,
      evidence: ["conversation_preference:avoid_term"],
    };
  }
  if (
    directRoleSurfacePresent(ROLE_UNAUTHORIZED_MUTATION_CORRECTION, normalized)
  ) {
    return {
      intent: "action_correction",
      content: answerFor("action_correction", language),
      confidence: 1.0,
      evidence: ["action_correction:unauthorized_mutation"] // #1175 R3: no earlier action is the refusal lane
        .concat(lastHistoryTurn(history, "assistant") ? [] : ["conversation_control:refusal:no earlier action"]),
    };
  }
  const updateRule = runtimeRuleFromText(prompt);
  if (updateRule) {
    return {
      intent: "behavior_rule_update",
      content: renderRuntimeRuleUpdate(updateRule, language),
      confidence: 1.0,
      evidence: [],
      solverEvents: [solverEvent("skill_compile:package", updateRule.id),
        solverEvent("behavior_rule:update", updateRule.legacy_behavior_rule_id)],
    };
  }

  const compiled = tryCompiledProcedure(prompt, language);
  if (compiled) return compiled;

  if (isBehaviorRulesCountQuery(normalized, history)) {
    const runtimeRules = collectRuntimeRules(history);
    const counts = behaviorRuleCounts(runtimeRules);
    return {
      intent: "behavior_rules_count",
      content: renderBehaviorRuleCount(runtimeRules, language),
      solverEvents: [solverEvent("behavior_rules:count", String(counts.total)),
        solverEvent("behavior_rules:built_in_count", String(counts.builtIn)),
        solverEvent("behavior_rules:runtime_count", String(counts.runtime)),
        solverEvent("reasoning:operation", "count_behavior_rules_catalog_plus_dialog_local_rules"),
        solverEvent("reasoning:result", `built_in=${counts.builtIn};runtime=${counts.runtime};total=${counts.total}`)],
      confidence: 1.0,
      evidence: [
        "behavior_rules:count",
        `behavior_rules:built_in_count:${counts.builtIn}`,
        `behavior_rules:runtime_count:${counts.runtime}`,
        `reasoning:result:total:${counts.total}`,
      ],
    };
  }

  if (isBehaviorRulesBriefFollowup(normalized, history)) {
    const runtimeRules = collectRuntimeRules(history);
    const counts = behaviorRuleCounts(runtimeRules);
    return {
      intent: "behavior_rules_brief",
      content: renderBehaviorRulesBrief(runtimeRules, language),
      confidence: 1.0,
      evidence: [
        "behavior_rules:brief",
        `behavior_rules:built_in_count:${counts.builtIn}`,
        `behavior_rules:runtime_count:${counts.runtime}`,
        `reasoning:result:total:${counts.total}`,
      ],
    };
  }

  if (isBehaviorRulesList(normalized)) {
    return {
      intent: "behavior_rules_list",
      content: renderBehaviorRuleList(collectRuntimeRules(history), language),
      confidence: 1.0,
      evidence: ["behavior_rules:list", "all"],
    };
  }

  const query = detailQuery(prompt);
  if (query) {
    const rule = findBehaviorRule(query);
    if (rule) {
      return {
        intent: "behavior_rule_detail",
        content: renderBehaviorRuleDetail(rule, language),
        confidence: 1.0,
        evidence: ["behavior_rule:read", rule.id],
      };
    }
  }

  if (isSelfIntroductionQuery(normalized)) {
    const language = selfAwarenessLanguage(prompt, normalized);
    return {
      intent: "identity",
      content: selfIntroductionContent(language, preferences),
      confidence: 1.0,
      evidence: [
        "identity:self_introduction",
        `language:${language}`,
        `assistant_name:${assistantNameStatus(preferences)}`,
      ],
    };
  }

  if (isArchitectureQuestion(normalized)) {
    const language = architectureLanguage(prompt, normalized);
    return {
      intent: "meta_explanation",
      content: architectureExplanationContent(language),
      confidence: 1.0,
      evidence: [
        "response:meta_explanation",
        "meta_explanation:self_awareness",
        `language:${language}`,
      ],
    };
  }

  if (isSelfFactQuery(normalized)) {
    return {
      intent: "self_facts",
      content: renderSelfFacts(preferences),
      confidence: 1.0,
      evidence: ["self_facts:list", "formal-ai"],
    };
  }

  if (isKnownFactQuery(normalized)) {
    const language = selfAwarenessLanguage(prompt, normalized);
    return {
      intent: "known_facts",
      content: renderKnownFacts(language, preferences),
      confidence: 1.0,
      evidence: ["known_facts:list", "formal-ai", `language:${language}`],
    };
  }

  // The `conversation_topic` rule of data/seed/handler-rules.lino: the topic is
  // the slot of a conversation_topic_opener surface, the wording is seeded.
  const topic = runHandlerRuleSet("conversation_topic", prompt, String(prompt || "").toLowerCase(), []);
  if (topic) return topic;

  const runtimeRule = runtimeRuleForPrompt(prompt, history);
  if (runtimeRule) {
    return {
      intent: "behavior_rule_custom",
      content: runtimeRule.response,
      confidence: 1.0,
      responseLink: `response:${runtimeRule.id}`,
      evidence: [],
      solverEvents: [solverEvent("compiled_skill:package", runtimeRule.linksNotation()),
        solverEvent("compiled_skill:replay", runtimeRule.rule_id),
        solverEvent("cache_hit", runtimeRule.id)],
    };
  }

  return null;
}

function containsAny(normalized, values) {
  if (!normalized || !Array.isArray(values)) return false;
  return values.some((value) => value && normalized.includes(String(value).toLowerCase()));
}

// Issue #386 feature-capability roles — mirror the ROLE_FEATURE_* consts in
// src/seed/roles.rs. Their surface forms live in
// data/seed/meanings-feature-capability.lino (loaded into MEANINGS_LINO).
// detectFeatureCapability walks the `feature_capability_alias` meanings in seed
// declaration order (= the historical FEATURE_CAPABILITIES priority) and takes
// the first whose multilingual aliases occur as a raw substring; the question
// gate and the two action gates reference the other roles. No surface word is
// named here — they all live in the data.
const ROLE_FEATURE_CAPABILITY_ALIAS = "feature_capability_alias";
const ROLE_FEATURE_CAPABILITY_QUESTION = "feature_capability_question";
const ROLE_FEATURE_ACTION_ARITHMETIC = "feature_action_arithmetic";
const ROLE_FEATURE_ACTION_PLANNING = "feature_action_planning";

// Issue #918: the features, their runtime switches and their localized labels
// and examples are data/seed/feature-capabilities.lino, read here as the
// native handler reads it; a language without an entry falls back to English.
function featureCapabilities() {
  const out = [];
  for (const root of codeTaskSeedRecords("feature-capabilities.lino")) {
    for (const node of codeTaskChildren(root, "feature")) {
      const localized = (kind) => {
        const group = codeTaskChildren(node, kind)[0];
        const table = {};
        for (const entry of (group && group.children) || []) table[entry.name] = String(entry.id || "");
        return table;
      };
      out.push({
        slug: String(node.id || "").replace(/^feature_capability_/u, ""),
        state: codeTaskChildValue(node, "state"),
        labels: localized("label"),
        examples: localized("example"),
      });
    }
  }
  return out;
}

function localizedValue(record, language) {
  if (!record || typeof record !== "object") return "";
  return record[language] || record.en || "";
}

// Walk the `feature_capability_alias` meanings in seed declaration order — the
// historical FEATURE_CAPABILITIES priority — and return the first capability
// whose multilingual forms occur as a raw substring of `normalized`, checked in
// the prompt's own language plus English (English prompts check English only).
// The matched meaning's slug, minus its `feature_capability_` prefix, keys
// FEATURE_CAPABILITIES, so no surface alias is named here. Mirrors
// detect_feature_capability in src/solver_handlers/feature_capability.rs (#386).
function detectFeatureCapability(normalized, language) {
  const languages = language === "en" ? ["en"] : [language, "en"];
  const meaning = firstRoleMatchInLanguagesRaw(
    ROLE_FEATURE_CAPABILITY_ALIAS,
    normalized,
    languages,
  );
  if (!meaning) return null;
  const prefix = "feature_capability_";
  if (!meaning.slug.startsWith(prefix)) return null;
  const slug = meaning.slug.slice(prefix.length);
  return featureCapabilities().find((feature) => feature.slug === slug) || null;
}

// A prompt is a capability question when one of the `feature_capability_question`
// interrogative cues occurs as a raw substring, checked in the prompt's own
// detected language only. A language the seed gives no cue reads as English,
// which also accepts a grammatical "is/are ... enabled/available" frame
// computed in code. Mirrors is_feature_capability_question in
// src/solver_handlers/feature_capability.rs (#386, R1188-U1).
function isFeatureCapabilityQuestion(normalized, language) {
  const mentions = (lang) =>
    mentionsRoleInLanguagesRaw(ROLE_FEATURE_CAPABILITY_QUESTION, normalized, [lang]);
  const cues = wordsForRoleInLanguages(ROLE_FEATURE_CAPABILITY_QUESTION, [language]);
  if (language !== "en" && cues.length > 0) return mentions(language);
  return mentions("en") || isEnglishAvailabilityQuestion(normalized);
}

// English-only grammatical "is/are ... enabled/available" availability frame —
// a grammatical pattern (not a vocabulary list), so it stays in code. Mirrors
// is_english_availability_question in
// src/solver_handlers/feature_capability.rs (#386).
function isEnglishAvailabilityQuestion(normalized) {
  return /\b(?:is|are)\s+(?:your\s+|the\s+|this\s+|formal-ai\s+)?[\w\s/-]{1,80}\s+(?:enabled|available)\b/.test(
    normalized,
  );
}

// True when a detected capability question is actually an action request that a
// dedicated handler should answer. The English action frames live in the
// `feature_action_arithmetic` / `feature_action_planning` meanings; they are
// read through wordsForRoleInLanguages restricted to English and reconstructed
// as space-padded forms (prefix for arithmetic, anywhere for planning), so no
// frame is named here. Mirrors is_feature_action_request in
// src/solver_handlers/feature_capability.rs (#386).
function isFeatureActionRequest(normalized, feature) {
  if (!feature) return false;
  if (feature.slug === "arithmetic") {
    return wordsForRoleInLanguages(ROLE_FEATURE_ACTION_ARITHMETIC, ["en"]).some(
      (frame) => normalized.startsWith(`${frame} `),
    );
  }
  if (feature.slug === "planning") {
    return wordsForRoleInLanguages(ROLE_FEATURE_ACTION_PLANNING, ["en"]).some(
      (frame) => normalized.includes(`${frame} `),
    );
  }
  return false;
}

function capabilityResponse(intent, language, values = {}) {
  let text = answerFor(intent, language);
  for (const [name, value] of Object.entries(values)) text = text.split(`{${name}}`).join(String(value));
  return text;
}

function webSearchStatusContent(language, available, providers) {
  return available
    ? capabilityResponse("feature_capability_web_search_available", language, { k: webSearchRrfK(), providers: providers || "none" })
    : capabilityResponse("feature_capability_web_search_unavailable_browser", language);
}

function featureAvailability(feature, preferences) {
  if (!feature) return { available: false, reason: "unknown" };
  if (feature.state === "web_search") {
    const providers = WEB_SEARCH_PROVIDERS.filter((provider) => !webSearchIsDisabled(provider.id));
    const online = typeof navigator === "undefined" || navigator.onLine !== false;
    return {
      available: online && providers.length > 0,
      reason: online && providers.length > 0 ? "none" : "offline_or_no_providers",
      providers,
    };
  }
  const switches = {
    diagnostic_mode: [Boolean(preferences && preferences.diagnosticsMode), "diagnostic_mode_off"],
    agent_mode: [Boolean(preferences && preferences.agentMode), "agent_mode_off"],
    definition_fusion: [definitionFusionByDefault(preferences || {}), "definition_fusion_explicit"],
  };
  const [available, reason] = switches[feature.state] || [true, "none"];
  return { available, reason: available ? "none" : reason };
}

function featureCapabilityContent(feature, language, availability) {
  if (feature.slug === "web_search") {
    const providers = availability.providers || [];
    return webSearchStatusContent(
      language,
      availability.available,
      providers.map((provider) => provider.id).join(", "),
    );
  }
  const label = localizedValue(feature.labels, language);
  const example = localizedValue(feature.examples, language);
  if (availability.available) {
    return capabilityResponse("feature_capability_available", language, { label, example });
  }
  const reason = answerFor(`feature_capability_reason_${availability.reason}`, language);
  return capabilityResponse("feature_capability_unavailable", language, { reason, label, example });
}

function tryFeatureCapabilityStatus(prompt, normalized, language, preferences) {
  if (!isFeatureCapabilityQuestion(normalized, language)) return null;
  const feature = detectFeatureCapability(normalized, language);
  if (!feature) return null;
  if (isFeatureActionRequest(normalized, feature)) return null;
  const availability = featureAvailability(feature, preferences || {});
  const providers = WEB_SEARCH_PROVIDERS.filter((provider) => !webSearchIsDisabled(provider.id));
  return {
    intent: "capabilities",
    content: featureCapabilityContent(feature, language, availability),
    confidence: availability.available ? 0.95 : 0.6,
    evidence: [
      "handler:capabilities",
      `feature:question:${feature.slug}`,
      availability.available
        ? `feature:available:${feature.slug}`
        : `feature:unavailable:${feature.slug}:${availability.reason}`,
      ...(feature.slug === "web_search" ? providers.map((provider) => `web_search:provider:${provider.id}`) : []),
      `language:${language}`,
    ],
  };
}

// Issue #386: recognise "what else can you do" / "что ещё ты умеешь" /
// "और क्या कर सकते" / "你还能做什么" by the capability_query_more meaning role
// rather than a hardcoded per-language phrase list. Recognition is
// language-agnostic because the surface words are script-specific; the response
// body is still chosen by the caller from detectLanguage. The prompt is
// re-normalised so trailing punctuation collapses to the canonical spacing the
// seed stores. Mirror of is_more_capabilities_prompt in
// src/solver_handlers/user_intent.rs.
function isMoreCapabilitiesPrompt(normalized) {
  return lexiconMentionsRole(ROLE_CAPABILITY_QUERY_MORE, normalizePrompt(normalized));
}

// Issue #386: recognise "what can you do" / "что ты умеешь" / "что за дичь" /
// "आप क्या कर सकते" / "你能做什么" by the capability_query meaning role — plus
// its follow-up capability_query_more, so "what else can you do" still counts —
// rather than a hardcoded per-language phrase list. Mirror of
// is_capability_query in src/solver_handlers/user_intent.rs.
function isCapabilityQuery(normalized) {
  const cleaned = normalizePrompt(normalized);
  return (
    lexiconMentionsRole(ROLE_CAPABILITY_QUERY, cleaned) ||
    lexiconMentionsRole(ROLE_CAPABILITY_QUERY_MORE, cleaned)
  );
}

function historyMentionsWebSearch(history) {
  if (!Array.isArray(history)) return false;
  return history.some((turn) => {
    const content = String(turn && turn.content ? turn.content : "").toLowerCase();
    return lexiconMentionsRoleSubstring(ROLE_WEB_SEARCH_HISTORY_SIGNAL, content);
  });
}

// R1188-U1: the capability listings are the seeded `capabilities` and
// `capabilities_more` responses, as rust/src/solver_handlers/user_intent.rs
// answers them; no listing or example prompt lives here.
function additionalCapabilitiesContent(language) {
  return answerFor("capabilities_more", language);
}

// True when the prompt asks how the assistant itself is built rather than
// requesting a task. Decomposes exactly like the Rust is_architecture_question:
// the prompt must address the assistant — carry an assistant_self_reference
// surface — and name an architecture_concept such as a language model, neural
// network, or the project's local rules. Both are matched as raw substrings
// across all four languages; no architecture word is hardcoded here.
function isArchitectureQuestion(normalized) {
  if (!lexiconMentionsRoleSubstring(ROLE_ASSISTANT_SELF_REFERENCE, normalized)) {
    return false;
  }
  return lexiconMentionsRoleSubstring(ROLE_ARCHITECTURE_CONCEPT, normalized);
}

function architectureLanguage(prompt, normalized) {
  return selfAwarenessLanguage(prompt, normalized);
}

function architectureExplanationContent(language) {
  // Issue #918: the seeded meta_explanation_architecture record, filled with
  // this surface's notes in one pass as architecture_explanation_body fills
  // the native ones.
  const surface = BROWSER_SURFACE;
  const values = { surface_label: surface.label, surface: surface.slug, runtime: surface.runtime, memory: surface.memory, web_search: surface.webSearch };
  return answerFor("meta_explanation_architecture", language).replace(/\{([^{}]*)\}/gu, (whole, name) =>
    Object.prototype.hasOwnProperty.call(values, name) ? String(values[name]) : whole);
}

function tryArchitectureExplanation(prompt, normalized) {
  if (!isArchitectureQuestion(normalized)) return null;
  const language = architectureLanguage(prompt, normalized);
  return {
    intent: "meta_explanation",
    content: architectureExplanationContent(language),
    confidence: 1.0,
    evidence: ["response:meta_explanation", "meta_explanation:architecture", `language:${language}`],
  };
}

function tryCapabilities(prompt, normalized, preferences, history) {
  const language = detectLanguage(prompt);
  const featureStatus = tryFeatureCapabilityStatus(prompt, normalized, language, preferences);
  if (featureStatus) return featureStatus;
  const moreCapabilities = isMoreCapabilitiesPrompt(normalized);
  if (!isCapabilityQuery(normalized)) return null;
  if (moreCapabilities) {
    const priorSearch = historyMentionsWebSearch(history);
    return {
      intent: "capabilities",
      content: additionalCapabilitiesContent(language),
      confidence: 1.0,
      evidence: [
        "handler:capabilities",
        "capabilities:follow_up",
        ...(priorSearch ? ["capabilities:history:prior-web-search"] : []),
        `language:${language}`,
      ],
    };
  }
  const content = answerFor("capabilities", language);
  return {
    intent: "capabilities",
    content,
    confidence: 1.0,
    evidence: ["handler:capabilities", `language:${language}`],
  };
}

// Issue #386: the source/target language of a translation prompt is read from
// the lexicon, not a hardcoded per-language phrase ladder. Each translation
// source/target marker meaning enumerates its surfaces across all four
// languages and is defined_by the language_* meaning it names; detection walks
// those meanings in declaration order (en, ru, hi, zh) and resolves the code
// through defined_by. Mirrors detect_source_language / detect_target_language
// in src/translation/language_markers.rs.
function detectTranslationSourceLanguage(normalized) {
  return detectTranslationMarkerLanguage(
    ROLE_TRANSLATION_SOURCE_MARKER,
    normalized,
  );
}

function detectTranslationTargetLanguage(normalized) {
  return detectTranslationMarkerLanguage(
    ROLE_TRANSLATION_TARGET_MARKER,
    normalized,
  );
}

// Offline phrase registry for the browser worker.
//
// The Rust pipeline (`src/translation/pipeline.rs`) resolves any pair
// of surfaces through Wiktionary + Wikidata using cached HTTP
// responses. The worker mirrors that with a live `liveWiktionaryTranslate`
// fallback below (MediaWiki action API is CORS-friendly via
// `origin=*`), and first looks up the stock phrases of the seed so the demo
// stays snappy when the network is slow. Each is a meaning carrying the role
// `translation_phrase` (data/seed/meanings-translation-phrases.lino,
// R1188-U1): the first surface of a language is the form deformalization
// renders, every surface is a phrasing formalization recognizes.
const ROLE_TRANSLATION_PHRASE = "translation_phrase";

function translationPhraseRegistry() {
  return meaningsWithRole(ROLE_TRANSLATION_PHRASE).map((meaning) => ({
    token: meaning.slug,
    forms: Object.fromEntries((meaning.lexemes || []).map((lexeme) => [lexeme.language, lexeme.words])),
  }));
}

const TRANSLATION_TERMINAL_PUNCTUATION = ["?", "!", ".", "。", "？", "！", "．"];

function normalizeTranslationAlias(surface) {
  return Array.from(String(surface || "").toLowerCase())
    .filter((character) => /[\p{L}\p{N}]/u.test(character))
    .join("");
}

function formalizeSurface(surface, source) {
  const normalized = normalizeTranslationAlias(surface);
  if (!normalized) return null;
  const entry = translationPhraseRegistry().find((phrase) =>
    (phrase.forms[source] || []).some((form) => normalizeTranslationAlias(form) === normalized));
  return entry ? entry.token : null;
}

// The surface among `candidates`, offered in `target` for `surface` in
// `source`, that survives the round trip best (Rust
// `Translation::round_trip_surface`, R1188-U19): the crate's
// `roundTripChoice`, with the first candidate standing wherever the round
// trip cannot tell them apart.
function roundTripSurface(surface, source, target, candidates) {
  if (candidates.length === 0) return null;
  const { roundTripChoice } = crateModule("crate/round_trip_translation.mjs");
  return candidates[roundTripChoice(String(surface || ""), source, target, candidates) ?? 0];
}

function deformalizeMeaning(token, target, surface, source) {
  const entry = translationPhraseRegistry().find((phrase) => phrase.token === token);
  return roundTripSurface(surface, source, target, (entry && entry.forms[target]) || []);
}

function canonicalTokenForNormalized(normalized) {
  if (!normalized) return null;
  const entry = translationPhraseRegistry().find((phrase) =>
    Object.values(phrase.forms).some((forms) => forms.some((form) => normalizeTranslationAlias(form) === normalized)));
  return entry ? entry.token : null;
}

function canonicalMeaningToken(raw) {
  return canonicalTokenForNormalized(raw) || raw;
}

function normalizeMeaningText(surface) {
  const raw = normalizeTranslationAlias(surface);
  return canonicalMeaningToken(raw);
}
