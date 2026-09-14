// Unit coverage for the browser production scripts under `src/web/` (issue #895).
//
// These are the exact files a browser downloads as `<script>` tags — not build
// output — so the coverage they produce is an honest denominator for the site's
// client-side code. `loadBrowserScript` runs each one through `node:vm` with the
// real repository path as the script filename, which is what makes
// `node --test --experimental-test-coverage` attribute the lines to
// `src/web/<name>.js`.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  createBrowserContext,
  createStorage,
  loadBrowserScript,
  plain,
} from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../..");

function load(relativePath, overrides = {}) {
  const context = createBrowserContext(overrides);
  loadBrowserScript(context, relativePath);
  return context;
}

test("preferences round-trip through the Links Notation storage format", () => {
  const storage = createStorage();
  const context = load("src/web/preferences.js", { localStorage: storage });
  const preferences = context.FormalAiPreferences;

  assert.equal(preferences.STORAGE_KEY, "formal-ai.preferences.v1");
  assert.deepEqual(plain(preferences.load()), {}, "empty storage yields no preferences");

  const record = { language: "ru", theme: "dark", showTrace: true, hideHints: false };
  const text = preferences.format(record);
  assert.match(text, /^demo_preferences\n/, "the document is rooted at demo_preferences");
  assert.match(text, /^ {2}language "ru"$/m);
  assert.match(text, /^ {2}showTrace "on"$/m, "booleans serialize as on/off");
  assert.match(text, /^ {2}hideHints "off"$/m);

  assert.deepEqual(
    plain(preferences.parse(text)),
    record,
    "parsing the formatted document restores the exact record",
  );

  preferences.save(record);
  assert.equal(
    storage.getItem(preferences.STORAGE_KEY),
    text,
    "save() persists the same Links Notation document under the versioned key",
  );
  assert.deepEqual(plain(preferences.load()), record, "load() reads back what save() wrote");
});

test("preferences reject documents that are not rooted at demo_preferences", () => {
  const context = load("src/web/preferences.js");
  const preferences = context.FormalAiPreferences;

  assert.equal(preferences.parse(""), null);
  assert.equal(preferences.parse("other_root\n  language \"ru\""), null);
  assert.deepEqual(
    plain(preferences.parse('demo_preferences\n  language "ru"\nnot an entry\n')),
    { language: "ru" },
    "unparsable lines are skipped rather than corrupting the record",
  );
});

test("preferences survive a storage backend that throws", () => {
  const hostile = {
    getItem() {
      throw new Error("storage disabled");
    },
    setItem() {
      throw new Error("storage disabled");
    },
    removeItem() {},
  };
  const context = load("src/web/preferences.js", { localStorage: hostile });
  const preferences = context.FormalAiPreferences;

  assert.deepEqual(plain(preferences.load()), {}, "a throwing read degrades to defaults");
  assert.doesNotThrow(() => preferences.save({ language: "en" }));
});

test("i18n resolves every supported language and falls back to the default", () => {
  const context = load("src/web/i18n.js");
  const i18n = context.FormalAiI18n;

  assert.equal(i18n.DEFAULT_LANGUAGE, "en");
  assert.deepEqual(plain(i18n.SUPPORTED_LANGUAGES), ["en", "ru", "zh", "hi"]);

  for (const language of i18n.SUPPORTED_LANGUAGES) {
    assert.equal(
      i18n.resolveLanguage(language),
      language,
      `${language} resolves to itself`,
    );
  }

  assert.equal(i18n.normalizeLanguageTag("ru-RU"), "ru", "regional tags collapse to the base");
  assert.equal(i18n.normalizeLanguageTag("zh-Hans"), "zh");
  assert.equal(i18n.normalizeLanguageTag("EN-us"), "en", "tags are case-insensitive");
  assert.equal(
    i18n.resolveLanguage("pt-BR"),
    i18n.DEFAULT_LANGUAGE,
    "an unsupported language falls back to the default rather than throwing",
  );
  assert.equal(i18n.resolveLanguage(undefined), i18n.DEFAULT_LANGUAGE);
});

