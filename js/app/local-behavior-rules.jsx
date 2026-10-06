// Local fallback behaviour rules: the built-in rule records, runtime rules
// taught in chat, and the multilingual "list/count/explain rules" queries.

import {
  ASSISTANT_FREE_TIME_ANSWER, ASSISTANT_NAME_ANSWER, IDENTITY_ANSWER, UNKNOWN_ANSWER,
} from "./app-constants.jsx";
import { normalizePrompt } from "./local-prompts.jsx";
import { localContainsAny } from "./local-self-knowledge.jsx";

export function localBehaviorRuleId(value) {
  let hash = 2166136261;
  const text = String(value || "");
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 16777619) >>> 0;
  }
  return `behavior_rule_runtime_${hash.toString(16)}`;
}

function localCodeSpans(text) {
  return String(text || "")
    .split("`")
    .map((part, index) => (index % 2 === 1 ? part.trim() : ""))
    .filter(Boolean);
}

// Issue #144: mirror the worker's multilingual `When X then Y` grammar so the
// local fallback recognizes the same teach forms even without WASM.
const LOCAL_BEHAVIOR_RULE_KEYWORD_PAIRS = [
  ["when ", " then "],
  ["when ", " do "],
  ["когда ", " тогда "],
  ["когда ", " делай "],
  ["когда ", " сделай "],
  ["когда ", " отвечай "],
  ["когда ", " отвечать "],
  ["если ", " то "],
  ["जब ", " तब "],
  ["जब ", " तो "],
  ["当 ", " 时 "],
  ["当 ", " 则 "],
  ["当 ", " 回答 "],
  ["当 ", "时回答 "],
  ["当 ", "则回答 "],
];

