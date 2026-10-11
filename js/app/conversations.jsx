// Conversations projected from the append-only event log: ids, titles,
// grouping, Markdown export and message records.

import { normalizeUiLanguagePreference } from "./user-context.jsx";

function randomItem(items) {
  return items[Math.floor(Math.random() * items.length)];
}

// Issue #27: conversations are grouped slices of the append-only event log.
// Each event records the id of the conversation that produced it; the UI then
// filters events on read. New ids are generated locally so they stay portable
// across browsers (no server round-trip required).
export function generateConversationId() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `conv-${crypto.randomUUID()}`;
  }
  const random = Math.random().toString(16).slice(2, 10);
  return `conv-${Date.now().toString(16)}-${random}`;
}

export function deriveConversationTitle(text) {
  const trimmed = String(text || "").trim().replace(/\s+/g, " ");
  if (!trimmed) {
    return "New conversation";
  }
  if (trimmed.length <= 60) {
    return trimmed;
  }
  return `${trimmed.slice(0, 57)}…`;
}

// Group append-only events into per-conversation summaries (id, title,
// timestamps, message count). Events without a conversationId are aggregated
// under the synthetic "legacy" bucket so existing demos remain visible after
// the schema upgrade.
export function groupConversations(events, options = {}) {
  const safe = Array.isArray(events) ? events : [];
  const map = new Map();

  const ensureEntry = (id, event = {}) => {
    let entry = map.get(id);
    if (!entry) {
      entry = {
        id,
        title: id === "legacy" ? "Earlier conversation" : "",
        firstAt: event.sentAt || "",
        lastAt: event.sentAt || "",
        deletedAt: "",
        messageCount: 0,
        deleted: false,
      };
      map.set(id, entry);
    }
    return entry;
  };

  for (let index = 0; index < safe.length; index += 1) {
    const event = safe[index];
    if (!event) {
      continue;
    }
    // Issue #541 (R4): demo turns live in their own dedicated conversation
    // and must never surface in the user's sidebar — listing them would
    // suggest the user can navigate into and edit them, breaking the
    // "demo never overwrites your work" guarantee.
    if (event.isDemo) {
      continue;
    }
    const kind = event.kind || "message";
    const id = event.conversationId || "legacy";
    if (kind === "conversation_deleted") {
      const entry = ensureEntry(id, event);
      entry.deleted = true;
      entry.deletedAt = event.sentAt || entry.deletedAt || "";
      if (!entry.title && event.conversationTitle) {
        entry.title = event.conversationTitle;
      }
      if (event.sentAt && (!entry.lastAt || event.sentAt > entry.lastAt)) {
        entry.lastAt = event.sentAt;
      }
      continue;
    }
    if (kind !== "message") {
      continue;
    }
    const entry = ensureEntry(id, event);
    if (event.role === "user" && !entry.title && event.conversationTitle) {
      entry.title = event.conversationTitle;
    } else if (event.role === "user" && !entry.title) {
      entry.title = deriveConversationTitle(event.content);
    }
    if (event.sentAt && (!entry.firstAt || event.sentAt < entry.firstAt)) {
      entry.firstAt = event.sentAt;
    }
    if (event.sentAt && (!entry.lastAt || event.sentAt > entry.lastAt)) {
      entry.lastAt = event.sentAt;
    }
    entry.messageCount += 1;
  }
  const showDeleted = Boolean(options.showDeleted);
  const list = Array.from(map.values()).filter((entry) =>
    showDeleted ? entry.deleted : !entry.deleted,
  );
  list.sort((left, right) => {
    if (left.lastAt && right.lastAt) {
      return right.lastAt.localeCompare(left.lastAt);
    }
    return 0;
  });
  return list;
}

