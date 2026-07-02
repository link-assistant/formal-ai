// Worker module 4 of 21. Loaded by ../formal_ai_worker.js.
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

// Issue #386 translation roles — mirror the ROLE_TRANSLATION_* constants in
// src/seed/roles.rs. Each role's slot-marked surface words live in
// data/seed/meanings-translation.lino (loaded into MEANINGS_LINO); the
// detection and extraction helpers ask the lexicon for those forms by meaning,
// slot, and script instead of hardcoding a per-language phrase list. This is
// the JS mirror of src/translation/language_markers.rs and
// src/translation/prompt.rs.
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
// Project the translation-extraction markers out of the meaning lexicon, once.
// Each field is a semantic role narrowed to the slot and script its strategy
// needs, in declaration order — the code names a role and a shape, never a
// surface word. Mirrors markers() in src/translation/prompt.rs.
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
  return extractChineseUnquotedTranslationSurface(trimmed, lower);
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
  const cleaned = String(candidate || "").trim();
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

function behaviorRuleRecords() {
  const greeting = answerFor("greeting", "en");
  const farewell = answerFor("farewell", "en");
  const assistantFreeTime = answerFor("assistant_free_time", "en");
  const identity = answerFor("identity", "en");
  const assistantName = answerFor("assistant_name", "en");
  return [
    {
      id: "rule_greeting",
      topic: "greetings",
      intent: "greeting",
      label: "Greeting rule",
      matches: "`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases",
      response: greeting,
      source: "data/seed/intent-routing.lino + multilingual responses",
      whenThen: `When the user says \`Hi\`, \`Hello\`, or \`Hey\` then respond with \`${greeting}\`.`,
    },
    {
      id: "rule_assistant_free_time",
      topic: "small_talk",
      intent: "assistant_free_time",
      label: "Assistant free-time rule",
      matches:
        "`What do you do in your free time?`, `Что делаешь в свободное время?`, and equivalent small-talk seed phrases",
      response: assistantFreeTime,
      source: "data/seed/intent-routing.lino + multilingual responses",
      whenThen: `When the user asks what I do in free time then respond with \`${assistantFreeTime}\`.`,
    },
    {
      id: "rule_farewell",
      topic: "farewells",
      intent: "farewell",
      label: "Farewell rule",
      matches: "`bye`, `goodbye`, `poka`, and multilingual farewell seed phrases",
      response: farewell,
      source: "data/seed/intent-routing.lino + multilingual responses",
      whenThen: `When the user says \`bye\`, \`goodbye\`, or \`пока\` then respond with \`${farewell}\`.`,
    },
    {
      id: "rule_identity",
      topic: "identity",
      intent: "identity",
      label: "Identity rule",
      matches: "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
      response: identity,
      source: "data/seed/identity.lino + multilingual responses",
      whenThen: `When the user asks \`Who are you?\` or \`Кто ты?\` then respond with \`${identity}\`.`,
    },
    {
      id: "rule_assistant_name",
      topic: "assistant_name",
      intent: "assistant_name",
      label: "Assistant name rule",
      matches: "`What is your name?`, `Как твое имя?`, and equivalent name prompts",
      response: assistantName,
      source: "data/seed/intent-routing.lino + multilingual responses",
      whenThen: `When the user asks \`What is your name?\` or \`Как твое имя?\` then respond with \`${assistantName}\`, unless the assistant name setting is configured.`,
    },
    {
      id: "rule_capabilities",
      topic: "capabilities",
      intent: "capabilities",
      label: "Capabilities rule",
      matches: "`What can you do?`, `Что ты умеешь?`, and equivalent capability prompts",
      response: "Lists the supported symbolic chat capabilities.",
      source: "src/solver_handlers/user_intent.rs",
      whenThen:
        "When the user asks `What can you do?` or `Что ты умеешь?` then respond with the multilingual capability listing.",
    },
    {
      id: "rule_write_program",
      topic: "write_program",
      intent: "write_program",
      label: "Program template rule",
      // Built from the live catalog so the advertised tasks stay in lock-step
      // with `WRITE_PROGRAM_TASKS` (mirrors `supported_program_tasks` on the
      // Rust side, issue #330).
      matches:
        "`write_program(language, task)` with languages " +
        `${Object.keys(WRITE_PROGRAM_LANGUAGES).join(", ")} and tasks ` +
        `${Object.keys(WRITE_PROGRAM_TASKS).join(", ")}`,
      response: "Returns a minimal program from the parameterized template catalog.",
      source: "data/seed/hello-world-programs.lino + src/coding/catalog.rs",
      whenThen:
        "When the user requests a program with supported `language` and `task` parameters then respond with the matching template through the single `write_program` intent.",
    },
    {
      id: "rule_unknown",
      topic: "unknown_fallback",
      intent: "unknown",
      label: "Unknown fallback rule",
      matches: "Any prompt that no earlier rule or handler can answer",
      response: answerFor("unknown", "en"),
      source: "data/seed/multilingual-responses.lino",
      whenThen:
        "When no earlier rule or handler matches the prompt then respond with the multilingual unknown-intent guide (`List behavior rules`, `Show behavior rule`, `When I say … answer …`, `Report issue`, `Export memory`).",
    },
  ];
}

