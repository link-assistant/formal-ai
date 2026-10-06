// App-wide constants: version stamps, the issue repository, the canonical
// answers the local fallback reuses and the example prompts in the sidebar.

// The meta tag is stamped with the published crate version by
// `scripts/stamp-pages-artifact.sh` during the GitHub Pages deploy. When the
// site is served straight from the source tree (e.g. local Playwright runs)
// the placeholder is preserved verbatim; we fall back to `"dev"` so issue
// reports never advertise a hardcoded stale version like `0.16.0`.
export const APP_VERSION = (() => {
  const raw = document.querySelector('meta[name="formal-ai-version"]')?.content;
  if (!raw || raw.startsWith("__") || raw.endsWith("__")) {
    return "dev";
  }
  return raw;
})();

const ASSET_VERSION =
  typeof window !== "undefined" ? window.FORMAL_AI_ASSET_VERSION || "" : "";

export const ISSUE_REPOSITORY = "link-assistant/formal-ai";

export const ISSUE_LABELS = "bug";

export const SOURCE_CODE_URL = `https://github.com/${ISSUE_REPOSITORY}`;

export const UNKNOWN_ANSWER =
  "I don't know how to answer that yet. I cannot answer that from local Links Notation rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.";

export const IDENTITY_ANSWER =
  "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";

export const ASSISTANT_FREE_TIME_ANSWER =
  "I do not have free time the way a person does. Between prompts I am idle; when the dialog is active, I help with tasks, rules, and explanations.";

export const ASSISTANT_NAME_ANSWER =
  "I'm formal AI, and currently I don't have a name. But you can name me as you like.";

export const COURTESY_ACKNOWLEDGEMENTS = [
  "Glad to hear it.",
  "You're welcome.",
  "Good to hear.",
  "Happy to hear that.",
];

export const COURTESY_FOLLOW_UPS = [
  "What would you like to do next?",
  "Do you want to discuss something else?",
  "Is there anything else you want to work on?",
  "Would you like to explore another topic?",
];

// Issue #27: the sidebar advertises every prompt family that has a deterministic
// symbolic rule or seed-backed answer in the engine. The list intentionally
// mirrors the multilingual + hello-world end-to-end tests so any regression in
// the seed surfaces immediately when a user clicks the prompt.
export const EXAMPLE_PROMPTS = [
  { label: "Greeting (en)", text: "Hi" },
  { label: "Greeting (ru)", text: "Привет" },
  { label: "Greeting (hi)", text: "नमस्ते" },
  { label: "Greeting (zh)", text: "你好" },
  { label: "Farewell (en)", text: "Goodbye" },
  { label: "Farewell (ru)", text: "До свидания" },
  { label: "Farewell (hi)", text: "अलविदा" },
  { label: "Farewell (zh)", text: "再见" },
  { label: "Identity (en)", text: "Who are you?" },
  { label: "Identity (ru)", text: "Кто ты?" },
  { label: "Identity (hi)", text: "तुम कौन हो?" },
  { label: "Identity (zh)", text: "你是谁?" },
  { label: "Clarification (en)", text: "I don't understand" },
  { label: "Clarification (ru)", text: "не понял" },
  { label: "Clarification (hi)", text: "समझ नहीं आया" },
  { label: "Clarification (zh)", text: "我不明白" },
  { label: "Capabilities (en)", text: "What can you do?" },
  { label: "Capabilities (ru)", text: "Что ты умеешь?" },
  { label: "Behavior rules", text: "List behavior rules" },
  { label: "Self facts", text: "List all facts you know about yourself" },
  { label: "Hello world (Rust)", text: "Write me hello world program in Rust" },
  { label: "Hello world (Python)", text: "Create a hello world example in Python" },
  { label: "Hello world (JavaScript)", text: "Write hello world in JavaScript" },
  { label: "Hello world (TypeScript)", text: "Write hello world in TypeScript" },
  { label: "Hello world (Go)", text: "Show hello world in Go" },
  { label: "Hello world (C)", text: "Show hello world in C" },
  { label: "Calculation (en)", text: "What is 2 + 2?" },
  { label: "Calculation (ru)", text: "Сколько будет два плюс два?" },
  { label: "Concept (en)", text: "What is Rust?" },
  { label: "Concept (en/Wikipedia)", text: "Who is Donald Trump?" },
  { label: "Concept (ru/Wikipedia)", text: "Кто такой Илон Маск?" },
  { label: "Concept (ru)", text: "Что такое Википедия?" },
  { label: "Concept (hi)", text: "विकिपीडिया क्या है?" },
  { label: "Concept (zh)", text: "维基百科是什么?" },
  { label: "Concept in context", text: "What is IIR in machine learning?" },
  { label: "Summarization", text: "Summarize this conversation" },
  { label: "Brainstorming", text: "Brainstorm 5 small tools for link notation." },
  { label: "Fact Q&A (en)", text: "Who wrote The Lord of the Rings?" },
  { label: "Fact Q&A (ru)", text: "столица россии" },
  { label: "Fact Q&A (hi)", text: "जापान की राजधानी क्या है?" },
  { label: "Fact Q&A (zh)", text: "日本的首都是什么?" },
  { label: "Navigate URL", text: "Navigate to github.com" },
  { label: "Fetch URL", text: "Сделай запрос к google.com" },
  { label: "Web search", text: "Search the web for Nikola Tesla" },
  { label: "Coreference", text: "What features make it different from C?" },
  { label: "Roleplay", text: "Pretend you are Albert Einstein and explain relativity to a teenager." },
  { label: "Idiom (ru)", text: "Купи слона" },
  { label: "Recall (en)", text: "When did I ask about Rust?" },
  { label: "Recall (cross-conv)", text: "Find Wikipedia in another conversation" },
  { label: "Export memory", text: "Export memory" },
  { label: "Import memory", text: "Import memory" },
];

export function withAssetVersion(path) {
  if (!ASSET_VERSION) {
    return path;
  }
  const separator = path.includes("?") ? "&" : "?";
  return `${path}${separator}v=${encodeURIComponent(ASSET_VERSION)}`;
}
