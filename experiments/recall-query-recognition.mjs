// Experiment: verify recognizeRecallQuery + buildRecallReport handle the
// expected EN/RU/ZH phrasings without needing the full browser harness.
//
// Run with: node experiments/recall-query-recognition.mjs

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const appPath = path.resolve(__dirname, "..", "src", "web", "app.js");
const src = fs.readFileSync(appPath, "utf8");

function sliceFn(name) {
  const start = src.indexOf(`function ${name}`);
  if (start < 0) throw new Error(`could not find ${name}`);
  // Skip past the parameter list (which may contain a destructured "{ ... }")
  // before counting body braces.
  const parenStart = src.indexOf("(", start);
  let parenDepth = 0;
  let i = parenStart;
  for (; i < src.length; i++) {
    if (src[i] === "(") parenDepth++;
    else if (src[i] === ")") {
      parenDepth--;
      if (parenDepth === 0) {
        i++;
        break;
      }
    }
  }
  // Now find the opening brace of the body.
  while (i < src.length && src[i] !== "{") i++;
  let depth = 0;
  for (; i < src.length; i++) {
    if (src[i] === "{") depth++;
    else if (src[i] === "}") {
      depth--;
      if (depth === 0) return src.slice(start, i + 1);
    }
  }
  throw new Error(`unterminated ${name}`);
}

function sliceConst(name) {
  const start = src.indexOf(`const ${name}`);
  if (start < 0) throw new Error(`could not find const ${name}`);
  const semi = src.indexOf("];", start);
  if (semi < 0) throw new Error(`could not find end of const ${name}`);
  return src.slice(start, semi + 2);
}

const helpers = [
  sliceConst("RECALL_QUERY_PATTERNS"),
  sliceConst("RECALL_OTHER_SUFFIXES"),
  sliceConst("RECALL_OTHER_PREFIXES"),
  sliceFn("normalizeMemoryPrompt"),
  sliceFn("stripRecallTerm"),
  sliceFn("recoverOriginalRange"),
  sliceFn("recognizeRecallQuery"),
  sliceFn("deriveConversationTitle"),
  sliceFn("buildRecallReport"),
].join("\n");

const fn = new Function(`${helpers}\nreturn { recognizeRecallQuery, buildRecallReport };`);
const { recognizeRecallQuery, buildRecallReport } = fn();

const cases = [
  "When did I ask about Rust?",
  "when did i mention Donald Trump",
  "search my conversations for Wikipedia",
  "find Donald Trump in another conversation",
  "find \"Wikipedia\" in my other conversations",
  "Recall Rust",
  "Когда я спрашивал про Илона Маска?",
  "найди Илон Маск в другой беседе",
  "поиск по беседам Wikipedia",
  "我什么时候问过 Rust",
  "在对话中搜索 Donald Trump",
  "搜索我的对话 Rust",
  "Hi",
  "Who are you?",
];

for (const c of cases) {
  console.log(JSON.stringify(c), "→", recognizeRecallQuery(c));
}

const events = [
  {
    kind: "message",
    role: "user",
    content: "What is Rust?",
    sentAt: "2026-05-15T10:00:00Z",
    conversationId: "conv-1",
    conversationTitle: "Rust basics",
  },
  {
    kind: "message",
    role: "assistant",
    content: "Rust is a memory-safe systems language.",
    sentAt: "2026-05-15T10:00:05Z",
    conversationId: "conv-1",
  },
  {
    kind: "message",
    role: "user",
    content: "Who is Donald Trump?",
    sentAt: "2026-05-15T11:00:00Z",
    conversationId: "conv-2",
    conversationTitle: "Donald Trump bio",
  },
  {
    kind: "message",
    role: "assistant",
    content: "Donald Trump is the 47th president…",
    sentAt: "2026-05-15T11:00:05Z",
    conversationId: "conv-2",
  },
  {
    kind: "message",
    role: "user",
    content: "Tell me more about Rust",
    sentAt: "2026-05-15T12:00:00Z",
    conversationId: "conv-3",
    conversationTitle: "Rust follow-up",
  },
];

console.log("\n--- Recall report: term='Rust' scope=all ---");
console.log(buildRecallReport({ events, term: "Rust", scope: "all", currentConversationId: "conv-3" }).content);

console.log("\n--- Recall report: term='Rust' scope=other current=conv-3 ---");
console.log(buildRecallReport({ events, term: "Rust", scope: "other", currentConversationId: "conv-3" }).content);

console.log("\n--- Recall report: term='Trump' scope=all ---");
console.log(buildRecallReport({ events, term: "Trump", scope: "all", currentConversationId: "conv-3" }).content);

console.log("\n--- Recall report: term='Haskell' scope=all (no matches) ---");
console.log(buildRecallReport({ events, term: "Haskell", scope: "all", currentConversationId: "conv-3" }).content);
