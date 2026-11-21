// Whole record readers shared by the browser, worker and Node hosts.
// Dependencies stay in the seed loader; this factory owns no parser cache.
const formalAiSeedRecordReaders = (function () {
  "use strict";
  return function (findChildren, findChildValue, splitRefList, splitList, toLower) {
  function extractConcepts(root) {
    if (!root || !Array.isArray(root.children)) return [];
    // Concept files are flat — each top-level child is one record.
    var concepts = [];
    var iterate = function (item) {
      if (!item || !item.name) return;
      if (!item.name.startsWith("concept_")) return;
      var aliases = findChildValue(item, "aliases");
      var contexts = findChildValue(item, "contexts");
      var contextLinks = findChildValue(item, "context_links");
      var localized = findChildren(item, "localized").map(function (loc) {
        var locAliases = findChildValue(loc, "aliases");
        return {
          language: loc.id,
          term: findChildValue(loc, "term"),
          aliases: splitRefList(locAliases),
          summary: findChildValue(loc, "summary"),
          source: findChildValue(loc, "source"),
          sourceKind: findChildValue(loc, "source_kind"),
        };
      });
      concepts.push({
        slug: item.name,
        term: findChildValue(item, "term"),
        category: findChildValue(item, "category") || "concept",
        summary: findChildValue(item, "summary"),
        source: findChildValue(item, "source"),
        sourceKind: findChildValue(item, "source_kind") || "project-docs",
        wikidata: findChildValue(item, "wikidata"),
        aliases: splitRefList(aliases),
        contexts: splitRefList(contexts),
        contextLinks: splitRefList(contextLinks),
        localized: localized,
      });
    };
    if (root.name && root.name.startsWith("concept_")) {
      iterate(root);
    } else if (Array.isArray(root.children)) {
      root.children.forEach(iterate);
    }
    return concepts;
  }
  // Extract disambiguating context records (`concept-contexts.lino`).
  // Each context is anchored by a Wikidata Q-ID and carries per-language
  // localized labels plus a `|`-separated alias list (free-text phrases the
  // user might type in the four supported languages).
  function extractConceptContexts(root) {
    if (!root || !Array.isArray(root.children)) return [];
    var out = [];
    var visit = function (parent) {
      var entries = findChildren(parent, "context");
      for (var i = 0; i < entries.length; i += 1) {
        var entry = entries[i];
        if (!entry.id) continue;
        var aliases = findChildValue(entry, "aliases");
        var labels = findChildren(entry, "label").map(function (label) {
          return { language: label.id, text: findChildValue(label, "text") };
        });
        out.push({
          slug: entry.id,
          wikidata: findChildValue(entry, "wikidata"),
          aliases: splitRefList(aliases),
          labels: labels,
        });
      }
    };
    visit(root);
    for (var i = 0; i < root.children.length; i += 1) {
      visit(root.children[i]);
    }
    return out;
  }
  function extractFacts(root) {
    if (!root || !Array.isArray(root.children)) return [];
    var facts = [];
    var visit = function (item) {
      if (!item || !item.name || item.name.indexOf("fact_") !== 0) return;
      var localized = findChildren(item, "localized").map(function (loc) {
        return {
          language: loc.id,
          subjectLabel: findChildValue(loc, "subject_label"),
          valueLabel: findChildValue(loc, "value_label"),
          summary: findChildValue(loc, "summary"),
          source: findChildValue(loc, "source"),
          sourceKind: findChildValue(loc, "source_kind"),
        };
      });
      facts.push({
        slug: item.name,
        intent: findChildValue(item, "intent") || "fact_lookup",
        category: findChildValue(item, "category"),
        wikidata: splitList(findChildValue(item, "wikidata")),
        relation: findChildValue(item, "relation"),
        subjectQid: findChildValue(item, "subject_qid"),
        valueQid: findChildValue(item, "value_qid"),
        subjectLabel: findChildValue(item, "subject_label"),
        valueLabel: findChildValue(item, "value_label"),
        subjectAliases: splitList(findChildValue(item, "subject_aliases")).map(toLower),
        questionKeywords: splitList(findChildValue(item, "question_keywords")).map(toLower),
        summary: findChildValue(item, "summary"),
        releaseTimeline: findChildValue(item, "release_timeline"),
        source: findChildValue(item, "source"),
        sourceKind: findChildValue(item, "source_kind"),
        localized: localized,
      });
    };
    if (root.name && root.name.indexOf("fact_") === 0) {
      visit(root);
    } else {
      root.children.forEach(visit);
    }
    return facts;
  }
    return { extractConcepts: extractConcepts,
      extractConceptContexts: extractConceptContexts, extractFacts: extractFacts };
  };
})();