test("i18n detects the language from the browser's preference list", () => {
  const context = load("src/web/i18n.js", {
    navigator: { language: "ru-RU", languages: ["ru-RU", "en-US"] },
  });
  const i18n = context.FormalAiI18n;

  assert.deepEqual(plain(i18n.browserLanguages()), ["ru-RU", "en-US"]);
  assert.equal(i18n.detectLanguage(), "ru");

  const unsupported = load("src/web/i18n.js", {
    navigator: { language: "pt-BR", languages: ["pt-BR"] },
  });
  assert.equal(
    unsupported.FormalAiI18n.detectLanguage(),
    "en",
    "an unsupported browser locale detects as the default language",
  );
});

test("syntax highlighting escapes markup and tags known languages", () => {
  const context = load("src/web/syntax-highlight.js");
  const highlight = context.FormalAiHighlight;

  assert.equal(
    highlight.escapeHtml('<script>&"'),
    "&lt;script&gt;&amp;&quot;",
    "every HTML-significant character is escaped",
  );

  const languages = highlight.listLanguages();
  assert.ok(languages.includes("rust"), "the project's own language is highlightable");
  assert.ok(languages.includes("javascript"));

  const rust = highlight.highlight("let x = 1;", "rust");
  assert.equal(rust.language, "rust");
  assert.match(rust.value, /hljs-keyword">let</, "keywords are wrapped in highlight spans");
  assert.match(rust.value, /hljs-number">1</);

  const unknown = highlight.highlight("<b>plain</b>", "not-a-language");
  assert.ok(
    !unknown.value.includes("<b>"),
    "an unknown language still escapes the source instead of injecting markup",
  );
});

test("memory events round-trip through the exported Links Notation bundle", () => {
  const context = load("src/web/memory.js");
  const memory = context.FormalAiMemory;

  assert.equal(memory.ROOT, "demo_memory");
  assert.equal(memory.BUNDLE_ROOT, "formal_ai_bundle");

  const events = [
    { id: "e1", conversationId: "c1", content: "hello" },
    { id: "e2", conversationId: "c1", content: 'quote " and \\ backslash' },
  ];
  const document = memory.exportLinksNotation(events);
  assert.match(document, /^demo_memory\n/);

  const parsed = memory.parseLinksNotation(document);
  assert.equal(parsed.length, 2);
  assert.equal(parsed[0].id, "e1");
  assert.equal(
    parsed[1].content,
    'quote " and \\ backslash',
    "escaped characters survive the round-trip",
  );
});

test("memory events reduce to stable doublet records", () => {
  const context = load("src/web/memory.js");
  const memory = context.FormalAiMemory;

  const event = { id: "e1", type: "user_message", conversationId: "c1", content: "hello" };
  const first = memory.reduceEventToDoublets(event);
  const second = memory.reduceEventToDoublets({ ...event });

  assert.equal(first.recordType, "MemoryEvent");
  assert.equal(first.schemaVersion, memory.LINK_STORE_SCHEMA_VERSION);
  assert.equal(first.sourceId, "e1");
  assert.deepEqual(
    plain(first),
    plain(second),
    "the same event always reduces to the same identifiers, so the store is idempotent",
  );
  assert.ok(first.links.length > 0, "the reduction emits doublets");

  const different = memory.reduceEventToDoublets({ ...event, id: "e2" });
  assert.notEqual(
    different.stableId,
    first.stableId,
    "distinct events get distinct stable identifiers",
  );
});

test("the seed loader parses the shipped seed files", () => {
  // Pages and the worker load `seed-files.js` before `seed_loader.js`: the
  // inventory is generated from `data/meta/seed-registry.lino` so that adding a
  // seed file never edits a list two branches share (issue #991).
  const context = createBrowserContext();
  loadBrowserScript(context, "src/web/seed-files.js");
  loadBrowserScript(context, "src/web/seed_loader.js");
  const seed = context.FormalAiSeed;

  assert.ok(
    seed.DEFAULT_FILES.includes("seed/agent-info.lino"),
    "agent info is one of the files the site loads at startup",
  );

  const text = readFileSync(path.join(REPO_ROOT, "data/seed/agent-info.lino"), "utf8");
  const tree = seed.parse(text);
  assert.ok(tree, "a shipped seed file parses");
  assert.ok(tree.children.length > 0, "the parsed tree has entries");

  const info = seed.extractAgentInfo([tree]);
  assert.ok(info && typeof info === "object", "agent info is extracted from the parsed tree");
});

test("the seed loader has no inventory of its own", () => {
  // The failure mode worth pinning: a host that forgets `seed-files.js` gets an
  // empty list and a loader that says so, rather than a second copy of the
  // inventory quietly drifting from `data/meta/seed-registry.lino`.
  const seed = load("src/web/seed_loader.js").FormalAiSeed;

  assert.deepEqual(plain(seed.DEFAULT_FILES), []);
});

test("the seed loader parses indentation into a nested tree", () => {
  const context = load("src/web/seed_loader.js");
  const seed = context.FormalAiSeed;

  const tree = seed.parse('root\n  child "value"\n    grandchild "deep"\n');
  assert.equal(tree.name, "root");
  assert.equal(tree.children.length, 1);
  assert.equal(tree.children[0].name, "child");
  assert.equal(tree.children[0].value, "value");
  assert.equal(tree.children[0].children[0].value, "deep");
});

test("site chrome resolves locale and theme preferences", () => {
  // Pages load `preferences.js` before `site-chrome.js`; the chrome delegates
  // its storage to that namespace, so the pair is what has to be tested.
  const storage = createStorage();
  const context = createBrowserContext({ localStorage: storage });
  loadBrowserScript(context, "src/web/preferences.js");
  loadBrowserScript(context, "src/web/site-chrome.js");
  const chrome = context.FormalAiSiteChrome;

  assert.deepEqual(plain(chrome.SUPPORTED_LOCALES), ["en", "ru", "zh", "hi"]);
  assert.equal(chrome.normalizeLocale("ru-RU"), "ru");
  assert.equal(
    chrome.normalizeLocale("pt-BR"),
    undefined,
    "an unsupported locale normalizes to nothing so the caller falls back",
  );
  assert.equal(
    chrome.resolveLocale("pt-BR"),
    "en",
    "resolution supplies the default when normalization finds no match",
  );
  assert.equal(chrome.resolveTheme("dark"), "dark");
  assert.equal(chrome.resolveTheme("nonsense"), chrome.resolveTheme(null));

  chrome.writePreference("language", "zh");
  assert.equal(
    chrome.readPreferences().language,
    "zh",
    "a written preference is readable back through the same storage",
  );
});

test("site chrome builds elements through its own `h` helper", () => {
  const context = load("src/web/site-chrome.js");
  const chrome = context.FormalAiSiteChrome;

  const child = chrome.h("span", { className: "inner" }, "text");
  const parent = chrome.h("div", { className: "outer" }, child);
  assert.ok(parent, "the helper returns an element");
  assert.ok(parent.children.includes(child), "children are appended to the parent");
});

test("site chrome reads the stamped version and ignores an un-stamped build", () => {
  function chromeWithVersion(content) {
    const context = createBrowserContext();
    context.document.querySelector = (selector) =>
      selector === 'meta[name="formal-ai-version"]'
        ? { getAttribute: () => content }
        : null;
    loadBrowserScript(context, "src/web/site-chrome.js");
    return context.FormalAiSiteChrome.readVersion();
  }

  assert.equal(chromeWithVersion("v1.2.3"), "1.2.3", "the leading v is stripped");
  assert.equal(chromeWithVersion("1.2.3"), "1.2.3");
  assert.equal(
    chromeWithVersion("__FORMAL_AI_VERSION__"),
    "",
    "an un-stamped local build shows no version rather than the placeholder",
  );
  assert.equal(chromeWithVersion(null), "");
});

test("the download page resolves release assets, checksums and verification commands", () => {
  const context = createBrowserContext({
    navigator: {
      language: "en",
      languages: ["en"],
      userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
    },
  });
  loadBrowserScript(context, "src/web/preferences.js");
  loadBrowserScript(context, "src/web/download/download.js");
  const download = context.FormalAiDownload;

  assert.equal(
    download.detectOperatingSystem(),
    "macos",
    "the operating system is detected from the user agent",
  );

  const primary = download.primaryOptionFor("macos");
  assert.equal(primary.os, "macos");

  const release = {
    tag_name: "v1.2.3",
    assets: [
      {
        name: `${primary.assetPrefix}-1.2.3.${primary.extension}`,
        browser_download_url: "https://example.invalid/desktop",
      },
      { name: "SHA256SUMS.txt", browser_download_url: "https://example.invalid/sums" },
    ],
  };
  assert.equal(download.releaseVersion(release), "1.2.3", "the tag's v prefix is stripped");

  const assets = download.assetsByName(release);
  assert.deepEqual(
    plain(download.candidateAssetNames(primary, release)),
    [`${primary.assetPrefix}-1.2.3.${primary.extension}`, `${primary.assetPrefix}.${primary.extension}`],
    "the versioned name is tried before the legacy unversioned one",
  );
  assert.equal(
    download.resolveDownloadHref(primary, assets, release),
    "https://example.invalid/desktop",
    "the versioned asset is the one offered for download",
  );
  assert.equal(
    download.resolveChecksumHref(assets),
    "https://example.invalid/sums",
    "the checksum file is published alongside the download",
  );
  assert.equal(
    download.resolveDownloadHref(primary, {}, release),
    undefined,
    "a release missing the asset offers no href instead of a broken link",
  );

  const digest = "a".repeat(64);
  const sums = `${digest}  ${primary.assetPrefix}-1.2.3.${primary.extension}\n${"b".repeat(64)}  other.zip\n`;
  assert.equal(
    download.checksumForFile(sums, `${primary.assetPrefix}-1.2.3.${primary.extension}`),
    digest,
    "the digest for the requested file is picked out of SHA256SUMS.txt",
  );
  assert.equal(
    download.checksumForFile(sums, "missing.dmg"),
    undefined,
    "an unlisted file has no checksum rather than a wrong one",
  );

  const commands = download.verificationCommands(release);
  assert.deepEqual(
    plain(commands).map((entry) => entry.key),
    ["windowsCommand", "macosCommand", "linuxCommand"],
    "every supported platform gets a verification command",
  );
  for (const entry of commands) {
    assert.ok(
      entry.command.includes("1.2.3") || entry.command.includes("SHA256SUMS.txt"),
      `${entry.key} references the concrete release`,
    );
  }
});

// Every page config is a production script the corresponding page loads. Loading
// it against a real `site-chrome.js` proves the config it declares is accepted
// by the chooser the site actually ships.
for (const page of [
  "src/web/cli/cli.js",
  "src/web/docs/docs.js",
  "src/web/telegram/telegram.js",
  "src/web/vscode/vscode.js",
  "src/web/landing.js",
]) {
  test(`${page} registers its page against the shipped site chrome`, () => {
    const context = createBrowserContext();
    loadBrowserScript(context, "src/web/site-chrome.js");
    const created = [];
    const chooser = context.FormalAiSiteChrome.createChooser;
    context.FormalAiSiteChrome.createChooser = (config) => {
      created.push(config);
      return chooser(config);
    };

    loadBrowserScript(context, page);

    assert.equal(created.length, 1, "the page config calls the chooser exactly once");
    const config = created[0];
    assert.ok(config.rootId, "the page declares the element it mounts into");
    assert.ok(config.repoUrl, "the page links back to the repository");
    if (config.sections) {
      assert.ok(config.sections.length > 0, "the page declares at least one section");
      for (const section of config.sections) {
        assert.ok(section.id, "every section has an id");
      }
    }
  });
}

test("page configs skip cleanly when the site chrome is missing", () => {
  // A page whose chrome script failed to load must not throw; the guard at the
  // top of each config is what keeps a broken CDN from blanking the page.
  for (const page of ["src/web/cli/cli.js", "src/web/landing.js"]) {
    const context = createBrowserContext();
    assert.doesNotThrow(() => loadBrowserScript(context, page), `${page} degrades quietly`);
  }
});
