// UI language (via window.FormalAiI18n) and the user-context facts shared
// with the worker and included in issue reports.

import { normalizeAssistantName } from "./interface-commands.jsx";
import {
  DEFAULT_FOLLOW_UP_PROBABILITY_PERCENT, DEFAULT_GUESS_PROBABILITY_PERCENT,
  DEFAULT_TEMPERATURE_TEXT, PREFERENCE_DEFAULTS, formatSliderValue, normalizeSliderPreference,
} from "./preferences.jsx";

export function i18nApi() {
  return typeof window !== "undefined" && window.FormalAiI18n
    ? window.FormalAiI18n
    : null;
}

export function normalizeUiLanguagePreference(value) {
  if (!value || value === "auto") return "auto";
  const api = i18nApi();
  const normalized = api && api.normalizeLanguageTag
    ? api.normalizeLanguageTag(value)
    : String(value).toLowerCase().split(/[-_]/)[0];
  return normalized || "auto";
}

export function detectUiLanguage(preference) {
  const api = i18nApi();
  if (api && api.detectLanguage) {
    return api.detectLanguage(preference === "auto" ? "" : preference);
  }
  return "en";
}

export function translateUi(key, language, params) {
  const api = i18nApi();
  if (api && api.t) {
    return api.t(key, language, params);
  }
  return key;
}

function browserLanguagesList() {
  if (typeof navigator === "undefined") return [];
  if (Array.isArray(navigator.languages) && navigator.languages.length > 0) {
    return Array.from(navigator.languages);
  }
  return navigator.language ? [navigator.language] : [];
}

function currentColorScheme(themePreference) {
  if (themePreference === "light" || themePreference === "dark") {
    return themePreference;
  }
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return "unknown";
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function resolvedLocale() {
  try {
    return Intl.DateTimeFormat().resolvedOptions().locale || "";
  } catch (_error) {
    return "";
  }
}

function resolvedTimeZone() {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "";
  } catch (_error) {
    return "";
  }
}

export function collectUserContext({
  uiLanguage,
  uiLanguagePreference,
  themePreference,
  uiSkin,
  glassOpacity,
  chatStyle,
  composerStyle,
  composerAction,
  toolbarIconPack,
  locationPreference,
  assistantName,
  guessProbability,
  temperature,
  followUpProbability,
  definitionFusion,
  thinkingDetailLevel,
  experimentalOcr,
}) {
  const browserLanguages = browserLanguagesList();
  const nav = typeof navigator !== "undefined" ? navigator : {};
  const userAgent = nav.userAgent || "";
  const screenInfo =
    typeof screen !== "undefined"
      ? `${screen.width}x${screen.height} @${window.devicePixelRatio || 1}x`
      : "";
  const viewportInfo =
    typeof window !== "undefined" ? `${window.innerWidth}x${window.innerHeight}` : "";
  return {
    uiLanguage,
    uiLanguagePreference,
    themePreference,
    uiSkin,
    glassOpacity,
    chatStyle,
    composerStyle,
    composerAction,
    toolbarIconPack,
    browserLanguage: nav.language || "",
    browserLanguages: browserLanguages.join(", "),
    locale: resolvedLocale(),
    timeZone: resolvedTimeZone(),
    colorScheme: currentColorScheme(themePreference),
    viewport: viewportInfo,
    screen: screenInfo,
    userAgent,
    platform:
      (nav.userAgentData && nav.userAgentData.platform) ||
      nav.platform ||
      "",
    online: typeof nav.onLine === "boolean" ? (nav.onLine ? "yes" : "no") : "",
    preferredLocation: locationPreference || "",
    assistantName: normalizeAssistantName(assistantName) || "not set",
    guessProbability: formatSliderValue(guessProbability),
    temperature: String(normalizeSliderPreference(temperature, 0)),
    followUpProbability: formatSliderValue(followUpProbability),
    definitionFusion,
    thinkingDetailLevel,
    experimentalOcr: experimentalOcr ? "on" : "off",
    locationInference:
      locationPreference
        ? `user-provided preference: ${locationPreference}`
        : "time zone / locale only; exact geolocation was not requested",
  };
}

