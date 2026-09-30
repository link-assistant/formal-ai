// Issue #825: autocomplete in every own-input-box surface. The engine in
// js/app/autocomplete.js is framework-free, so these tests drive ranking,
// the keyboard contract and the localStorage input history directly, plus
// the catalog parity of the strings the composer listbox renders.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  COMMAND_SUGGESTIONS,
  createAutocompleteController,
  examplePromptSuggestions,
  historySuggestions,
  normalizeForMatch,
  rankSuggestions,
  recallInputValues,
  rememberInputValue,
} from "../../../js/app/autocomplete.js";

import { REPO_ROOT } from "./support/browser-runtime.mjs";

test("normalizeForMatch is case- and diacritic-insensitive", () => {
  assert.equal(normalizeForMatch("Тёма"), "тема");
  assert.equal(normalizeForMatch("  Switch TO Theme "), "switch to theme");
  assert.equal(normalizeForMatch(""), "");
});

test("ranking prefers prefix, then word start, then substring", () => {
  const items = [
    { value: "chat style cards" },
    { value: "switch to dark theme" },
    { value: "ui skin glass" },
  ];
  assert.deepEqual(
    rankSuggestions("sw", items).map((item) => item.value),
    ["switch to dark theme"],
  );
  assert.deepEqual(
    rankSuggestions("glass", items).map((item) => item.value),
    ["ui skin glass"],
  );
  const mixed = [{ value: "foggy glass" }, { value: "glass clear" }];
  assert.equal(rankSuggestions("glass", mixed)[0].value, "glass clear");
  assert.equal(rankSuggestions("zebra", items).length, 0);
  assert.equal(rankSuggestions("e", Array.from({ length: 20 }, (_, i) => ({ value: `e${i}` }))).length, 6);
});

test("ranking works across the UI languages the catalog ships", () => {
  const items = [
    { value: "switch to Russian" },
    { value: "переключить тему" },
    { value: "切换到深色主题" },
  ];
  assert.equal(rankSuggestions("рус", items)[0].value, "switch to Russian");
  assert.equal(rankSuggestions("тему", items)[0].value, "переключить тему");
  assert.equal(rankSuggestions("深色", items)[0].value, "切换到深色主题");
});

function keyEvent(key, { shiftKey = false } = {}) {
  return { key, shiftKey, preventDefault() {} };
}

test("the controller opens on a partial word and completes via keyboard", () => {
  const completed = [];
  const controller = createAutocompleteController({
    getItems: () => [...COMMAND_SUGGESTIONS],
    onComplete: (value) => completed.push(value),
  });

  controller.handleInput("sw");
  const state = controller.snapshot();
  assert.equal(state.open, true);
  assert.ok(state.items.length >= 2);
  assert.equal(state.activeIndex, 0);

  assert.equal(controller.handleKeyDown(keyEvent("ArrowDown")), "handled");
  assert.equal(controller.snapshot().activeIndex, 1);
  assert.equal(controller.handleKeyDown(keyEvent("ArrowUp")), "handled");
  assert.equal(controller.snapshot().activeIndex, 0);

  assert.equal(controller.handleKeyDown(keyEvent("Enter")), "handled");
  assert.deepEqual(completed, ["switch to dark theme"]);
  assert.equal(controller.snapshot().open, false);
});

test("Tab completes, Escape closes and releases the keys, short input stays closed", () => {
  const completed = [];
  const controller = createAutocompleteController({
    getItems: () => [{ value: "ui skin glass" }],
    onComplete: (value) => completed.push(value),
  });

  controller.handleInput("ui");
  assert.equal(controller.snapshot().open, true);
  assert.equal(controller.handleKeyDown(keyEvent("Tab")), "handled");
  assert.deepEqual(completed, ["ui skin glass"]);

  controller.handleInput("ui");
  assert.equal(controller.handleKeyDown(keyEvent("Escape")), "handled");
  assert.equal(controller.snapshot().open, false);
  assert.equal(controller.handleKeyDown(keyEvent("Enter")), "ignored");

  controller.handleInput("u");
  assert.equal(controller.snapshot().open, false);
  controller.handleInput("");
  assert.equal(controller.snapshot().open, false);
});

test("only the word being typed is matched, so prose does not pop the list", () => {
  const controller = createAutocompleteController({
    getItems: (query) => [{ value: `dark ${query}` }, { value: "unrelated" }],
    onComplete: () => {},
  });
  controller.handleInput("please switch to dark theme no");
  const state = controller.snapshot();
  assert.equal(state.open, true);
  assert.deepEqual(
    state.items.map((item) => item.value),
    ["dark no"],
  );
});

test("input history is deduped, newest-first, capped, and failure-tolerant", () => {
  const store = new Map();
  globalThis.window = {
    localStorage: {
      getItem: (key) => (store.has(key) ? store.get(key) : null),
      setItem: (key, value) => store.set(key, value),
    },
  };
  try {
    rememberInputValue("composerPrompts", "hello");
    rememberInputValue("composerPrompts", "switch to dark theme");
    rememberInputValue("composerPrompts", "hello");
    assert.deepEqual(recallInputValues("composerPrompts"), ["hello", "switch to dark theme"]);
    for (let i = 0; i < 12; i += 1) rememberInputValue("composerPrompts", `prompt ${i}`);
    assert.ok(recallInputValues("composerPrompts").length <= 8);
    rememberInputValue("composerPrompts", "   ");
    assert.ok(!recallInputValues("composerPrompts").includes(""));

    globalThis.window = {
      localStorage: {
        getItem: () => {
          throw new Error("quota");
        },
        setItem: () => {
          throw new Error("quota");
        },
      },
    };
    rememberInputValue("composerPrompts", "still fine");
    assert.deepEqual(recallInputValues("composerPrompts"), []);
  } finally {
    delete globalThis.window;
  }
});

test("suggestion adapters keep their source labels", () => {
  assert.deepEqual(examplePromptSuggestions([{ text: "hello", label: "Greeting" }, { text: "" }, null]), [
    { value: "hello", source: "example", hint: "Greeting" },
  ]);
  assert.deepEqual(historySuggestions(["a", "", "b", null]), [
    { value: "a", source: "history" },
    { value: "b", source: "history" },
  ]);
});

test("every command suggestion is unique and non-empty", () => {
  const values = COMMAND_SUGGESTIONS.map((item) => item.value);
  assert.equal(new Set(values).size, values.length);
  for (const value of values) assert.ok(value.trim().length >= 2);
});

test("the catalog carries the autocomplete strings in every locale", () => {
  const catalog = readFileSync(path.join(REPO_ROOT, "js/i18n-catalog.lino"), "utf8");
  for (const locale of ["en", "ru", "zh", "hi"]) {
    const block = catalog.split(new RegExp(`^${locale}$`, "m"))[1]?.split(/^[a-z]{2}$/m)[0] ?? "";
    for (const key of ["autocomplete", "listLabel"]) {
      assert.ok(block.includes(key), `${locale} has ${key}`);
    }
    for (const source of ["command", "history", "example"]) {
      assert.ok(block.includes(source), `${locale} labels the ${source} source`);
    }
  }
});
