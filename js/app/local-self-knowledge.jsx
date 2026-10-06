// Local fallback self-knowledge: self facts, known facts, self-introduction,
// conversation topic and architecture answers.

import { IDENTITY_ANSWER } from "./app-constants.jsx";
import { normalizeAssistantName } from "./interface-commands.jsx";
import { normalizePrompt } from "./local-prompts.jsx";
import { normalizeBlueprintComposition } from "./preferences.jsx";

function localAssistantNameStatus(preferences = {}) {
  const name = normalizeAssistantName(preferences.assistantName);
  return name ? `configured:${name}` : "browser_preference_when_set_else_not_configured";
}

function localLinoEscape(value) {
  return String(value || "").replaceAll("\\", "\\\\").replaceAll('"', '\\"').replaceAll("\n", "\\n");
}

const LOCAL_BROWSER_SURFACE = {
  slug: "browser",
  label: "browser demo with JavaScript and WebAssembly worker",
  runtime: "JavaScript UI plus a WebAssembly worker mirror of the solver",
  memory: "browser IndexedDB/local storage plus worker state and imported memory",
  webSearch: "available through browser CORS-readable providers when online and not blocked",
  limits: "browser settings, import/export controls, and IndexedDB-backed memory belong to this surface",
};

function localModeStatus(enabled) {
  return enabled ? "enabled" : "disabled";
}

function localDefinitionFusionStatus(preferences = {}) {
  return preferences.definitionFusion === "auto" ? "enabled_by_default" : "explicit_only";
}

function localBlueprintCompositionStatus(preferences = {}) {
  return normalizeBlueprintComposition(preferences.blueprintComposition);
}

export function localSelfFacts(preferences = {}) {
  const assistantName = localAssistantNameStatus(preferences);
  const surface = LOCAL_BROWSER_SURFACE;
  return [
    "Facts I know about myself in this environment:",
    "",
    `- **Execution surface**: ${surface.label} (\`${surface.slug}\`).`,
    `- **Runtime**: ${surface.runtime}.`,
    `- **Memory**: ${surface.memory}.`,
    `- **Web search**: ${surface.webSearch}.`,
    `- **Surface limits**: ${surface.limits}.`,
    "- **Local rules**: local Links Notation rules and seed facts are checked first.",
    "",
    "```links",
    "self_fact_model",
    '  subject "formal-ai"',
    '  relation "model"',
    '  object "formal-ai"',
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
    `  object "${localLinoEscape(surface.runtime)}"`,
    "self_fact_memory",
    '  subject "formal-ai"',
    '  relation "memory"',
    `  object "${localLinoEscape(surface.memory)}"`,
    "self_fact_web_search",
    '  subject "formal-ai"',
    '  relation "web_search"',
    `  object "${localLinoEscape(surface.webSearch)}"`,
    "self_fact_assistant_name",
    '  subject "formal-ai"',
    '  relation "assistant_name"',
    `  object "${localLinoEscape(assistantName)}"`,
    "self_fact_agent_mode",
    '  subject "formal-ai"',
    '  relation "agent_mode"',
    `  object "${localModeStatus(preferences.agentMode)}"`,
    "self_fact_diagnostics",
    '  subject "formal-ai"',
    '  relation "diagnostic_mode"',
    `  object "${localModeStatus(preferences.diagnosticsMode)}"`,
    "self_fact_definition_fusion",
    '  subject "formal-ai"',
    '  relation "definition_fusion"',
    `  object "${localDefinitionFusionStatus(preferences)}"`,
    "self_fact_blueprint_composition",
    '  subject "formal-ai"',
    '  relation "blueprint_composition"',
    `  object "${localBlueprintCompositionStatus(preferences)}"`,
    "```",
    "",
    "Read behavior with `List behavior rules`; teach one with When `prompt` then `answer` (or When I say `prompt`, answer `answer`).",
  ].join("\n");
}

