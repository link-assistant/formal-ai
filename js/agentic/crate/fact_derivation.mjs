// Fact records derived from committed Wikidata captures (issue #1172 R9),
// the crate twin of rust/src/seed/fact_derivation.rs (`derived_facts`) for the
// agentic port; js/worker/formal_ai_worker_fact_derivation.js is the browser
// worker's. A record is a subject entity of data/seed/fact-captures.lino whose
// first claim's property grounds a fact_relation meaning that
// data/seed/fact-realization.lino realizes, with its value item captured too;
// the record has the browser shape (`subjectAliases`, `questionKeywords`,
// `localized`, ...), which solver_handlers_benchmark_prompts.mjs reads.

import { cached, parseLino, readText } from '../host.mjs';
import { splitPipeList } from './seed.mjs';
import { meaning } from './seed_meanings.mjs';

const FACT_CAPTURES_FILE = "data/seed/fact-captures.lino";
const FACT_REALIZATION_FILE = "data/seed/fact-realization.lino";
const FACT_SLOT_PATTERN = /\{(subject|value)(?::([a-z_]+))?\}/g;

/**
 * The single top-level record of a parsed seed file named `name`, or null.
 * @param {string} text
 * @param {string} name
 * @returns {object|null}
 */
function factSeedRoot(text, name) {
  if (!text) return null;
  const root = parseLino(text);
  if (root.name === name) return root;
  return (root.children || []).find((child) => child.name === name) || null;
}

/**
 * @param {object} node
 * @param {string} name
 * @returns {Array<object>}
 */
function factChildren(node, name) {
  return ((node && node.children) || []).filter((child) => child.name === name);
}

/**
 * @param {object} node
 * @param {string} name
 * @returns {string}
 */
function factChildText(node, name) {
  const child = factChildren(node, name)[0];
  return child ? String(child.id || "") : "";
}

/**
 * @param {string} value
 * @returns {Array<string>}
 */
function factList(value) {
  return splitPipeList(String(value || ""));
}

/**
 * The captured entities in seed order: labels, aliases, sitelinks, claims.
 * @param {object|null} root
 * @returns {Array<object>}
 */
function factCapturedEntities(root) {
  return factChildren(root, "entity").map((node) => {
    const entity = { qid: node.id, labels: new Map(), aliases: new Map(), sitelinks: new Map(), claims: [] };
    for (const child of node.children || []) {
      const items = factList(child.id);
      if (child.name === "label" && items.length === 2) entity.labels.set(items[0], items[1]);
      else if (child.name === "alias" && items.length > 1) entity.aliases.set(items[0], items.slice(1));
      else if (child.name === "sitelink" && items.length === 2) entity.sitelinks.set(items[0], items[1]);
      else if (child.name === "claim" && items.length === 3) {
        entity.claims.push({ property: items[0], value: items[1], rank: items[2] });
      }
    }
    return entity;
  });
}

/**
 * The realization tables of fact-realization.lino.
 * @param {object} root
 * @returns {object}
 */
function factRealizationRules(root) {
  const keyed = (name) => new Map(factChildren(root, name).map((node) => [node.id, node]));
  const source = factChildren(root, "source")[0] || null;
  const variants = new Map();
  for (const node of factChildren(root, "label_variant")) {
    const space = String(node.id).indexOf(" ");
    if (space > 0) variants.set(node.id.slice(0, space), factList(node.id.slice(space + 1)));
  }
  const preferred = new Map();
  for (const node of factChildren(root, "preferred_label")) {
    preferred.set(node.id, new Map((node.children || []).map((entry) => [entry.name, factChildText(entry, "text")])));
  }
  const surfaces = new Map();
  for (const node of factChildren(root, "prompt_surface")) {
    const space = String(node.id).indexOf(" ");
    if (space > 0) surfaces.set(node.id.slice(0, space), factList(node.id.slice(space + 1)));
  }
  const articles = new Map();
  for (const [language, node] of keyed("article")) {
    articles.set(language, {
      definite: factChildText(node, "definite"),
      heads: factList(factChildText(node, "definite_head")),
      possessive: factList(factChildText(node, "possessive")),
    });
  }
  const morphology = new Map();
  for (const [language, node] of keyed("morphology")) {
    const cases = new Map();
    for (const caseNode of factChildren(node, "case")) {
      cases.set(caseNode.id, {
        endings: factChildren(caseNode, "ending")
          .map((ending) => factList(ending.id))
          .filter((pair) => pair.length === 2)
          .sort((left, right) => right[0].length - left[0].length),
        afterConsonant: factChildText(caseNode, "after_consonant"),
      });
    }
    morphology.set(language, {
      initialMark: factChildText(node, "initial_mark"),
      acronyms: factChildText(node, "indeclinable_acronym") === "true",
      consonants: factChildText(node, "consonants"),
      cases,
    });
  }
  const relations = new Map();
  for (const [slug, node] of keyed("relation")) {
    relations.set(slug, {
      intent: factChildText(node, "intent"),
      category: factChildText(node, "category"),
      sourceKind: factChildText(node, "source_kind"),
      sourcePage: factChildText(node, "source_page"),
      order: factList(factChildText(node, "wikidata_order")),
      keywords: factList(factChildText(node, "question_keywords")),
      sentences: new Map(factChildren(node, "sentence").map((entry) => [entry.id, factChildText(entry, "text")])),
    });
  }
  return {
    languages: factList(factChildText(root, "languages")),
    variants,
    absorbedMark: factChildText(root, "absorbed_mark"),
    source: source === null ? null : {
      site: factChildText(source, "site"),
      url: factChildText(source, "url"),
      titleSpace: factChildText(source, "title_space"),
    },
    relations,
    preferred,
    surfaces,
    articles,
    morphology,
  };
}

