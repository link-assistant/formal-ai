// Append-only Links Notation event log for the demo.
//
// The browser demo records every user/assistant turn as an Event in an
// IndexedDB object store. The store is treated as append-only: writes can only
// add new records, and the public API does not expose a "forget" operation.
// Users can export the full log as Links Notation text and import a previous
// log into a new browser session — Links Notation is the portable format
// shared with the Rust solver and the seed data under `data/`.
//
// Storage layout:
//
//   demo_memory
//     event "id1"
//       role "user"
//       content "Hi"
//       sentAt "2026-05-15T12:00:00.000Z"
//     event "id2"
//       role "assistant"
//       intent "greeting"
//       content "Hi, how may I help you?"
//       sentAt "2026-05-15T12:00:01.000Z"
//
// Only existing entries are encoded; absent fields are omitted so the format
// stays human-readable in DevTools and on disk.

(function (global) {
  "use strict";

  var DB_NAME = "formal-ai-demo";
  var DB_VERSION = 1;
  var STORE_NAME = "events";
  var ROOT_HEADER = "demo_memory";
  var EXPORT_FIELDS = [
    "role",
    "intent",
    "content",
    "sentAt",
    "demoLabel",
    "evidence",
  ];

  var dbPromise = null;

  function hasIndexedDb() {
    try {
      return typeof global.indexedDB !== "undefined";
    } catch (_error) {
      return false;
    }
  }

  function openDatabase() {
    if (dbPromise) return dbPromise;
    if (!hasIndexedDb()) {
      dbPromise = Promise.resolve(null);
      return dbPromise;
    }
    dbPromise = new Promise(function (resolve, reject) {
      var request = global.indexedDB.open(DB_NAME, DB_VERSION);
      request.onupgradeneeded = function () {
        var db = request.result;
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          db.createObjectStore(STORE_NAME, {
            keyPath: "id",
            autoIncrement: true,
          });
        }
      };
      request.onsuccess = function () {
        resolve(request.result);
      };
      request.onerror = function () {
        reject(request.error);
      };
    });
    return dbPromise;
  }

  function escapeValue(value) {
    return String(value).replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  }

  function unescapeValue(value) {
    return value.replace(/\\"/g, '"').replace(/\\\\/g, "\\");
  }

  function serializeEvidence(evidence) {
    if (!Array.isArray(evidence)) return "";
    return evidence.filter(function (item) {
      return typeof item === "string" && item.length > 0;
    }).join("|");
  }

  function parseEvidence(value) {
    if (!value) return [];
    return value.split("|").filter(Boolean);
  }

  function formatEvent(event) {
    var lines = [];
    var id = event && event.id != null ? String(event.id) : "";
    lines.push('  event "' + escapeValue(id) + '"');
    for (var index = 0; index < EXPORT_FIELDS.length; index += 1) {
      var key = EXPORT_FIELDS[index];
      var raw = event ? event[key] : undefined;
      if (raw === undefined || raw === null) continue;
      var value = key === "evidence" ? serializeEvidence(raw) : raw;
      if (typeof value !== "string" || value.length === 0) {
        if (key !== "evidence") {
          value = String(raw);
        } else {
          continue;
        }
      }
      lines.push('    ' + key + ' "' + escapeValue(value) + '"');
    }
    return lines.join("\n");
  }

  function exportLinksNotation(events) {
    var safe = Array.isArray(events) ? events : [];
    if (safe.length === 0) return ROOT_HEADER + "\n";
    var parts = [ROOT_HEADER];
    for (var index = 0; index < safe.length; index += 1) {
      parts.push(formatEvent(safe[index]));
    }
    return parts.join("\n") + "\n";
  }

  function parseLinksNotation(text) {
    if (typeof text !== "string" || !text.trim()) return [];
    var lines = text.split(/\r?\n/);
    var header = (lines.shift() || "").trim();
    if (header !== ROOT_HEADER) return [];
    var events = [];
    var current = null;
    var eventPattern = /^\s{2}event\s+"((?:[^"\\]|\\.)*)"\s*$/;
    var fieldPattern = /^\s{4}([a-zA-Z0-9_]+)\s+"((?:[^"\\]|\\.)*)"\s*$/;
    for (var index = 0; index < lines.length; index += 1) {
      var line = lines[index];
      if (!line || !line.trim()) continue;
      var eventMatch = eventPattern.exec(line);
      if (eventMatch) {
        if (current) events.push(current);
        current = { id: unescapeValue(eventMatch[1]) };
        continue;
      }
      var fieldMatch = fieldPattern.exec(line);
      if (fieldMatch && current) {
        var key = fieldMatch[1];
        var raw = unescapeValue(fieldMatch[2]);
        current[key] = key === "evidence" ? parseEvidence(raw) : raw;
      }
    }
    if (current) events.push(current);
    return events;
  }

  function withStore(mode, action) {
    return openDatabase().then(function (db) {
      if (!db) return null;
      return new Promise(function (resolve, reject) {
        var transaction = db.transaction(STORE_NAME, mode);
        transaction.onerror = function () {
          reject(transaction.error);
        };
        transaction.oncomplete = function () {
          resolve(result);
        };
        var store = transaction.objectStore(STORE_NAME);
        var result = null;
        action(store, function (value) {
          result = value;
        }, reject);
      });
    });
  }

  function appendEvent(event) {
    if (!event || typeof event !== "object") {
      return Promise.resolve(null);
    }
    var record = {
      role: String(event.role || ""),
      content: String(event.content || ""),
      sentAt: String(event.sentAt || new Date().toISOString()),
    };
    if (event.intent) record.intent = String(event.intent);
    if (event.demoLabel) record.demoLabel = String(event.demoLabel);
    if (Array.isArray(event.evidence)) record.evidence = event.evidence.slice();
    return withStore("readwrite", function (store, setResult) {
      var request = store.add(record);
      request.onsuccess = function () {
        record.id = request.result;
        setResult(record);
      };
    });
  }

  function listEvents() {
    return withStore("readonly", function (store, setResult) {
      var request = store.getAll();
      request.onsuccess = function () {
        var items = Array.isArray(request.result) ? request.result : [];
        items.sort(function (left, right) {
          return Number(left.id) - Number(right.id);
        });
        setResult(items);
      };
    }).then(function (value) {
      return Array.isArray(value) ? value : [];
    });
  }

  function importEvents(events) {
    var safe = Array.isArray(events) ? events : [];
    if (safe.length === 0) return Promise.resolve(0);
    return withStore("readwrite", function (store, setResult) {
      var inserted = 0;
      var index = 0;

      function insertNext() {
        if (index >= safe.length) {
          setResult(inserted);
          return;
        }
        var raw = safe[index];
        index += 1;
        if (!raw || typeof raw !== "object") {
          insertNext();
          return;
        }
        var record = {
          role: String(raw.role || ""),
          content: String(raw.content || ""),
          sentAt: String(raw.sentAt || new Date().toISOString()),
        };
        if (raw.intent) record.intent = String(raw.intent);
        if (raw.demoLabel) record.demoLabel = String(raw.demoLabel);
        if (Array.isArray(raw.evidence)) record.evidence = raw.evidence.slice();
        var request = store.add(record);
        request.onsuccess = function () {
          inserted += 1;
          insertNext();
        };
        request.onerror = function () {
          insertNext();
        };
      }

      insertNext();
    }).then(function (value) {
      return typeof value === "number" ? value : 0;
    });
  }

  global.FormalAiMemory = {
    appendEvent: appendEvent,
    listEvents: listEvents,
    importEvents: importEvents,
    exportLinksNotation: exportLinksNotation,
    parseLinksNotation: parseLinksNotation,
    formatEvent: formatEvent,
    DB_NAME: DB_NAME,
    STORE_NAME: STORE_NAME,
    ROOT: ROOT_HEADER,
  };
})(typeof window !== "undefined" ? window : globalThis);
