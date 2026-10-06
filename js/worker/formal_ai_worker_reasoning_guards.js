// Reasoning guards shared with the native surface.
//
// 1. Request-legality advisory (issue #836), mirroring
//    rust/src/legality_warning.rs: the seeded catalogue
//    (data/seed/legality-patterns.lino) names the categories, dispositions,
//    reasons, jurisdiction notes, trigger phrases and the exemption framings;
//    the wording is the legality_warn / legality_refuse templates in
//    data/seed/multilingual-responses-legality.lino. Warn, don't police: a
//    warn category with a legitimate framing proceeds unflagged, the narrow
//    refuse set is framing-proof.
// 2. Issue #1173 R5: no web-search surface answers with the canned
//    description of the search machinery ("Web search requested for …");
//    such an answer degrades to the localized `web_search_unavailable`
//    response that names the query, as rust/src/solver_handlers/
//    web_requests/live_search.rs does.

/**
 * The value of the first child named `name` (Rust `find_child_value`).
 * @param {{children: object[]}} node
 * @param {string} name
 * @returns {string}
 */
function guardChildValue(node, name) {
  const child = (node.children || []).find((candidate) => candidate.name === name);
  return child ? String(child.value || "") : "";
}

/**
 * The phrases of a catalogue record, in order.
 * @param {{children: object[]}} node
 * @returns {string[]}
 */
function guardPhrases(node) {
  return (node.children || [])
    .filter((child) => child.name === "phrase" && child.value)
    .map((child) => String(child.value));
}

let cachedLegalityCatalogue = null;

/**
 * The seeded legality patterns and exemptions, in file order.
 * @returns {{patterns: object[], exemptions: object[]}}
 */
function legalityCatalogue() {
  if (cachedLegalityCatalogue) return cachedLegalityCatalogue;
  const text = seedRawText(SEED_RAW, "legality-patterns.lino");
  if (!text) return { patterns: [], exemptions: [] };
  const roots = parseLinoTree(text).children;
  const patterns = [];
  const exemptions = [];
  for (const root of roots) {
    if (root.name === "legality_patterns") {
      for (const record of root.children) {
        if (record.name !== "pattern") continue;
        const disposition = guardChildValue(record, "disposition");
        const category = guardChildValue(record, "category");
        if ((disposition !== "refuse" && disposition !== "warn") || category === "") continue;
        patterns.push({
          category,
          disposition,
          reason: guardChildValue(record, "reason"),
          jurisdictionNote: guardChildValue(record, "jurisdiction_note"),
          phrases: guardPhrases(record),
        });
      }
    } else if (root.name === "legality_exemptions") {
      for (const record of root.children) {
        if (record.name !== "exemption") continue;
        const framing = guardChildValue(record, "framing");
        if (framing === "") continue;
        exemptions.push({ framing, phrases: guardPhrases(record) });
      }
    }
  }
  cachedLegalityCatalogue = { patterns, exemptions };
  return cachedLegalityCatalogue;
}

/**
 * Does any phrase occur in the normalized or lowered prompt?
 * @param {string[]} phrases
 * @param {string} normalized
 * @param {string} lower
 * @returns {boolean}
 */
function guardPhraseMatches(phrases, normalized, lower) {
  return phrases.some((phrase) => normalized.includes(phrase) || lower.includes(phrase));
}

/**
 * Assess a request against the catalogue (Rust `legality_warning::assess`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {{pattern: object, exemptFraming: string}|null}
 */
function assessLegality(prompt, normalized) {
  const catalogue = legalityCatalogue();
  const lower = String(prompt || "").toLowerCase();
  const text = String(normalized || "");
  const pattern = catalogue.patterns.find((candidate) =>
    guardPhraseMatches(candidate.phrases, text, lower));
  if (!pattern) return null;
  const exemption = catalogue.exemptions.find((candidate) =>
    guardPhraseMatches(candidate.phrases, text, lower));
  return { pattern, exemptFraming: exemption ? exemption.framing : "" };
}

/**
 * The English seed template with `{placeholder}` slots filled (the native
 * advisory always renders the English template).
 * @param {string} intent
 * @param {{name: string, value: string}[]} values
 * @returns {string}
 */
function guardEnglishTemplate(intent, values) {
  const table = MULTILINGUAL_ANSWERS[intent] || {};
  const raw = table.en || table.unknown;
  let out = raw ? (typeof raw === "string" ? raw : String(raw.text || "")) : "";
  for (const value of values) out = out.split(`{${value.name}}`).join(value.value);
  return out;
}

/**
 * `legality_warning` precedence row (Rust `handle_legality_warning`): the
 * advisory answer when the catalogue says warn or refuse, null when the
 * request is unflagged or a legitimate framing suppressed its warning.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryLegalityWarning(prompt, normalized) {
  const assessment = assessLegality(prompt, normalized);
  if (!assessment) return null;
  const pattern = assessment.pattern;
  const evidence = [
    "handler:legality_warning",
    `legality:category:${pattern.category}`,
    `legality:disposition:${pattern.disposition}`,
  ];
  if (assessment.exemptFraming !== "") {
    evidence.push(`legality:framing:${assessment.exemptFraming}`);
  }
  if (pattern.disposition === "warn" && assessment.exemptFraming !== "") return null;
  const refuse = pattern.disposition === "refuse";
  const intent = refuse ? "legality_refuse" : "legality_warn";
  evidence.push(`response:${intent}`);
  return {
    intent,
    content: guardEnglishTemplate(intent, [
      { name: "category", value: pattern.category },
      { name: "category_readable", value: pattern.category.split("_").join(" ") },
      { name: "reason", value: pattern.reason },
      { name: "jurisdiction_note", value: pattern.jurisdictionNote },
    ]),
    confidence: refuse ? 0.9 : 0.7,
    evidence,
  };
}

/** The canned openers of the deleted search-machinery description (#1173). */
const CANNED_WEB_SEARCH_OPENERS = [
  "Web search requested",
  "Поиск в интернете запрошен",
  "Providers considered",
];

/**
 * Issue #1173 R5: a web-search answer may never be the canned description
 * of the search machinery; it degrades to the seeded
 * `web_search_unavailable` response naming the query instead.
 * @param {object} answer
 * @param {string} language
 * @returns {object}
 */
function guardCannedWebSearchAnswer(answer, language) {
  if (!answer || typeof answer.content !== "string") return answer;
  const canned = CANNED_WEB_SEARCH_OPENERS.some((opener) => answer.content.includes(opener));
  if (!canned) return answer;
  const table = MULTILINGUAL_ANSWERS.web_search_unavailable || {};
  const raw = table[language] || table.unknown || table.en;
  const template = raw ? (typeof raw === "string" ? raw : String(raw.text || "")) : "";
  const query = String(answer.query || answer.formalizedObject || "");
  return Object.assign({}, answer, {
    intent: "web_search",
    content: template.split("{query}").join(query),
    confidence: 0,
    evidence: (Array.isArray(answer.evidence) ? answer.evidence : [])
      .concat(["response:web_search_unavailable", "web_search:canned_description_refused"]),
  });
}
