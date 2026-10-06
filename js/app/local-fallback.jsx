// Local fallback answerer used when the worker is unavailable: behaviour
// rules, courtesy replies and varied unknown-answer openers.

import {
  ASSISTANT_FREE_TIME_ANSWER, COURTESY_ACKNOWLEDGEMENTS, COURTESY_FOLLOW_UPS, IDENTITY_ANSWER,
  UNKNOWN_ANSWER,
} from "./app-constants.jsx";
import {
  localBehaviorRuleDetail, localBehaviorRuleId, localBehaviorRulesCount, localBehaviorRulesList,
  localCollectRuntimeRules, localDetailQuery, localFindBehaviorRule, localIsBehaviorRulesCount,
  localIsBehaviorRulesList, localLocalizedText, localRuleResponse, localRuntimeRuleForPrompt,
  localRuntimeRuleFromText, localRuntimeRuleWhenThen,
} from "./local-behavior-rules.jsx";
import {
  isAssistantNamePrompt, isIdentityPrompt, isLocalAssistantFreeTimePrompt,
  localAssistantNameAnswer, localPromptLanguage, normalizePrompt,
} from "./local-prompts.jsx";
import {
  localArchitectureExplanation, localConversationTopic, localConversationTopicContent,
  localIsArchitectureQuestion, localIsKnownFactQuery, localIsSelfFactQuery,
  localIsSelfIntroductionQuery, localKnownFacts, localSelfAwarenessLanguage, localSelfFacts,
  localSelfIntroductionContent,
} from "./local-self-knowledge.jsx";
import { PREFERENCE_DEFAULTS, normalizeSliderPreference } from "./preferences.jsx";

function tryLocalBehaviorRules(prompt, normalized, history, preferences = {}) {
  const language = localSelfAwarenessLanguage(prompt, normalized);
  const updateRule = localRuntimeRuleFromText(prompt);
  if (updateRule) {
    const whenThen = localRuntimeRuleWhenThen(updateRule, language);
    const title = localLocalizedText(language, {
      en: "Behavior rule recorded for this dialog.",
      ru: "Правило поведения записано для этого диалога.",
      hi: "इस संवाद के लिए व्यवहार नियम record किया गया.",
      zh: "已为本对话记录行为规则。",
    });
    const sendHint =
      language === "ru"
        ? `Отправьте \`${updateRule.trigger}\` сейчас, и я отвечу настроенным ответом. Экспортируйте память, чтобы сохранить это правило вместе с диалогом.`
        : language === "hi"
          ? `\`${updateRule.trigger}\` अभी भेजें और मैं configured response से उत्तर दूँगा. इस rule message को dialog के साथ रखने के लिए memory export करें.`
          : language === "zh"
            ? `现在发送 \`${updateRule.trigger}\`，我会使用配置的回答。导出 memory 可把这条规则消息随对话一起保存。`
            : `Send \`${updateRule.trigger}\` now and I will answer with the configured response. Export memory to keep this rule message with the dialog.`;
    return {
      intent: "behavior_rule_update",
      content: [
        title,
        "",
        whenThen,
        "",
        "```links",
        updateRule.id,
        '  type "behavior_rule_runtime"',
        `  match_prompt "${updateRule.trigger.replaceAll('"', '\\"')}"`,
        `  answer "${updateRule.answer.replaceAll('"', '\\"')}"`,
        `  when_then "${whenThen.replaceAll('"', '\\"')}"`,
        '  source "user_message"',
        "```",
        "",
        sendHint,
      ].join("\n"),
    };
  }
  const runtimeRules = localCollectRuntimeRules(history);
  if (localIsBehaviorRulesCount(normalized, history)) {
    return {
      intent: "behavior_rules_count",
      content: localBehaviorRulesCount(runtimeRules, language),
    };
  }
  if (localIsBehaviorRulesList(normalized)) {
    return { intent: "behavior_rules_list", content: localBehaviorRulesList(runtimeRules, language) };
  }
  const query = localDetailQuery(prompt);
  if (query) {
    const rule = localFindBehaviorRule(query);
    if (rule) {
      return { intent: "behavior_rule_detail", content: localBehaviorRuleDetail(rule, language) };
    }
  }
  if (localIsSelfIntroductionQuery(normalized)) {
    const language = localSelfAwarenessLanguage(prompt, normalized);
    return {
      intent: "identity",
      content: localSelfIntroductionContent(language, preferences),
    };
  }
  if (localIsSelfFactQuery(normalized)) {
    return { intent: "self_facts", content: localSelfFacts(preferences) };
  }
  if (localIsKnownFactQuery(normalized)) {
    const language = localSelfAwarenessLanguage(prompt, normalized);
    return { intent: "known_facts", content: localKnownFacts(language, preferences) };
  }
  const topic = localConversationTopic(prompt, normalized);
  if (topic) {
    const language = localSelfAwarenessLanguage(prompt, normalized);
    return { intent: "conversation_topic", content: localConversationTopicContent(topic, language) };
  }
  const runtimeRule = localRuntimeRuleForPrompt(prompt, history);
  if (runtimeRule) {
    return { intent: "behavior_rule_custom", content: runtimeRule.answer };
  }
  return null;
}

