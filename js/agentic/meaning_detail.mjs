import { nonNullObject as isObject } from '../logging/object-values.js';
// Derive meaning details from a complete Wikidata publisher response.

import { cached, childrenNamed, parseLino, readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { byteOrder, trimEnd } from './crate/rust_str.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { surfacePresent } from './crate/seed_meanings.mjs';

const SINGULAR_FEATURE = 'Q110786';
const PLURAL_FEATURE = 'Q146786';
const NUMBER_FEATURES = [SINGULAR_FEATURE, PLURAL_FEATURE];
const SOURCE_ENDPOINT = 'https://www.wikidata.org/w/api.php?action=wbgetentities&ids=';
const MEANINGS = 'data/seed/meanings-translation.lino';

const field = (node, name) => childrenNamed(node, name)[0]?.value ?? '';
const validLanguageCode = (code) => /^[a-z0-9-]{1,20}$/u.test(code);
const wikidataItem = (id) => typeof id === 'string' && /^Q[0-9]{1,18}$/u.test(id);
const validLexeme = (id) => /^L[0-9]{1,18}$/u.test(id);

function safeSurface(value) {
  const text = typeof value === 'string' ? value.trim() : '';
  return text && new TextEncoder().encode(text).length <= 300 && !/[#"\p{Cc}]/u.test(text) ? text : null;
}

function parseConcept(node) {
  const name = node.name;
  const grounded_in = field(node, 'grounded-in');
  if (!wikidataItem(grounded_in) || !/^[a-z_-]+$/u.test(name)) return null;
  const entries = childrenNamed(node, 'source-lexeme').filter((child) => validLexeme(child.id));
  const sources = entries.map((child) => child.id);
  const required_senses = entries.filter((child) => field(child, 'sense')).map((child) => child.id);
  if (!sources.length || sources.length > 20) return null;
  const extras = [];
  for (const lexeme of childrenNamed(node, 'lexeme').filter((child) => validLanguageCode(child.id))) {
    for (const surface of childrenNamed(lexeme, 'surface')) {
      const text = safeSurface(field(surface, 'text'));
      if (text !== null) extras.push({ language: lexeme.id, text });
    }
  }
  const keywords = [name];
  for (const surface of childrenNamed(node, 'surface')) {
    const text = safeSurface(field(surface, 'text'));
    if (text !== null) keywords.push(text.toLowerCase());
  }
  keywords.push(...extras.map((extra) => extra.text.toLowerCase()));
  return {
    name, grounded_in, sources, required_senses, extras, keywords,
    search_query: agenticMessage('meaning-detail-search-query', { name, lexemes: sources.join(' ') }),
    source_url: `${SOURCE_ENDPOINT}${sources.join('%7C')}&format=json&origin=*`,
    kb_path: `meanings-${name}-detail.lino`,
  };
}

export function concepts() {
  return cached('meaning-detail-concepts', () => {
    const root = parseLino(readText(MEANINGS));
    const containers = root.name === 'meanings' ? [root] : childrenNamed(root, 'meanings');
    return containers.flatMap((container) => container.children).map(parseConcept).filter((concept) => concept !== null);
  });
}

/** Resolve one registered meaning for a grammatical-detail request. */
export function conceptForTask(prompt) {
  if (!isMeaningDetailTask(prompt)) return null;
  const lower = normalizePrompt(prompt);
  const candidates = concepts().filter((concept) => concept.keywords.some((keyword) => surfacePresent(lower, normalizePrompt(keyword))));
  return candidates.length === 1 ? candidates[0] : null;
}

export function isMeaningDetailTask(prompt) {
  const lower = prompt.toLowerCase();
  const markers = cached('meaning-detail-keywords', () => agenticMessage('meaning_detail_keywords').split('|').filter(Boolean));
  return markers.some((keyword) => lower.includes(keyword));
}

/** serde_json `Map` iteration order (a `BTreeMap`: keys sorted). */
const sortedKeys = (object) => Object.keys(object).sort(byteOrder);

/** Mirrors `fn parse_entities`: `Map<id, RawLexeme>` in sorted key order. */
function parseEntities(text, groundedIn) {
  const map = new Map();
  let doc;
  try {
    doc = JSON.parse(text);
  } catch {
    return map;
  }
  const entities = isObject(doc) ? doc.entities : undefined;
  if (!isObject(entities)) return map;
  for (const id of sortedKeys(entities)) {
    const raw = parseLexemeEntity(id, entities[id], groundedIn);
    if (raw !== null) map.set(id, raw);
  }
  return map;
}

/** Mirrors `fn parse_lexeme_entity`. */
function parseLexemeEntity(id, entity, groundedIn) {
  if (!isObject(entity) || entity.id !== id || entity.type !== 'lexeme') return null;
  const languageItem = entity.language;
  const category = entity.lexicalCategory;
  if (!wikidataItem(languageItem) || category !== 'Q1084' || !isObject(entity.lemmas)) return null;
  const languageCode = sortedKeys(entity.lemmas)[0];
  if (!languageCode || !validLanguageCode(languageCode)) return null;
  const lemma = safeSurface(entity.lemmas[languageCode]?.value);
  if (lemma === null) return null;
  const senses = Array.isArray(entity.senses) ? entity.senses : [];
  const sense = senses.find((entry) => matchesSubidentifier(id, entry?.id, 'S') &&
    Array.isArray(entry.claims?.P5137) && entry.claims.P5137.some((claim) => claim?.rank !== 'deprecated' &&
      claim?.mainsnak?.datavalue?.value?.id === groundedIn))?.id ?? null;
  const forms = [];
  if (Array.isArray(entity.forms)) {
    for (const form of entity.forms) {
      const raw = parseForm(form, id, languageCode);
      if (raw !== null) forms.push(raw);
    }
  }
  return { language_item: languageItem, language_code: languageCode, lemma, category, sense, forms };
}

function matchesSubidentifier(lexeme, identifier, kind) {
  const prefix = `${lexeme}-${kind}`;
  return typeof identifier === 'string' && identifier.startsWith(prefix) && /^[0-9]+$/u.test(identifier.slice(prefix.length));
}

/** Mirrors `fn parse_form`. */
function parseForm(form, lexeme, language) {
  if (!matchesSubidentifier(lexeme, form?.id, 'F')) return null;
  const text = safeSurface(form.representations?.[language]?.value);
  if (text === null) return null;
  const features = Array.isArray(form.grammaticalFeatures) ? form.grammaticalFeatures.filter((item) => typeof item === 'string') : [];
  return { id: form.id, text, features };
}

const hasFeature = (form, feature) => form.features.includes(feature);

/** Mirrors `fn case_key`. */
function caseKey(form) {
  return form.features.filter((feature) => !NUMBER_FEATURES.includes(feature)).sort(byteOrder).join('\u0000');
}

/** Mirrors `fn select_singular`. */
function selectSingular(raw) {
  const singulars = raw.forms.filter((form) => hasFeature(form, SINGULAR_FEATURE));
  return singulars.find((form) => form.text === raw.lemma) ?? singulars[0] ?? null;
}

/** Mirrors `fn select_plural`. */
function selectPlural(raw, singular) {
  const plurals = raw.forms.filter((form) => hasFeature(form, PLURAL_FEATURE));
  const key = singular === null ? null : caseKey(singular);
  const matched = key === null ? undefined : plurals.find((form) => caseKey(form) === key);
  return matched ?? plurals[0] ?? null;
}

/** Mirrors `fn form_suffix`. */
function formSuffix(formId) {
  const parts = formId.split('-');
  return parts[parts.length - 1];
}

/** Mirrors `fn derive_source`. */
function deriveSource(id, raw) {
  const singular = selectSingular(raw);
  const plural = selectPlural(raw, singular);
  const forms = [];
  if (singular !== null) forms.push({ suffix: formSuffix(singular.id), text: singular.text, number: 'singular', feature: SINGULAR_FEATURE });
  if (plural !== null) forms.push({ suffix: formSuffix(plural.id), text: plural.text, number: 'plural', feature: PLURAL_FEATURE });
  return { id, language: raw.language_code, language_item: raw.language_item, category: raw.category, sense: raw.sense, forms };
}

/** Derive grammatical forms from the complete requested Wikidata batch. */
export function conceptLexemes(concept, fetched) {
  if (typeof fetched !== 'string' || new TextEncoder().encode(fetched).length > 1048576) return null;
  const entities = parseEntities(fetched, concept.grounded_in);
  const sources = [];
  for (const id of concept.sources) {
    if (!entities.has(id)) return null;
    const source = deriveSource(id, entities.get(id));
    if (source.forms.length !== NUMBER_FEATURES.length ||
        (source.sense === null && concept.required_senses.includes(id))) return null;
    sources.push(source);
  }
  return { sources, extras: concept.extras };
}

/** Mirrors `fn language_name`. */
function languageName(code) {
  return { en: 'english', ru: 'russian', hi: 'hindi', zh: 'chinese' }[code] ?? 'unknown';
}

/**
 * Mirrors `fn render_block`: the enriched meaning block (two-space indent).
 * The `# wikidata ...` comments are templates in agentic-messages.lino.
 */
export function renderBlock(concept, lexemes) {
  let out = '';
  const line = (text) => {
    out += `${text}\n`;
  };
  line(`  ${concept.name}`);
  line(`    grounded-in ${concept.grounded_in}`);
  line(agenticMessage('meaning_detail_defined_by_line'));
  line('    role compositional_lemma');
  for (const sourceLexeme of lexemes.sources) {
    const name = languageName(sourceLexeme.language);
    const lemma = sourceLexeme.forms.length ? sourceLexeme.forms[0].text : null;
    const commentLemma = sourceLexeme.language === 'ru';
    const lemmaSuffix = commentLemma && lemma !== null ? ` ${lemma}` : '';
    line(agenticMessage('meaning_detail_source_lexeme_line', { id: sourceLexeme.id, name, lemma_suffix: lemmaSuffix }));
    line(agenticMessage('meaning_detail_language_line', { item: sourceLexeme.language_item, name }));
    line(agenticMessage('meaning_detail_category_line', { category: sourceLexeme.category }));
    for (const form of sourceLexeme.forms) {
      line(agenticMessage('meaning_detail_form_line', { id: sourceLexeme.id, suffix: form.suffix, text: form.text }));
      line(agenticMessage('meaning_detail_feature_line', { feature: form.feature, number: form.number }));
    }
    if (sourceLexeme.sense !== null) line(agenticMessage('meaning_detail_sense_line', { sense: sourceLexeme.sense }));
    for (const form of sourceLexeme.forms) {
      const commentText = commentLemma ? ` ${form.text}` : '';
      line(agenticMessage('meaning_detail_surface_line', {
        id: sourceLexeme.id, suffix: form.suffix, name, number: form.number, comment_text: commentText,
      }));
      line(`      text ${form.text}`);
      line(`      language ${sourceLexeme.language}`);
      line('      part_of_speech noun');
      line(`      grammatical_number ${form.number}`);
      if (sourceLexeme.sense !== null) line(agenticMessage('meaning_detail_sense_line', { sense: sourceLexeme.sense }));
    }
  }
  const seenLanguages = [];
  for (const extra of lexemes.extras) if (!seenLanguages.includes(extra.language)) seenLanguages.push(extra.language);
  for (const language of seenLanguages) {
    line(`    lexeme ${language}`);
    for (const extra of lexemes.extras.filter((candidate) => candidate.language === language)) {
      line('      surface');
      line(`        text ${extra.text}`);
      line('        part_of_speech noun');
    }
  }
  return out;
}

/** Mirrors `fn enrich_block`. @param {object} concept @param {string|null} fetched */
export function enrichBlock(concept, fetched) {
  const lexemes = conceptLexemes(concept, fetched);
  return lexemes === null ? null : renderBlock(concept, lexemes);
}

/** Mirrors `fn final_answer_for`. */
export function finalAnswerFor(concept, block) {
  return agenticMessage('meaning_detail_final_answer', { name: concept.name, path: concept.kb_path, block: trimEnd(block) });
}