// Issue #140: the prefilled `Report issue` URL is encoded as `?body=…` and
// GitHub caps the request line at 8192 chars. The verbose User Context block
// previously listed one field per line; now we combine related fields so a
// typical 5-turn dialog fits comfortably under the cap. Defaults and
// not-set values are omitted (UI Skin / Chat Style / Composer Style /
// Composer Action / Online status / Preferred Location), since they are
// uninteresting without the matching memory export.
function formatUiLanguagesField(active, browserLanguagesStr) {
  const browserLanguages = browserLanguagesStr
    ? String(browserLanguagesStr)
        .split(",")
        .map((entry) => entry.trim())
        .filter(Boolean)
    : [];
  const activeStr = String(active || "").trim();
  if (!activeStr && browserLanguages.length === 0) return "unknown";
  const lower = activeStr.toLowerCase();
  const primary = (lang) => String(lang).split(/[-_]/)[0].toLowerCase();
  const matchIndex = browserLanguages.findIndex(
    (lang) => primary(lang) === lower || lang.toLowerCase() === lower,
  );
  if (matchIndex >= 0) {
    return browserLanguages
      .map((lang, idx) => (idx === matchIndex ? `*${lang}*` : lang))
      .join(", ");
  }
  if (!activeStr) return browserLanguages.join(", ");
  if (browserLanguages.length === 0) return `*${activeStr}*`;
  return `*${activeStr}*, ${browserLanguages.join(", ")}`;
}

function formatUiField(context) {
  const parts = [];
  if (context.viewport) parts.push(`${context.viewport} viewport`);
  if (context.screen) parts.push(`${context.screen} screen`);
  if (context.userAgent) parts.push(`${context.userAgent} browser`);
  if (context.platform) parts.push(`${context.platform} platform`);
  return parts.join(", ");
}

function formatLocaleField(context) {
  const locale = context.locale ? String(context.locale).trim() : "";
  const timeZone = context.timeZone ? String(context.timeZone).trim() : "";
  if (locale && timeZone) return `${locale} (${timeZone})`;
  if (locale) return locale;
  if (timeZone) return timeZone;
  return "";
}

function formatThemeField(context) {
  const preference = context.themePreference || "auto";
  const scheme = context.colorScheme || "";
  if (scheme && scheme !== preference) return `${preference} (${scheme})`;
  return preference;
}

// Issue #839: returns the User Context section as data (`{label, value}`), not
// as rendered Markdown — the shared builder in `./issue-report.js` decides how
// a field looks so the web and the CLI cannot drift.
export function userContextFields(context) {
  const safe = context && typeof context === "object" ? context : {};
  const entries = [];
  const push = (label, value) => {
    if (value === undefined || value === null) return;
    const text = String(value).trim();
    if (!text) return;
    entries.push({ label, value: text });
  };

  push("UI languages", formatUiLanguagesField(safe.uiLanguage, safe.browserLanguages));
  // Issue #386: omit settings that are set exactly to their default so the
  // report keeps space for the dialog itself. A field is reported only when it
  // differs from the shipped default (or carries an explicit user value).
  const themePreference = safe.themePreference || PREFERENCE_DEFAULTS.theme;
  if (themePreference !== PREFERENCE_DEFAULTS.theme) {
    push("Theme", formatThemeField(safe));
  }
  push("UI", formatUiField(safe));
  push("Locale", formatLocaleField(safe));
  if (safe.preferredLocation) {
    push("Preferred location", safe.preferredLocation);
  }
  if (safe.guessProbability !== DEFAULT_GUESS_PROBABILITY_PERCENT) {
    push("Guess probability", `${safe.guessProbability || "unknown"}%`);
  }
  if (safe.temperature !== DEFAULT_TEMPERATURE_TEXT) {
    push("Temperature", safe.temperature);
  }
  if (safe.followUpProbability !== DEFAULT_FOLLOW_UP_PROBABILITY_PERCENT) {
    push("Follow-up probability", `${safe.followUpProbability || "unknown"}%`);
  }
  const thinkingDetailLevel =
    safe.thinkingDetailLevel || PREFERENCE_DEFAULTS.thinkingDetailLevel;
  if (thinkingDetailLevel !== PREFERENCE_DEFAULTS.thinkingDetailLevel) {
    push("Thinking detail", thinkingDetailLevel);
  }
  const toolbarIconPack = safe.toolbarIconPack || PREFERENCE_DEFAULTS.toolbarIconPack;
  if (toolbarIconPack !== PREFERENCE_DEFAULTS.toolbarIconPack) {
    push("Toolbar icon pack", toolbarIconPack);
  }
  // Issue #386: the inference-only location ("time zone / locale only") is the
  // default, so it is omitted. An explicit preference is reported above.

  return entries;
}
