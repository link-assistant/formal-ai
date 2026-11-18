// User preferences: shipped defaults, option lists, localStorage persistence
// and the normalizers that keep stored values within their allowed ranges.

// Issue #444: the assistant may consult a small set of external *trusted*
// services (wikiHow, Stack Exchange, the MediaWiki sister projects, GitHub) when
// it answers procedural "how to X" prompts and project lookups. The maintainer
// asked for a settings section to opt in or out of each one. This single
// data-driven catalog drives the preference defaults, the worker prefs payload,
// the reset descriptors, and the settings panel checkboxes, so adding a new
// trusted service later means appending one row here (plus an i18n label) rather
// than editing six call sites. `key` mirrors the `settings_key` recorded in
// `data/seed/sources-registry.lino`; every service is opt-out (default enabled),
// so existing behavior is preserved unless the user turns one off.


const LEGACY_EXPANDED_SIDEBAR_KEYS = [
  "sidebarSettingsCollapsed",
  "sidebarToolsCollapsed",
  "sidebarTraceCollapsed",
  "sidebarDesktopCollapsed",
  "sidebarServicesCollapsed",
];

import {PREFERENCE_DEFAULTS, EXTERNAL_TRUSTED_SERVICES} from "../preferences/defaults.js";
export {PREFERENCE_DEFAULTS, EXTERNAL_TRUSTED_SERVICES} from "../preferences/defaults.js";

// Issue #386: precompute the formatted default values so the issue report can
// omit any User Context field that matches its shipped default. Keeping these
// derived from PREFERENCE_DEFAULTS means they stay in sync if a default moves.
export const DEFAULT_GUESS_PROBABILITY_PERCENT = formatSliderValue(
  PREFERENCE_DEFAULTS.guessProbability,
);

export const DEFAULT_TEMPERATURE_TEXT = String(
  normalizeSliderPreference(PREFERENCE_DEFAULTS.temperature, 0),
);

export const DEFAULT_FOLLOW_UP_PROBABILITY_PERCENT = formatSliderValue(
  PREFERENCE_DEFAULTS.followUpProbability,
);

// Issue #386: the settings panel lets the user reset each setting (or all of
// them) back to the shipped default. A setting is "modified" when its current
// value differs from PREFERENCE_DEFAULTS; numeric sliders are compared
// numerically so 0.8 and "0.8" are treated as equal.
export function settingIsDefault(key, value) {
  const fallback = PREFERENCE_DEFAULTS[key];
  if (typeof fallback === "number") {
    return Number(value) === fallback;
  }
  return value === fallback;
}

const UI_SKINS = ["flat", "glass", "material", "contrast"];

const CHAT_STYLES = ["cards", "compact", "bubbles"];

const COMPOSER_STYLES = ["flat", "glass-soft", "glass-clear", "bubble"];

const COMPOSER_ACTIONS = ["attach", "plus"];

const TOOLBAR_ICON_PACKS = [
  "fontawesome",
  "material-symbols",
  "bootstrap-icons",
  "ionicons",
  "remix-icon",
  "tabler-icons",
  "names",
];

const DEFINITION_FUSION_MODES = ["explicit", "auto"];

// Issue #340: blueprint program-composition strategies. "composed" projects the
// program from the detected capabilities; "documented" always emits the full
// annotated program with every optional region present.
const BLUEPRINT_COMPOSITION_MODES = ["composed", "documented"];

const THINKING_DETAIL_LEVELS = ["brief", "standard", "detailed"];

// Issue #513: the three-way operating modes shown in the toolbar radio group.
export const MODE_OPTIONS = ["chat", "agent", "fullAuto"];

export const MODE_LABEL_KEYS = {
  chat: "buttons.chat",
  agent: "buttons.agent",
  fullAuto: "buttons.fullAuto",
};

export const MODE_TITLE_KEYS = {
  chat: "titles.agentOff",
  agent: "titles.agentOn",
  fullAuto: "titles.fullAuto",
};

// Issue #514: the renderer mirrors the desktop tool vocabulary so it can send a
// per-tool grant map to the native router instead of the old all-or-nothing
// grant. Keep this list in sync with desktop/lib/tool-router.cjs.
export const DESKTOP_TOOL_OPTIONS = Object.freeze([
  "eval_js",
  "write_file",
  "edit_file",
  "multi_edit",
  "code_exec",
  "shell",
  "fs.read",
  "fs.write",
  "fs.list",
  "fs.move",
  "shell.run",
  "http.fetch",
  "http.post",
  "dom.query",
  "dom.extract",
  "archive.pack",
  "archive.unpack",
  "process.status",
]);