function localLooksLikeRuntimeRuleUpdate(text) {
  const raw = String(text || "");
  const lower = raw.toLowerCase();
  if (
    (lower.includes("when i say") && (lower.includes("answer") || lower.includes("reply"))) ||
    (lower.includes("if i ask") && (lower.includes("answer") || lower.includes("reply"))) ||
    lower.includes("add behavior rule") ||
    lower.includes("update behavior rule") ||
    (lower.includes("когда я скажу") && lower.includes("ответ")) ||
    (lower.includes("если я спрошу") && lower.includes("ответ")) ||
    lower.includes("добавь правило поведения") ||
    lower.includes("обнови правило поведения")
  ) {
    return true;
  }
  for (const [head, link] of LOCAL_BEHAVIOR_RULE_KEYWORD_PAIRS) {
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

export function localRuntimeRuleFromText(text) {
  if (!localLooksLikeRuntimeRuleUpdate(text)) return null;
  const spans = localCodeSpans(text);
  if (spans.length < 2) return null;
  const trigger = spans[0].trim();
  const answer = spans[1].trim();
  if (!trigger || !answer) return null;
  return {
    id: localBehaviorRuleId(`${trigger}\n${answer}`),
    trigger,
    answer,
  };
}

function localBehaviorRuleRecords() {
  return [
    {
      id: "rule_greeting",
      topic: "greetings",
      intent: "greeting",
      label: "Greeting rule",
      matches: "`Hi`, `Hello`, and `Hey`",
      response: "Hi, how may I help you?",
      source: "local fallback",
      whenThen:
        "When the user says `Hi`, `Hello`, or `Hey` then respond with `Hi, how may I help you?`.",
    },
    {
      id: "rule_identity",
      topic: "identity",
      intent: "identity",
      label: "Identity rule",
      matches: "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
      response: IDENTITY_ANSWER,
      source: "local fallback",
      whenThen: `When the user asks \`Who are you?\` or \`Кто ты?\` then respond with the identity answer.`,
    },
    {
      id: "rule_assistant_free_time",
      topic: "small_talk",
      intent: "assistant_free_time",
      label: "Assistant free-time rule",
      matches:
        "`What do you do in your free time?`, `Что делаешь в свободное время?`, and equivalent small-talk seed phrases",
      response: ASSISTANT_FREE_TIME_ANSWER,
      source: "local fallback",
      whenThen: `When the user asks what I do in free time then respond with \`${ASSISTANT_FREE_TIME_ANSWER}\`.`,
    },
    {
      id: "rule_assistant_name",
      topic: "assistant_name",
      intent: "assistant_name",
      label: "Assistant name rule",
      matches: "`What is your name?`, `Как твое имя?`, and equivalent name prompts",
      response: ASSISTANT_NAME_ANSWER,
      source: "local fallback",
      whenThen:
        "When the user asks `What is your name?` or `Как твое имя?` then respond with the assistant-name answer, unless the assistant name setting is configured.",
    },
    {
      id: "rule_unknown",
      topic: "unknown_fallback",
      intent: "unknown",
      label: "Unknown fallback rule",
      matches: "Any prompt that no earlier rule can answer",
      response: UNKNOWN_ANSWER,
      source: "local fallback",
      whenThen:
        "When no earlier rule or handler matches the prompt then respond with the unknown-intent guide.",
    },
  ];
}

const LOCAL_BEHAVIOR_RULE_TOPIC_ORDER = [
  "greetings",
  "identity",
  "small_talk",
  "assistant_name",
  "unknown_fallback",
];

export function localLocalizedText(language, values) {
  return values[language] || values.en;
}

function localBehaviorRuleTopicLabel(topic, language) {
  const labels = {
    greetings: { en: "Greetings", ru: "Приветствия", hi: "अभिवादन", zh: "问候" },
    identity: { en: "Identity", ru: "Идентичность", hi: "पहचान", zh: "身份" },
    small_talk: {
      en: "Small talk",
      ru: "Светская беседа",
      hi: "हल्की बातचीत",
      zh: "闲聊",
    },
    assistant_name: {
      en: "Assistant name",
      ru: "Имя ассистента",
      hi: "सहायक का नाम",
      zh: "助手名称",
    },
    unknown_fallback: {
      en: "Unknown fallback",
      ru: "Резервный ответ",
      hi: "अज्ञात अनुरोध का वैकल्पिक उत्तर",
      zh: "未知请求回退",
    },
  };
  return localLocalizedText(language, labels[topic] || {
    en: "Other",
    ru: "Другое",
    hi: "अन्य",
    zh: "其他",
  });
}

function localBehaviorRuleListIntro(language) {
  return localLocalizedText(language, {
    en: "Behavior rules I can inspect in this dialog (grouped by topic, each shown as a `When X then Y` statement):",
    ru: "Правила поведения, которые я могу показать в этом диалоге (сгруппированы по темам; каждое показано как инструкция `Когда X тогда Y`):",
    hi: "व्यवहार नियम जिन्हें मैं इस संवाद में दिखा सकता हूँ (विषय के अनुसार समूहित; हर नियम `जब X तब Y` कथन के रूप में है):",
    zh: "我可以查看的行为规则（按主题分组；每条都显示为 `当 X 时 Y` 语句）：",
  });
}

export function localRuntimeRuleWhenThen(rule, language) {
  if (language === "ru") return `Когда пользователь говорит \`${rule.trigger}\`, ответь \`${rule.answer}\`.`;
  if (language === "hi") return `जब उपयोगकर्ता \`${rule.trigger}\` कहे, तब \`${rule.answer}\` उत्तर दें.`;
  if (language === "zh") return `当用户说 \`${rule.trigger}\` 时，回答 \`${rule.answer}\`。`;
  return `When the user says \`${rule.trigger}\` then respond with \`${rule.answer}\`.`;
}

export function localRuleResponse(rule, language) {
  if (rule.id === "rule_greeting") {
    if (language === "ru") return "Здравствуйте! Чем могу помочь?";
    if (language === "hi") return "नमस्ते! मैं आपकी क्या मदद कर सकता हूँ?";
    if (language === "zh") return "你好，请问我可以帮你什么？";
  }
  if (rule.id === "rule_assistant_free_time") {
    return localLocalizedText(language, {
      en: ASSISTANT_FREE_TIME_ANSWER,
      ru: "У меня нет свободного времени в человеческом смысле. Между запросами я бездействую; когда диалог активен, помогаю с задачами, правилами и объяснениями.",
      hi: "मेरे पास मनुष्यों जैसा खाली समय नहीं है. prompts के बीच मैं निष्क्रिय रहता हूँ; dialog सक्रिय हो तो tasks, rules और explanations में मदद करता हूँ.",
      zh: "我没有人类意义上的空闲时间。两次提示之间我处于空闲状态；对话活跃时，我帮助处理任务、规则和解释。",
    });
  }
  if (rule.id === "rule_assistant_name") {
    return localLocalizedText(language, {
      en: "Returns the assistant-name answer; browser surfaces can override it from the assistant name setting.",
      ru: "Возвращает ответ об имени ассистента; браузерные поверхности могут переопределить его настройкой имени ассистента.",
      hi: "assistant-name उत्तर लौटाता है; browser surfaces assistant name setting से इसे बदल सकते हैं.",
      zh: "返回助手名称回答；浏览器界面可通过助手名称设置覆盖它。",
    });
  }
  return rule.response;
}

function localRuleLabel(rule, language) {
  const labels = {
    rule_greeting: {
      en: "Greeting rule",
      ru: "Правило приветствия",
      hi: "अभिवादन नियम",
      zh: "问候规则",
    },
    rule_identity: {
      en: "Identity rule",
      ru: "Правило идентичности",
      hi: "पहचान नियम",
      zh: "身份规则",
    },
    rule_assistant_free_time: {
      en: "Assistant free-time rule",
      ru: "Правило свободного времени ассистента",
      hi: "सहायक खाली समय नियम",
      zh: "助手空闲时间规则",
    },
    rule_assistant_name: {
      en: "Assistant name rule",
      ru: "Правило имени ассистента",
      hi: "सहायक नाम नियम",
      zh: "助手名称规则",
    },
    rule_unknown: {
      en: "Unknown fallback rule",
      ru: "Резервное правило для неизвестного запроса",
      hi: "अज्ञात अनुरोध का वैकल्पिक नियम",
      zh: "未知请求回退规则",
    },
  };
  return labels[rule.id] ? localLocalizedText(language, labels[rule.id]) : rule.label;
}

function localRuleMatches(rule, language) {
  const matches = {
    rule_greeting: {
      en: "`Hi`, `Hello`, and `Hey`",
      ru: "`Hi`, `Hello`, `Hey` и многоязычные seed-фразы приветствия",
      hi: "`Hi`, `Hello`, `Hey` और बहुभाषी greeting seed phrases",
      zh: "`Hi`、`Hello`、`Hey` 以及多语言问候 seed 短语",
    },
    rule_identity: {
      en: "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
      ru: "`Who are you?`, `Кто ты?` и равнозначные вопросы об идентичности",
      hi: "`Who are you?`, `Кто ты?` और समान identity prompts",
      zh: "`Who are you?`、`Кто ты?` 以及等价身份提示",
    },
    rule_assistant_free_time: {
      en: "`What do you do in your free time?`, `Что делаешь в свободное время?`, and equivalent small-talk seed phrases",
      ru: "`What do you do in your free time?`, `Что делаешь в свободное время?` и равнозначные seed-фразы светской беседы",
      hi: "`What do you do in your free time?`, `Что делаешь в свободное время?` और समान small-talk seed phrases",
      zh: "`What do you do in your free time?`、`Что делаешь в свободное время?` 以及等价闲聊 seed 短语",
    },
    rule_assistant_name: {
      en: "`What is your name?`, `Как твое имя?`, and equivalent name prompts",
      ru: "`What is your name?`, `Как твое имя?` и равнозначные вопросы об имени",
      hi: "`What is your name?`, `Как твое имя?` और समान name prompts",
      zh: "`What is your name?`、`Как твое имя?` 以及等价名称提示",
    },
    rule_unknown: {
      en: "Any prompt that no earlier rule can answer",
      ru: "Любой запрос, на который не ответило более раннее правило",
      hi: "कोई भी prompt जिसका उत्तर पहले का rule नहीं दे सकता",
      zh: "任何前面的规则无法回答的提示",
    },
  };
  return matches[rule.id] ? localLocalizedText(language, matches[rule.id]) : rule.matches;
}

function localRuleWhenThen(rule, language) {
  const response = localRuleResponse(rule, language);
  if (rule.id === "rule_greeting") {
    if (language === "ru") return `Когда пользователь говорит \`Hi\`, \`Hello\`, \`Hey\` или многоязычную фразу приветствия, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता \`Hi\`, \`Hello\`, \`Hey\` या बहुभाषी greeting phrase कहे, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户说 \`Hi\`、\`Hello\`、\`Hey\` 或多语言问候短语时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_identity") {
    if (language === "ru") return "Когда пользователь спрашивает `Who are you?` или `Кто ты?`, ответь сообщением об идентичности.";
    if (language === "hi") return "जब उपयोगकर्ता `Who are you?` या `Кто ты?` पूछे, तब identity answer दें.";
    if (language === "zh") return "当用户问 `Who are you?` 或 `Кто ты?` 时，回答身份说明。";
  }
  if (rule.id === "rule_assistant_free_time") {
    if (language === "ru") return `Когда пользователь спрашивает, что я делаю в свободное время, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता पूछे कि मैं खाली समय में क्या करता हूँ, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户问我空闲时间做什么时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_assistant_name") {
    if (language === "ru") return "Когда пользователь спрашивает `What is your name?` или `Как твое имя?`, ответь сообщением об имени ассистента; если настройка имени есть, включи настроенное имя.";
    if (language === "hi") return "जब उपयोगकर्ता `What is your name?` या `Как твое имя?` पूछे, तब assistant-name उत्तर दें; अगर setting है, तो configured name शामिल करें.";
    if (language === "zh") return "当用户问 `What is your name?` 或 `Как твое имя?` 时，回答助手名称；如果有名称设置，则包含配置的名称。";
  }
  if (rule.id === "rule_unknown") {
    if (language === "ru") return "Когда ни одно более раннее правило не подходит к запросу, ответь подсказкой для неизвестного намерения.";
    if (language === "hi") return "जब कोई पहले का rule prompt से मेल न खाए, तब unknown-intent guide दें.";
    if (language === "zh") return "当前面的规则都不匹配提示时，回答未知意图指南。";
  }
  return rule.whenThen;
}

function localBehaviorRuleListFooter(language) {
  if (language === "ru") {
    return [
      "",
      "Прочитать одно правило можно командой `Покажи правило unknown`.",
      "Научить этот диалог можно так: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``.",
      "Многоязычные формы: английская ``When `X` then `Y` ``, хинди ``जब `X` तब `Y` ``, китайская ``当 `X` 时 `Y` ``.",
      "Запись добавляется только в конец: экспортируйте память, чтобы сохранить сообщение с правилом вместе с диалогом.",
    ];
  }
  if (language === "hi") {
    return [
      "",
      "एक नियम पढ़ने के लिए `Show behavior rule unknown` भेजें.",
      "इस संवाद को सिखाएँ: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. दूसरा रूप: ``When I say `your prompt`, answer `your answer` ``.",
      "बहुभाषी रूप: रूसी ``Когда `X` тогда `Y` ``, अंग्रेज़ी ``When `X` then `Y` ``, चीनी ``当 `X` 时 `Y` ``.",
      "लेखन केवल append-only है: नियम संदेश को संवाद के साथ रखने के लिए memory export करें.",
    ];
  }
  if (language === "zh") {
    return [
      "",
      "要读取一条规则，请发送 `Show behavior rule unknown`。",
      "可以这样教当前对话：``当 `你的提示` 时 `你的回答` ``。也可以发送：``When I say `your prompt`, answer `your answer` ``。",
      "多语言形式：俄语 ``Когда `X` тогда `Y` ``，印地语 ``जब `X` तब `Y` ``，英语 ``When `X` then `Y` ``。",
      "写入是 append-only：导出 memory 可把这条规则消息随对话一起保存。",
    ];
  }
  return [
    "",
    "Read one with `Show behavior rule unknown`.",
    "Teach this dialog with: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.",
    "Multilingual forms: Russian ``Когда `X` тогда `Y` ``, Hindi ``जब `X` तब `Y` ``, Chinese ``当 `X` 时 `Y` ``.",
    "The write is append-only: export memory to preserve the rule message with the dialog.",
  ];
}

export function localBehaviorRulesList(runtimeRules, language = "en") {
  const lines = [localBehaviorRuleListIntro(language), ""];
  const groups = new Map();
  for (const rule of localBehaviorRuleRecords()) {
    const order = LOCAL_BEHAVIOR_RULE_TOPIC_ORDER.indexOf(rule.topic);
    const safeOrder = order === -1 ? LOCAL_BEHAVIOR_RULE_TOPIC_ORDER.length : order;
    if (!groups.has(safeOrder)) {
      groups.set(safeOrder, {
        label: localBehaviorRuleTopicLabel(rule.topic, language),
        rules: [],
      });
    }
    groups.get(safeOrder).rules.push(rule);
  }
  const ordered = Array.from(groups.entries()).sort((a, b) => a[0] - b[0]);
  ordered.forEach(([, group], index) => {
    lines.push(`### ${group.label}`);
    for (const rule of group.rules) {
      lines.push(`- \`${rule.id}\` -> ${localRuleWhenThen(rule, language)}`);
    }
    if (index + 1 < ordered.length) lines.push("");
  });
  if (Array.isArray(runtimeRules) && runtimeRules.length > 0) {
    lines.push("", `### ${localLocalizedText(language, {
      en: "Dialog-local rules taught in this conversation",
      ru: "Правила, изученные в этом диалоге",
      hi: "इस संवाद में सिखाए गए स्थानीय नियम",
      zh: "本对话中学到的局部规则",
    })}`);
    for (const rule of runtimeRules) {
      lines.push(`- \`${rule.id}\` -> ${localRuntimeRuleWhenThen(rule, language)}`);
    }
  }
  lines.push(...localBehaviorRuleListFooter(language));
  return lines.join("\n");
}

export function localBehaviorRulesCount(runtimeRules, language = "en") {
  const runtimeRuleCount = Array.isArray(runtimeRules) ? runtimeRules.length : 0;
  const builtInCount = localBehaviorRuleRecords().length;
  const total = builtInCount + runtimeRuleCount;
  const summary = localLocalizedText(language, {
    en: `Total behavior rules: ${total} (built-in: ${builtInCount}; dialog-local: ${runtimeRuleCount}).`,
    ru: `Всего правил: ${total} (встроенных: ${builtInCount}; изученных в этом диалоге: ${runtimeRuleCount}).`,
    hi: `कुल व्यवहार नियम: ${total} (built-in: ${builtInCount}; dialog-local: ${runtimeRuleCount}).`,
    zh: `行为规则总数：${total}（内置：${builtInCount}；本对话：${runtimeRuleCount}）。`,
  });
  const reasoning = localLocalizedText(language, {
    en: "Reasoning: I count the built-in behavior-rule catalog and add dialog-local rules compiled from earlier user turns.",
    ru: "Рассуждение: я считаю встроенный каталог правил поведения и добавляю правила, скомпилированные из предыдущих сообщений пользователя.",
    hi: "Reasoning: मैं built-in behavior-rule catalog गिनता हूँ और पहले user turns से compiled dialog-local rules जोड़ता हूँ.",
    zh: "Reasoning：我统计内置行为规则目录，并加上从此前用户消息编译出的本对话规则。",
  });
  return [
    summary,
    "",
    reasoning,
    "",
    "```links",
    "behavior_rules_count",
    `  built_in_rules "${builtInCount}"`,
    `  dialog_local_rules "${runtimeRuleCount}"`,
    `  total_rules "${total}"`,
    '  algorithm "localBehaviorRuleRecords + localCollectRuntimeRules(history:user)"',
    "```",
  ].join("\n");
}

export function localBehaviorRuleDetail(rule, language = "en") {
  const label = localRuleLabel(rule, language);
  const whenThen = localRuleWhenThen(rule, language);
  const matches = localRuleMatches(rule, language);
  const response = localRuleResponse(rule, language);
  const changeHint = localLocalizedText(language, {
    en: "To change this behavior in the current dialog, send: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.",
    ru: "Чтобы изменить это поведение в текущем диалоге, отправьте: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``.",
    hi: "इस व्यवहार को वर्तमान संवाद में बदलने के लिए भेजें: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. दूसरा रूप: ``When I say `your prompt`, answer `your answer` ``.",
    zh: "要在当前对话中改变此行为，请发送：``当 `你的提示` 时 `你的回答` ``。也可以发送：``When I say `your prompt`, answer `your answer` ``。",
  });
  return [
    label,
    "",
    whenThen || "",
    "",
    "```links",
    rule.id,
    `  topic "${(rule.topic || "").replaceAll('"', '\\"')}"`,
    `  intent "${rule.intent}"`,
    `  matches "${matches.replaceAll('"', '\\"')}"`,
    `  response "${response.replaceAll('"', '\\"')}"`,
    `  source "${rule.source}"`,
    `  when_then "${(whenThen || "").replaceAll('"', '\\"')}"`,
    "```",
    "",
    changeHint,
  ].join("\n");
}

function localCleanRuleQuery(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[\s`"':._,\-?!]+|[\s`"':._,\-?!]+$/g, "")
    .toLowerCase();
}

export function localDetailQuery(prompt) {
  const lower = String(prompt || "").toLowerCase();
  for (const prefix of ["show behavior rule", "read behavior rule", "show rule", "read rule"]) {
    if (lower.startsWith(prefix)) {
      return localCleanRuleQuery(String(prompt || "").slice(prefix.length));
    }
  }
  if (lower.includes("rule_unknown")) return "unknown";
  return "";
}

export function localFindBehaviorRule(query) {
  const cleaned = localCleanRuleQuery(query);
  const withoutPrefix = cleaned.startsWith("rule_") ? cleaned.slice(5) : cleaned;
  return localBehaviorRuleRecords().find(
    (rule) =>
      rule.id === cleaned ||
      rule.id === `rule_${withoutPrefix}` ||
      rule.intent === cleaned ||
      rule.intent === withoutPrefix,
  );
}

export function localRuntimeRuleForPrompt(prompt, history) {
  const normalizedPrompt = normalizePrompt(prompt);
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index] || {};
    if (String(turn.role || "").toLowerCase() !== "user") continue;
    const rule = localRuntimeRuleFromText(turn.content);
    if (rule && normalizePrompt(rule.trigger) === normalizedPrompt) {
      return rule;
    }
  }
  return null;
}

export function localCollectRuntimeRules(history) {
  const turns = Array.isArray(history) ? history : [];
  const seen = new Set();
  const rules = [];
  for (const turn of turns) {
    if (String((turn || {}).role || "").toLowerCase() !== "user") continue;
    const rule = localRuntimeRuleFromText((turn || {}).content);
    if (rule && !seen.has(rule.id)) {
      seen.add(rule.id);
      rules.push(rule);
    }
  }
  return rules;
}

const LOCAL_BEHAVIOR_RULES_LIST_PATTERNS = [
  "show behavior rules",
  "show rules",
  "show list of your rules",
  "list your rules",
  "покажи правила поведения",
  "покажи правила",
  "покажи список своих правил",
  "перечисли свои правила",
  "व्यवहार के नियम सूचीबद्ध करें",
  "नियम दिखाओ",
  "अपने नियमों की सूची दिखाओ",
  "अपने नियम गिनाओ",
  "列出行为规则",
  "显示规则",
  "显示你的规则列表",
  "列出你的规则",
];

function matchesLocalBehaviorRulesListPattern(normalized) {
  return LOCAL_BEHAVIOR_RULES_LIST_PATTERNS.some((pattern) => {
    const text = normalizePrompt(pattern);
    return text && (normalized === text || normalized.includes(text));
  });
}

export function localIsBehaviorRulesList(normalized) {
  return (
    matchesLocalBehaviorRulesListPattern(normalized) ||
    normalized.includes("list behavior rules") ||
    normalized.includes("list all behavior rules") ||
    normalized.includes("show behavior rules") ||
    isSupportedLanguageBehaviorRulesListQuery(normalized) ||
    normalized.includes("список правил поведения")
  );
}

export function localIsBehaviorRulesCount(normalized, history) {
  const priorRuleListContext = localPriorBehaviorRulesListContext(history);
  const english =
    localContainsAny(normalized, ["rules", "rule list", "rules list"]) &&
    localContainsAny(normalized, ["how many", "number of", "count"]) &&
    (localContainsAny(normalized, ["all", "total", "there", "existing", "current", "behavior"]) ||
      priorRuleListContext);
  const russian =
    localContainsAny(normalized, ["правил", "правила"]) &&
    normalized.includes("сколько") &&
    (localContainsAny(normalized, ["всего", "все", "текущих", "поведения"]) ||
      priorRuleListContext);
  const hindi =
    localContainsAny(normalized, ["नियम", "नियमों"]) &&
    normalized.includes("कितने") &&
    (localContainsAny(normalized, ["कुल", "सभी", "व्यवहार"]) || priorRuleListContext);
  const chinese =
    localContainsAny(normalized, ["规则", "規則"]) &&
    normalized.includes("多少") &&
    (localContainsAny(normalized, ["总共", "总共有", "所有", "行为", "行為"]) ||
      priorRuleListContext);

  return english || russian || hindi || chinese;
}

function localPriorBehaviorRulesListContext(history) {
  const turns = Array.isArray(history) ? history : [];
  return turns.some((turn) => {
    const role = String((turn || {}).role || "").toLowerCase();
    const content = String((turn || {}).content || "");
    if (role === "user") return localIsBehaviorRulesList(normalizePrompt(content));
    return (
      role === "assistant" &&
      content.includes("rule_greeting") &&
      content.includes("rule_unknown")
    );
  });
}

function isSupportedLanguageBehaviorRulesListQuery(normalized) {
  return (
    isEnglishBehaviorRulesListQuery(normalized) ||
    isRussianBehaviorRulesListQuery(normalized) ||
    isHindiBehaviorRulesListQuery(normalized) ||
    isChineseBehaviorRulesListQuery(normalized)
  );
}

function isEnglishBehaviorRulesListQuery(normalized) {
  const mentionsRules =
    normalized.includes("rules") ||
    normalized.includes("rule list") ||
    normalized.includes("rules list");
  const asksToList =
    normalized.includes("list") ||
    normalized.includes("show") ||
    normalized.includes("what") ||
    normalized.includes("which");
  const pointsAtAssistantRules =
    normalized.includes("behavior") ||
    normalized.includes("your") ||
    normalized.includes("own") ||
    normalized.includes("current") ||
    normalized.includes("existing");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isRussianBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("правил") || normalized.includes("правила");
  const asksToList =
    normalized.includes("список") ||
    normalized.includes("перечисли") ||
    normalized.includes("покажи") ||
    normalized.includes("какие");
  const pointsAtAssistantRules =
    normalized.includes("поведения") ||
    normalized.includes("своих") ||
    normalized.includes("свои") ||
    normalized.includes("твоих") ||
    normalized.includes("твои") ||
    normalized.includes("собственные") ||
    normalized.includes("список правил");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isHindiBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("नियम") || normalized.includes("नियमों");
  const asksToList =
    normalized.includes("सूची") ||
    normalized.includes("सूचीबद्ध") ||
    normalized.includes("दिखाओ") ||
    normalized.includes("दिखाएं") ||
    normalized.includes("बताओ") ||
    normalized.includes("गिनाओ") ||
    normalized.includes("कौन");
  const pointsAtAssistantRules =
    normalized.includes("व्यवहार") ||
    normalized.includes("अपने") ||
    normalized.includes("तुम्हारे") ||
    normalized.includes("आपके") ||
    normalized.includes("नियमों की सूची");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isChineseBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("规则") || normalized.includes("規則");
  const asksToList =
    normalized.includes("列出") ||
    normalized.includes("显示") ||
    normalized.includes("顯示") ||
    normalized.includes("展示") ||
    normalized.includes("哪些") ||
    normalized.includes("什么");
  const pointsAtAssistantRules =
    normalized.includes("行为") ||
    normalized.includes("行為") ||
    normalized.includes("你的") ||
    normalized.includes("您的") ||
    normalized.includes("自己") ||
    normalized.includes("规则列表") ||
    normalized.includes("規則列表");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}
