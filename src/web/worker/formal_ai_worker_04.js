// Worker module 5 of 21. Loaded by ../formal_ai_worker.js.
function conversationTopic(prompt, normalized) {
  // Recognized surfaces — the let-us-talk-about-X phrasings in every supported
  // language — carry the conversation_topic_opener role; each is a prefix whose
  // text before the … slot is the matchable opener, in declaration order. A
  // form whose action is "scan" is also matched anywhere in the prompt, not only
  // at the start, so an opener that follows a greeting is still found. No
  // per-language opener list lives here — only the concept. Mirrors
  // conversation_topic in src/solver_handlers/benchmark_prompts.rs (issue #386).
  const forms = roleWordForms(ROLE_CONVERSATION_TOPIC_OPENER);
  for (const form of forms) {
    if (normalized.startsWith(form.before)) {
      return cleanConversationTopic(normalized.slice(form.before.length));
    }
  }
  const lower = String(prompt || "").toLowerCase();
  for (const form of forms) {
    if (form.action !== "scan") continue;
    const index = lower.indexOf(form.before);
    if (index >= 0) {
      return cleanConversationTopic(lower.slice(index + form.before.length));
    }
  }
  return "";
}

function conversationTopicContent(topic, language) {
  if (language === "ru") {
    return `Можем. Тема: ${topic}. Я могу начать с краткого определения, контекста или конкретного вопроса; если веб-поиск доступен, публичные факты можно уточнить через внешний источник.`;
  }
  if (language === "hi") {
    return `हम बात कर सकते हैं. विषय: ${topic}. मैं छोटी परिभाषा, संदर्भ, या किसी конкрет प्रश्न से शुरू कर सकता हूँ; web search उपलब्ध हो तो public facts बाहरी स्रोत से जाँचे जा सकते हैं.`;
  }
  if (language === "zh") {
    return `可以聊。主题: ${topic}。我可以从简短定义、上下文或具体问题开始; 如果 web search 可用, 公开事实可以通过外部来源核对。`;
  }
  return `We can talk about ${topic}. I can start with a short definition, context, or a specific question; when web search is available, public facts can be checked against an external source.`;
}

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