export function resizeComposerInput(element) {
  if (!element) return;
  element.style.height = "auto";
  const computed = getComputedStyle(element);
  const maxHeight = parseFloat(computed.maxHeight);
  const borderHeight =
    (parseFloat(computed.borderTopWidth) || 0) +
    (parseFloat(computed.borderBottomWidth) || 0);
  const scrollBorderBoxHeight = element.scrollHeight + borderHeight;
  const target = Number.isFinite(maxHeight)
    ? Math.min(scrollBorderBoxHeight, maxHeight)
    : scrollBorderBoxHeight;
  element.style.height = `${Math.max(target, 0)}px`;
  element.style.overflowY =
    element.scrollHeight > target - borderHeight + 1 ? "auto" : "hidden";
}

export function localizeTool(tool, language) {
  if (!tool || !Array.isArray(tool.localized)) {
    return tool || {};
  }
  const normalized = normalizeUiLanguagePreference(language) || "en";
  const localized =
    tool.localized.find((entry) => entry.language === normalized) ||
    tool.localized.find((entry) => entry.language === "en");
  if (!localized) {
    return tool;
  }
  return {
    ...tool,
    name: localized.name || tool.name,
    description: localized.description || tool.description,
  };
}

export function messagesForConversation(events, conversationId) {
  if (!conversationId) {
    return [];
  }
  const safe = Array.isArray(events) ? events : [];
  const out = [];
  for (let index = 0; index < safe.length; index += 1) {
    const event = safe[index];
    if (!event || event.kind && event.kind !== "message") continue;
    if ((event.conversationId || "legacy") !== conversationId) continue;
    // Issue #541 (R4): defensive — if a caller ever asks for a demo
    // conversation id, only return its demo-flagged events; conversely a
    // real conversation id must skip any stray demo-flagged event that
    // somehow shares the id.
    const evidence = Array.isArray(event.evidence) ? event.evidence : [];
    out.push(
      createMessage(event.role || "assistant", String(event.content || ""), {
        intent: event.intent,
        evidence,
        iframeUrl: event.iframeUrl || null,
        detectedFailure: event.detectedFailure === true,
      }),
    );
  }
  return out;
}

// Issue #386: serialize a whole stored conversation to Markdown so it can be
// copied from the conversations list. Each turn becomes a `### <author>`
// section followed by the message body. When `includeReasoning` is set (the
// diagnostics surface is on) the per-turn reasoning steps — persisted as
// separate `reasoning` events recorded just before each assistant message —
// are appended after that AI message as a Markdown ordered list, so the export
// mirrors what the diagnostics panel shows on screen.
export function conversationToMarkdown(events, conversationId, options = {}) {
  if (!conversationId) return "";
  const includeReasoning = options.includeReasoning === true;
  const userLabel = options.userLabel || "You";
  const assistantLabel = options.assistantLabel || "formal-ai";
  const reasoningLabel = options.reasoningLabel || "Reasoning";
  const safe = Array.isArray(events) ? events : [];
  const blocks = [];
  const title = (options.title || "").trim();
  if (title) {
    blocks.push(`# ${title}`);
  }
  let pendingReasoning = [];
  for (const event of safe) {
    if (!event) continue;
    if ((event.conversationId || "legacy") !== conversationId) continue;
    const kind = event.kind || "message";
    if (kind === "reasoning") {
      const detail = String(event.content || "").trim();
      if (detail) pendingReasoning.push(detail);
      continue;
    }
    if (kind !== "message") continue;
    const role = event.role || "assistant";
    const label = role === "user" ? userLabel : assistantLabel;
    const lines = [`### ${label}`, "", String(event.content || "")];
    if (role === "assistant" && includeReasoning && pendingReasoning.length > 0) {
      lines.push("", `#### ${reasoningLabel}`, "");
      pendingReasoning.forEach((step, index) => {
        lines.push(`${index + 1}. ${step}`);
      });
    }
    blocks.push(lines.join("\n"));
    pendingReasoning = [];
  }
  return blocks.join("\n\n");
}

export function randomInt(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function timeLabel() {
  return new Date().toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function createMessage(role, content, extra = {}) {
  return {
    id: `${role}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    role,
    author: role === "user" ? "You" : "formal-ai",
    content,
    sentAt: timeLabel(),
    ...extra,
  };
}
