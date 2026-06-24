// formal-ai shared site chrome (issue #479).
//
// The landing page (/) and the documentation hub (/docs/) are structurally the
// same page: a top bar (brand + language/theme switchers), a hero, a grid of
// "chooser" cards, and a footer. Rather than duplicate that machinery, both
// pages provide a small config object to `createChooser()` here.
//
// This module reuses the exact `data-theme` contract and the same
// Links-Notation-backed `formal-ai.preferences.v1` storage (via preferences.js)
// that download.js and the chat app use, so a visitor's theme/locale choice is
// shared across /, /app/, /docs/ and /download/. It makes no network calls.
//
// The proven download.js helpers (h/segmentedControl/locale+theme resolution)
// are intentionally re-implemented here verbatim rather than imported, so the
// heavily-tested download page stays an independent island while the two new
// pages share one source of truth.

(function (global) {
  "use strict";

  var SUPPORTED_LOCALES = ["en", "ru", "zh", "hi"];

  // Chrome labels shared by every page (switchers + footer). Page-specific copy
  // (heading/eyebrow/summary/card text) overrides these via `config.copy`.
  var LABELS = {
    en: {
      language: "Language",
      theme: "Theme",
      themeAuto: "Auto",
      themeLight: "Light",
      themeDark: "Dark",
      footerVersion: "Version",
      footerSource: "Source on GitHub",
      sourceEyebrow: "Open source",
      copyLabel: "Copy",
      copiedLabel: "Copied",
    },
    ru: {
      language: "Язык",
      theme: "Тема",
      themeAuto: "Авто",
      themeLight: "Светлая",
      themeDark: "Тёмная",
      footerVersion: "Версия",
      footerSource: "Исходный код на GitHub",
      sourceEyebrow: "Открытый код",
      copyLabel: "Копировать",
      copiedLabel: "Скопировано",
    },
    zh: {
      language: "语言",
      theme: "主题",
      themeAuto: "自动",
      themeLight: "浅色",
      themeDark: "深色",
      footerVersion: "版本",
      footerSource: "GitHub 源代码",
      sourceEyebrow: "开源",
      copyLabel: "复制",
      copiedLabel: "已复制",
    },
    hi: {
      language: "भाषा",
      theme: "थीम",
      themeAuto: "ऑटो",
      themeLight: "लाइट",
      themeDark: "डार्क",
      footerVersion: "संस्करण",
      footerSource: "GitHub पर सोर्स कोड",
      sourceEyebrow: "ओपन सोर्स",
      copyLabel: "कॉपी करें",
      copiedLabel: "कॉपी हो गया",
    },
  };

  // ---------------------------------------------------------------------------
  // Locale detection (identical contract to download.js)
  // ---------------------------------------------------------------------------

  function normalizeLocale(tag) {
    var lower = String(tag || "").toLowerCase();
    for (var i = 0; i < SUPPORTED_LOCALES.length; i += 1) {
      if (lower === SUPPORTED_LOCALES[i] || lower.indexOf(SUPPORTED_LOCALES[i] + "-") === 0) {
        return SUPPORTED_LOCALES[i];
      }
    }
    if (lower.indexOf("zh") === 0) return "zh";
    return undefined;
  }

  function detectLocaleFromBrowser() {
    var nav = typeof navigator !== "undefined" ? navigator : {};
    var languages = nav.languages || (nav.language ? [nav.language] : ["en"]);
    for (var i = 0; i < languages.length; i += 1) {
      var match = normalizeLocale(languages[i]);
      if (match) return match;
    }
    return "en";
  }

  function resolveLocale(localePreference) {
    var normalized = normalizeLocale(localePreference);
    if (normalized) return normalized;
    return detectLocaleFromBrowser();
  }

  function resolveTheme(themePreference) {
    if (themePreference === "dark") return "dark";
    if (themePreference === "light") return "light";
    if (
      typeof global.matchMedia === "function" &&
      global.matchMedia("(prefers-color-scheme: dark)").matches
    ) {
      return "dark";
    }
    return "light";
  }

  // ---------------------------------------------------------------------------
  // Preference round-trip (shared with the chat app via preferences.js)
  // ---------------------------------------------------------------------------

  function readPreferences() {
    if (global.FormalAiPreferences && typeof global.FormalAiPreferences.load === "function") {
      return global.FormalAiPreferences.load({});
    }
    return {};
  }

  function writePreference(key, value) {
    if (!global.FormalAiPreferences || typeof global.FormalAiPreferences.save !== "function") {
      return;
    }
    var current = global.FormalAiPreferences.load({});
    current[key] = value;
    global.FormalAiPreferences.save(current);
  }

  function readVersion() {
    if (typeof document === "undefined") return "";
    var meta = document.querySelector('meta[name="formal-ai-version"]');
    var content = meta && meta.getAttribute("content");
    // The Pages stamp pipeline replaces __FORMAL_AI_VERSION__ with a real
    // semver; if the placeholder survives (un-stamped local build) we omit the
    // label rather than print the placeholder.
    if (!content || content.indexOf("__") === 0 || !/^v?\d/.test(content)) return "";
    return content.charAt(0).toLowerCase() === "v" ? content.slice(1) : content;
  }

  // ---------------------------------------------------------------------------
  // Tiny hyperscript helper (identical to download.js)
  // ---------------------------------------------------------------------------

  function h(tag, props) {
    var el = document.createElement(tag);
    if (props) {
      Object.keys(props).forEach(function (key) {
        var value = props[key];
        if (value == null || value === false) return;
        if (key === "class") {
          el.className = value;
        } else if (key === "text") {
          el.textContent = value;
        } else if (key.indexOf("on") === 0 && typeof value === "function") {
          el.addEventListener(key.slice(2).toLowerCase(), value);
        } else if (value === true) {
          el.setAttribute(key, "");
        } else {
          el.setAttribute(key, String(value));
        }
      });
    }
    for (var i = 2; i < arguments.length; i += 1) {
      appendChild(el, arguments[i]);
    }
    return el;
  }

  function appendChild(parent, child) {
    if (child == null || child === false) return;
    if (Array.isArray(child)) {
      child.forEach(function (item) {
        appendChild(parent, item);
      });
      return;
    }
    if (typeof child === "string" || typeof child === "number") {
      parent.appendChild(document.createTextNode(String(child)));
      return;
    }
    parent.appendChild(child);
  }

  function segmentedControl(options, activeValue, onSelect, ariaLabel, className) {
    return h(
      "div",
      { class: className, role: "group", "aria-label": ariaLabel },
      options.map(function (option) {
        return h("button", {
          type: "button",
          class: activeValue === option.value ? "active" : "",
          "aria-pressed": activeValue === option.value ? "true" : "false",
          "data-value": option.value,
          text: option.label,
          onClick: function () {
            onSelect(option.value);
          },
        });
      }),
    );
  }

  // ---------------------------------------------------------------------------
  // Chooser page factory (landing + docs)
  // ---------------------------------------------------------------------------
  //
  // config = {
  //   rootId:       string  — id of the <main> to render into
  //   topbarClass:  string  — class for the <header> top bar
  //   brandHref:    string  — where the brand link points (home)
  //   repoUrl:      string  — hero "Source on GitHub" big-button target
  //   exposeAs:     string  — window global to publish the api on (for e2e)
  //   destinations: [{ id, href, external?, icon, titleKey, descKey, actionKey }]
  //   sections:     optional [{ id?, titleKey?, introKey?, steps?:[key],
  //                   commands?:[{ command, labelKey?, noteKey?, testid? }],
  //                   links?:[{ href, labelKey, external?, testid? }], noteKey? }]
  //                 — install pages (#554) use these for copy-paste commands.
  //   copy:         { <locale>: { heading, eyebrow, summary, ...cardKeys } }
  // }
  //
  // Both `destinations` and `sections` are optional; a page may render only a
  // card grid (landing/docs), only sections, or both.

  function createChooser(config) {
    var state = { locale: "en", themePreference: "auto" };

    function text(locale, key) {
      var c = config.copy || {};
      return (
        (c[locale] && c[locale][key]) ||
        (c.en && c.en[key]) ||
        (LABELS[locale] && LABELS[locale][key]) ||
        LABELS.en[key] ||
        key
      );
    }

    function applyDocumentChrome() {
      if (typeof document === "undefined") return;
      document.documentElement.setAttribute("data-theme", resolveTheme(state.themePreference));
      document.documentElement.lang = state.locale;
    }

    function navCard(locale, destination) {
      var props = {
        class: "nav-card",
        href: destination.href,
        "data-testid": "nav-" + destination.id,
      };
      // External links open in a new tab with safe rel; in-site routes stay in
      // the same tab so theme/locale (in localStorage) carry over seamlessly.
      if (destination.external) {
        props.target = "_blank";
        props.rel = "noopener noreferrer";
      }
      return h(
        "a",
        props,
        h("span", { class: "nav-card-icon", "aria-hidden": "true", text: destination.icon }),
        h("span", { class: "nav-card-title", text: text(locale, destination.titleKey) }),
        h("span", { class: "nav-card-desc", text: text(locale, destination.descKey) }),
        h("span", { class: "nav-card-action", text: text(locale, destination.actionKey) }),
      );
    }

    // -------------------------------------------------------------------------
    // Optional info sections (issue #554): the install pages for the VS Code
    // extension, the CLI and the Telegram bot need more than a card grid — they
    // carry copy-paste install commands, ordered step lists and direct links.
    // These render only when `config.sections` is provided, so the landing and
    // docs pages (no sections) are byte-for-byte unchanged.
    // -------------------------------------------------------------------------

    // Copy `value` to the clipboard, flipping the button label to "Copied" for a
    // moment. Uses the async Clipboard API where available and falls back to a
    // hidden textarea + execCommand so it still works under the strict CSP and
    // on older engines.
    function copyToClipboard(value, button, locale) {
      var original = text(locale, "copyLabel");
      var flip = function () {
        button.textContent = text(locale, "copiedLabel");
        if (typeof global.setTimeout === "function") {
          global.setTimeout(function () {
            button.textContent = original;
          }, 1500);
        }
      };
      var fallback = function () {
        try {
          var ta = document.createElement("textarea");
          ta.value = value;
          ta.setAttribute("readonly", "");
          ta.style.position = "absolute";
          ta.style.left = "-9999px";
          document.body.appendChild(ta);
          ta.select();
          document.execCommand("copy");
          document.body.removeChild(ta);
        } catch (error) {
          /* clipboard unavailable; leave the visible command for manual copy */
        }
      };
      if (
        global.navigator &&
        global.navigator.clipboard &&
        typeof global.navigator.clipboard.writeText === "function"
      ) {
        global.navigator.clipboard.writeText(value).then(flip, function () {
          fallback();
          flip();
        });
      } else {
        fallback();
        flip();
      }
    }

    // command = { command, labelKey?, noteKey?, testid? }
    function commandBlock(locale, command) {
      var button = h("button", {
        type: "button",
        class: "copy-button",
        "data-testid": command.testid ? "copy-" + command.testid : null,
        "aria-label": text(locale, "copyLabel"),
        text: text(locale, "copyLabel"),
      });
      button.addEventListener("click", function () {
        copyToClipboard(command.command, button, locale);
      });
      return h(
        "div",
        { class: "command-block" },
        command.labelKey
          ? h("span", { class: "command-label", text: text(locale, command.labelKey) })
          : null,
        h(
          "div",
          { class: "command-row" },
          h(
            "pre",
            { class: "command-pre" },
            h("code", {
              "data-testid": command.testid ? "command-" + command.testid : null,
              text: command.command,
            }),
          ),
          button,
        ),
        command.noteKey
          ? h("p", { class: "command-note", text: text(locale, command.noteKey) })
          : null,
      );
    }

    // section = { id?, titleKey?, introKey?, steps?:[key], commands?:[command],
    //             links?:[{href,labelKey,external?,testid?}], noteKey? }
    function sectionBlock(locale, section) {
      return h(
        "section",
        {
          class: "info-section",
          "data-testid": section.id ? "section-" + section.id : null,
        },
        section.titleKey
          ? h("h2", { class: "info-title", text: text(locale, section.titleKey) })
          : null,
        section.introKey
          ? h("p", { class: "info-intro", text: text(locale, section.introKey) })
          : null,
        section.steps
          ? h(
              "ol",
              { class: "info-steps" },
              section.steps.map(function (key) {
                return h("li", { text: text(locale, key) });
              }),
            )
          : null,
        section.commands
          ? section.commands.map(function (command) {
              return commandBlock(locale, command);
            })
          : null,
        section.links
          ? h(
              "div",
              { class: "info-links" },
              section.links.map(function (link) {
                var props = {
                  class: "info-link",
                  href: link.href,
                  "data-testid": link.testid || null,
                };
                if (link.external) {
                  props.target = "_blank";
                  props.rel = "noopener noreferrer";
                }
                return h("a", props, text(locale, link.labelKey));
              }),
            )
          : null,
        section.noteKey
          ? h("p", { class: "info-note", text: text(locale, section.noteKey) })
          : null,
      );
    }

    function render() {
      var root = document.getElementById(config.rootId);
      if (!root) return;
      var locale = state.locale;
      root.textContent = "";

      var topbar = h(
        "header",
        { class: config.topbarClass || "landing-topbar" },
        h(
          "a",
          { class: "brand", href: config.brandHref || "./", "data-testid": "brand-home" },
          h("span", { class: "brand-mark", "aria-hidden": "true", text: "◆" }),
          h("span", { text: "formal-ai" }),
        ),
        h(
          "div",
          { class: "topbar-controls" },
          segmentedControl(
            SUPPORTED_LOCALES.map(function (value) {
              return { value: value, label: value.toUpperCase() };
            }),
            locale,
            function (value) {
              state.locale = value;
              writePreference("uiLanguage", value);
              applyDocumentChrome();
              render();
            },
            text(locale, "language"),
            "locale-switch",
          ),
          segmentedControl(
            [
              { value: "auto", label: text(locale, "themeAuto") },
              { value: "light", label: text(locale, "themeLight") },
              { value: "dark", label: text(locale, "themeDark") },
            ],
            state.themePreference,
            function (value) {
              state.themePreference = value;
              writePreference("theme", value);
              applyDocumentChrome();
              render();
            },
            text(locale, "theme"),
            "theme-switch",
          ),
        ),
      );

      // The source-code entry point is a big, prominent call-to-action button in
      // the hero (issue #479: the maintainer asked that "the source code on the
      // landing is a big button", not a small footer link). It reuses the exact
      // <span>(action) + <strong>(label) shape of the /download page's
      // .primary-download button, mirroring how vk-bot-desktop surfaces its
      // primary action, and opens the repository in a new tab.
      var sourceCta = config.repoUrl
        ? h(
            "a",
            {
              class: "source-cta",
              href: config.repoUrl,
              target: "_blank",
              rel: "noopener noreferrer",
              "data-testid": "source-cta",
            },
            h("span", { class: "source-cta-icon", "aria-hidden": "true", text: "</>" }),
            h(
              "span",
              { class: "source-cta-text" },
              h("span", { class: "source-cta-eyebrow", text: text(locale, "sourceEyebrow") }),
              h("strong", { class: "source-cta-label", text: text(locale, "footerSource") }),
            ),
          )
        : null;

      var hero = h(
        "section",
        { class: "hero" },
        h("p", { class: "eyebrow", text: text(locale, "eyebrow") }),
        h("h1", { text: text(locale, "heading") }),
        h("p", {
          class: "summary",
          "data-testid": config.rootId + "-summary",
          text: text(locale, "summary"),
        }),
        sourceCta,
      );

      // Optional detailed install sections, rendered between the hero and the
      // card grid (issue #554). Absent on the landing/docs chooser pages.
      var sections =
        config.sections && config.sections.length
          ? h(
              "div",
              { class: "info-sections", "data-testid": "info-sections" },
              config.sections.map(function (section) {
                return sectionBlock(locale, section);
              }),
            )
          : null;

      // The card grid is optional too: an install page may carry only sections,
      // or sections plus a couple of cross-links back into the site.
      var cards =
        config.destinations && config.destinations.length
          ? h(
              "section",
              { class: "nav-cards", "data-testid": "nav-cards" },
              config.destinations.map(function (destination) {
                return navCard(locale, destination);
              }),
            )
          : null;

      // The footer no longer carries a small "Source on GitHub" text link — the
      // source code is surfaced as the big .source-cta button in the hero above
      // (issue #479). The footer keeps only the stamped version label.
      var version = readVersion();
      var footer = h(
        "footer",
        { class: "landing-footer" },
        version
          ? h("span", {
              class: "landing-version",
              "data-testid": "landing-version",
              text: text(locale, "footerVersion") + " " + version,
            })
          : null,
      );

      root.appendChild(topbar);
      root.appendChild(hero);
      if (sections) root.appendChild(sections);
      if (cards) root.appendChild(cards);
      root.appendChild(footer);
    }

    function init() {
      var prefs = readPreferences();
      state.themePreference =
        prefs.theme === "dark" || prefs.theme === "light" ? prefs.theme : "auto";
      state.locale = resolveLocale(prefs.uiLanguage);

      applyDocumentChrome();
      render();

      // Follow OS theme changes while in "auto".
      if (typeof global.matchMedia === "function") {
        var media = global.matchMedia("(prefers-color-scheme: dark)");
        var onChange = function () {
          if (state.themePreference === "auto") {
            applyDocumentChrome();
            render();
          }
        };
        if (typeof media.addEventListener === "function") {
          media.addEventListener("change", onChange);
        } else if (typeof media.addListener === "function") {
          media.addListener(onChange);
        }
      }
    }

    var api = {
      config: config,
      state: state,
      text: text,
      render: render,
      init: init,
      applyDocumentChrome: applyDocumentChrome,
      resolveLocale: resolveLocale,
      resolveTheme: resolveTheme,
      SUPPORTED_LOCALES: SUPPORTED_LOCALES,
    };

    if (config.exposeAs) {
      global[config.exposeAs] = api;
    }

    if (typeof document !== "undefined") {
      if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
      } else {
        init();
      }
    }

    return api;
  }

  global.FormalAiSiteChrome = {
    SUPPORTED_LOCALES: SUPPORTED_LOCALES,
    LABELS: LABELS,
    normalizeLocale: normalizeLocale,
    detectLocaleFromBrowser: detectLocaleFromBrowser,
    resolveLocale: resolveLocale,
    resolveTheme: resolveTheme,
    readPreferences: readPreferences,
    writePreference: writePreference,
    readVersion: readVersion,
    h: h,
    appendChild: appendChild,
    segmentedControl: segmentedControl,
    createChooser: createChooser,
  };
})(typeof window !== "undefined" ? window : globalThis);