export const DESKTOP_TOOL_I18N_KEYS = Object.freeze({
  "fs.read": "computer_fs_read",
  "fs.write": "computer_fs_write",
  "fs.list": "computer_fs_list",
  "fs.move": "computer_fs_move",
  "shell.run": "computer_shell_run",
  "http.fetch": "computer_http_fetch",
  "http.post": "computer_http_post",
  "dom.query": "computer_dom_query",
  "dom.extract": "computer_dom_extract",
  "archive.pack": "computer_archive_pack",
  "archive.unpack": "computer_archive_unpack",
  "process.status": "computer_process_status",
});

// Issue #511/#514: per-tool labels and descriptions live in the i18n catalog
// (permissions.tool.<key>.{label,description}) so the desktop permission panel
// translates with the active UI language instead of shipping hardcoded English.
// Links Notation treats dots as path separators, so dotted computer-use tool names
// map to stable catalog keys while the native permission map keeps the exact names.
// Issue #324: source that drives the assistant's response language.
const RESPONSE_LANGUAGE_MODES = ["last_message", "preferred", "ui"];

// Issue #324: languages the assistant can be pinned to via `preferredLanguage`.
const PREFERRED_RESPONSE_LANGUAGES = ["en", "ru", "hi", "zh"];

export const CONTEXT_PANEL_MIN_WIDTH = 220;

const CONTEXT_PANEL_MAX_WIDTH = 560;

const CONTEXT_PANEL_MIN_CHAT_WIDTH = 360;

const CONTEXT_PANEL_RESIZER_WIDTH = 10;

function readStoredPreferencesRecord() {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return null;
  }
  const parser = window.FormalAiPreferences.parse;
  if (typeof parser !== "function") {
    return null;
  }
  try {
    const storage = window.localStorage;
    if (!storage) return null;
    const key = window.FormalAiPreferences.STORAGE_KEY || "formal-ai.preferences.v1";
    return parser(storage.getItem(key));
  } catch (_error) {
    return null;
  }
}

function migrateStoredSidebarCollapsePreferences(preferences, storedRecord) {
  if (!storedRecord || typeof storedRecord !== "object") {
    return preferences;
  }
  let next = preferences;
  for (const key of LEGACY_EXPANDED_SIDEBAR_KEYS) {
    if (!Object.prototype.hasOwnProperty.call(storedRecord, key)) {
      if (next === preferences) next = { ...preferences };
      next[key] = false;
    }
  }
  return next;
}

export function loadPreferences() {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return { ...PREFERENCE_DEFAULTS };
  }
  try {
    const storedRecord = readStoredPreferencesRecord();
    const preferences = window.FormalAiPreferences.load(PREFERENCE_DEFAULTS);
    return migrateStoredSidebarCollapsePreferences(preferences, storedRecord);
  } catch (_error) {
    return { ...PREFERENCE_DEFAULTS };
  }
}

export function persistPreferences(values) {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return;
  }
  try {
    window.FormalAiPreferences.save(values);
  } catch (_error) {
    // localStorage may be unavailable (private mode, sandboxed iframe); ignore.
  }
}

export function clampNumber(value, min, max, fallback) {
  const number = Number(value);
  if (!Number.isFinite(number)) return fallback;
  return Math.min(max, Math.max(min, number));
}

export function normalizeSliderPreference(value, fallback) {
  return clampNumber(value, 0, 1, fallback);
}

export function formatSliderValue(value) {
  return String(Math.round(normalizeSliderPreference(value, 0) * 100));
}

export function contextPanelMaxWidth() {
  if (typeof window === "undefined") {
    return CONTEXT_PANEL_MAX_WIDTH;
  }
  const viewportWidth =
    window.visualViewport && window.visualViewport.width
      ? window.visualViewport.width
      : window.innerWidth;
  const available = Math.round(
    viewportWidth - CONTEXT_PANEL_MIN_CHAT_WIDTH - CONTEXT_PANEL_RESIZER_WIDTH,
  );
  return Math.max(
    CONTEXT_PANEL_MIN_WIDTH,
    Math.min(CONTEXT_PANEL_MAX_WIDTH, available),
  );
}

export function normalizeContextPanelWidth(value) {
  return Math.round(
    clampNumber(
      value,
      CONTEXT_PANEL_MIN_WIDTH,
      contextPanelMaxWidth(),
      PREFERENCE_DEFAULTS.contextPanelWidth,
    ),
  );
}

export function normalizeThemePreference(value) {
  return ["auto", "light", "dark"].includes(value) ? value : "auto";
}

export function normalizeUiSkin(value) {
  return UI_SKINS.includes(value) ? value : PREFERENCE_DEFAULTS.uiSkin;
}

export function normalizeChatStyle(value) {
  return CHAT_STYLES.includes(value) ? value : PREFERENCE_DEFAULTS.chatStyle;
}

export function normalizeComposerStyle(value) {
  return COMPOSER_STYLES.includes(value) ? value : PREFERENCE_DEFAULTS.composerStyle;
}

