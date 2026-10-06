// Recall queries ("what did I say about X") over the append-only event log.

import { deriveConversationTitle } from "./conversations.jsx";
import { normalizeMemoryPrompt } from "./interface-commands.jsx";

// Issue #27 R11: natural-language cross-conversation recall. The user types
// something like "when did I ask about Rust?" or "find Donald Trump in another
// conversation" and the assistant projects the append-only memory log onto
// matching events grouped by conversation. Patterns are prefix-based so the
// remainder of the prompt becomes the search term verbatim (after trimming
// quotes and trailing punctuation). `scope = 'other'` excludes the current
// conversation; `scope = 'all'` searches every conversation including the
// current one.
const RECALL_QUERY_PATTERNS = [
  { prefix: "when did i ask about ", scope: "all" },
  { prefix: "when did i ask ", scope: "all" },
  { prefix: "when did i mention ", scope: "all" },
  { prefix: "when did i talk about ", scope: "all" },
  { prefix: "have i asked about ", scope: "all" },
  { prefix: "have i mentioned ", scope: "all" },
  { prefix: "did i ask about ", scope: "all" },
  { prefix: "did i mention ", scope: "all" },
  { prefix: "search my conversations for ", scope: "all" },
  { prefix: "search conversations for ", scope: "all" },
  { prefix: "search my chats for ", scope: "all" },
  { prefix: "recall ", scope: "all" },
  { prefix: "когда я спрашивал про ", scope: "all" },
  { prefix: "когда я спрашивал о ", scope: "all" },
  { prefix: "когда я спрашивал ", scope: "all" },
  { prefix: "когда я упоминал ", scope: "all" },
  { prefix: "поиск по беседам ", scope: "all" },
  { prefix: "поиск в беседах ", scope: "all" },
  { prefix: "найди в беседах ", scope: "all" },
  { prefix: "我什么时候问过 ", scope: "all" },
  { prefix: "我什么时候问过", scope: "all" },
  { prefix: "我什么时候提到 ", scope: "all" },
  { prefix: "我什么时候提到", scope: "all" },
  { prefix: "搜索我的对话 ", scope: "all" },
  { prefix: "搜索我的对话", scope: "all" },
  { prefix: "在对话中搜索 ", scope: "all" },
  { prefix: "在对话中搜索", scope: "all" },
];

// Suffix forms ("...in another conversation", "...在其他对话中") that mark the
// recall as cross-conversation-only. The remainder before the suffix becomes
// the search term.
const RECALL_OTHER_SUFFIXES = [
  " in another conversation",
  " in other conversations",
  " in my other conversations",
  " in my conversations",
  " in another chat",
  " in other chats",
  " в другой беседе",
  " в других беседах",
  " в других чатах",
  "在其他对话中",
  "在另一个对话中",
];

// "find X in another conversation" — `find ` is the lead-in for the other-scope
// recall when paired with one of the suffixes above.
const RECALL_OTHER_PREFIXES = [
  "find ",
  "search for ",
  "look for ",
  "найди ",
  "поищи ",
  "查找 ",
  "查找",
  "搜索 ",
  "搜索",
];

