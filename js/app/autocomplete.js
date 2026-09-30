// Issue #825: one autocomplete implementation for every surface where the
// app ships its own input box (the chat composer and the free-text settings
// fields). The logic lives here, framework-free, so the same ranking,
// keyboard contract and input history are testable in node
// (rust/tests/web/issue-825-autocomplete.test.mjs) and reusable by any
// future non-React surface.
//
// Suggestion sources are read-only: the natural-language preference
// commands the chat parser already accepts (COMMAND_SUGGESTIONS mirrors the
// term tables in main.jsx — when a command is added there it is added here),
// the example prompts, and the user's own earlier prompts (localStorage,
// never synced anywhere).

const HISTORY_LIMIT = 8;
const SUGGESTION_LIMIT = 6;

/** Lowercase + strip combining marks so "Тёма" matches "тема". */
function normalizeForMatch(value) {
  return String(value || "")
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .trim();
}

/**
 * Rank `items` against `query`. Items are { value, hint?, source? }.
 * Score: 3 = prefix match, 2 = word-start match, 1 = substring match.
 * Non-matching items are dropped; ties keep the caller's order.
 */
function rankSuggestions(query, items, limit = SUGGESTION_LIMIT) {
  const needle = normalizeForMatch(query);
  if (!needle) return [];
  const ranked = [];
  for (const item of items || []) {
    if (!item || typeof item.value !== "string" || !item.value) continue;
    const haystack = normalizeForMatch(item.value);
    let score = 0;
    if (haystack.startsWith(needle)) score = 3;
    else {
      const wordStart = haystack
        .split(/[^\p{L}\p{N}]+/u)
        .some((word) => word.startsWith(needle));
      if (wordStart) score = 2;
      else if (haystack.includes(needle)) score = 1;
    }
    if (score > 0) ranked.push({ item, score });
  }
  return ranked
    .sort((a, b) => b.score - a.score || a.item.value.localeCompare(b.item.value))
    .slice(0, limit)
    .map((entry) => entry.item);
}

/**
 * Keyboard-driven suggestion state machine. `getItems(query)` returns the
 * candidate pool; `onComplete(value)` fires exactly once per accepted
 * suggestion. `handleKeyDown` returns "handled" when the event was consumed
 * (the host must then skip its own Enter/Tab handling) and "ignored"
 * otherwise. Pure apart from the onComplete callback, so tests drive it
 * without a DOM.
 */
function createAutocompleteController({ getItems, limit = SUGGESTION_LIMIT, onComplete }) {
  if (typeof getItems !== "function") throw new Error("getItems is required");
  if (typeof onComplete !== "function") throw new Error("onComplete is required");
  let open = false;
  let activeIndex = -1;
  let items = [];

  function snapshot() {
    return { open, items: [...items], activeIndex };
  }

  function close() {
    open = false;
    activeIndex = -1;
    items = [];
  }

  function commit(index) {
    const item = items[index];
    if (!item) {
      close();
      return;
    }
    close();
    onComplete(item.value);
  }

  function handleInput(value) {
    const query = String(value || "");
    const trailingWord = query.match(/[^\n]*$/)[0].trim();
    items = rankSuggestions(trailingWord, getItems(trailingWord), limit);
    // Only a partial word is completable: a finished sentence would turn
    // every keystroke into a popup.
    open = items.length > 0 && /[^\s]/.test(trailingWord) && trailingWord.length >= 2;
    activeIndex = open ? 0 : -1;
  }

  function handleKeyDown(event) {
    if (!open || items.length === 0) return "ignored";
    const key = event && event.key;
    if (key === "ArrowDown") {
      activeIndex = (activeIndex + 1) % items.length;
      return "handled";
    }
    if (key === "ArrowUp") {
      activeIndex = (activeIndex - 1 + items.length) % items.length;
      return "handled";
    }
    if (key === "Tab" || (key === "Enter" && !event.shiftKey)) {
      commit(activeIndex >= 0 ? activeIndex : 0);
      return "handled";
    }
    if (key === "Escape") {
      close();
      return "handled";
    }
    return "ignored";
  }

  return { snapshot, close, commit, handleInput, handleKeyDown };
}

/**
 * Input-history store for the free-text settings fields (assistant name,
 * location). localStorage is per-browser and can throw (private mode,
 * blocked storage) — every access is guarded and failures simply yield no
 * history.
 */
function storage() {
  try {
    return window.localStorage;
  } catch (_error) {
    return null;
  }
}

function recallInputValues(key) {
  const store = storage();
  if (!store) return [];
  try {
    const parsed = JSON.parse(store.getItem(`formalAiAutocomplete:${key}`) || "[]");
    return Array.isArray(parsed) ? parsed.filter((v) => typeof v === "string") : [];
  } catch (_error) {
    return [];
  }
}

function rememberInputValue(key, value) {
  const trimmed = String(value || "").trim();
  if (!trimmed) return;
  const store = storage();
  if (!store) return;
  const next = [trimmed, ...recallInputValues(key).filter((v) => v !== trimmed)].slice(0, HISTORY_LIMIT);
  try {
    store.setItem(`formalAiAutocomplete:${key}`, JSON.stringify(next));
  } catch (_error) {
    // History is a convenience; a full quota must not break the input.
  }
}

// The natural-language preference commands the chat parser accepts. These
// mirror the term tables in main.jsx (theme, ui language, ui skin, chat
// style, composer style) — suggestions are only honest if the parser
// actually understands them.
const COMMAND_SUGGESTIONS = [
  { value: "switch to dark theme", source: "command" },
  { value: "switch to light theme", source: "command" },
  { value: "switch to auto theme", source: "command" },
  { value: "switch to English", source: "command" },
  { value: "switch to Russian", source: "command" },
  { value: "switch to Chinese", source: "command" },
  { value: "switch to Hindi", source: "command" },
  { value: "ui skin glass", source: "command" },
  { value: "ui skin contrast", source: "command" },
  { value: "ui skin flat", source: "command" },
  { value: "chat style compact", source: "command" },
  { value: "chat style bubbles", source: "command" },
  { value: "chat style cards", source: "command" },
  { value: "composer style glass clear", source: "command" },
  { value: "composer style glass", source: "command" },
  { value: "composer style flat", source: "command" },
  { value: "composer style bubble", source: "command" },
];

/** Map the app's example prompts into suggestion items. */
function examplePromptSuggestions(prompts) {
  return (prompts || [])
    .filter((entry) => entry && typeof entry.text === "string" && entry.text)
    .map((entry) => ({ value: entry.text, source: "example", hint: entry.label || null }));
}

/** Prompt history as suggestion items, newest first. */
function historySuggestions(values) {
  return (values || [])
    .filter((value) => typeof value === "string" && value.trim())
    .map((value) => ({ value, source: "history" }));
}

export {
  COMMAND_SUGGESTIONS,
  createAutocompleteController,
  examplePromptSuggestions,
  historySuggestions,
  normalizeForMatch,
  rankSuggestions,
  recallInputValues,
  rememberInputValue,
  SUGGESTION_LIMIT,
};
