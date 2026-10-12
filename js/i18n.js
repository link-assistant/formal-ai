(function (global) {
  "use strict";

  var DEFAULT_LANGUAGE = "en";
  var SUPPORTED_LANGUAGES = ["en", "ru", "zh", "hi"];
  var PUBLISHED_RUNTIME_SOURCE = "lino-i18n@0.3.0";
  var LOADING_RUNTIME_SOURCE = "lino-i18n-loading";
  var UNAVAILABLE_RUNTIME_SOURCE = "lino-i18n-unavailable";
  var CATALOG_URL = "i18n-catalog.lino";
  // Permission/Services and per-message strings live in companion files so each
  // catalog stays under the Links Notation line limit (see
  // scripts/check-file-size.rs). lino-i18n's `loadCatalogs` fetches them
  // concurrently and merges their per-locale keys in this order.
  var CATALOG_URLS = [
    CATALOG_URL,
    "i18n-catalog-permissions.lino",
    "i18n-catalog-messages.lino",
  ];
  var runtimeEngine = null;
  var CATALOG = {};

  function normalizeLanguageTag(value) {
    var raw = String(value || "").toLowerCase().trim();
    if (!raw || raw === "auto") return "";
    var base = raw.split(/[-_]/)[0];
    return SUPPORTED_LANGUAGES.indexOf(base) >= 0 ? base : "";
  }

  function resolveLanguage(preference, candidates) {
    var explicit = normalizeLanguageTag(preference);
    if (explicit) return explicit;
    var list = Array.isArray(candidates) ? candidates : [candidates];
    for (var index = 0; index < list.length; index += 1) {
      var normalized = normalizeLanguageTag(list[index]);
      if (normalized) return normalized;
    }
    return DEFAULT_LANGUAGE;
  }

  function browserLanguages() {
    var nav = global.navigator || {};
    if (Array.isArray(nav.languages) && nav.languages.length > 0) {
      return nav.languages.slice();
    }
    return nav.language ? [nav.language] : [];
  }

  function detectLanguage(preference) {
    return resolveLanguage(preference, browserLanguages());
  }

  function cacheBustedUrl(path) {
    var version = String(global.FORMAL_AI_ASSET_VERSION || "").trim();
    if (!version || /^__.*__$/.test(version)) return path;
    return (
      path +
      (path.indexOf("?") >= 0 ? "&" : "?") +
      "v=" +
      encodeURIComponent(version)
    );
  }

  function t(key, language, params) {
    var lang = normalizeLanguageTag(language) || DEFAULT_LANGUAGE;
    if (!runtimeEngine || typeof runtimeEngine.t !== "function") {
      return String(key);
    }
    try {
      return runtimeEngine.t(String(key), params || {}, {
        locale: lang,
        defaultValue: String(key),
      });
    } catch (_error) {
      return String(key);
    }
  }

  function dispatchReady() {
    if (typeof global.dispatchEvent !== "function") return;
    try {
      if (typeof global.CustomEvent === "function") {
        global.dispatchEvent(
          new global.CustomEvent("formal-ai:i18n-ready", {
            detail: { source: api.ENGINE_SOURCE, error: api.lastError },
          }),
        );
      } else {
        global.dispatchEvent({ type: "formal-ai:i18n-ready" });
      }
    } catch (_error) {
      // Rendering already falls back to stable keys when event dispatch is unavailable.
    }
  }

  function bundledRuntimeModule() {
    var vendor = global.FormalAiVendor || {};
    var module = vendor.LinoI18n || global.LinoI18n || null;
    if (!module) {
      return Promise.reject(new Error("bundled lino-i18n runtime is not available"));
    }
    return Promise.resolve(module);
  }

  function loadPublishedRuntime() {
    return bundledRuntimeModule()
      .then(function (module) {
        if (!module || typeof module.createI18n !== "function") {
          throw new Error("lino-i18n did not export createI18n");
        }
        if (typeof module.loadCatalogs !== "function") {
          throw new Error("lino-i18n did not export loadCatalogs");
        }
        if (typeof global.fetch !== "function") {
          throw new Error("fetch is not available");
        }
        return module
          .loadCatalogs(CATALOG_URLS.map(cacheBustedUrl), {
            fetch: global.fetch.bind(global),
            requestInit: { cache: "no-cache" },
          })
          .then(function (catalog) {
            return [module, catalog];
          });
      })
      .then(function (results) {
        var module = results[0];
        CATALOG = results[1];
        runtimeEngine = module.createI18n({
          locales: CATALOG,
          defaultLocale: DEFAULT_LANGUAGE,
          fallback: [DEFAULT_LANGUAGE],
        });
        api.CATALOG = CATALOG;
        api.ENGINE_SOURCE = PUBLISHED_RUNTIME_SOURCE;
        api.lastError = null;
        dispatchReady();
        return api;
      })
      .catch(function (error) {
        runtimeEngine = null;
        api.ENGINE_SOURCE = UNAVAILABLE_RUNTIME_SOURCE;
        api.lastError = error && error.message ? error.message : String(error);
        dispatchReady();
        return api;
      });
  }

  var api = {
    DEFAULT_LANGUAGE: DEFAULT_LANGUAGE,
    SUPPORTED_LANGUAGES: SUPPORTED_LANGUAGES.slice(),
    CATALOG: CATALOG,
    CATALOG_URL: CATALOG_URL,
    CATALOG_URLS: CATALOG_URLS.slice(),
    ENGINE_SOURCE: LOADING_RUNTIME_SOURCE,
    PUBLISHED_RUNTIME_SOURCE: PUBLISHED_RUNTIME_SOURCE,
    lastError: null,
    browserLanguages: browserLanguages,
    detectLanguage: detectLanguage,
    normalizeLanguageTag: normalizeLanguageTag,
    resolveLanguage: resolveLanguage,
    t: t,
    ready: Promise.resolve(null),
  };

  global.FormalAiI18n = api;
  api.ready = loadPublishedRuntime();
})(typeof window !== "undefined" ? window : globalThis);