const BEHAVIOR_RULE_TOPIC_ORDER = [
  "greetings",
  "small_talk",
  "farewells",
  "identity",
  "assistant_name",
  "capabilities",
  "write_program",
  "unknown_fallback",
];

function localizedText(language, values) {
  return values[language] || values.en;
}

function behaviorRuleTopicLabel(topic, language) {
  switch (topic) {
    case "greetings":
      return localizedText(language, {
        en: "Greetings",
        ru: "Приветствия",
        hi: "अभिवादन",
        zh: "问候",
      });
    case "small_talk":
      return localizedText(language, {
        en: "Small talk",
        ru: "Светская беседа",
        hi: "हल्की बातचीत",
        zh: "闲聊",
      });
    case "farewells":
      return localizedText(language, {
        en: "Farewells",
        ru: "Прощания",
        hi: "विदाई",
        zh: "告别",
      });
    case "identity":
      return localizedText(language, {
        en: "Identity",
        ru: "Идентичность",
        hi: "पहचान",
        zh: "身份",
      });
    case "assistant_name":
      return localizedText(language, {
        en: "Assistant name",
        ru: "Имя ассистента",
        hi: "सहायक का नाम",
        zh: "助手名称",
      });
    case "capabilities":
      return localizedText(language, {
        en: "Capabilities",
        ru: "Возможности",
        hi: "क्षमताएँ",
        zh: "能力",
      });
    case "write_program":
      return localizedText(language, {
        en: "Program templates",
        ru: "Шаблоны программ",
        hi: "Program templates",
        zh: "程序模板",
      });
    case "unknown_fallback":
      return localizedText(language, {
        en: "Unknown fallback",
        ru: "Резервный ответ",
        hi: "अज्ञात अनुरोध का वैकल्पिक उत्तर",
        zh: "未知请求回退",
      });
    default:
      return localizedText(language, {
        en: "Other",
        ru: "Другое",
        hi: "अन्य",
        zh: "其他",
      });
  }
}

function behaviorRuleTopicOrder(topic) {
  const index = BEHAVIOR_RULE_TOPIC_ORDER.indexOf(topic);
  return index === -1 ? BEHAVIOR_RULE_TOPIC_ORDER.length : index;
}

function behaviorRuleListIntro(language) {
  return localizedText(language, {
    en: "Behavior rules I can inspect in this dialog (grouped by topic, each shown as a `When X then Y` statement):",
    ru: "Правила поведения, которые я могу показать в этом диалоге (сгруппированы по темам; каждое показано как инструкция `Когда X тогда Y`):",
    hi: "व्यवहार नियम जिन्हें मैं इस संवाद में दिखा सकता हूँ (विषय के अनुसार समूहित; हर नियम `जब X तब Y` कथन के रूप में है):",
    zh: "我可以查看的行为规则（按主题分组；每条都显示为 `当 X 时 Y` 语句）：",
  });
}

