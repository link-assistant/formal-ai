"use strict";

// Local-database sync client for the desktop main process.
//
// Issue #347 / R5c (ROADMAP D1): the desktop shell keeps its conversation memory
// in the browser (IndexedDB) while the native CLI / server keeps the same event
// log on disk. This module reconciles the two over the local server's
// Links-Notation memory endpoints:
//
//   GET  /v1/memory/since?event=<id>   -> demo_memory delta document (.lino text)
//   POST /v1/memory/import             -> { object, added, total }
//
// Per R7 the payloads are Links Notation (not JSON), so pulls return raw `.lino`
// text and pushes send `.lino` text. The reconciler tracks the last event id it
// has seen so repeated pulls only transfer the delta. Dependencies are injected
// so the merge/track logic is unit-testable without a live server.

function memoryUrl(apiBase, path) {
  const base = String(apiBase || "").replace(/\/+$/, "");
  return `${base}${path}`;
}

async function pullSince(apiBase, lastSeen, fetchImpl) {
  const fetcher = fetchImpl || globalThis.fetch;
  if (typeof fetcher !== "function") {
    throw new Error("no fetch implementation is configured");
  }
  const suffix = lastSeen ? `?event=${encodeURIComponent(lastSeen)}` : "";
  const response = await fetcher(memoryUrl(apiBase, `/v1/memory/since${suffix}`), {
    method: "GET",
  });
  if (!response.ok) {
    throw new Error(`memory pull failed: HTTP ${response.status}`);
  }
  return typeof response.text === "function" ? await response.text() : "";
}

async function push(apiBase, linoText, fetchImpl) {
  const fetcher = fetchImpl || globalThis.fetch;
  if (typeof fetcher !== "function") {
    throw new Error("no fetch implementation is configured");
  }
  const response = await fetcher(memoryUrl(apiBase, "/v1/memory/import"), {
    method: "POST",
    headers: { "Content-Type": "text/plain" },
    body: String(linoText || ""),
  });
  if (!response.ok) {
    throw new Error(`memory push failed: HTTP ${response.status}`);
  }
  return typeof response.json === "function" ? await response.json() : { added: 0, total: 0 };
}

// Pull every `demo_memory` record id from a `.lino` document. The memory log
// records each event as `event <id>` (or a quoted id), so we read the id that
// follows the `event` keyword to advance the watermark.
function eventIdsFromLino(linoText) {
  const ids = [];
  const lines = String(linoText || "").split("\n");
  for (const line of lines) {
    const match = line.match(/^\s*event\s+"?([^"\s]+)"?/);
    if (match) {
      ids.push(match[1]);
    }
  }
  return ids;
}

function createMemorySync(options = {}) {
  const apiBase = options.apiBase || "";
  const fetchImpl = options.fetchImpl || globalThis.fetch;
  let lastSeen = options.lastSeen || null;

  async function pull() {
    const delta = await pullSince(apiBase, lastSeen, fetchImpl);
    const ids = eventIdsFromLino(delta);
    if (ids.length > 0) {
      lastSeen = ids[ids.length - 1];
    }
    return { delta, added: ids.length, lastSeen };
  }

  async function sendLocal(linoText) {
    const result = await push(apiBase, linoText, fetchImpl);
    return result;
  }

  return {
    getLastSeen: () => lastSeen,
    setLastSeen: (value) => {
      lastSeen = value || null;
      return lastSeen;
    },
    pull,
    push: sendLocal,
  };
}

module.exports = {
  memoryUrl,
  pullSince,
  push,
  eventIdsFromLino,
  createMemorySync,
};