function stripRecallTerm(term) {
  return String(term || "")
    .replace(/^["'«»『「]+/, "")
    .replace(/["'«»』」]+$/, "")
    .replace(/[!?.,;:。!?,;:、]+$/g, "")
    .trim();
}

// Extract the substring from `original` that corresponds to characters at
// positions [start, end) of the lowercased normalised form. We do not have a
// strict 1:1 character map because normalisation can collapse whitespace, so
// approximate by walking the original and skipping characters that the
// normaliser would also skip. The result is good enough to preserve user
// casing for terms like "Donald Trump" or "Илона Маска".
function recoverOriginalRange(original, normalized, start, end) {
  // Walk through `original` character by character, advancing a normalised
  // cursor whenever we emit a character that would survive normalisation.
  // When the normalised cursor enters [start, end), we capture characters
  // from `original` instead of from `normalized`.
  let nIdx = 0;
  let captured = "";
  let i = 0;
  let prevWasSpace = false;
  while (i < original.length && nIdx < end) {
    const ch = original[i];
    const lower = ch.toLowerCase();
    // Mirror normalizeMemoryPrompt's whitespace collapse: \s plus the
    // unicode-space block U+00A0 / U+2000–U+200B used by the seed corpus.
    if (/[\s\u00A0\u2000-\u200B]/.test(ch)) {
      if (!prevWasSpace) {
        if (nIdx >= start) captured += " ";
        nIdx++;
        prevWasSpace = true;
      }
      i++;
      continue;
    }
    prevWasSpace = false;
    if (nIdx >= start) captured += ch;
    nIdx += lower.length;
    i++;
  }
  return captured.trim();
}

export function recognizeRecallQuery(text) {
  const original = String(text || "").trim();
  if (!original) return null;
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized) return null;

  // Try "find X in another conversation" — prefix + suffix combo.
  for (const suffix of RECALL_OTHER_SUFFIXES) {
    const suffixIdx = normalized.lastIndexOf(suffix);
    if (suffixIdx < 0) continue;
    const beforeSuffix = normalized.slice(0, suffixIdx);
    for (const prefix of RECALL_OTHER_PREFIXES) {
      if (beforeSuffix.startsWith(prefix)) {
        const normalizedTerm = stripRecallTerm(beforeSuffix.slice(prefix.length));
        if (!normalizedTerm) continue;
        const originalTerm = stripRecallTerm(
          recoverOriginalRange(original, normalized, prefix.length, suffixIdx),
        );
        return { term: originalTerm || normalizedTerm, scope: "other" };
      }
    }
  }

  // Prefix-only patterns ("when did I ask about X", "recall X").
  for (const { prefix, scope } of RECALL_QUERY_PATTERNS) {
    if (normalized.startsWith(prefix)) {
      const normalizedTerm = stripRecallTerm(normalized.slice(prefix.length));
      if (!normalizedTerm) continue;
      const originalTerm = stripRecallTerm(
        recoverOriginalRange(original, normalized, prefix.length, normalized.length),
      );
      return { term: originalTerm || normalizedTerm, scope };
    }
  }
  return null;
}

// Build a Markdown report of every message whose lowercased content contains
// `term`, grouped by conversation. `scope === 'other'` filters out the active
// conversation. `triggerText` is the user's recall request itself — skip
// events whose content equals it so the recall never matches the prompt that
// triggered it. `events` is the full append-only log from FormalAiMemory.
export function buildRecallReport({ events, term, scope, currentConversationId, triggerText }) {
  const safeEvents = Array.isArray(events) ? events : [];
  const needle = String(term || "").toLowerCase();
  if (!needle) {
    return {
      content: "No search term recognised in the recall request.",
      matches: [],
    };
  }
  const triggerNormalized = String(triggerText || "").trim().toLowerCase();
  const groups = new Map();
  for (const event of safeEvents) {
    if (!event || (event.kind && event.kind !== "message")) continue;
    const content = String(event.content || "");
    if (!content.toLowerCase().includes(needle)) continue;
    if (triggerNormalized && content.trim().toLowerCase() === triggerNormalized) {
      continue;
    }
    const id = event.conversationId || "legacy";
    if (scope === "other" && id === (currentConversationId || "")) continue;
    let entry = groups.get(id);
    if (!entry) {
      entry = { id, title: "", events: [] };
      groups.set(id, entry);
    }
    if (!entry.title && event.role === "user" && event.conversationTitle) {
      entry.title = event.conversationTitle;
    }
    entry.events.push(event);
  }

  const groupList = Array.from(groups.values());
  // Fill in titles from the first user message of each group when the recorded
  // title is missing (legacy events without a conversationTitle field).
  for (const group of groupList) {
    if (!group.title) {
      const firstUser = group.events.find((e) => e.role === "user");
      if (firstUser && firstUser.content) {
        group.title = deriveConversationTitle(firstUser.content);
      } else if (group.id === "legacy") {
        group.title = "Earlier conversation";
      } else {
        group.title = "Untitled conversation";
      }
    }
    group.events.sort((a, b) => String(a.sentAt || "").localeCompare(String(b.sentAt || "")));
  }
  groupList.sort((left, right) => {
    const lLast = left.events[left.events.length - 1]?.sentAt || "";
    const rLast = right.events[right.events.length - 1]?.sentAt || "";
    return String(rLast).localeCompare(String(lLast));
  });

  const totalMatches = groupList.reduce((sum, g) => sum + g.events.length, 0);
  if (totalMatches === 0) {
    const scopeNote = scope === "other" ? " in any other conversation" : "";
    return {
      content: `No mentions of "${term}" found${scopeNote}.`,
      matches: [],
    };
  }

  const lines = [];
  const conversationCount = groupList.length;
  lines.push(
    `Found **${totalMatches}** mention${totalMatches === 1 ? "" : "s"} of "${term}" across **${conversationCount}** conversation${conversationCount === 1 ? "" : "s"}.`,
  );
  for (const group of groupList) {
    lines.push("");
    lines.push(`### ${group.title}`);
    for (const event of group.events) {
      const stamp = event.sentAt ? event.sentAt : "(no timestamp)";
      const role = event.role === "user" ? "user" : "assistant";
      const snippet = String(event.content || "").replace(/\s+/g, " ").trim();
      const trimmed = snippet.length > 160 ? `${snippet.slice(0, 157)}…` : snippet;
      lines.push(`- ${stamp} — ${role}: ${trimmed}`);
    }
  }
  return { content: lines.join("\n"), matches: groupList };
}