/**
 * An entity's surface in `language`: its preferred label, else its label in
 * the first captured variant the language lists, else its label.
 * @param {object} rules
 * @param {object} entity
 * @param {string} language
 * @returns {string}
 */
function factSurface(rules, entity, language) {
  const preferred = rules.preferred.get(entity.qid);
  if (preferred && preferred.get(language)) return preferred.get(language);
  for (const variant of rules.variants.get(language) || [language]) {
    if (entity.labels.get(variant)) return entity.labels.get(variant);
  }
  return entity.labels.get(language) || "";
}

/**
 * The definite form of `text` (the article rule of `language`).
 * @param {object} rules
 * @param {string} text
 * @param {string} language
 * @returns {string}
 */
function factDefinite(rules, text, language) {
  const article = rules.articles.get(language);
  if (!article || !article.definite || !text) return text;
  if (text.toLowerCase().startsWith(`${article.definite} `)) return text;
  const words = text.split(" ");
  return article.heads.includes(words[words.length - 1].toLowerCase()) ? `${article.definite} ${text}` : text;
}

/**
 * True for an all-capital abbreviation of two or more letters.
 * @param {string} word
 * @returns {boolean}
 */
function factAcronym(word) {
  const letters = Array.from(word).filter((character) => character.toLowerCase() !== character.toUpperCase());
  return letters.length > 1 && letters.every((character) => character === character.toUpperCase());
}

/**
 * `text` in grammatical case `name` of `language`: the first word that is
 * neither an initial nor an acronym takes the longest matching ending, else
 * the after-consonant suffix; a word no rule covers stays as it is.
 * @param {object} rules
 * @param {string} text
 * @param {string} language
 * @param {string} name
 * @returns {string}
 */
function factInflect(rules, text, language, name) {
  const morphology = rules.morphology.get(language);
  const rule = morphology && morphology.cases.get(name);
  if (!rule) return text;
  const words = text.split(" ");
  for (let index = 0; index < words.length; index += 1) {
    const word = words[index];
    if (morphology.initialMark && word.endsWith(morphology.initialMark)) continue;
    if (morphology.acronyms && factAcronym(word)) continue;
    const ending = rule.endings.find((pair) => word.toLowerCase().endsWith(pair[0]));
    if (ending) {
      words[index] = word.slice(0, word.length - ending[0].length) + ending[1];
    } else if (rule.afterConsonant && word && morphology.consonants.includes(word.slice(-1).toLowerCase())) {
      words[index] = word + rule.afterConsonant;
    } else {
      return text;
    }
    return words.join(" ");
  }
  return text;
}

/**
 * @param {object} rules
 * @param {string} text
 * @param {string} language
 * @param {string} form "" | "definite" | a case name
 * @returns {string}
 */
function factForm(rules, text, language, form) {
  if (!form) return text;
  return form === "definite" ? factDefinite(rules, text, language) : factInflect(rules, text, language, form);
}

/**
 * Fill a sentence template; a slot ending in the absorbed mark takes the
 * place of the same mark right after it.
 * @param {object} rules
 * @param {string} template
 * @param {string} language
 * @param {object} subject
 * @param {object} value
 * @returns {string}
 */
function factRealize(rules, template, language, subject, value) {
  let out = "";
  let position = 0;
  for (const match of template.matchAll(FACT_SLOT_PATTERN)) {
    out += template.slice(position, match.index);
    const entity = match[1] === "subject" ? subject : value;
    const filled = factForm(rules, factSurface(rules, entity, language), language, match[2] || "");
    out += filled;
    position = match.index + match[0].length;
    const mark = rules.absorbedMark;
    if (mark && filled.endsWith(mark) && template.startsWith(mark, position)) position += mark.length;
  }
  return out + template.slice(position);
}