function behaviorRuleCounts(runtimeRules) {
  const runtime = Array.isArray(runtimeRules) ? runtimeRules.length : 0;
  const builtIn = behaviorRuleRecords().length;
  return { builtIn, runtime, total: builtIn + runtime };
}

function renderBehaviorRuleCount(runtimeRules, language = "en") {
  const { builtIn, runtime, total } = behaviorRuleCounts(runtimeRules);
  const summary = localizedText(language, {
    en: `Total behavior rules: ${total} (built-in: ${builtIn}; dialog-local: ${runtime}).`,
    ru: `Всего правил: ${total} (встроенных: ${builtIn}; изученных в этом диалоге: ${runtime}).`,
    hi: `कुल व्यवहार नियम: ${total} (built-in: ${builtIn}; dialog-local: ${runtime}).`,
    zh: `行为规则总数：${total}（内置：${builtIn}；本对话：${runtime}）。`,
  });
  const reasoning = localizedText(language, {
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
    `  built_in_rules "${builtIn}"`,
    `  dialog_local_rules "${runtime}"`,
    `  total_rules "${total}"`,
    '  algorithm "behavior_rule_records + collect_runtime_rules(prior_turn:user)"',
    "```",
  ].join("\n");
}

function renderBehaviorRulesBrief(runtimeRules, language = "en") {
  const { builtIn, runtime, total } = behaviorRuleCounts(runtimeRules);
  const groups = localizedText(language, {
    en: "greetings, farewells, small talk, identity, assistant name, capabilities, program templates, and the unknown fallback",
    ru: "приветствия, прощания, светская беседа, идентичность, имя ассистента, возможности, шаблоны программ и резервный ответ",
    hi: "अभिवादन, विदाई, हल्की बातचीत, पहचान, सहायक का नाम, क्षमताएँ, program templates, और unknown fallback",
    zh: "问候、告别、闲聊、身份、助手名称、能力、程序模板和未知请求回退",
  });
  return localizedText(language, {
    en: `Briefly: ${total} behavior rules (${builtIn} built-in, ${runtime} dialog-local): ${groups}.`,
    ru: `Всего: ${total} правил поведения (${builtIn} встроенных, ${runtime} из диалога). Кратко: ${groups}.`,
    hi: `कुल: ${total} व्यवहार नियम (${builtIn} built-in, ${runtime} dialog-local). संक्षेप में: ${groups}.`,
    zh: `总计：${total} 条行为规则（${builtIn} 条内置，${runtime} 条来自对话）。简要：${groups}。`,
  });
}

function runtimeRulesHeading(language) {
  return localizedText(language, {
    en: "Dialog-local rules taught in this conversation",
    ru: "Правила, изученные в этом диалоге",
    hi: "इस संवाद में सिखाए गए स्थानीय नियम",
    zh: "本对话中学到的局部规则",
  });
}

function behaviorRuleListFooter(language) {
  if (language === "ru") {
    return [
      "",
      "Прочитать одно правило можно командой `Покажи правило unknown` или `Покажи правило rule_greeting`.",
      "Научить этот диалог можно так: ``Когда `ваш запрос` тогда `ваш ответ` ``. Другие формы: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``; ``Если я спрошу `ваш запрос`, ответь `ваш ответ` ``; ``Когда `ваш запрос` делай `ваш ответ` ``.",
      "Многоязычные формы: английская ``When `X` then `Y` ``, хинди ``जब `X` तब `Y` ``, китайская ``当 `X` 时 `Y` ``.",
      "Запись добавляется только в конец: экспортируйте память, чтобы сохранить сообщение с правилом вместе с диалогом.",
    ];
  }
  if (language === "hi") {
    return [
      "",
      "एक नियम पढ़ने के लिए `Show behavior rule unknown` या `Show behavior rule rule_greeting` भेजें.",
      "इस संवाद को सिखाएँ: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. अन्य रूप: ``When I say `your prompt`, answer `your answer` ``; ``If I ask `your prompt`, reply `your answer` ``; ``जब `आपका प्रश्न` तो `आपका उत्तर` ``.",
      "बहुभाषी रूप: रूसी ``Когда `X` тогда `Y` ``, अंग्रेज़ी ``When `X` then `Y` ``, चीनी ``当 `X` 时 `Y` ``.",
      "लेखन केवल append-only है: नियम संदेश को संवाद के साथ रखने के लिए memory export करें.",
    ];
  }
  if (language === "zh") {
    return [
      "",
      "要读取一条规则，请发送 `Show behavior rule unknown` 或 `Show behavior rule rule_greeting`。",
      "可以这样教当前对话：``当 `你的提示` 时 `你的回答` ``。等价形式：``When I say `your prompt`, answer `your answer` ``；``If I ask `your prompt`, reply `your answer` ``；``当 `你的提示` 则 `你的回答` ``。",
      "多语言形式：俄语 ``Когда `X` тогда `Y` ``，印地语 ``जब `X` तब `Y` ``，英语 ``When `X` then `Y` ``。",
      "写入是 append-only：导出 memory 可把这条规则消息随对话一起保存。",
    ];
  }
  return [
    "",
    "Read one with `Show behavior rule unknown` or `Show behavior rule rule_greeting`.",
    "Teach this dialog with: ``When `your prompt` then `your answer` ``. Equivalent forms: ``When I say `your prompt`, answer `your answer` ``; ``If I ask `your prompt`, reply `your answer` ``; ``When `your prompt` do `your answer` ``.",
    "Multilingual forms: Russian ``Когда `X` тогда `Y` `` / ``Когда `X` делай `Y` ``, Hindi ``जब `X` तब `Y` ``, Chinese ``当 `X` 时 `Y` ``.",
    "The write is append-only: export memory to preserve the rule message with the dialog.",
  ];
}

function localizedRuleResponse(rule, language) {
  if (rule.id === "rule_write_program") {
    return localizedText(language, {
      en: "Returns a minimal program from the parameterized template catalog.",
      ru: "Возвращает минимальную программу из параметризованного каталога шаблонов.",
      hi: "parameterized template catalog से minimal program लौटाता है.",
      zh: "从参数化模板目录返回一个最小程序。",
    });
  }
  switch (rule.id) {
    case "rule_greeting":
      return answerFor("greeting", language);
    case "rule_farewell":
      return answerFor("farewell", language);
    case "rule_assistant_free_time":
      return answerFor("assistant_free_time", language);
    case "rule_identity":
      return answerFor("identity", language);
    case "rule_assistant_name":
      return localizedText(language, {
        en: "Returns the assistant-name answer; browser surfaces can override it from the assistant name setting.",
        ru: "Возвращает ответ об имени ассистента; браузерные поверхности могут переопределить его настройкой имени ассистента.",
        hi: "assistant-name उत्तर लौटाता है; browser surfaces assistant name setting से इसे बदल सकते हैं.",
        zh: "返回助手名称回答；浏览器界面可通过助手名称设置覆盖它。",
      });
    case "rule_capabilities":
      return localizedText(language, {
        en: "Lists the supported symbolic chat capabilities.",
        ru: "Перечисляет поддерживаемые возможности символьного чата.",
        hi: "समर्थित symbolic chat क्षमताओं को सूचीबद्ध करता है.",
        zh: "列出支持的符号聊天能力。",
      });
    case "rule_unknown":
      return answerFor("unknown", language);
    default:
      return rule.response;
  }
}

function localizedRuleLabel(rule, language) {
  if (rule.id === "rule_write_program") {
    return localizedText(language, {
      en: "Program template rule",
      ru: "Правило шаблона программы",
      hi: "Program template rule",
      zh: "程序模板规则",
    });
  }
  const labels = {
    rule_greeting: {
      en: "Greeting rule",
      ru: "Правило приветствия",
      hi: "अभिवादन नियम",
      zh: "问候规则",
    },
    rule_farewell: {
      en: "Farewell rule",
      ru: "Правило прощания",
      hi: "विदाई नियम",
      zh: "告别规则",
    },
    rule_assistant_free_time: {
      en: "Assistant free-time rule",
      ru: "Правило свободного времени ассистента",
      hi: "सहायक खाली समय नियम",
      zh: "助手空闲时间规则",
    },
    rule_identity: {
      en: "Identity rule",
      ru: "Правило идентичности",
      hi: "पहचान नियम",
      zh: "身份规则",
    },
    rule_assistant_name: {
      en: "Assistant name rule",
      ru: "Правило имени ассистента",
      hi: "सहायक नाम नियम",
      zh: "助手名称规则",
    },
    rule_capabilities: {
      en: "Capabilities rule",
      ru: "Правило возможностей",
      hi: "क्षमता नियम",
      zh: "能力规则",
    },
    rule_unknown: {
      en: "Unknown fallback rule",
      ru: "Резервное правило для неизвестного запроса",
      hi: "अज्ञात अनुरोध का वैकल्पिक नियम",
      zh: "未知请求回退规则",
    },
  };
  return labels[rule.id] ? localizedText(language, labels[rule.id]) : rule.label;
}

function localizedRuleMatches(rule, language) {
  if (rule.id === "rule_write_program") {
    return localizedText(language, {
      en: "`write_program(language, task)` with supported languages and tasks",
      ru: "`write_program(language, task)` с поддерживаемыми языками и задачами",
      hi: "supported languages और tasks वाला `write_program(language, task)`",
      zh: "带受支持语言和任务的 `write_program(language, task)`",
    });
  }
  const matches = {
    rule_greeting: {
      en: "`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases",
      ru: "`Hi`, `Hello`, `Hey` и многоязычные seed-фразы приветствия",
      hi: "`Hi`, `Hello`, `Hey` और बहुभाषी greeting seed phrases",
      zh: "`Hi`、`Hello`、`Hey` 以及多语言问候 seed 短语",
    },
    rule_farewell: {
      en: "`bye`, `goodbye`, `poka`, and multilingual farewell seed phrases",
      ru: "`bye`, `goodbye`, `poka` и многоязычные seed-фразы прощания",
      hi: "`bye`, `goodbye`, `poka` और बहुभाषी farewell seed phrases",
      zh: "`bye`、`goodbye`、`poka` 以及多语言告别 seed 短语",
    },
    rule_assistant_free_time: {
      en: "`What do you do in your free time?`, `Что делаешь в свободное время?`, and equivalent small-talk seed phrases",
      ru: "`What do you do in your free time?`, `Что делаешь в свободное время?` и равнозначные seed-фразы светской беседы",
      hi: "`What do you do in your free time?`, `Что делаешь в свободное время?` और समान small-talk seed phrases",
      zh: "`What do you do in your free time?`、`Что делаешь в свободное время?` 以及等价闲聊 seed 短语",
    },
    rule_identity: {
      en: "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
      ru: "`Who are you?`, `Кто ты?` и равнозначные вопросы об идентичности",
      hi: "`Who are you?`, `Кто ты?` और समान identity prompts",
      zh: "`Who are you?`、`Кто ты?` 以及等价身份提示",
    },
    rule_assistant_name: {
      en: "`What is your name?`, `Как тебя зовут?`, and equivalent name prompts",
      ru: "`What is your name?`, `Как тебя зовут?` и равнозначные вопросы об имени",
      hi: "`What is your name?`, `Как тебя зовут?` और समान name prompts",
      zh: "`What is your name?`、`Как тебя зовут?` 以及等价名称提示",
    },
    rule_capabilities: {
      en: "`What can you do?`, `Что ты умеешь?`, and equivalent capability prompts",
      ru: "`What can you do?`, `Что ты умеешь?` и равнозначные вопросы о возможностях",
      hi: "`What can you do?`, `Что ты умеешь?` और समान capability prompts",
      zh: "`What can you do?`、`Что ты умеешь?` 以及等价能力提示",
    },
    rule_unknown: {
      en: "Any prompt that no earlier rule or handler can answer",
      ru: "Любой запрос, на который не ответило более раннее правило или обработчик",
      hi: "कोई भी prompt जिसका उत्तर पहले का rule या handler नहीं दे सकता",
      zh: "任何前面的规则或处理器无法回答的提示",
    },
  };
  return matches[rule.id] ? localizedText(language, matches[rule.id]) : rule.matches;
}

function localizedRuleWhenThen(rule, language) {
  if (rule.id === "rule_write_program") {
    if (language === "ru") {
      return "Когда пользователь просит программу с поддерживаемыми параметрами `language` и `task`, ответь соответствующим шаблоном через единое намерение `write_program`.";
    }
    if (language === "hi") {
      return "जब उपयोगकर्ता supported `language` और `task` parameters वाला program माँगे, तब single `write_program` intent से matching template दें.";
    }
    if (language === "zh") {
      return "当用户请求带受支持 `language` 和 `task` 参数的程序时，通过单个 `write_program` 意图选择匹配模板。";
    }
    return rule.whenThen;
  }
  const response = localizedRuleResponse(rule, language);
  if (rule.id === "rule_greeting") {
    if (language === "ru") return `Когда пользователь говорит \`Hi\`, \`Hello\`, \`Hey\` или многоязычную фразу приветствия, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता \`Hi\`, \`Hello\`, \`Hey\` या बहुभाषी greeting phrase कहे, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户说 \`Hi\`、\`Hello\`、\`Hey\` 或多语言问候短语时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_farewell") {
    if (language === "ru") return `Когда пользователь говорит \`bye\`, \`goodbye\`, \`poka\` или многоязычную фразу прощания, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता \`bye\`, \`goodbye\`, \`poka\` या बहुभाषी farewell phrase कहे, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户说 \`bye\`、\`goodbye\`、\`poka\` 或多语言告别短语时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_assistant_free_time") {
    if (language === "ru") return `Когда пользователь спрашивает, что я делаю в свободное время, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता पूछे कि मैं खाली समय में क्या करता हूँ, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户问我空闲时间做什么时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_identity") {
    if (language === "ru") return `Когда пользователь спрашивает \`Who are you?\` или \`Кто ты?\`, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता \`Who are you?\` या \`Кто ты?\` पूछे, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户问 \`Who are you?\` 或 \`Кто ты?\` 时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_assistant_name") {
    if (language === "ru") return "Когда пользователь спрашивает `What is your name?` или `Как тебя зовут?`, ответь сообщением об имени ассистента; если поверхность поддерживает настройку имени, включи настроенное имя.";
    if (language === "hi") return "जब उपयोगकर्ता `What is your name?` या `Как тебя зовут?` पूछे, तब assistant-name उत्तर दें; अगर surface में assistant-name setting है, तो configured name शामिल करें.";
    if (language === "zh") return "当用户问 `What is your name?` 或 `Как тебя зовут?` 时，回答助手名称；如果界面有助手名称设置，则包含配置的名称。";
  }
  if (rule.id === "rule_capabilities") {
    if (language === "ru") return "Когда пользователь спрашивает `What can you do?` или `Что ты умеешь?`, ответь многоязычным списком возможностей.";
    if (language === "hi") return "जब उपयोगकर्ता `What can you do?` या `Что ты умеешь?` पूछे, तब बहुभाषी capability listing दें.";
    if (language === "zh") return "当用户问 `What can you do?` 或 `Что ты умеешь?` 时，回答多语言能力列表。";
  }
  if (rule.id === "rule_unknown") {
    if (language === "ru") return "Когда ни одно более раннее правило или обработчик не подходит к запросу, ответь многоязычной подсказкой для неизвестного намерения (`Покажи правила`, `Покажи правило`, `Когда ... тогда ...`, `Сообщить о проблеме`, `Экспорт памяти`).";
    if (language === "hi") return "जब कोई पहले का rule या handler prompt से मेल न खाए, तब unknown-intent guide दें (`नियम दिखाएँ`, `rule दिखाएँ`, `जब ... तब ...`, `Report issue`, `Export memory`).";
    if (language === "zh") return "当前面的规则或处理器都不匹配提示时，回答未知意图指南（`显示规则`、`显示规则详情`、`当 ... 时 ...`、`报告问题`、`导出 memory`）。";
  }
  return rule.whenThen;
}

function runtimeRuleWhenThen(rule, language) {
  if (language === "ru") {
    return `Когда пользователь говорит \`${rule.trigger}\`, ответь \`${rule.answer}\`.`;
  }
  if (language === "hi") {
    return `जब उपयोगकर्ता \`${rule.trigger}\` कहे, तब \`${rule.answer}\` उत्तर दें.`;
  }
  if (language === "zh") {
    return `当用户说 \`${rule.trigger}\` 时，回答 \`${rule.answer}\`。`;
  }
  return `When the user says \`${rule.trigger}\` then respond with \`${rule.answer}\`.`;
}

function renderBehaviorRuleList(runtimeRules, language = "en") {
  const lines = [behaviorRuleListIntro(language), ""];
  const groups = new Map();
  for (const rule of behaviorRuleRecords()) {
    const order = behaviorRuleTopicOrder(rule.topic);
    if (!groups.has(order)) {
      groups.set(order, { label: behaviorRuleTopicLabel(rule.topic, language), rules: [] });
    }
    groups.get(order).rules.push(rule);
  }
  const ordered = Array.from(groups.entries()).sort((a, b) => a[0] - b[0]);
  ordered.forEach(([, group], index) => {
    lines.push(`### ${group.label}`);
    for (const rule of group.rules) {
      lines.push(`- \`${rule.id}\` -> ${localizedRuleWhenThen(rule, language)}`);
    }
    if (index + 1 < ordered.length) lines.push("");
  });
  if (Array.isArray(runtimeRules) && runtimeRules.length > 0) {
    lines.push("", `### ${runtimeRulesHeading(language)}`);
    for (const rule of runtimeRules) {
      lines.push(
        `- \`${rule.id}\` -> ${runtimeRuleWhenThen(rule, language)}`,
      );
    }
  }
  lines.push(...behaviorRuleListFooter(language));
  return lines.join("\n");
}

function renderBehaviorRuleDetail(rule, language = "en") {
  const label = localizedRuleLabel(rule, language);
  const whenThen = localizedRuleWhenThen(rule, language);
  const matches = localizedRuleMatches(rule, language);
  const response = localizedRuleResponse(rule, language);
  const changeHint = localizedText(language, {
    en: "To change this behavior in the current dialog, send: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.",
    ru: "Чтобы изменить это поведение в текущем диалоге, отправьте: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``.",
    hi: "इस व्यवहार को वर्तमान संवाद में बदलने के लिए भेजें: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. दूसरा रूप: ``When I say `your prompt`, answer `your answer` ``.",
    zh: "要在当前对话中改变此行为，请发送：``当 `你的提示` 时 `你的回答` ``。也可以发送：``When I say `your prompt`, answer `your answer` ``。",
  });
  return [
    label,
    "",
    whenThen,
    "",
    "```links",
    rule.id,
    `  topic "${escapeBehaviorRuleValue(rule.topic)}"`,
    `  intent "${escapeBehaviorRuleValue(rule.intent)}"`,
    `  matches "${escapeBehaviorRuleValue(matches)}"`,
    `  response "${escapeBehaviorRuleValue(response)}"`,
    `  source "${escapeBehaviorRuleValue(rule.source)}"`,
    `  when_then "${escapeBehaviorRuleValue(whenThen)}"`,
    "```",
    "",
    changeHint,
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
  const title = localizedText(language, {
    en: "Behavior rule recorded for this dialog.",
    ru: "Правило поведения записано для этого диалога.",
    hi: "इस संवाद के लिए व्यवहार नियम record किया गया.",
    zh: "已为本对话记录行为规则。",
  });
  const sendHint =
    language === "ru"
      ? `Отправьте \`${rule.trigger}\` сейчас, и я отвечу настроенным ответом. Экспортируйте память, чтобы сохранить это правило вместе с диалогом.`
      : language === "hi"
        ? `\`${rule.trigger}\` अभी भेजें और मैं configured response से उत्तर दूँगा. इस rule message को dialog के साथ रखने के लिए memory export करें.`
        : language === "zh"
          ? `现在发送 \`${rule.trigger}\`，我会使用配置的回答。导出 memory 可把这条规则消息随对话一起保存。`
          : `Send \`${rule.trigger}\` now and I will answer with the configured response. Export memory to keep this rule message with the dialog.`;
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

function cleanConversationTopic(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[`"':._,\-\s!?]+|[`"':._,\-\s!?]+$/gu, "");
}
