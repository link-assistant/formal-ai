// Local (no-worker) prompt recognition: identity, free-time and assistant-name
// questions answered by the in-page fallback.

import { ASSISTANT_NAME_ANSWER } from "./app-constants.jsx";
import { normalizeAssistantName } from "./interface-commands.jsx";

export function normalizePrompt(prompt) {
  return String(prompt || "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
}

export function isIdentityPrompt(normalized) {
  const tokens = normalized ? normalized.split(/\s+/) : [];
  const has = (token) => tokens.includes(token);
  return (
    [
      "who are you",
      "what are you",
      "who is formal ai",
      "what is formal ai",
      "who is formalai",
      "what is formalai",
      "tell me about yourself",
      "introduce yourself",
      "let s get acquainted",
      "lets get acquainted",
      "let us get acquainted",
      "let s get to know each other",
      "кто ты",
      "что ты",
      "расскажи о себе",
      "расскажи мне о себе",
      "расскажи про себя",
      "опиши себя",
      "представься",
      "давай знакомиться",
      "давай познакомимся",
      "давайте познакомимся",
      "चलो परिचय करते हैं",
      "आइए परिचय करें",
      "चलो एक दूसरे को जानें",
      "你是谁",
      "我们认识一下吧",
      "认识一下吧",
      "让我们认识一下",
    ].includes(normalized) ||
    (has("who") && has("you")) ||
    (has("what") && has("you")) ||
    ((has("who") || has("what")) && has("formal") && has("ai")) ||
    (has("tell") && has("yourself")) ||
    (has("introduce") && has("yourself")) ||
    (has("let") && has("s") && has("acquainted")) ||
    (has("lets") && has("acquainted")) ||
    (has("let") && has("us") && has("acquainted")) ||
    (has("know") && has("each") && has("other")) ||
    (has("кто") && has("ты")) ||
    (has("что") && has("ты")) ||
    (has("расскажи") && has("себе")) ||
    (has("опиши") && has("себя")) ||
    (has("давай") && has("знакомиться")) ||
    (has("давай") && has("познакомимся")) ||
    (has("давайте") && has("познакомимся")) ||
    (has("चलो") && has("परिचय")) ||
    (has("आइए") && has("परिचय"))
  );
}

export function isLocalAssistantFreeTimePrompt(normalized) {
  return [
    "what do you do in your free time",
    "what do you do in free time",
    "how do you spend your free time",
    "what do you do when you are not working",
    "что делаешь в свободное время",
    "что ты делаешь в свободное время",
    "чем занимаешься в свободное время",
    "чем ты занимаешься в свободное время",
    "что делаешь когда свободен",
    "खाली समय में क्या करते हो",
    "आप खाली समय में क्या करते हैं",
    "फुर्सत में क्या करते हो",
    "你空闲时间做什么",
    "你有空的时候做什么",
    "你业余时间做什么",
  ].includes(normalized);
}

export function localPromptLanguage(prompt) {
  const raw = String(prompt || "");
  if (/[\u0400-\u04ff]/u.test(raw)) return "ru";
  if (/[\u0900-\u097f]/u.test(raw)) return "hi";
  if (/[\u3400-\u9fff]/u.test(raw)) return "zh";
  return "en";
}

export function isAssistantNamePrompt(normalized) {
  const tokens = normalized ? normalized.split(/\s+/) : [];
  const has = (token) => tokens.includes(token);
  return (
    [
      "what is your name",
      "what s your name",
      "what's your name",
      "do you have a name",
      "what should i call you",
      "как твое имя",
      "как твоё имя",
      "как тебя зовут",
      "у тебя есть имя",
      "आपका नाम क्या है",
      "तुम्हारा नाम क्या है",
      "你叫什么名字",
      "您叫什么名字",
      "你的名字是什么",
      "你有名字吗",
    ].includes(normalized) ||
    (has("what") && has("your") && has("name")) ||
    (has("you") && has("have") && has("name")) ||
    (has("call") && has("you")) ||
    (has("как") && has("тебя") && has("зовут"))
  );
}

export function localAssistantNameAnswer(prompt, preferences = {}) {
  const name = normalizeAssistantName(preferences.assistantName);
  const raw = String(prompt || "");
  if (name && /[а-яё]/iu.test(raw)) {
    return `Меня зовут ${name}. Я formal AI.`;
  }
  if (name && /[\u0900-\u097f]/u.test(raw)) {
    return `मेरा नाम ${name} है। मैं formal AI हूँ।`;
  }
  if (name && /[\u3400-\u9fff]/u.test(raw)) {
    return `我的名字是 ${name}。我是 formal AI。`;
  }
  if (name) {
    return `My name is ${name}. I'm formal AI.`;
  }
  if (/[а-яё]/iu.test(raw)) {
    return "Я formal AI, и сейчас у меня нет имени. Но вы можете назвать меня как хотите.";
  }
  if (/[\u0900-\u097f]/u.test(raw)) {
    return "मैं formal AI हूँ, और अभी मेरा कोई नाम नहीं है। लेकिन आप मुझे अपनी पसंद का नाम दे सकते हैं।";
  }
  if (/[\u3400-\u9fff]/u.test(raw)) {
    return "我是 formal AI,目前还没有名字。不过您可以按自己的喜好给我起名。";
  }
  return ASSISTANT_NAME_ANSWER;
}