export function normalizeComposerAction(value) {
  return COMPOSER_ACTIONS.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.composerAction;
}

export function normalizeToolbarIconPack(value) {
  return TOOLBAR_ICON_PACKS.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.toolbarIconPack;
}

export function normalizeDefinitionFusion(value) {
  return DEFINITION_FUSION_MODES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.definitionFusion;
}

export function normalizeBlueprintComposition(value) {
  return BLUEPRINT_COMPOSITION_MODES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.blueprintComposition;
}

export function normalizeThinkingDetailLevel(value) {
  return THINKING_DETAIL_LEVELS.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.thinkingDetailLevel;
}

// Issue #541 (R5): the minimum message-animation budget is a millisecond count
// clamped to a sane range. 0 means "show the answer immediately" (no animation);
// the upper bound keeps a mis-set preference from freezing the UI for minutes.
// Non-numeric / NaN input falls back to the shipped 2s default.
const MIN_MESSAGE_ANIMATION_MAX_MS = 8000;

export function normalizeAnimationBudgetMs(value) {
  const number = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(number)) {
    return PREFERENCE_DEFAULTS.minMessageAnimationMs;
  }
  const clamped = Math.min(Math.max(number, 0), MIN_MESSAGE_ANIMATION_MAX_MS);
  return Math.round(clamped);
}

export {OUTPUT_STANDARDS, normalizeOutputStandard, normalizeLexicalDensity} from "../preferences/controlled-output.js";

export function normalizeResponseLanguageMode(value) {
  return RESPONSE_LANGUAGE_MODES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.responseLanguage;
}

// Issue #513: resolve the persisted operating mode. Falls back to the legacy
// `agentMode` boolean so users who saved preferences before the radio existed
// keep their agent opt-in (true -> "agent"), and unknown values reset to chat.
export function normalizeMode(value, legacyAgentMode) {
  // A newer client that writes mode="chat" also writes the derived agentMode as
  // false, so the only way to see mode="chat" alongside a truthy legacy
  // agentMode is a pre-#513 preference store that only had `agentMode "on"`
  // (mode defaulting to "chat"). In that case upgrade to the agent radio.
  if (MODE_OPTIONS.includes(value) && !(value === "chat" && legacyAgentMode)) {
    return value;
  }
  return legacyAgentMode ? "agent" : PREFERENCE_DEFAULTS.mode;
}

export function normalizeDesktopToolGrants(value) {
  const normalized = {};
  const apply = (tool, granted) => {
    if (!DESKTOP_TOOL_OPTIONS.includes(tool) || typeof granted !== "boolean") {
      return;
    }
    normalized[tool] = granted;
  };
  if (value && typeof value === "object" && !Array.isArray(value)) {
    DESKTOP_TOOL_OPTIONS.forEach((tool) => {
      if (value[tool] === true || value[tool] === false) {
        apply(tool, value[tool]);
      }
    });
    return normalized;
  }
  const text = String(value || "").trim();
  if (!text) {
    return normalized;
  }
  text.split(/[,;\n]+/).forEach((entry) => {
    const match = /^\s*([a-z0-9_]+)\s*[:=]\s*([a-z0-9_-]+)\s*$/i.exec(entry);
    if (!match) {
      return;
    }
    const state = match[2].toLowerCase();
    if (["on", "true", "1", "grant", "granted"].includes(state)) {
      apply(match[1], true);
    } else if (["off", "false", "0", "decline", "declined", "deny", "denied"].includes(state)) {
      apply(match[1], false);
    }
  });
  return normalized;
}

export function serializeDesktopToolGrants(grants) {
  const safe = grants && typeof grants === "object" ? grants : {};
  return DESKTOP_TOOL_OPTIONS
    .filter((tool) => safe[tool] === true || safe[tool] === false)
    .map((tool) => `${tool}:${safe[tool] ? "on" : "off"}`)
    .join(",");
}

export function desktopToolRouterGrants(mode, grants) {
  const active = mode !== "chat";
  const safe = grants && typeof grants === "object" ? grants : {};
  const out = { all: false };
  DESKTOP_TOOL_OPTIONS.forEach((tool) => {
    out[tool] = active && safe[tool] === true;
  });
  return out;
}

export function desktopToolGrantCount(grants) {
  const safe = grants && typeof grants === "object" ? grants : {};
  return DESKTOP_TOOL_OPTIONS.filter((tool) => safe[tool] === true).length;
}

export function desktopToolGrantState(grants, tool) {
  const safe = grants && typeof grants === "object" ? grants : {};
  if (safe[tool] === true) return "granted";
  if (safe[tool] === false) return "declined";
  return "undecided";
}

export function normalizePreferredLanguage(value) {
  return PREFERRED_RESPONSE_LANGUAGES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.preferredLanguage;
}