function chooseVariant(variants, randomize) {
  if (!Array.isArray(variants) || variants.length === 0) return "";
  if (!randomize || variants.length === 1) return variants[0];
  return variants[Math.floor(Math.random() * variants.length)] || variants[0];
}

function shouldIncludeCourtesyFollowUp(probability, randomize) {
  const normalized = normalizeSliderPreference(
    probability,
    PREFERENCE_DEFAULTS.followUpProbability,
  );
  if (normalized <= 0) return false;
  if (normalized >= 1) return true;
  if (!randomize) return normalized >= 0.5;
  return Math.random() < normalized;
}

function courtesyResponseContent(preferences = {}) {
  const temperature = normalizeSliderPreference(
    preferences.temperature,
    PREFERENCE_DEFAULTS.temperature,
  );
  const randomize = temperature > 0;
  const acknowledgement = chooseVariant(COURTESY_ACKNOWLEDGEMENTS, randomize);
  if (!shouldIncludeCourtesyFollowUp(preferences.followUpProbability, randomize)) {
    return acknowledgement;
  }
  const followUp = chooseVariant(COURTESY_FOLLOW_UPS, randomize);
  return `${acknowledgement} ${followUp}`;
}

export function localFallbackAnswer(prompt, history = [], preferences = {}) {
  const normalized = normalizePrompt(prompt);
  const behaviorRule = tryLocalBehaviorRules(prompt, normalized, history, preferences);
  if (behaviorRule) {
    return behaviorRule;
  }
  if (localIsArchitectureQuestion(normalized)) {
    const language = /[\u0400-\u04ff]/u.test(String(prompt || "")) ? "ru" : "en";
    return { intent: "meta_explanation", content: localArchitectureExplanation(language) };
  }
  if (["hi", "hello", "hey"].includes(normalized)) {
    return {
      intent: "greeting",
      content: "Hi, how may I help you?",
    };
  }

  if (isLocalAssistantFreeTimePrompt(normalized)) {
    return {
      intent: "assistant_free_time",
      content: localRuleResponse(
        { id: "rule_assistant_free_time", response: ASSISTANT_FREE_TIME_ANSWER },
        localPromptLanguage(prompt),
      ),
    };
  }

  const courtesyResponses = new Set([
    "thanks",
    "thank you",
    "i am fine thank you",
    "i am fine thanks",
    "i m fine thank you",
    "i m fine thanks",
    "ого чето начал соображать",
    "ого чёто начал соображать",
    "ого чё то начал соображать",
    "ого что то начал соображать",
  ]);
  if (courtesyResponses.has(normalized)) {
    return {
      intent: "courtesy_response",
      content: courtesyResponseContent(preferences),
    };
  }

  if (isAssistantNamePrompt(normalized)) {
    return {
      intent: "assistant_name",
      content: localAssistantNameAnswer(prompt, preferences),
    };
  }

  if (isIdentityPrompt(normalized)) {
    return {
      intent: "identity",
      content: IDENTITY_ANSWER,
    };
  }

  return {
    intent: "unknown",
    content: localUnknownAnswerWithVariation(prompt),
  };
}

// Mirrors `src/engine.rs::UNKNOWN_OPENERS_EN` so the React fallback (used when
// the worker is unavailable, e.g. on `file://`) presents the same set of
// variations as the worker and Rust solver. Only the English pool is kept
// here because the React fallback never reaches non-English seeds.
const LOCAL_UNKNOWN_OPENERS = [
  "I don't know how to answer that yet.",
  "I didn't understand you.",
  "I'm not sure how to respond to that yet.",
  "I haven't learned to answer that yet.",
  "That one is new to me.",
];

function localSelectUnknownOpener(prompt) {
  const trimmed = String(prompt || "").trim();
  if (trimmed === "") return LOCAL_UNKNOWN_OPENERS[0];
  const id = localBehaviorRuleId(`unknown_opener\n${trimmed}`);
  const hex = id.split("_").pop() || "0";
  const value = parseInt(hex, 16) || 0;
  return LOCAL_UNKNOWN_OPENERS[value % LOCAL_UNKNOWN_OPENERS.length];
}

function localUnknownAnswerWithVariation(prompt) {
  const opener = localSelectUnknownOpener(prompt);
  const body = String(UNKNOWN_ANSWER || "").trimStart();
  for (const known of LOCAL_UNKNOWN_OPENERS) {
    if (body.startsWith(known)) {
      const rest = body.slice(known.length).trimStart();
      return rest ? `${opener} ${rest}` : opener;
    }
  }
  const idx = body.indexOf(". ");
  if (idx >= 0) {
    return `${opener} ${body.slice(idx + 2).trimStart()}`;
  }
  return `${opener} ${body}`;
}