/**
 * The page of `entity` in `language`'s edition, from its captured sitelink.
 * @param {object} rules
 * @param {object} entity
 * @param {string} language
 * @returns {string}
 */
function factSourceUrl(rules, entity, language) {
  if (!rules.source) return "";
  const title = entity.sitelinks.get(rules.source.site.split("{language}").join(language));
  if (!title) return "";
  return rules.source.url
    .split("{language}").join(language)
    .split("{title}").join(title.split(" ").join(rules.source.titleSpace));
}

/**
 * The label-index surfaces of a subject: each captured label (and preferred
 * label) per language, with the article and morphology forms of it, then the
 * recorded prompt surfaces; lowercased and deduplicated in that order.
 * @param {object} rules
 * @param {object} entity
 * @returns {Array<string>}
 */
function factLabelSurfaces(rules, entity) {
  const out = [];
  const add = (text) => {
    const lower = String(text || "").toLowerCase();
    if (lower && !out.includes(lower)) out.push(lower);
  };
  const keys = rules.languages.slice();
  for (const language of rules.languages) {
    for (const variant of rules.variants.get(language) || []) if (!keys.includes(variant)) keys.push(variant);
  }
  const preferred = rules.preferred.get(entity.qid) || new Map();
  for (const key of keys) {
    for (const label of [entity.labels.get(key), preferred.get(key)]) {
      if (!label) continue;
      add(label);
      const article = rules.articles.get(key);
      if (article && article.definite) {
        const lead = `${article.definite} `;
        if (label.toLowerCase().startsWith(lead)) add(label.slice(lead.length));
        add(factDefinite(rules, label, key));
        if (!label.includes(" ")) for (const ending of article.possessive) add(label + ending);
      }
      const morphology = rules.morphology.get(key);
      if (morphology) for (const name of morphology.cases.keys()) add(factInflect(rules, label, key, name));
    }
  }
  for (const surface of rules.surfaces.get(entity.qid) || []) add(surface);
  return out;
}

/**
 * The fact records the captures and the realization rules derive, in capture
 * order (`derived_facts`).
 * @returns {Array<object>}
 */
export function derivedFacts() {
  return cached("derived-capture-facts", () => buildDerivedFacts());
}

function buildDerivedFacts() {
  const captures = factSeedRoot(readText(FACT_CAPTURES_FILE), "fact_captures");
  const realization = factSeedRoot(readText(FACT_REALIZATION_FILE), "fact_realization");
  if (captures === null || realization === null) return [];
  const rules = factRealizationRules(realization);
  const entities = factCapturedEntities(captures);
  const byQid = new Map(entities.map((entity) => [entity.qid, entity]));
  const relationOf = new Map();
  for (const slug of rules.relations.keys()) {
    const property = meaning(slug)?.wikidata ?? "";
    if (property) relationOf.set(property, slug);
  }
  const records = [];
  for (const subject of entities) {
    const claim = subject.claims[0];
    const slug = claim && relationOf.get(claim.property);
    const value = claim && byQid.get(claim.value);
    if (!slug || !value) continue;
    const relation = rules.relations.get(slug);
    const localized = rules.languages
      .filter((language) => relation.sentences.has(language))
      .map((language) => ({
        language,
        subjectLabel: factDefinite(rules, factSurface(rules, subject, language), language),
        valueLabel: factSurface(rules, value, language),
        summary: factRealize(rules, relation.sentences.get(language), language, subject, value),
        source: factSourceUrl(rules, relation.sourcePage === "subject" ? subject : value, language),
        sourceKind: relation.sourceKind,
      }));
    const lead = localized[0] || {};
    records.push({
      slug: `fact_${slug}_${subject.qid.toLowerCase()}`,
      intent: relation.intent || "fact_lookup",
      category: relation.category,
      wikidata: relation.order.map((role) => (role === "subject" ? subject.qid : value.qid)),
      relation: slug,
      subjectQid: subject.qid,
      valueQid: value.qid,
      subjectLabel: lead.subjectLabel || "",
      valueLabel: lead.valueLabel || "",
      subjectAliases: factLabelSurfaces(rules, subject),
      questionKeywords: relation.keywords.map((keyword) => keyword.toLowerCase()),
      summary: lead.summary || "",
      source: lead.source || "",
      sourceKind: relation.sourceKind,
      localized,
    });
  }
  return records;
}
