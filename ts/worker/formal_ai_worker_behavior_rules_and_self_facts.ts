// Translation marker detection, the behavior-rule listing, count, brief and
// detail answers, and the self-fact and self-introduction answers.
// Loaded by ../formal_ai_worker.js.
function stableBehaviorRuleId(prefix, value) {
  const fromWasm = wasmStableId(prefix, value);
  if (fromWasm) return fromWasm;
  let hash = 0xcbf29ce484222325n;
  const sourceBytes = new TextEncoder().encode(String(value || ""));
  for (const byte of sourceBytes) {
    hash ^= BigInt(byte);
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return `${prefix}_${hash.toString(16).padStart(16, "0")}`;
}

function extractQuotedPhrase(text) {
  const source = String(text || "");
  const pairs = [
    ['"', '"'],
    ["'", "'"],
    ["`", "`"],
    ["«", "»"],
  ];
  for (const [open, close] of pairs) {
    const start = source.indexOf(open);
    if (start === -1) continue;
    const end = source.indexOf(close, start + open.length);
    if (end !== -1) return source.slice(start + open.length, end);
  }
  return null;
}

// Issue #386 translation roles mirror src/seed/roles.rs. Their slot-marked
// forms live in meanings-translation.lino; helpers query them by role/shape.
const ROLE_TRANSLATION_SOURCE_MARKER = "translation_source_marker";
const ROLE_TRANSLATION_TARGET_MARKER = "translation_target_marker";
const ROLE_RESPONSE_LANGUAGE_MARKER = "response_language_marker";
const ROLE_TRANSLATION_TARGET_DIRECTION = "translation_target_direction";
const ROLE_TRANSLATION_UNQUOTED_FRAME = "translation_unquoted_frame";
const ROLE_TRANSLATION_INTO_MARKER = "translation_into_marker";
const ROLE_TRANSLATION_OBJECT_MARKER = "translation_object_marker";

// The ISO 639-1 code of the language_* meaning that defines a marker. Mirrors
// language_code in src/translation/language_markers.rs: the surface *names* of
// each language live in the seed; only this slug -> code bridge stays in code.
function translationLanguageCode(meaning) {
  return meaningDefinedLanguageCode(meaning);
}

// The first marker meaning of `role` (in declaration order) any of whose
// surface words is a substring of `normalized` reports its language. Plain
// substring matching — not the boundary-aware surfacePresent — is intentional
// and mirrors detect_marker_language in src/translation/language_markers.rs: a
// CJK marker like 从中文 has no word spaces, and a Cyrillic marker like
// "с английского" must match inside a longer sentence.
function detectTranslationMarkerLanguage(role, normalized) {
  for (const meaning of meaningsWithRole(role)) {
    if (meaning.words.some((word) => normalized.includes(word))) {
      const code = translationLanguageCode(meaning);
      if (code) return code;
    }
  }
  return null;
}

function detectResponseLanguage(normalized) {
  return detectTranslationMarkerLanguage(ROLE_RESPONSE_LANGUAGE_MARKER, normalized);
}

// The role naming every "I cannot understand this" surface. Its phrase table
// lives in data/seed/meanings-translation.lino; only the role name stays in
// code. Mirrors ROLE_COMPREHENSION_FAILURE_MARKER in
// src/translation/language_markers.rs (issue #556).
const ROLE_COMPREHENSION_FAILURE_MARKER = "comprehension_failure_marker";

// True when the user reports they cannot understand the prior answer — any
// seeded surface ("do not understand", "не понимаю", "समझ नहीं", "不懂", …)
// appears in `normalized`. Plain substring matching mirrors
// detect_comprehension_failure in src/translation/language_markers.rs, so a
// CJK marker with no word spaces still matches inside a longer sentence.
function detectComprehensionFailure(normalized) {
  const text = String(normalized || "").toLowerCase();
  return meaningsWithRole(ROLE_COMPREHENSION_FAILURE_MARKER).some((meaning) =>
    meaning.words.some((word) => text.includes(word)),
  );
}

let cachedTranslationMarkers = null;
// Cache the role/slot/script projection mirrored by translation/prompt.rs.
function translationMarkers() {
  if (cachedTranslationMarkers) return cachedTranslationMarkers;
  const scriptForms = (role, script) =>
    roleWordForms(role)
      .filter((form) => script(form.text))
      .map((form) => form.text);
  const bareScriptForms = (role, script) =>
    roleWordForms(role)
      .filter((form) => form.slot === "bare" && script(form.text))
      .map((form) => form.before);
  cachedTranslationMarkers = {
    circumfixFrames: roleWordForms(ROLE_TRANSLATION_UNQUOTED_FRAME)
      .filter((form) => form.slot === "circumfix")
      .map((form) => [form.before, form.after]),
    suffixFrames: roleWordForms(ROLE_TRANSLATION_UNQUOTED_FRAME).filter((form) => form.slot === "suffix").map((form) => form.after),
    hindiVerbStems: bareScriptForms(
      ROLE_TRANSLATION_UNQUOTED_FRAME,
      containsDevanagari,
    ),
    hindiTargetMarkers: scriptForms(
      ROLE_TRANSLATION_INTO_MARKER,
      containsDevanagari,
    ),
    hindiObjectMarkers: scriptForms(
      ROLE_TRANSLATION_OBJECT_MARKER,
      containsDevanagari,
    ),
    chineseCommandPrefixes: scriptForms(
      ROLE_TRANSLATION_OBJECT_MARKER,
      containsCjk,
    ),
    chineseCommandMarkers: scriptForms(
      ROLE_TRANSLATION_INTO_MARKER,
      containsCjk,
    ),
    chineseTranslatePrefixes: bareScriptForms(
      ROLE_TRANSLATION_UNQUOTED_FRAME,
      containsCjk,
    ),
    chineseTargetMarkers: scriptForms(
      ROLE_TRANSLATION_TARGET_DIRECTION,
      containsCjk,
    ),
  };
  return cachedTranslationMarkers;
}

// Issue #216: extract the surface from unquoted translation prompts such as
// `translate apple to russian`, `переведи яблоко на английский`,
// `apple का हिंदी में अनुवाद करो`, or `把 apple 翻译成中文`. Returns null when
// the prompt already contains a quoted fragment or does not match a supported
// verb + target-marker pattern. Issue #386: every marker is now projected from
// the lexicon by role/slot/script — translationMarkers() above — so this code
// names the *shape* of each frame, never its words.
function extractUnquotedTranslationSurface(text) {
  const source = String(text || "").trim();
  const trimmed = source.replace(/[.!?。]+$/u, "");
  const lower = trimmed.toLowerCase();
  const markers = translationMarkers();

  for (const [prefix, marker] of markers.circumfixFrames) {
    const extracted = extractBetweenPrefixAndMarker(
      trimmed,
      lower,
      prefix,
      marker,
    );
    if (extracted) return extracted;
  }

  const hindi = extractHindiUnquotedTranslationSurface(trimmed, lower);
  if (hindi) return hindi;
  const chinese = extractChineseUnquotedTranslationSurface(trimmed, lower);
  if (chinese) return chinese;
  const suffix = markers.suffixFrames.map((frame) => [frame, lower.lastIndexOf(frame)])
    .find(([frame, index]) => index !== -1 && lower.slice(index + frame.length).trim());
  return suffix ? cleanUnquotedTranslationSurface(trimmed.slice(0, suffix[1])) : null;
}

// The surface that sits between a circumfix frame's prefix and its trailing
// marker (e.g. "translate " … " to "). Mirrors extract_between_prefix_and_marker
// in src/translation/prompt.rs.
function extractBetweenPrefixAndMarker(original, lower, prefix, marker) {
  if (!lower.startsWith(prefix)) return null;
  const afterPrefix = lower.slice(prefix.length);
  const markerIndex = afterPrefix.indexOf(marker);
  if (markerIndex === -1) return null;
  return cleanUnquotedTranslationSurface(
    original.slice(prefix.length, prefix.length + markerIndex),
  );
}

function cleanUnquotedTranslationSurface(candidate) {
  const cleaned = String(candidate || "").trim().replace(/[-–—:]\s*$/u, "").trim();
  if (!cleaned || /["'«»`“”‘’]/u.test(cleaned)) return null;
  return cleaned;
}

// Head-final Hindi: "<surface> <object-marker> <target> में अनुवाद". Gated on a
// Devanagari translate stem (अनुवाद); the target markers and object markers are
// the Devanagari forms of the into-marker and object-marker roles. Mirrors
// extract_hindi_unquoted_surface in src/translation/prompt.rs.
function extractHindiUnquotedTranslationSurface(original, lower) {
  const markers = translationMarkers();
  if (!markers.hindiVerbStems.some((stem) => lower.includes(stem))) return null;
  for (const targetMarker of markers.hindiTargetMarkers) {
    const targetIndex = lower.indexOf(targetMarker);
    if (targetIndex === -1) continue;
    const beforeTarget = lower.slice(0, targetIndex);
    for (const surfaceMarker of markers.hindiObjectMarkers) {
      const surfaceEnd = beforeTarget.lastIndexOf(surfaceMarker);
      if (surfaceEnd !== -1) {
        return cleanUnquotedTranslationSurface(original.slice(0, surfaceEnd));
      }
    }
  }
  return null;
}

function firstMarkerOffset(text, markers) {
  let best = null;
  for (const marker of markers) {
    const offset = text.indexOf(marker);
    if (offset !== -1 && (best === null || offset < best)) best = offset;
  }
  return best;
}

// Head-initial Chinese: a command prefix (把/将) + command marker (翻译成 …), or a
// bare translate stem (翻译/翻譯) + target marker (成/为/到). Both prefix sets and
// marker sets are the Han forms of the object-marker / into-marker / unquoted-
// frame / target-direction roles. Mirrors extract_chinese_unquoted_surface in
// src/translation/prompt.rs.
function extractChineseUnquotedTranslationSurface(original, lower) {
  const markers = translationMarkers();
  for (const prefix of markers.chineseCommandPrefixes) {
    if (!lower.startsWith(prefix)) continue;
    const rest = lower.slice(prefix.length);
    const markerIndex = firstMarkerOffset(rest, markers.chineseCommandMarkers);
    if (markerIndex !== null) {
      return cleanUnquotedTranslationSurface(
        original.slice(prefix.length, prefix.length + markerIndex),
      );
    }
  }

  for (const prefix of markers.chineseTranslatePrefixes) {
    if (!lower.startsWith(prefix)) continue;
    const rest = lower.slice(prefix.length);
    const markerIndex = firstMarkerOffset(rest, markers.chineseTargetMarkers);
    if (markerIndex !== null) {
      return cleanUnquotedTranslationSurface(
        original.slice(prefix.length, prefix.length + markerIndex),
      );
    }
  }
  return null;
}

function escapeBehaviorRuleValue(value) {
  return String(value || "")
    .replaceAll("\\", "\\\\")
    .replaceAll('"', '\\"')
    .replaceAll("\n", "\\n");
}

// R1188-U1: the built-in behavior rules, in listing order (consecutive rules
// share a topic group). Mirrors BEHAVIOR_RULES in
// rust/src/solver_handlers/behavior_rules.rs. `intent` keys both the seeded
// response a rule answers with and the `behavior_rule_<field>_<intent>` texts
// that describe it (data/seed/multilingual-responses-behavior-rules.lino);
// `topic` keys its `behavior_rule_topic_<topic>` heading. No prose or example
// prompt lives here.
const SEEDED_RESPONSES_SOURCE = "data/seed/intent-routing.lino + multilingual responses";
const BEHAVIOR_RULES = [
  { id: "rule_greeting", topic: "greetings", intent: "greeting", source: SEEDED_RESPONSES_SOURCE },
  { id: "rule_farewell", topic: "farewells", intent: "farewell", source: SEEDED_RESPONSES_SOURCE },
  { id: "rule_assistant_free_time", topic: "small_talk", intent: "assistant_free_time", source: SEEDED_RESPONSES_SOURCE },
  { id: "rule_identity", topic: "identity", intent: "identity", source: "data/seed/identity.lino + multilingual responses" },
  { id: "rule_assistant_name", topic: "assistant_name", intent: "assistant_name", source: "data/seed/intent-routing.lino + browser preferences" },
  { id: "rule_capabilities", topic: "capabilities", intent: "capabilities", source: "src/solver_handlers/user_intent.rs" },
  { id: "rule_write_program", topic: "write_program", intent: "write_program", source: "data/seed/hello-world-programs.lino + src/coding/catalog/" },
  { id: "rule_unknown", topic: "unknown_fallback", intent: "unknown", source: "data/seed/multilingual-responses.lino" },
];

// Mirrors `fn catalog_text`: the seeded `behavior_rule_<name>` text in
// `language`, its `{slots}` filled in one pass.
function behaviorRuleText(name, language, values = {}) {
  return String(answerFor(`behavior_rule_${name}`, language)).replace(/\{([^{}]*)\}/gu, (whole, slot) =>
    Object.prototype.hasOwnProperty.call(values, slot) ? String(values[slot]) : whole);
}

// Mirrors `fn rule_text`.
function behaviorRuleField(field, rule, language) {
  return behaviorRuleText(`${field}_${rule.intent}`, language, {
    languages: Object.keys(WRITE_PROGRAM_LANGUAGES).join(", "),
    tasks: Object.keys(WRITE_PROGRAM_TASKS).join(", "),
  });
}

function behaviorRuleRecords() {
  return BEHAVIOR_RULES.map((rule) => ({ ...rule, label: behaviorRuleField("label", rule, "en") }));
}

function behaviorRuleCounts(runtimeRules) {
  const runtime = Array.isArray(runtimeRules) ? runtimeRules.length : 0;
  const builtIn = behaviorRuleRecords().length;
  return { builtIn, runtime, total: builtIn + runtime };
}

function renderBehaviorRuleCount(runtimeRules, language = "en") {
  const { builtIn, runtime, total } = behaviorRuleCounts(runtimeRules);
  return [
    behaviorRuleText("count_summary", language, { total, built_in: builtIn, runtime }),
    "",
    behaviorRuleText("count_reasoning", language),
    "",
    "```links",
    "behavior_rules_count",
    `  built_in_rules "${builtIn}"`,
    `  dialog_local_rules "${runtime}"`,
    `  total_rules "${total}"`,
    '  algorithm "behavior_rule_records + collect_runtime_rules(prior_turn:user)"',
    "```",
  ].join("\n");
}

function renderBehaviorRulesBrief(runtimeRules, language = "en") {
  const { builtIn, runtime, total } = behaviorRuleCounts(runtimeRules);
  const groups = behaviorRuleText("brief_groups", language);
  return behaviorRuleText("brief", language, { total, built_in: builtIn, runtime, groups });
}

// Mirrors `fn rule_response`: the rule's own seeded response, or the seeded
// description of an answer that is no single response.
function localizedRuleResponse(rule, language) {
  const described = `behavior_rule_response_${rule.intent}`;
  return answerFor(MULTILINGUAL_ANSWERS[described] ? described : rule.intent, language);
}

function localizedRuleWhenThen(rule, language) {
  return behaviorRuleText(`when_then_${rule.intent}`, language, { response: localizedRuleResponse(rule, language) });
}

function runtimeRuleWhenThen(rule, language) {
  return behaviorRuleText("runtime_when_then", language, { trigger: rule.trigger, response: rule.answer });
}

function renderBehaviorRuleList(runtimeRules, language = "en") {
  const lines = [behaviorRuleText("list_intro", language), ""];
  let previousTopic = null;
  for (const rule of BEHAVIOR_RULES) {
    if (rule.topic !== previousTopic) {
      if (previousTopic !== null) lines.push("");
      lines.push(`### ${behaviorRuleText(`topic_${rule.topic}`, language)}`);
      previousTopic = rule.topic;
    }
    lines.push(`- \`${rule.id}\` -> ${localizedRuleWhenThen(rule, language)}`);
  }
  if (Array.isArray(runtimeRules) && runtimeRules.length > 0) {
    lines.push("", `### ${behaviorRuleText("runtime_heading", language)}`);
    for (const rule of runtimeRules) {
      lines.push(`- \`${rule.id}\` -> ${runtimeRuleWhenThen(rule, language)}`);
    }
  }
  lines.push("", ...["read", "teach", "forms", "append"].map((part) => behaviorRuleText(`list_footer_${part}`, language)));
  return lines.join("\n");
}

function renderBehaviorRuleDetail(rule, language = "en") {
  const whenThen = localizedRuleWhenThen(rule, language);
  return [
    behaviorRuleField("label", rule, language),
    "",
    whenThen,
    "",
    "```links",
    rule.id,
    `  topic "${escapeBehaviorRuleValue(rule.topic)}"`,
    `  intent "${escapeBehaviorRuleValue(rule.intent)}"`,
    `  matches "${escapeBehaviorRuleValue(behaviorRuleField("matches", rule, language))}"`,
    `  response "${escapeBehaviorRuleValue(localizedRuleResponse(rule, language))}"`,
    `  source "${escapeBehaviorRuleValue(rule.source)}"`,
    `  when_then "${escapeBehaviorRuleValue(whenThen)}"`,
    "```",
    "",
    behaviorRuleText("change_hint", language),
  ].join("\n");
}

function assistantNameStatus(preferences) {
  const name = normalizeAssistantNamePreference(
    preferences && preferences.assistantName,
  );
  return name ? `configured:${name}` : "browser_preference_when_set_else_not_configured";
}

const BROWSER_SURFACE = {
  slug: "browser",
  label: "browser demo with JavaScript and WebAssembly worker",
  runtime: "JavaScript UI plus a WebAssembly worker mirror of the solver",
  memory: "browser IndexedDB/local storage plus worker state and imported memory",
  webSearch: "available through browser CORS-readable providers when online and not blocked",
  limits: "browser settings, import/export controls, and IndexedDB-backed memory belong to this surface",
};

function modeStatus(enabled) {
  return enabled ? "enabled" : "disabled";
}

function definitionFusionStatus(preferences) {
  return preferences && preferences.definitionFusion === "auto"
    ? "enabled_by_default"
    : "explicit_only";
}

function blueprintCompositionStatus(preferences) {
  return normalizeBlueprintComposition(
    preferences && preferences.blueprintComposition,
  );
}

function renderSelfFacts(preferences) {
  const assistantName = assistantNameStatus(preferences);
  const surface = BROWSER_SURFACE;
  return [
    "Facts I know about myself in this environment:",
    "",
    `- **Execution surface**: ${surface.label} (\`${surface.slug}\`).`,
    `- **Runtime**: ${surface.runtime}.`,
    `- **Memory**: ${surface.memory}.`,
    `- **Web search**: ${surface.webSearch}.`,
    `- **Surface limits**: ${surface.limits}.`,
    "- **Local rules**: local links rules and seed facts are checked first.",
    "",
    "```links",
    "self_fact_model",
    '  subject "formal-ai"',
    '  relation "model"',
    `  object "${escapeBehaviorRuleValue(AGENT_INFO.model || "formal-ai")}"`,
    "self_fact_policy",
    '  subject "formal-ai"',
    '  relation "policy"',
    '  object "deterministic symbolic AI; no neural network inference"',
    "self_fact_environment",
    '  subject "formal-ai"',
    '  relation "execution_surface"',
    `  object "${surface.slug}"`,
    "self_fact_runtime",
    '  subject "formal-ai"',
    '  relation "runtime"',
    `  object "${escapeBehaviorRuleValue(surface.runtime)}"`,
    "self_fact_memory",
    '  subject "formal-ai"',
    '  relation "memory"',
    `  object "${escapeBehaviorRuleValue(surface.memory)}"`,
    "self_fact_web_search",
    '  subject "formal-ai"',
    '  relation "web_search"',
    `  object "${escapeBehaviorRuleValue(surface.webSearch)}"`,
    "self_fact_assistant_name",
    '  subject "formal-ai"',
    '  relation "assistant_name"',
    `  object "${escapeBehaviorRuleValue(assistantName)}"`,
    "self_fact_agent_mode",
    '  subject "formal-ai"',
    '  relation "agent_mode"',
    `  object "${modeStatus(preferences && preferences.agentMode)}"`,
    "self_fact_diagnostics",
    '  subject "formal-ai"',
    '  relation "diagnostic_mode"',
    `  object "${modeStatus(preferences && preferences.diagnosticsMode)}"`,
    "self_fact_definition_fusion",
    '  subject "formal-ai"',
    '  relation "definition_fusion"',
    `  object "${definitionFusionStatus(preferences)}"`,
    "self_fact_blueprint_composition",
    '  subject "formal-ai"',
    '  relation "blueprint_composition"',
    `  object "${blueprintCompositionStatus(preferences)}"`,
    "```",
    "",
    "Read behavior with `List behavior rules`; teach one with When `prompt` then `answer` (or When I say `prompt`, answer `answer`).",
  ].join("\n");
}

function renderKnownFacts(language, preferences) {
  const surface = BROWSER_SURFACE;
  const assistantName = assistantNameStatus(preferences);
  const links = [
    "```links",
    "known_fact_local_seed",
    '  source "local_links_notation_seed"',
    '  scope "built-in rules, concepts, facts, tools, and response templates"',
    "known_fact_internet",
    '  source "environment_aware_web_search"',
    `  scope "${escapeBehaviorRuleValue(surface.webSearch)}"`,
    "known_fact_memory",
    '  source "conversation_memory"',
    `  scope "${escapeBehaviorRuleValue(surface.memory)}"`,
    "known_fact_environment",
    '  subject "formal-ai"',
    '  relation "execution_surface"',
    `  object "${surface.slug}"`,
    "known_fact_self",
    '  subject "formal-ai"',
    '  relation "model"',
    `  object "${escapeBehaviorRuleValue(AGENT_INFO.model || "formal-ai")}"`,
    "known_fact_assistant_name",
    '  subject "formal-ai"',
    '  relation "assistant_name_setting"',
    `  object "${escapeBehaviorRuleValue(assistantName)}"`,
    "known_fact_surface_limits",
    '  source "environment_directory"',
    `  scope "${escapeBehaviorRuleValue(surface.limits)}"`,
    "```",
  ].join("\n");
  if (language === "ru") {
    return [
      `Я могу использовать несколько классов фактов в текущей среде \`${surface.slug}\`:`,
      "",
      "- **Локальные факты и правила**: встроенный seed Links Notation, включая правила, понятия, инструменты и ответы.",
      `- **Интернет**: ${surface.webSearch}; это не означает, что весь интернет предзагружен в локальную память.`,
      `- **Память диалога**: ${surface.memory}.`,
      "- **Факты о себе**: модель `formal-ai`, политика исполнения, поверхность и источники ответов.",
      `- **Ограничения среды**: ${surface.limits}.`,
      "",
      links,
      "",
      "Для конкретного факта задайте прямой вопрос; порядок проверки: локальные правила, память, затем веб-поиск, если он доступен в этой среде.",
    ].join("\n");
  }
  if (language === "hi") {
    return [
      `मैं current \`${surface.slug}\` environment में इन fact sources का उपयोग कर सकता हूँ:`,
      "",
      "- **Local facts and rules**: Links Notation seed में rules, concepts, tools और response templates.",
      `- **Internet**: ${surface.webSearch}; पूरा internet local memory में preload नहीं है.`,
      `- **Conversation memory**: ${surface.memory}.`,
      "- **Self facts**: model `formal-ai`, execution surface और answer sources.",
      `- **Surface limits**: ${surface.limits}.`,
      "",
      links,
      "",
      "किसी खास fact के लिए सीधे पूछें; मैं local rules और memory पहले देखता हूँ, फिर environment अनुमति दे तो web search इस्तेमाल करता हूँ.",
    ].join("\n");
  }
  if (language === "zh") {
    return [
      `在当前 \`${surface.slug}\` 环境中, 我可以使用这些事实来源:`,
      "",
      "- **本地事实和规则**: Links Notation seed 中的规则、概念、工具和回复模板。",
      `- **Internet**: ${surface.webSearch}; 整个互联网不会预加载到本地记忆中。`,
      `- **Conversation memory**: ${surface.memory}。`,
      "- **Self facts**: model `formal-ai`, execution surface 和 answer sources。",
      `- **Surface limits**: ${surface.limits}。`,
      "",
      links,
      "",
      "如果需要某个具体事实, 请直接提问; 我会先检查本地规则和记忆, 环境允许时再使用 web search。",
    ].join("\n");
  }
  return [
    `I can use several classes of facts in the current \`${surface.slug}\` environment:`,
    "",
    "- **Local facts and rules**: built-in Links Notation seed data, including rules, concepts, tools, and response templates.",
    `- **Internet**: ${surface.webSearch}; the whole internet is not preloaded into local memory.`,
    `- **Conversation memory**: ${surface.memory}.`,
    "- **Self facts**: model `formal-ai`, execution policy, active surface, and answer sources.",
    `- **Surface limits**: ${surface.limits}.`,
    "",
    links,
    "",
    "Ask for a specific fact directly; I check local rules and memory first, then use web search only when this environment allows it.",
  ].join("\n");
}

function renderRuntimeRuleUpdate(rule, language = "en") {
  const whenThenText = runtimeRuleWhenThen(rule, language);
  const title = behaviorRuleText("update_title", language);
  const sendHint = behaviorRuleText("update_send_hint", language, { trigger: rule.trigger });
  return [
    title,
    "",
    whenThenText,
    "",
    "```links",
    rule.id,
    '  type "behavior_rule_runtime"',
    `  match_prompt "${escapeBehaviorRuleValue(rule.trigger)}"`,
    `  answer "${escapeBehaviorRuleValue(rule.answer)}"`,
    `  when_then "${escapeBehaviorRuleValue(whenThenText)}"`,
    '  source "user_message"',
    "```",
    "",
    sendHint,
  ].join("\n");
}

// Issue #386: recognise a request to list the assistant's behavior rules by
// *meaning*, not a hardcoded per-language phrase list. The standalone phrases
// (role rule_listing_phrase) and the three compositional dimensions
// (rule_listing_subject / rule_listing_request / rule_listing_scope) live in
// data/seed/meanings-behavior-rules.lino. Mirror of is_behavior_rules_list in
// src/solver_handlers/behavior_rules.rs.
function isBehaviorRulesList(normalized) {
  return (
    matchesBehaviorRulesListSeedPattern(normalized) ||
    lexiconMentionsRoleSubstring(ROLE_RULE_LISTING_PHRASE, normalized) ||
    isSupportedLanguageBehaviorRulesListQuery(normalized)
  );
}

function isBehaviorRulesCountQuery(normalized, history) {
  const hasPriorRuleList = previousAssistantIsBehaviorRuleList(history);
  const hasPhraseScope = lexiconMentionsRoleSubstring(
    ROLE_RULE_LISTING_PHRASE,
    normalized,
  );
  const present = (role, language) =>
    wordsForRoleInLanguages(role, [language]).some((word) =>
      normalized.includes(word),
    );
  return ["en", "ru", "hi", "zh"].some(
    (language) =>
      present(ROLE_RULE_COUNT_REQUEST, language) &&
      present(ROLE_RULE_LISTING_SUBJECT, language) &&
      (hasPriorRuleList ||
        hasPhraseScope ||
        present(ROLE_RULE_COUNT_SCOPE, language) ||
        present(ROLE_RULE_LISTING_SCOPE, language)),
  );
}

function isBehaviorRulesBriefFollowup(normalized, history) {
  if (!previousAssistantIsBehaviorRuleList(history)) return false;
  return ["en", "ru", "hi", "zh"].some((language) =>
    wordsForRoleInLanguages(ROLE_RULE_BRIEF_REQUEST, [language]).some((word) =>
      normalized.includes(word),
    ),
  );
}

function previousAssistantIsBehaviorRuleList(history) {
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index] || {};
    if (String(turn.role || "").toLowerCase() !== "assistant") continue;
    const payload = String(turn.content || "").toLowerCase();
    return (
      payload.includes("rule_greeting") &&
      payload.includes("rule_write_program") &&
      payload.includes("rule_unknown")
    );
  }
  return false;
}

function behaviorRuleResponseLanguage(normalized, detectedLanguage) {
  const lower = String(normalized || "").toLowerCase();
  for (const { marker, language } of conceptResponseLanguageMarkers()) {
    if (lower.includes(marker)) return language;
  }
  return detectedLanguage || "en";
}

function matchesBehaviorRulesListSeedPattern(normalized) {
  return PROMPT_PATTERNS.some((pattern) => {
    if (!pattern || pattern.intent !== "behavior_rules_list" || !pattern.text) {
      return false;
    }
    const text = normalizePrompt(pattern.text);
    if (!text) return false;
    switch (pattern.kind) {
      case "keyword":
      case "phrase":
        return normalized === text || normalized.includes(text);
      case "prefix":
        return normalized.startsWith(text);
      case "suffix":
        return normalized.endsWith(text);
      default:
        return false;
    }
  });
}

// True when the prompt, within one supported language's vocabulary, names the
// rule subject, asks to enumerate it, and scopes the request to the assistant's
// own behavior. The three dimensions are read from the meaning lexicon
// (rule_listing_subject / rule_listing_request / rule_listing_scope) rather than
// hardcoded per-language word lists. The per-language AND is preserved: every
// dimension must be evidenced within the SAME language (wordsForRoleInLanguages),
// matched as a raw substring to keep the legacy stem match byte-for-byte. Mirror
// of is_supported_language_behavior_rules_list_query in
// src/solver_handlers/behavior_rules.rs.
function isSupportedLanguageBehaviorRulesListQuery(normalized) {
  const present = (role, language) =>
    wordsForRoleInLanguages(role, [language]).some((word) =>
      normalized.includes(word),
    );
  return ["en", "ru", "hi", "zh"].some(
    (language) =>
      present(ROLE_RULE_LISTING_SUBJECT, language) &&
      present(ROLE_RULE_LISTING_REQUEST, language) &&
      present(ROLE_RULE_LISTING_SCOPE, language),
  );
}

// Issue #386: recognise a request to list the assistant's own facts by
// *meaning*, not a hardcoded per-language phrase list. The self_fact_query role
// gathers every surface from data/seed/meanings-intent.lino; mirror of
// is_self_fact_query in src/solver_handlers/self_awareness.rs. The prompt is
// re-normalized first because some call sites pass a merely-lowercased string
// (trailing "?" intact) and the boundary-aware matcher expects punctuation
// already collapsed to spaces.
function isSelfFactQuery(normalized) {
  return lexiconMentionsRole(ROLE_SELF_FACT_QUERY, normalizePrompt(normalized));
}

// Issue #386: recognise "introduce yourself" / "расскажи о себе" /
// "अपना परिचय दो" / "介绍一下你自己" by the self_introduction_request meaning
// role. The pre-check is preserved verbatim: an empty prompt, or one that is
// really a self-fact query, must not be treated as an introduction request, so
// "list all facts you know about yourself" still routes to the self-fact
// branch. Mirror of is_self_introduction_query in
// src/solver_handlers/self_awareness.rs.
function isSelfIntroductionQuery(normalized) {
  const cleaned = normalizePrompt(normalized);
  if (!cleaned || isSelfFactQuery(cleaned)) return false;
  return lexiconMentionsRole(ROLE_SELF_INTRODUCTION_REQUEST, cleaned);
}

function selfAwarenessLanguage(prompt, normalized) {
  // Issue #386: language is detected purely by Unicode script ranges. The
  // Cyrillic range below already subsumes the former second-person pronoun
  // list (ty/tebya/tvoy/vy/...), every member of which is Cyrillic, so no raw
  // word list is needed -- the script range is the universal signal. Mirror of
  // self_awareness_language in src/solver_handlers/self_awareness.rs.
  const text = `${String(prompt || "").toLowerCase()} ${String(normalized || "")}`;
  if (/[\u0400-\u04ff]/u.test(text)) return "ru";
  if (/[\u0900-\u097f]/u.test(text)) return "hi";
  if (/[\u4e00-\u9fff]/u.test(text)) return "zh";
  return detectLanguage(prompt);
}

function selfIntroductionContent(language, preferences) {
  const identity = answerFor("identity", language);
  const name = normalizeAssistantNamePreference(
    preferences && preferences.assistantName,
  );
  if (!name) return identity;
  if (language === "ru") return `Меня зовут ${name}. ${identity}`;
  if (language === "hi") return `मेरा नाम ${name} है। ${identity}`;
  if (language === "zh") return `我的名字是 ${name}。${identity}`;
  return `My name is ${name}. ${identity}`;
}
