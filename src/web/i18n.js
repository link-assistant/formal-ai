(function (global) {
  "use strict";

  var DEFAULT_LANGUAGE = "en";
  var SUPPORTED_LANGUAGES = ["en", "ru", "zh", "hi"];
  var PUBLISHED_RUNTIME_SOURCE = "lino-i18n@0.1.1";
  var LOADING_RUNTIME_SOURCE = "lino-i18n-loading";
  var UNAVAILABLE_RUNTIME_SOURCE = "lino-i18n-unavailable";
  var CATALOG_URL = "i18n-catalog.lino";
  // Permission/Services strings live in a second file so each catalog stays
  // under the Links Notation line limit (see scripts/check-file-size.rs). Both
  // files are fetched and their per-locale keys merged before parsing.
  var CATALOG_URLS = [CATALOG_URL, "i18n-catalog-permissions.lino"];
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

  function fetchOneCatalog(url) {
    return global
      .fetch(cacheBustedUrl(url), { cache: "no-cache" })
      .then(function (response) {
        if (!response || !response.ok) {
          var status = response ? "HTTP " + response.status : "no response";
          throw new Error("failed to load " + url + ": " + status);
        }
        return response.text();
      });
  }

  function fetchCatalogTexts() {
    if (typeof global.fetch !== "function") {
      return Promise.reject(new Error("fetch is not available"));
    }
    return Promise.all(CATALOG_URLS.map(fetchOneCatalog));
  }

  function mergeCatalogObjects(target, source) {
    Object.keys(source).forEach(function (locale) {
      var existing = target[locale] || {};
      var incoming = source[locale] || {};
      Object.keys(incoming).forEach(function (key) {
        existing[key] = incoming[key];
      });
      target[locale] = existing;
    });
    return target;
  }

  function catalogObjectFromParsed(parsed) {
    var output = {};
    if (Array.isArray(parsed)) {
      parsed.forEach(function (entry) {
        if (entry && entry.locale && entry.translations) {
          output[entry.locale] = entry.translations;
        }
      });
      return output;
    }
    if (
      parsed &&
      typeof parsed.forEach === "function" &&
      typeof parsed.get === "function"
    ) {
      parsed.forEach(function (translations, locale) {
        output[locale] = translations;
      });
      return output;
    }
    if (parsed && typeof parsed === "object") {
      Object.keys(parsed).forEach(function (locale) {
        output[locale] = parsed[locale];
      });
    }
    return output;
  }

  function loadPublishedRuntime() {
    return Promise.all([bundledRuntimeModule(), fetchCatalogTexts()])
      .then(function (results) {
        var module = results[0];
        var catalogTexts = results[1];
        if (!module || typeof module.createI18n !== "function") {
          throw new Error("lino-i18n did not export createI18n");
        }
        if (typeof module.parseLinoCatalogs !== "function") {
          throw new Error("lino-i18n did not export parseLinoCatalogs");
        }
        CATALOG = {};
        catalogTexts.forEach(function (catalogText) {
          mergeCatalogObjects(
            CATALOG,
            catalogObjectFromParsed(module.parseLinoCatalogs(catalogText)),
          );
        });
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
