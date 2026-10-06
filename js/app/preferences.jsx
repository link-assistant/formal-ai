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
export const EXTERNAL_TRUSTED_SERVICES = [
  { key: "externalServiceWikihow", label: "settings.externalServiceWikihow" },
  { key: "externalServiceStackExchange", label: "settings.externalServiceStackExchange" },
  { key: "externalServiceMediawikiFamily", label: "settings.externalServiceMediawikiFamily" },
  { key: "externalServiceGithub", label: "settings.externalServiceGithub" },
  // Issue #1138 plan 01 L4: the lexical tier joined the live, opt-out-able
  // group when it gained a `need_kinds`, so the two settings keys that silence
  // a dictionary belong here too. Without them a user could opt out of every
  // procedural source and still have no way to opt out of the ones that answer
  // "what does this word mean".
  { key: "externalServiceWiktionary", label: "settings.externalServiceWiktionary" },
  { key: "externalServiceWordnet", label: "settings.externalServiceWordnet" },
];

const LEGACY_EXPANDED_SIDEBAR_KEYS = [
  "sidebarSettingsCollapsed",
  "sidebarToolsCollapsed",
  "sidebarTraceCollapsed",
  "sidebarDesktopCollapsed",
  "sidebarServicesCollapsed",
];

export const PREFERENCE_DEFAULTS = {
  demoMode: true,
  diagnosticsMode: false,
  contextPanelWidth: 300,
  // Issue #27: each sidebar section is a VS Code-style collapsible region; the
  // last expand/collapse state is persisted via FormalAiPreferences so opening
  // the demo never reshuffles the user's layout.
  sidebarMenuCollapsed: true,
  sidebarPromptsCollapsed: false,
  sidebarToolsCollapsed: true,
  sidebarTraceCollapsed: true,
  sidebarConversationsCollapsed: false,
  sidebarSettingsCollapsed: true,
  sidebarDesktopCollapsed: true,
  sidebarServicesCollapsed: true,
  // Issue #153: the side panel is collapsible to give the chat full viewport
  // width on desktop. The drawer view on mobile stays controlled by the
  // separate `mobileMenuOpen` toggle so phones can still slide it in.
  sidebarCollapsed: false,
  showDeletedConversations: false,
  // Issue #27: random greeting variations are opt-in but default to on so
  // newcomers see the multilingual surface immediately.
  greetingVariations: true,
  // Issue #488: user-facing thinking can be compact or detailed without
  // changing the raw diagnostics available to maintainers.
  // Issue #541 (R8): default to the 50% midpoint ("standard"), which surfaces
  // only the high-level human-readable steps (not the mechanical sub-steps), so
  // newcomers are not overwhelmed; "detailed" remains one notch away for power
  // users and still renders fully human-readable prose (no symbolic syntax).
  thinkingDetailLevel: "standard",
  // Issue #541 (R5): minimum wall-clock time, in milliseconds, that a freshly
  // produced assistant answer spends animating its reasoning + reveal so the
  // user *feels* the thinking happen even when the deterministic engine answers
  // instantly. 0 = immediate display; the shipped default is a relaxed 2s.
  minMessageAnimationMs: 2000,
  // Issue #82: user-tunable assistant behavior. The default still guesses
  // likely typo matches, while the sliders let cautious users ask first and
  // deterministic users turn random response variation off with temperature=0.
  guessProbability: 0.8,
  temperature: 0.7,
  // Issue #160 follow-up: polite courtesy responses can either leave the
  // initiative with the user or ask/propose the next action. This probability
  // controls whether the next-action sentence is appended.
  followUpProbability: 0.75,
  // Issue #63: definition fusion remains explicit-only by default, with an
  // opt-in mode that treats plain "What is X?" prompts as merge requests.
  definitionFusion: "explicit",
  // Issue #340: how composite-program blueprints project their annotated recipe
  // template into the program shown to the user.
  //   "composed" (default) — emit only the regions the request actually named,
  //                          so the program is a projection of the decomposition;
  //   "documented"         — always emit the fully documented program with every
  //                          optional region (error handling, comments) present.
  blueprintComposition: "composed",
  experimentalOcr: false,
  // Issue #444: external trusted-service opt-outs. Defined data-driven from
  // EXTERNAL_TRUSTED_SERVICES; every service ships enabled (opt-out model) so the
  // assistant keeps consulting them unless the user disables one in settings.
  ...Object.fromEntries(EXTERNAL_TRUSTED_SERVICES.map((service) => [service.key, true])),
  associativeProjectPromotion: true,
  theme: "auto",
  location: "",
  assistantName: "",
  // Issue #27: id of the conversation the user last typed in; on reload the
  // demo restores its event log into the main transcript. Empty string means
  // "no conversation yet — start a fresh one on first user input".
  currentConversationId: "",
  // Issue #27: Chat (single-turn Q&A) vs Agent (multi-step plan + execute) mode.
  // Persisted so the user keeps their preferred operating surface across
  // reloads. Agent mode in the browser sandbox decomposes the prompt into
  // sequential sub-tasks and runs each through the existing solver; a future
  // iteration will wire it to docker / WebVM execution.
  agentMode: false,
  // Issue #513: three-way operating mode replacing the binary agent toggle.
  //   "chat"     — single-turn reasoning, no command execution;
  //   "agent"    — multi-step plan + execute, capabilities gated on grant;
  //   "fullAuto" — agent mode that runs permitted commands automatically.
  // The legacy `agentMode` boolean is derived as `mode !== "chat"` so existing
  // readers (worker prefs, desktop tool grants) keep working unchanged.
  mode: "chat",
  // Issue #514: first-run Agent/Full Auto onboarding and per-tool grant
  // decisions are persisted independently. The grant string is a compact
  // Links-friendly map such as `shell:on,http_fetch:off`.
  agentOnboardingSeen: false,
  desktopToolGrants: "",
  // Issue #94: "auto" follows navigator.languages; explicit values use the
  // supported UI language catalog.
  uiLanguage: "auto",
  // Issue #324: which language drives the assistant's responses.
  //   "last_message" (default) — answer in the detected language of the prompt;
  //   "preferred"             — pin responses to `preferredLanguage`;
  //   "ui"                    — follow the UI-language preference.
  // The default reproduces the deterministic "reply in the message's language"
  // behavior, so a Russian prompt is answered in Russian.
  responseLanguage: "last_message",
  // Issue #324: the explicit language used when `responseLanguage` is
  // "preferred". One of the supported response languages (en/ru/hi/zh).
  preferredLanguage: "en",
  // Issues #108/#110: UI, chat, and input surfaces are configurable while the
  // defaults stay flat and cheap to render.
  uiSkin: "flat",
  glassOpacity: 0.78,
  chatStyle: "cards",
  composerStyle: "flat",
  composerAction: "attach",
  toolbarIconPack: "fontawesome",
};

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