export function localKnownFacts(language, preferences = {}) {
  const surface = LOCAL_BROWSER_SURFACE;
  const assistantName = localAssistantNameStatus(preferences);
  const links = [
    "```links",
    "known_fact_local_seed",
    '  source "local_links_notation_seed"',
    '  scope "built-in rules, concepts, facts, tools, and response templates"',
    "known_fact_internet",
    '  source "environment_aware_web_search"',
    `  scope "${localLinoEscape(surface.webSearch)}"`,
    "known_fact_memory",
    '  source "conversation_memory"',
    `  scope "${localLinoEscape(surface.memory)}"`,
    "known_fact_environment",
    '  subject "formal-ai"',
    '  relation "execution_surface"',
    `  object "${surface.slug}"`,
    "known_fact_self",
    '  subject "formal-ai"',
    '  relation "model"',
    '  object "formal-ai"',
    "known_fact_assistant_name",
    '  subject "formal-ai"',
    '  relation "assistant_name_setting"',
    `  object "${localLinoEscape(assistantName)}"`,
    "known_fact_surface_limits",
    '  source "environment_directory"',
    `  scope "${localLinoEscape(surface.limits)}"`,
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

export function localContainsAny(normalized, needles) {
  return needles.some((needle) => normalized.includes(needle));
}

export function localIsSelfFactQuery(normalized) {
  return (
    normalized.includes("facts you know about yourself") ||
    normalized.includes("self facts") ||
    normalized.includes("факты о себе") ||
    normalized.includes("какие факты ты знаешь о себе")
  );
}

export function localIsSelfIntroductionQuery(normalized) {
  const cleaned = normalizePrompt(normalized);
  if (!cleaned || localIsSelfFactQuery(cleaned)) return false;
  return (
    cleaned === "tell me about yourself" ||
    cleaned === "introduce yourself" ||
    cleaned.includes("tell me about yourself") ||
    cleaned.includes("introduce yourself") ||
    cleaned.includes("let s get acquainted") ||
    cleaned.includes("lets get acquainted") ||
    cleaned.includes("let us get acquainted") ||
    cleaned.includes("let s get to know each other") ||
    cleaned.includes("расскажи о себе") ||
    cleaned.includes("расскажи мне о себе") ||
    cleaned.includes("расскажи про себя") ||
    cleaned.includes("опиши себя") ||
    cleaned.includes("представься") ||
    cleaned.includes("давай знакомиться") ||
    cleaned.includes("давай познакомимся") ||
    cleaned.includes("давайте познакомимся") ||
    cleaned.includes("चलो परिचय करते हैं") ||
    cleaned.includes("आइए परिचय करें") ||
    cleaned.includes("चलो एक दूसरे को जानें") ||
    cleaned.includes("我们认识一下") ||
    cleaned.includes("认识一下吧") ||
    cleaned.includes("让我们认识一下")
  );
}

export function localSelfAwarenessLanguage(prompt, normalized) {
  const text = `${String(prompt || "").toLowerCase()} ${String(normalized || "")}`;
  if (/[\u0400-\u04ff]/u.test(text) || localContainsAny(text, ["ты", "теб", "у тебя"])) {
    return "ru";
  }
  if (/[\u0900-\u097f]/u.test(text)) return "hi";
  if (/[\u4e00-\u9fff]/u.test(text)) return "zh";
  return "en";
}

export function localSelfIntroductionContent(language, preferences = {}) {
  const identity = IDENTITY_ANSWER;
  const name = normalizeAssistantName(preferences.assistantName);
  if (!name) return identity;
  if (language === "ru") return `Меня зовут ${name}. ${identity}`;
  if (language === "hi") return `मेरा नाम ${name} है। ${identity}`;
  if (language === "zh") return `我的名字是 ${name}。${identity}`;
  return `My name is ${name}. ${identity}`;
}

function localCleanConversationTopic(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[`"':._,\-\s!?]+|[`"':._,\-\s!?]+$/gu, "");
}

export function localConversationTopic(prompt, normalized) {
  const source = String(prompt || "");
  const lower = source.toLowerCase();
  for (const prefix of [
    "let's talk about ",
    "lets talk about ",
    "can we talk about ",
    "talk about ",
    "давай поговорим о ",
    "давай поговорим об ",
    "давайте поговорим о ",
    "давайте поговорим об ",
    "поговорим о ",
    "поговорим об ",
    "обсудим ",
    "चलो बात करें ",
    "बात करें ",
    "聊聊",
    "谈谈",
  ]) {
    if (String(normalized || "").startsWith(prefix)) {
      return localCleanConversationTopic(String(normalized || "").slice(prefix.length));
    }
  }
  const marker = "поговорим о ";
  const index = lower.indexOf(marker);
  if (index >= 0) return localCleanConversationTopic(lower.slice(index + marker.length));
  return "";
}

export function localConversationTopicContent(topic, language) {
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

export function localIsKnownFactQuery(normalized) {
  const english =
    (normalized.includes("facts") &&
      localContainsAny(normalized, ["what", "which", "list", "show"]) &&
      localContainsAny(normalized, [
        "you know",
        "do you know",
        "you have",
        "available to you",
        "in your knowledge",
        "known to you",
      ])) ||
    localContainsAny(normalized, [
      "what do you know in general",
      "what do you know about the world",
      "what is known to you",
      "what knowledge do you have",
    ]);
  const russian =
    (normalized.includes("факт") &&
      localContainsAny(normalized, ["какие", "что", "перечисли", "покажи", "назови"]) &&
      localContainsAny(normalized, [
        "ты знаешь",
        "знаешь",
        "тебе извест",
        "у тебя есть",
        "твои знания",
        "что ты знаешь",
      ])) ||
    localContainsAny(normalized, [
      "что тебе вообще известно",
      "что тебе известно",
      "что ты вообще знаешь",
      "что ты знаешь об окружающем мире",
      "известно об окружающем мире",
      "знаешь про окружающий мир",
      "знаешь об окружающем мире",
    ]);
  const hindi = localContainsAny(normalized, [
    "आप क्या जानते हैं",
    "तुम क्या जानते हो",
    "आपको क्या पता है",
  ]);
  const chinese = localContainsAny(normalized, ["你知道什么", "您知道什么", "你知道哪些"]);
  return english || russian || hindi || chinese;
}

export function localIsArchitectureQuestion(normalized) {
  const mentionsAssistant = localContainsAny(normalized, [
    "you",
    "your",
    "formal ai",
    "ты",
    "теб",
    "твоя",
    "твой",
    "тво",
    "вы",
  ]);
  if (!mentionsAssistant) return false;
  return localContainsAny(normalized, [
    "llm",
    "large language model",
    "language model",
    "openai api",
    "openai",
    "neural inference",
    "neural network",
    "links notation rules",
    "local rules",
    "world model",
    "model of the world",
    "бям",
    "языковая модель",
    "языковой моделью",
    "нейросет",
    "нейрон",
    "локальных правил",
    "локальных правилах",
    "область знаний",
    "модель окружающего мира",
    "модель мира",
    "принцип работы",
    "идея твоей разработки",
    "идея твоего проекта",
    "зачем тебя разработ",
    "ссылк",
  ]);
}

export function localArchitectureExplanation(language) {
  const surface = LOCAL_BROWSER_SURFACE;
  if (language === "ru") {
    return `Я не LLM-рантайм и не выполняю нейросетевой инференс. Текущая среда: ${surface.label} (\`${surface.slug}\`). Рантайм: ${surface.runtime}. У проекта есть OpenAI-совместимые API-форматы, но ответы строит детерминированный solver: сначала он проверяет локальный seed Links Notation, правила и память (${surface.memory}); затем веб-поиск используется только с учетом среды: ${surface.webSearch}. Весь интернет не загружен в локальные правила целиком.`;
  }
  return `I am not an LLM runtime and I do not perform neural inference. Current environment: ${surface.label} (\`${surface.slug}\`). Runtime: ${surface.runtime}. The project exposes OpenAI-compatible API shapes, but answers come from a deterministic solver: it checks the local Links Notation seed, rules, and memory (${surface.memory}) first; web search is used only when this environment allows it: ${surface.webSearch}. The whole internet is not preloaded into local rules.`;
}
