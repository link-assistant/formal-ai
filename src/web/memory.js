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
  var BUNDLE_HEADER = "formal_ai_bundle";
  // Schema is intentionally additive. Older logs without "kind" still parse
  // as plain user/assistant turns. New "kind" values record reasoning steps,
  // tool invocations, decisions, and other internal events so the log is a
  // complete projection of what the agent did.
  var EXPORT_FIELDS = [
    "kind",
    "role",
    "intent",
    "tool",
    "inputs",
    "outputs",
    "content",
    "sentAt",
    "demoLabel",
    "evidence",
    // Issue #27: conversation grouping. Events keep their global ordering in
    // the append-only log; conversationId/conversationTitle just attribute each
    // event to a specific chat thread so the UI can filter on read.
    "conversationId",
    "conversationTitle",
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
    if (event.kind) record.kind = String(event.kind);
    if (event.intent) record.intent = String(event.intent);
    if (event.tool) record.tool = String(event.tool);
    if (event.inputs !== undefined && event.inputs !== null) {
      record.inputs =
        typeof event.inputs === "string"
          ? event.inputs
          : JSON.stringify(event.inputs);
    }
    if (event.outputs !== undefined && event.outputs !== null) {
      record.outputs =
        typeof event.outputs === "string"
          ? event.outputs
          : JSON.stringify(event.outputs);
    }
    if (event.demoLabel) record.demoLabel = String(event.demoLabel);
    if (Array.isArray(event.evidence)) record.evidence = event.evidence.slice();
    if (event.conversationId) record.conversationId = String(event.conversationId);
    if (event.conversationTitle)
      record.conversationTitle = String(event.conversationTitle);
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
        if (raw.kind) record.kind = String(raw.kind);
        if (raw.intent) record.intent = String(raw.intent);
        if (raw.tool) record.tool = String(raw.tool);
        if (raw.inputs !== undefined && raw.inputs !== null) {
          record.inputs =
            typeof raw.inputs === "string"
              ? raw.inputs
              : JSON.stringify(raw.inputs);
        }
        if (raw.outputs !== undefined && raw.outputs !== null) {
          record.outputs =
            typeof raw.outputs === "string"
              ? raw.outputs
              : JSON.stringify(raw.outputs);
        }
        if (raw.demoLabel) record.demoLabel = String(raw.demoLabel);
        if (Array.isArray(raw.evidence)) record.evidence = raw.evidence.slice();
        if (raw.conversationId) record.conversationId = String(raw.conversationId);
        if (raw.conversationTitle)
          record.conversationTitle = String(raw.conversationTitle);
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

  function indentBlock(text, indent) {
    var prefix = indent || "  ";
    return String(text || "")
      .split(/\r?\n/)
      .filter(function (line) { return line.length > 0; })
      .map(function (line) { return prefix + line; })
      .join("\n");
  }

  function infoFieldName(key) {
    return String(key || "")
      .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
      .replace(/[^a-zA-Z0-9_]+/g, "_")
      .replace(/^_+|_+$/g, "")
      .toLowerCase();
  }

  function appendInfoLine(lines, name, value) {
    if (!name || value === undefined || value === null || value === "") return;
    lines.push('  ' + name + ' "' + escapeValue(value) + '"');
  }

  // Combine app metadata + every seed file + the entire append-only event log
  // into a single Links Notation document. The output is the canonical
  // "report-ready" debug snapshot — paste it into a GitHub issue and the
  // maintainer can reconstruct the agent's full state.
  function exportBundle(options) {
    var settings = options || {};
    var seed = settings.seed || {};
    var events = Array.isArray(settings.events) ? settings.events : [];
    var info = settings.info || {};
    var preferences = settings.preferences || null;
    var lines = ["formal_ai_bundle"];
    lines.push('  exported_at "' + escapeValue(new Date().toISOString()) + '"');
    var preferredInfoFields = [
      "version",
      "url",
      "userAgent",
      "workerState",
      "mode",
      "uiLanguage",
      "uiLanguagePreference",
      "browserLanguage",
      "browserLanguages",
      "locale",
      "timeZone",
      "colorScheme",
      "viewport",
      "screen",
      "platform",
      "online",
      "locationInference",
    ];
    var writtenInfo = {};
    preferredInfoFields.forEach(function (key) {
      var name = infoFieldName(key);
      writtenInfo[key] = true;
      appendInfoLine(lines, name, info[key]);
    });
    Object.keys(info).sort().forEach(function (key) {
      if (writtenInfo[key]) return;
      appendInfoLine(lines, infoFieldName(key), info[key]);
    });
    var seedFiles = seed && seed.raw ? Object.keys(seed.raw) : [];
    if (seedFiles.length > 0) {
      lines.push("  seed_files");
      seedFiles.forEach(function (filename) {
        lines.push('    file "' + escapeValue(filename) + '"');
        lines.push(indentBlock(seed.raw[filename], "      "));
      });
    }
    if (preferences && typeof preferences === "object") {
      var prefKeys = Object.keys(preferences);
      if (prefKeys.length > 0) {
        lines.push("  preferences");
        prefKeys.forEach(function (key) {
          var raw = preferences[key];
          if (raw === undefined || raw === null) return;
          var value =
            typeof raw === "boolean" ? (raw ? "on" : "off") : String(raw);
          lines.push('    ' + key + ' "' + escapeValue(value) + '"');
        });
      }
    }
    lines.push("  " + ROOT_HEADER);
    events.forEach(function (event) {
      lines.push("  " + formatEvent(event));
    });
    return lines.join("\n") + "\n";
  }

  // Default export entry point. Always emits the full self-contained
  // `formal_ai_bundle` (seed + events + preferences + metadata) so a single
  // file is enough for a maintainer to replay the agent's state. Older code
  // paths and tests can still call `exportBundle`/`exportLinksNotation`
  // directly — `exportFullMemory` is the new canonical name.
  function exportFullMemory(options) {
    return exportBundle(options || {});
  }

  // Parse either a `formal_ai_bundle` document (the new full-memory format)
  // or the legacy `demo_memory` event log. Returns a small descriptor object
  // describing what was found so the caller can act on the structure:
  //
  //   { kind: "bundle", events, seedFiles, preferences, info, agentInfo }
  //   { kind: "memory", events, seedFiles: {}, preferences: null, info: {}, agentInfo: {} }
  //
  // `seedFiles` is a plain `{ filename: contents }` object; `info` carries
  // the bundle metadata (`exported_at`, `version`, `url`, `user_agent`,
  // `worker_state`, `mode`); `agentInfo` mirrors the `agent_info` map parsed
  // out of `seed/agent-info.lino` when present (used by
  // `suggestMigrations`).
  function importFullMemory(text) {
    var safe = typeof text === "string" ? text : "";
    var firstLine = safe.split(/\r?\n/, 1)[0] || "";
    if (firstLine.trim() === BUNDLE_HEADER) {
      return parseBundleDocument(safe);
    }
    return {
      kind: "memory",
      events: parseLinksNotation(safe),
      seedFiles: {},
      preferences: null,
      info: {},
      agentInfo: {},
    };
  }

  // Parser for the bundle document. Walks the indentation manually so it
  // does not pull in `FormalAiSeed.parseBundle` (which only returns the seed
  // files). The parser is forgiving: unknown sub-sections are skipped, and a
  // truncated document still yields whatever events were recoverable.
  function parseBundleDocument(text) {
    var lines = text.split(/\r?\n/);
    var info = {};
    var seedFiles = {};
    var preferences = null;
    var agentInfo = {};
    var memoryLines = [];
    var section = null; // null | "seed_files" | "preferences" | "memory"
    var currentSeedFile = null;
    var index = 0;
    while (index < lines.length) {
      var line = lines[index];
      index += 1;
      if (!line) continue;
      var indentMatch = /^( *)/.exec(line);
      var indent = indentMatch ? indentMatch[1].length : 0;
      var content = line.slice(indent);
      if (indent === 0) {
        section = null;
        continue;
      }
      if (indent === 2) {
        // Top-level subsection inside the bundle.
        if (content === "seed_files") {
          section = "seed_files";
          currentSeedFile = null;
          continue;
        }
        if (content === "preferences") {
          section = "preferences";
          preferences = {};
          continue;
        }
        if (content === ROOT_HEADER) {
          section = "memory";
          memoryLines = [ROOT_HEADER];
          continue;
        }
        var infoMatch = /^([a-zA-Z0-9_]+)\s+"((?:[^"\\]|\\.)*)"\s*$/.exec(content);
        if (infoMatch) {
          info[infoMatch[1]] = unescapeValue(infoMatch[2]);
        }
        section = null;
        continue;
      }
      if (section === "seed_files") {
        if (indent === 4) {
          var fileMatch = /^file\s+"((?:[^"\\]|\\.)*)"\s*$/.exec(content);
          if (fileMatch) {
            currentSeedFile = unescapeValue(fileMatch[1]);
            seedFiles[currentSeedFile] = "";
          }
          continue;
        }
        if (currentSeedFile && indent >= 6) {
          // Body lines for the current seed file. Strip the 6-space prefix.
          var body = line.length >= 6 ? line.slice(6) : "";
          if (seedFiles[currentSeedFile].length > 0) {
            seedFiles[currentSeedFile] += "\n";
          }
          seedFiles[currentSeedFile] += body;
        }
        continue;
      }
      if (section === "preferences") {
        if (indent === 4) {
          var prefMatch = /^([a-zA-Z0-9_]+)\s+"((?:[^"\\]|\\.)*)"\s*$/.exec(content);
          if (prefMatch) {
            var prefValue = unescapeValue(prefMatch[2]);
            preferences[prefMatch[1]] =
              prefValue === "on" ? true : prefValue === "off" ? false : prefValue;
          }
        }
        continue;
      }
      if (section === "memory") {
        // Strip the leading 2 spaces of bundle indentation so the captured
        // block matches the standalone `demo_memory` shape parsed by
        // `parseLinksNotation`.
        var stripped = line.length >= 2 ? line.slice(2) : line;
        memoryLines.push(stripped);
        continue;
      }
    }
    var events = memoryLines.length > 0
      ? parseLinksNotation(memoryLines.join("\n"))
      : [];
    if (seedFiles["seed/agent-info.lino"]) {
      agentInfo = parseAgentInfo(seedFiles["seed/agent-info.lino"]);
    } else if (seedFiles["data/seed/agent-info.lino"]) {
      // Tolerate the canonical path too — bundles produced by the CLI emit
      // `data/seed/...` while the browser emits `seed/...`.
      agentInfo = parseAgentInfo(seedFiles["data/seed/agent-info.lino"]);
    }
    return {
      kind: "bundle",
      events: events,
      seedFiles: seedFiles,
      preferences: preferences,
      info: info,
      agentInfo: agentInfo,
    };
  }

  // Tiny parser for the `agent_info` block — extracts `field "<key>" \n
  // value "<value>"` pairs so callers can look up the embedded seed
  // version, supported languages, etc. without pulling in the larger seed
  // loader.
  function parseAgentInfo(text) {
    var out = {};
    var lines = String(text || "").split(/\r?\n/);
    var currentField = null;
    var fieldPattern = /^\s{2}field\s+"((?:[^"\\]|\\.)*)"\s*$/;
    var valuePattern = /^\s{4}value\s+"((?:[^"\\]|\\.)*)"\s*$/;
    for (var i = 0; i < lines.length; i += 1) {
      var line = lines[i];
      var fieldMatch = fieldPattern.exec(line);
      if (fieldMatch) {
        currentField = unescapeValue(fieldMatch[1]);
        continue;
      }
      var valueMatch = valuePattern.exec(line);
      if (valueMatch && currentField) {
        out[currentField] = unescapeValue(valueMatch[1]);
        currentField = null;
      }
    }
    return out;
  }

  // Suggest known data migrations between an imported document and the
  // currently running app. The first migration check covers the seed
  // version baked into `agent-info.lino`. Returns an array of human-readable
  // strings — empty when no migration is needed. The function is pure so
  // tests can call it without a browser environment.
  function suggestMigrations(options) {
    var settings = options || {};
    var imported = settings.imported || {};
    var current = settings.current || {};
    var suggestions = [];
    var importedAgent = imported.agentInfo || {};
    var currentAgent = current.agentInfo || {};
    var importedVersion = importedAgent.version || (imported.info && imported.info.version);
    var currentVersion = currentAgent.version || (current.info && current.info.version);
    if (importedVersion && currentVersion && importedVersion !== currentVersion) {
      suggestions.push(
        "Seed version " + importedVersion + " → " + currentVersion +
          ": review the new entries in data/seed/ (multilingual responses, concepts, tools) — your imported memory was authored against an older seed.",
      );
    } else if (importedVersion && !currentVersion) {
      suggestions.push(
        "Imported bundle was authored against seed version " + importedVersion +
          " but the running app does not expose a seed version. Update the app to compare.",
      );
    }
    if (imported.kind === "memory") {
      suggestions.push(
        "Imported file is a legacy demo_memory log (no seed). The events were imported, but the seed at the time of capture is unknown — export from this session to upgrade to a full bundle.",
      );
    }
    return suggestions;
  }

  global.FormalAiMemory = {
    appendEvent: appendEvent,
    listEvents: listEvents,
    importEvents: importEvents,
    exportLinksNotation: exportLinksNotation,
    exportBundle: exportBundle,
    exportFullMemory: exportFullMemory,
    importFullMemory: importFullMemory,
    suggestMigrations: suggestMigrations,
    parseLinksNotation: parseLinksNotation,
    parseBundleDocument: parseBundleDocument,
    parseAgentInfo: parseAgentInfo,
    formatEvent: formatEvent,
    DB_NAME: DB_NAME,
    STORE_NAME: STORE_NAME,
    ROOT: ROOT_HEADER,
    BUNDLE_ROOT: BUNDLE_HEADER,
  };
})(typeof window !== "undefined" ? window : globalThis);
