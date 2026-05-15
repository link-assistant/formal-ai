// Tiny Links-Notation reader/writer for the demo's user preferences.
//
// The serialized shape mirrors the indented format used elsewhere in the
// repository (see `data/seed/*.lino`) so the same format that grounds the
// solver's knowledge also grounds the demo's UI state. Keeping the storage
// representation in Links Notation makes the persisted state portable and
// human-readable in DevTools.

(function (global) {
  "use strict";

  var STORAGE_KEY = "formal-ai.preferences.v1";
  var ROOT = "demo_preferences";

  function escapeValue(value) {
    return String(value).replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  }

  function unescapeValue(value) {
    return value.replace(/\\"/g, '"').replace(/\\\\/g, "\\");
  }

  function formatLinksNotation(record) {
    var lines = [ROOT];
    var keys = Object.keys(record);
    for (var index = 0; index < keys.length; index += 1) {
      var key = keys[index];
      var raw = record[key];
      if (raw === undefined || raw === null) {
        continue;
      }
      var value = typeof raw === "boolean" ? (raw ? "on" : "off") : raw;
      lines.push("  " + key + ' "' + escapeValue(value) + '"');
    }
    return lines.join("\n");
  }

  function parseLinksNotation(text) {
    if (typeof text !== "string" || !text.trim()) {
      return null;
    }
    var lines = text.split(/\r?\n/);
    if (lines.length === 0) {
      return null;
    }
    var header = lines.shift();
    if (!header || header.trim() !== ROOT) {
      return null;
    }
    var record = {};
    var entryPattern = /^\s+([a-zA-Z0-9_]+)\s+"((?:[^"\\]|\\.)*)"\s*$/;
    for (var index = 0; index < lines.length; index += 1) {
      var rawLine = lines[index];
      if (!rawLine.trim()) {
        continue;
      }
      var match = entryPattern.exec(rawLine);
      if (!match) {
        continue;
      }
      var key = match[1];
      var value = unescapeValue(match[2]);
      if (value === "on" || value === "off") {
        record[key] = value === "on";
      } else {
        record[key] = value;
      }
    }
    return record;
  }

  function getStorage() {
    try {
      if (typeof global.localStorage === "undefined") {
        return null;
      }
      return global.localStorage;
    } catch (_error) {
      return null;
    }
  }

  function loadPreferences(defaults) {
    var safeDefaults = defaults && typeof defaults === "object" ? defaults : {};
    var storage = getStorage();
    if (!storage) {
      return Object.assign({}, safeDefaults);
    }
    try {
      var raw = storage.getItem(STORAGE_KEY);
      var parsed = parseLinksNotation(raw);
      if (!parsed) {
        return Object.assign({}, safeDefaults);
      }
      return Object.assign({}, safeDefaults, parsed);
    } catch (_error) {
      return Object.assign({}, safeDefaults);
    }
  }

  function savePreferences(values) {
    var storage = getStorage();
    if (!storage) {
      return null;
    }
    try {
      var serialized = formatLinksNotation(values);
      storage.setItem(STORAGE_KEY, serialized);
      return serialized;
    } catch (_error) {
      return null;
    }
  }

  global.FormalAiPreferences = {
    load: loadPreferences,
    save: savePreferences,
    format: formatLinksNotation,
    parse: parseLinksNotation,
    STORAGE_KEY: STORAGE_KEY,
    ROOT: ROOT,
  };
})(typeof window !== "undefined" ? window : globalThis);