function detailQuery(prompt) {
  const lower = String(prompt || "").toLowerCase();
  const prefixes = [
    "show behavior rule",
    "read behavior rule",
    "describe behavior rule",
    "show rule",
    "read rule",
    "details for rule",
    "детали правила",
    "покажи правило",
    "прочитай правило",
  ];
  for (const prefix of prefixes) {
    if (lower.startsWith(prefix)) {
      return cleanRuleQuery(String(prompt || "").slice(prefix.length));
    }
  }
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

function codeSpans(text) {
  return String(text || "")
    .split("`")
    .map((part, index) => (index % 2 === 1 ? part.trim() : ""))
    .filter(Boolean);
}

// Issue #144 / #386: recognize behavior-rule updates expressed as `When X then
// Y` (and translations) in addition to the explicit `When I say … answer …`
// grammar. No keyword is named here any more — every surface lives in the
// embedded meaning lexicon (data/seed/meanings-skill-compiler.lino) and is read
// by semantic role, mirroring explicit_teaching_form + looks_like_skill_description
// in src/skill_compiler.rs:
//   * a teaching trigger lead that co-occurs with a teaching response verb, or a
//     standalone behaviour-rule edit directive (the explicit teaching form); and
//   * a when-then frame whose circumfix surface brackets the trigger and answer —
//     the literal before the … (U+2026) is the head, the literal after it is the
//     link; both must appear, head before link, with at least one backtick on
//     each side so the runtime extractor can pull the trigger and answer
//     deterministically.
const ROLE_SKILL_TEACHING_TRIGGER_LEAD = "skill_teaching_trigger_lead";
const ROLE_SKILL_TEACHING_RESPONSE_VERB = "skill_teaching_response_verb";
const ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE = "behavior_rule_edit_directive";
const ROLE_SKILL_WHEN_THEN_PAIR = "skill_when_then_pair";

function looksLikeRuntimeRuleUpdate(text) {
  const raw = String(text || "");
  const lower = raw.toLowerCase();
  if (
    (lexiconMentionsRoleSubstring(ROLE_SKILL_TEACHING_TRIGGER_LEAD, lower) &&
      lexiconMentionsRoleSubstring(ROLE_SKILL_TEACHING_RESPONSE_VERB, lower)) ||
    lexiconMentionsRoleSubstring(ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE, lower)
  ) {
    return true;
  }
  for (const form of roleWordForms(ROLE_SKILL_WHEN_THEN_PAIR)) {
    if (form.slot !== "circumfix") continue;
    const head = form.before;
    const link = form.after;
    const headPos = lower.indexOf(head);
    if (headPos === -1) continue;
    const tail = lower.slice(headPos + head.length);
    const linkPos = tail.indexOf(link);
    if (linkPos === -1) continue;
    const absoluteLinkPos = headPos + head.length + linkPos;
    const beforeLink = raw.slice(headPos, absoluteLinkPos);
    const afterLink = raw.slice(absoluteLinkPos + link.length);
    if (beforeLink.includes("`") && afterLink.includes("`")) return true;
  }
  return false;
}

function runtimeRuleFromText(text) {
  if (!looksLikeRuntimeRuleUpdate(text)) return null;
  const spans = codeSpans(text);
  if (spans.length < 2) return null;
  const trigger = spans[0].trim();
  const answer = spans[1].trim();
  if (!trigger || !answer) return null;
  return {
    id: stableBehaviorRuleId("behavior_rule_runtime", `${trigger}\n${answer}`),
    trigger,
    answer,
  };
}

function runtimeRuleForPrompt(prompt, history) {
  const normalizedPrompt = normalizePrompt(prompt);
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index] || {};
    if (String(turn.role || "").toLowerCase() !== "user") continue;
    const rule = runtimeRuleFromText(turn.content);
    if (rule && normalizePrompt(rule.trigger) === normalizedPrompt) {
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
  const updateRule = runtimeRuleFromText(prompt);
  if (updateRule) {
    return {
      intent: "behavior_rule_update",
      content: renderRuntimeRuleUpdate(updateRule, language),
      confidence: 1.0,
      evidence: ["behavior_rule:update", updateRule.id],
    };
  }

  if (isBehaviorRulesCountQuery(normalized, history)) {
    const runtimeRules = collectRuntimeRules(history);
    const counts = behaviorRuleCounts(runtimeRules);
    return {
      intent: "behavior_rules_count",
      content: renderBehaviorRuleCount(runtimeRules, language),
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

  const topic = conversationTopic(prompt, normalized);
  if (topic) {
    const language = selfAwarenessLanguage(prompt, normalized);
    return {
      intent: "conversation_topic",
      content: conversationTopicContent(topic, language),
      confidence: 0.75,
      evidence: [`conversation_topic:${topic}`, `language:${language}`],
    };
  }

  const runtimeRule = runtimeRuleForPrompt(prompt, history);
  if (runtimeRule) {
    return {
      intent: "behavior_rule_custom",
      content: runtimeRule.answer,
      confidence: 1.0,
      evidence: ["behavior_rule:match", runtimeRule.id],
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

const FEATURE_CAPABILITIES = [
  {
    slug: "web_search",
    state: "web_search",
    labels: { en: "web search", ru: "веб-поиск", hi: "web search", zh: "web search" },
    examples: {
      en: "Search the web for Nikola Tesla",
      ru: "Найди в интернете Никола Тесла",
      hi: "Search the web for Nikola Tesla",
      zh: "Search the web for Nikola Tesla",
    },
  },
  {
    slug: "diagnostics",
    state: "diagnostics",
    labels: { en: "diagnostics", ru: "диагностика", hi: "diagnostics", zh: "诊断" },
    examples: {
      en: "Turn on diagnostics",
      ru: "Включи диагностику",
      hi: "Turn on diagnostics",
      zh: "开启诊断",
    },
  },
  {
    slug: "agent_mode",
    state: "agent_mode",
    labels: { en: "agent mode", ru: "agent mode", hi: "agent mode", zh: "agent mode" },
    examples: {
      en: "Turn on agent mode",
      ru: "Включи agent mode",
      hi: "Turn on agent mode",
      zh: "开启 agent mode",
    },
  },
  {
    slug: "definition_fusion",
    state: "definition_fusion",
    labels: {
      en: "automatic definition fusion",
      ru: "автоматическое слияние определений",
      hi: "automatic definition fusion",
      zh: "自动 definition fusion",
    },
    examples: {
      en: "Turn on definition fusion",
      ru: "Включи слияние определений",
      hi: "Turn on definition fusion",
      zh: "开启 definition fusion",
    },
  },
  {
    slug: "configuration",
    state: "always",
    labels: {
      en: "message-driven configuration",
      ru: "настройка через сообщения",
      hi: "message-driven configuration",
      zh: "消息驱动设置",
    },
    examples: {
      en: "Switch to dark theme",
      ru: "Переключи тему на темную",
      hi: "Switch to dark theme",
      zh: "切换到深色主题",
    },
  },
  {
    slug: "memory_actions",
    state: "always",
    labels: {
      en: "memory import/export",
      ru: "импорт и экспорт памяти",
      hi: "memory import/export",
      zh: "记忆导入/导出",
    },
    examples: {
      en: "Export memory",
      ru: "Экспортируй память",
      hi: "Export memory",
      zh: "导出记忆",
    },
  },
  {
    slug: "greeting",
    state: "always",
    labels: { en: "greetings", ru: "приветствия", hi: "अभिवादन", zh: "问候" },
    examples: { en: "Hello", ru: "Привет", hi: "नमस्ते", zh: "你好" },
  },
  {
    slug: "write_program",
    state: "always",
    labels: {
      en: "program template generation",
      ru: "генерация программ",
      hi: "program template generation",
      zh: "程序生成",
    },
    examples: {
      en: "Write a Python program that counts to three",
      ru: "Напиши hello world на Rust",
      hi: "Write a Python program that counts to three",
      zh: "Write a Python program that counts to three",
    },
  },
  {
    slug: "concept_lookup",
    state: "always",
    labels: { en: "concept lookup", ru: "поиск понятий", hi: "concept lookup", zh: "概念查找" },
    examples: {
      en: "What is Wikipedia?",
      ru: "Что такое Википедия?",
      hi: "विकिपीडिया क्या है?",
      zh: "什么是维基百科？",
    },
  },
  {
    slug: "arithmetic",
    state: "always",
    labels: { en: "arithmetic", ru: "арифметика", hi: "अंकगणित", zh: "算术" },
    examples: {
      en: "What is 2 + 2?",
      ru: "Сколько будет 2 + 2?",
      hi: "2 + 2 क्या है?",
      zh: "2 + 2 等于多少？",
    },
  },
  {
    slug: "translation",
    state: "always",
    labels: { en: "translation", ru: "перевод", hi: "अनुवाद", zh: "翻译" },
    examples: {
      en: 'Translate "hello" to Russian',
      ru: 'Переведи "hello" на русский',
      hi: 'Translate "hello" to Hindi',
      zh: 'Translate "hello" to Chinese',
    },
  },
  {
    slug: "memory",
    state: "always",
    labels: {
      en: "conversation memory",
      ru: "память разговора",
      hi: "conversation memory",
      zh: "会话记忆",
    },
    examples: {
      en: "My name is Ada. What is my name?",
      ru: "Меня зовут Ада. Как меня зовут?",
      hi: "My name is Ada. What is my name?",
      zh: "My name is Ada. What is my name?",
    },
  },
  {
    slug: "demo_mode",
    state: "always",
    labels: { en: "demo mode", ru: "демо-режим", hi: "demo mode", zh: "演示模式" },
    examples: { en: "Turn off demo mode", ru: "Выключи демо", hi: "Turn off demo mode", zh: "关闭演示" },
  },
  {
    slug: "http_url",
    state: "always",
    labels: {
      en: "URL fetch/navigation",
      ru: "HTTP-запросы и переходы по URL",
      hi: "URL fetch/navigation",
      zh: "URL fetch/navigation",
    },
    examples: {
      en: "Navigate to example.com",
      ru: "Перейди на example.com",
      hi: "Navigate to example.com",
      zh: "Navigate to example.com",
    },
  },
  {
    slug: "javascript_execution",
    state: "always",
    labels: {
      en: "JavaScript execution",
      ru: "выполнение JavaScript",
      hi: "JavaScript execution",
      zh: "JavaScript execution",
    },
    examples: {
      en: "Run JavaScript: 1 + 1",
      ru: "Выполни JavaScript: 1 + 1",
      hi: "Run JavaScript: 1 + 1",
      zh: "Run JavaScript: 1 + 1",
    },
  },
  {
    slug: "planning",
    state: "always",
    labels: {
      en: "summaries, brainstorming, roleplay, and project planning",
      ru: "резюме, брейншторминг, роли и планирование проектов",
      hi: "summaries, brainstorming, roleplay, and project planning",
      zh: "总结、头脑风暴、角色扮演和项目计划",
    },
    examples: {
      en: "Brainstorm 5 project ideas",
      ru: "Предложи 5 идей проекта",
      hi: "Brainstorm 5 project ideas",
      zh: "Brainstorm 5 project ideas",
    },
  },
];

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
  return FEATURE_CAPABILITIES.find((feature) => feature.slug === slug) || null;
}

// A prompt is a capability question when one of the `feature_capability_question`
// interrogative cues occurs as a raw substring, checked in the prompt's own
// detected language only. English prompts additionally accept a grammatical
// "is/are ... enabled/available" frame computed in code. Mirrors
// is_feature_capability_question in
// src/solver_handlers/feature_capability.rs (#386).
function isFeatureCapabilityQuestion(normalized, language) {
  const mentions = (lang) =>
    mentionsRoleInLanguagesRaw(ROLE_FEATURE_CAPABILITY_QUESTION, normalized, [lang]);
  if (language === "ru") return mentions("ru");
  if (language === "zh") return mentions("zh");
  if (language === "hi") return mentions("hi");
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

function webSearchStatusContent(language, available, providers) {
  const providerList = providers || "none";
  const rrfK = webSearchRrfK();
  if (language === "ru") {
    return available
      ? `Да. В этой конфигурации веб-поиск включен: я могу использовать DuckDuckGo Instant Answer по умолчанию и доступные CORS-провайдеры (\`${providerList}\`) для явных запросов вроде \`Найди в интернете Никола Тесла\`. Результаты из top-10 по каждому провайдеру объединяются через reciprocal rank fusion (k = ${rrfK}). Если провайдеры отключены или заблокированы в браузерной сессии, я сообщу об этом вместо ответа "да".`
      : "Нет. В этой браузерной сессии веб-поиск сейчас недоступен: браузер offline или все CORS-readable поисковые провайдеры отключены после ошибок. Я могу отвечать по локальным правилам и кэшу, но не буду обращаться к поисковым системам.";
  }
  if (language === "zh") {
    return available
      ? `可以。当前配置启用了 web search：我会默认使用 DuckDuckGo Instant Answer，并可使用这些 CORS-readable provider（\`${providerList}\`）处理明确的搜索请求，例如 \`Search the web for Nikola Tesla\`。每个 provider 的 top-10 结果会用 reciprocal rank fusion 合并（k = ${rrfK}）。如果浏览器会话中所有 provider 被禁用或阻止，我会说明不可用，而不是回答可以。`
      : "不可以。当前浏览器会话中 web search 不可用：浏览器 offline，或所有 CORS-readable 搜索 provider 都因错误被禁用。我仍可使用本地规则和缓存回答，但不会调用搜索引擎。";
  }
  if (language === "hi") {
    return available
      ? `हाँ। इस configuration में web search enabled है: मैं default रूप से DuckDuckGo Instant Answer और उपलब्ध CORS-readable providers (\`${providerList}\`) का उपयोग explicit prompts जैसे \`Search the web for Nikola Tesla\` के लिए कर सकता हूँ। हर provider के top-10 results reciprocal rank fusion (k = ${rrfK}) से merge होते हैं। अगर browser session में providers disabled या blocked हों, तो मैं "हाँ" कहने के बजाय स्थिति बताऊँगा।`
      : "नहीं। इस browser session में web search अभी available नहीं है: browser offline है या सभी CORS-readable search providers errors के बाद disabled हैं। मैं local rules और cache से जवाब दे सकता हूँ, लेकिन search engines को call नहीं करूँगा।";
  }
  return available
    ? `Yes. Web search is enabled in this configuration: I can use DuckDuckGo Instant Answer by default plus the configured CORS-readable providers (\`${providerList}\`) for explicit prompts such as \`Search the web for Nikola Tesla\`. The top-10 results from each provider are merged with reciprocal rank fusion (k = ${rrfK}). If the browser session disables or blocks every provider, I will say that instead of claiming search is available.`
    : "No. Web search is unavailable in this browser session: the browser is offline or every CORS-readable search provider has been disabled after errors. I can still answer from local rules and cache, but I will not call search engines.";
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
  if (feature.state === "diagnostics") {
    const available = Boolean(preferences && preferences.diagnosticsMode);
    return { available, reason: available ? "none" : "diagnostics_off" };
  }
  if (feature.state === "agent_mode") {
    const available = Boolean(preferences && preferences.agentMode);
    return { available, reason: available ? "none" : "agent_mode_off" };
  }
  if (feature.state === "definition_fusion") {
    const available = definitionFusionByDefault(preferences || {});
    return { available, reason: available ? "none" : "definition_fusion_explicit" };
  }
  return { available: true, reason: "none" };
}

function unavailableReasonText(reason, language) {
  const reasons = {
    offline_or_no_providers: {
      en: "the browser is offline or no search providers are available",
      ru: "браузер offline или нет доступных поисковых провайдеров",
      hi: "browser offline है या कोई search provider available नहीं है",
      zh: "浏览器 offline，或没有可用搜索 provider",
    },
    diagnostics_off: {
      en: "diagnostics are off; enable them to show traces",
      ru: "диагностика выключена; включите ее, чтобы видеть трассировку",
      hi: "diagnostics off है; trace दिखाने के लिए इसे enable करें",
      zh: "诊断已关闭；开启后才会显示 trace",
    },
    agent_mode_off: {
      en: "agent mode is off; multi-step actions require explicit opt-in",
      ru: "agent mode выключен; для многошаговых действий нужен явный opt-in",
      hi: "agent mode off है; multi-step actions के लिए explicit opt-in चाहिए",
      zh: "agent mode 已关闭；多步骤操作需要显式启用",
    },
    definition_fusion_explicit: {
      en: "automatic definition fusion is set to explicit-only",
      ru: "автоматическое слияние определений работает только после включения режима auto",
      hi: "automatic definition fusion के लिए auto mode enable करना होगा",
      zh: "自动 definition fusion 需要切换到 auto 模式",
    },
  };
  return localizedValue(reasons[reason] || { en: "not available" }, language);
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
    if (language === "ru") {
      return `Да. Возможность «${label}» доступна в этой конфигурации. Пример сообщения: \`${example}\`.`;
    }
    if (language === "zh") {
      return `可以。当前配置中「${label}」可用。示例消息：\`${example}\`。`;
    }
    if (language === "hi") {
      return `हाँ। इस configuration में \`${label}\` available है। Example message: \`${example}\`.`;
    }
    return `Yes. ${label} is available in this configuration. Example message: \`${example}\`.`;
  }
  const reason = unavailableReasonText(availability.reason, language);
  if (language === "ru") {
    return `Нет. Возможность «${label}» сейчас недоступна в этой конфигурации: ${reason}. Пример сообщения после включения: \`${example}\`.`;
  }
  if (language === "zh") {
    return `不可以。当前配置中「${label}」不可用：${reason}。启用后的示例消息：\`${example}\`。`;
  }
  if (language === "hi") {
    return `नहीं। इस configuration में \`${label}\` अभी available नहीं है: ${reason}. Enable करने के बाद example message: \`${example}\`.`;
  }
  return `No. ${label} is not available in this configuration: ${reason}. Example message after enabling it: \`${example}\`.`;
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

function additionalCapabilitiesContent(language) {
  if (language === "ru") {
    return "Кроме уже названных возможностей, могу ещё:\n\n- **Арифметика**: вычислять выражения вроде «Сколько будет 2 + 2?»\n- **Перевод**: переводить короткие фразы между поддерживаемыми языками.\n- **Поиск понятий**: объяснять термины, например «Что такое Википедия?»\n- **Hello World**: генерировать минимальные программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Память диалога**: использовать предыдущие сообщения текущей сессии.\n- **Правила поведения**: показывать встроенные правила через `Покажи правила поведения` и `Покажи правило unknown`.\n- **Настройки и действия**: включать диагностику/демо/agent mode, менять тему, язык, стиль чата, экспортировать и импортировать память.";
  }
  return "Beyond the capability already discussed, I can also:\n\n- **Arithmetic**: evaluate expressions like `2 + 2`.\n- **Translation**: translate short phrases between supported languages.\n- **Concept lookup**: explain terms such as `What is Wikipedia?`.\n- **Hello World**: generate small programs in Rust, Python, JavaScript, Go, C, and more.\n- **Conversation memory**: use earlier messages from the current session.\n- **Behavior rules**: show built-in rules with `List behavior rules` and `Show behavior rule unknown`.\n- **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export.";
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
  const surface = BROWSER_SURFACE;
  if (language === "ru") {
    return `Я не LLM-рантайм и не выполняю нейросетевой инференс. Текущая среда: ${surface.label} (\`${surface.slug}\`). Рантайм: ${surface.runtime}. У проекта есть OpenAI-совместимые API-форматы, но ответы строит детерминированный solver: сначала он проверяет локальный seed Links Notation, правила и память (${surface.memory}); затем веб-поиск используется только с учетом среды: ${surface.webSearch}. Весь интернет не загружен в локальные правила целиком.`;
  }
  if (language === "hi") {
    return `मैं LLM runtime नहीं हूँ और neural inference नहीं चलाता. Current environment: ${surface.label} (\`${surface.slug}\`). Runtime: ${surface.runtime}. Project OpenAI-compatible API shapes देता है, लेकिन जवाब deterministic solver बनाता है: पहले local Links Notation seed, rules और memory (${surface.memory}) देखता है; फिर web search केवल environment अनुमति दे तो उपयोग करता है: ${surface.webSearch}. पूरा internet local rules में preload नहीं है.`;
  }
  if (language === "zh") {
    return `我不是 LLM runtime, 也不执行神经网络推理。当前环境: ${surface.label} (\`${surface.slug}\`)。Runtime: ${surface.runtime}。项目提供 OpenAI-compatible API 形状, 但回答由确定性的 solver 生成: 先检查本地 Links Notation seed、规则和记忆 (${surface.memory}); 然后只在当前环境允许时使用 web search: ${surface.webSearch}。整个互联网不会预加载到本地规则中。`;
  }
  return `I am not an LLM runtime and I do not perform neural inference. Current environment: ${surface.label} (\`${surface.slug}\`). Runtime: ${surface.runtime}. The project exposes OpenAI-compatible API shapes, but answers come from a deterministic solver: it checks the local Links Notation seed, rules, and memory (${surface.memory}) first; web search is used only when this environment allows it: ${surface.webSearch}. The whole internet is not preloaded into local rules.`;
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
        ...(priorSearch ? ["capabilities:history:prior_web_search"] : []),
        `language:${language}`,
      ],
    };
  }
  const content =
    language === "ru"
      ? "Я formal-ai — детерминированный символьный ИИ. Вот что я умею:\n\n- **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.\n- **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Веб-поиск**: ищу в интернете через DuckDuckGo, Wikipedia и Wikidata, когда поиск доступен.\n- **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»\n- **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»\n- **Перевод**: перевожу фразы между языками.\n- **Память**: помню контекст разговора в рамках сессии.\n- **Настройки и действия**: через сообщения можно включать диагностику/демо/agent mode, менять тему, язык, стиль чата и экспортировать или импортировать память.\n\nЯ работаю на основе локальных символьных правил, без нейросетевого инференса."
      : language === "zh"
        ? "我是 formal-ai —— 一个确定性的符号化 AI。以下是我的功能：\n\n- **问候**：回应「你好」等问候语。\n- **Hello World**：生成 Rust、Python、JavaScript、Go、C 等语言的示例程序。\n- **Web search**：在可用时通过 DuckDuckGo、Wikipedia 和 Wikidata 搜索互联网。\n- **概念查找**：解释术语，例如「什么是维基百科？」\n- **算术**：计算表达式，例如「2 + 2 等于多少？」\n- **翻译**：在语言之间翻译短语。\n- **记忆**：在会话中记住上下文。\n- **设置和操作**：可通过消息开启诊断、演示、agent mode，切换主题、语言、聊天样式，并导出或导入记忆。\n\n我基于本地符号规则运行，不进行神经网络推理。"
        : language === "hi"
          ? "मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI। मैं यह कर सकता हूँ:\n\n- **अभिवादन**: «नमस्ते» आदि का जवाब देना।\n- **Hello World**: Rust, Python, JavaScript, Go, C आदि में प्रोग्राम बनाना।\n- **Web search**: उपलब्ध होने पर DuckDuckGo, Wikipedia, और Wikidata से इंटरनेट में खोजना।\n- **अवधारणा खोज**: शब्दों को समझाना — जैसे «विकिपीडिया क्या है?»\n- **अंकगणित**: गणनाएँ — जैसे «2 + 2 क्या है?»\n- **अनुवाद**: भाषाओं के बीच अनुवाद।\n- **स्मृति**: सत्र में संदर्भ याद रखना।\n- **Settings और actions**: messages से diagnostics/demo/agent mode बदलना, theme/language/chat style बदलना, और memory export/import करना।\n\nमैं स्थानीय प्रतीकात्मक नियमों पर चलता हूँ, कोई न्यूरल इन्फेरेन्स नहीं।"
          : "I am formal-ai, a deterministic symbolic AI. Here is what I can do:\n\n- **Greetings**: respond to «Hi», «Hello», and similar.\n- **Hello World**: generate programs in Rust, Python, JavaScript, Go, C, and more.\n- **Web search**: search the internet through DuckDuckGo, Wikipedia, and Wikidata when available.\n- **Concept lookup**: explain terms — try «What is Wikipedia?»\n- **Arithmetic**: evaluate expressions — try «What is 2 + 2?»\n- **Translation**: translate phrases between languages.\n- **Memory**: recall context within the current session.\n- **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export from messages.\n\nI run on local symbolic rules, without any neural network inference.";
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

// Offline meaning registry for the browser worker.
//
// The Rust pipeline (`src/translation/pipeline.rs`) resolves any pair
// of surfaces through Wiktionary + Wikidata using cached HTTP
// responses. The worker mirrors that with a live `liveWiktionaryTranslate`
// fallback below (MediaWiki action API is CORS-friendly via
// `origin=*`), but keeps this small in-memory registry of greetings and
// stock phrases so the demo stays snappy when the network is slow.
// `primary` is the canonical form deformalization renders; `aliases` is a
// list of normalized alternative surfaces used during formalization.
const TRANSLATION_MEANING_REGISTRY = [
  {
    token: "greeting",
    primary: { en: "Hello", ru: "Привет", hi: "नमस्ते", zh: "你好" },
    aliases: {
      en: ["hello", "hi", "hey"],
      ru: ["привет", "здравствуйте", "здравствуй"],
      hi: ["नमस्ते", "नमस्कार"],
      zh: ["你好", "您好"],
    },
  },
  {
    token: "greeting_how_are_you",
    primary: {
      en: "How are you?",
      ru: "Как у тебя дела?",
      hi: "आप कैसे हैं?",
      zh: "你好吗？",
    },
    aliases: {
      en: ["howareyou", "hellohowareyou", "hihowareyou"],
      ru: [
        "какдела",
        "какутебядела",
        "какувасдела",
        "какваши дела",
        "какватидела",
        "какваши",
        "приветкакдела",
        "здравствуйтекаквашидела",
      ],
      hi: ["आपकैसेहैं", "तुमकैसेहो"],
      zh: ["你好吗", "你怎么样"],
    },
  },
  {
    token: "thank_you",
    primary: { en: "Thank you", ru: "Спасибо", hi: "धन्यवाद", zh: "谢谢" },
    aliases: {
      en: ["thanks", "thankyou", "thankyouverymuch"],
      ru: ["спасибо", "благодарю", "большоеспасибо"],
      hi: ["धन्यवाद", "शुक्रिया"],
      zh: ["谢谢", "多谢", "感谢"],
    },
  },
  {
    token: "you_are_welcome",
    primary: {
      en: "You are welcome",
      ru: "Пожалуйста",
      hi: "आपका स्वागत है",
      zh: "不客气",
    },
    aliases: {
      en: ["youarewelcome", "yourewelcome", "nottoworry"],
      ru: ["пожалуйста", "незачто"],
      hi: ["आपकास्वागतहै", "कोईबातनहीं"],
      zh: ["不客气", "不用谢"],
    },
  },
  {
    token: "goodbye",
    primary: { en: "Goodbye", ru: "До свидания", hi: "अलविदा", zh: "再见" },
    aliases: {
      en: ["goodbye", "bye", "seeyou", "byebye"],
      ru: ["досвидания", "пока", "прощай"],
      hi: ["अलविदा", "फिरमिलेंगे"],
      zh: ["再见", "拜拜"],
    },
  },
  {
    token: "good_morning",
    primary: { en: "Good morning", ru: "Доброе утро", hi: "सुप्रभात", zh: "早上好" },
    aliases: {
      en: ["goodmorning"],
      ru: ["доброеутро"],
      hi: ["सुप्रभात", "शुभप्रभात"],
      zh: ["早上好", "早安"],
    },
  },
  {
    token: "good_evening",
    primary: { en: "Good evening", ru: "Добрый вечер", hi: "शुभ संध्या", zh: "晚上好" },
    aliases: {
      en: ["goodevening"],
      ru: ["добрыйвечер"],
      hi: ["शुभसंध्या"],
      zh: ["晚上好", "晚安"],
    },
  },
  {
    token: "what_is_your_name",
    primary: {
      en: "What is your name?",
      ru: "Как тебя зовут?",
      hi: "तुम्हारा नाम क्या है?",
      zh: "你叫什么名字？",
    },
    aliases: {
      en: ["whatisyourname", "whatsyourname"],
      ru: ["кактебязовут", "каквасзовут"],
      hi: ["तुम्हारानामक्याहै", "आपकानामक्याहै"],
      zh: ["你叫什么名字", "您叫什么名字"],
    },
  },
  {
    token: "who_are_you",
    primary: {
      en: "Who are you?",
      ru: "Кто ты такой?",
      hi: "तुम कौन हो?",
      zh: "你是谁？",
    },
    aliases: {
      en: ["whoareyou"],
      ru: ["ктоты", "ктотытакой", "ктотытакая", "ктовы", "ктовытакой", "ктовытакая"],
      hi: ["तुमकौनहो", "आपकौनहैं"],
      zh: ["你是谁", "您是谁"],
    },
  },
  {
    token: "what_is_this",
    primary: {
      en: "What is this?",
      ru: "Что это такое?",
      hi: "यह क्या है?",
      zh: "这是什么？",
    },
    aliases: {
      en: ["whatisthis", "whatisit"],
      ru: ["чтоэто", "чтоэтотакое"],
      hi: ["यहक्याहै", "येक्याहै"],
      zh: ["这是什么", "這是什麼"],
    },
  },
  {
    token: "i_am_fine",
    primary: { en: "I am fine", ru: "У меня всё хорошо", hi: "मैं ठीक हूँ", zh: "我很好" },
    aliases: {
      en: ["iamfine", "imfine", "imdoingfine", "imdoingwell"],
      ru: ["уменявсёхорошо", "уменявсехорошо", "всёхорошо"],
      hi: ["मैंठीकहूँ", "मैंठीकहूं"],
      zh: ["我很好", "我挺好的"],
    },
  },
  {
    token: "yes",
    primary: { en: "Yes", ru: "Да", hi: "हाँ", zh: "是" },
    aliases: {
      en: ["yes", "yeah", "yep", "aye"],
      ru: ["да", "ага", "конечно"],
      hi: ["हाँ", "हां", "जी"],
      zh: ["是", "是的", "对"],
    },
  },
  {
    token: "no",
    primary: { en: "No", ru: "Нет", hi: "नहीं", zh: "不" },
    aliases: {
      en: ["no", "nope", "nah"],
      ru: ["нет", "неа"],
      hi: ["नहीं", "ना"],
      zh: ["不", "不是"],
    },
  },
  // Issue #216 / #217: the apple noun must be translatable in both
  // directions from the browser demo, including unquoted prompts.
  {
    token: "apple",
    primary: { en: "apple", ru: "яблоко", hi: "सेब", zh: "苹果" },
    aliases: {
      en: ["apple", "apples"],
      ru: [
        "яблоко",
        "яблока",
        "яблоку",
        "яблоком",
        "яблоке",
        "яблоки",
        "яблок",
        "яблокам",
        "яблоками",
        "яблоках",
      ],
      hi: ["सेब"],
      zh: ["苹果"],
    },
  },
];

const TRANSLATION_TERMINAL_PUNCTUATION = ["?", "!", ".", "。", "？", "！", "．"];

function normalizeTranslationAlias(surface) {
  return Array.from(String(surface || "").toLowerCase())
    .filter((character) => /[\p{L}\p{N}]/u.test(character))
    .join("");
}

function formalizeSurface(surface, source) {
  const normalized = normalizeTranslationAlias(surface);
  if (!normalized) return null;
  for (const entry of TRANSLATION_MEANING_REGISTRY) {
    const aliases = (entry.aliases && entry.aliases[source]) || [];
    if (aliases.some((alias) => normalizeTranslationAlias(alias) === normalized)) {
      return entry.token;
    }
    const primary = entry.primary && entry.primary[source];
    if (primary && normalizeTranslationAlias(primary) === normalized) {
      return entry.token;
    }
  }
  return null;
}

function deformalizeMeaning(token, target) {
  for (const entry of TRANSLATION_MEANING_REGISTRY) {
    if (entry.token !== token) continue;
    const primary = entry.primary && entry.primary[target];
    return primary || null;
  }
  return null;
}

function canonicalTokenForNormalized(normalized) {
  if (!normalized) return null;
  for (const entry of TRANSLATION_MEANING_REGISTRY) {
    const aliasesByLang = entry.aliases || {};
    for (const lang of Object.keys(aliasesByLang)) {
      const aliases = aliasesByLang[lang] || [];
      if (aliases.some((alias) => normalizeTranslationAlias(alias) === normalized)) {
        return entry.token;
      }
    }
    const primaryByLang = entry.primary || {};
    for (const lang of Object.keys(primaryByLang)) {
      if (normalizeTranslationAlias(primaryByLang[lang]) === normalized) {
        return entry.token;
      }
    }
  }
  return null;
}

function canonicalMeaningToken(raw) {
  return canonicalTokenForNormalized(raw) || raw;
}

function normalizeMeaningText(surface) {
  const raw = normalizeTranslationAlias(surface);
  return canonicalMeaningToken(raw);
}
