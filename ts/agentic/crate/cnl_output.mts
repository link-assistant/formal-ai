// Controlled output: reversible lexical projection, independent of input routing.
// Rust counterpart: rust/src/cnl_output.rs. Profiles and wording are shared data.
import { cached, childValue, childrenNamed, parseLino, readText } from '../host.mjs';

export const CNL_POLICY_PATH = 'data/seed/cnl-output.lino';

/** Read profile constraints and explicitly equivalent lexical realizations. */
export function cnlPolicy() {
  return cached('cnl-output-policy', () => parseOutputPolicy(readText(CNL_POLICY_PATH)));
}

/** Parse canonical policy data without requiring a solver or a runtime host. */
export function parseOutputPolicy(text, parse = parseLino) {
    const parsed = parse(text);
    const root = parsed.name === 'controlled-output' ? parsed : childrenNamed(parsed, 'controlled-output')[0];
    if (!root) throw new Error('cnl_policy_missing');
    const profiles = childrenNamed(root, 'profile').map(node => ({
      id: node.value, aliases: childValue(node, 'aliases').split('|'),
      maxWords: Number(childValue(node, 'maximum-words')) || null,
      forbidden: childValue(node, 'forbidden').split('|').filter(Boolean),
      required: childValue(node, 'required').split('|').filter(Boolean),
    }));
    const languages = childrenNamed(root, 'language').map(node => ({
      id: node.value, notice: childValue(node, 'notice'), frequency: childValue(node, 'frequency'),
      primes: childValue(node, 'primes').split('|').filter(Boolean),
      grammar: childValue(node, 'grammar').split('|').filter(Boolean),
      directives: childrenNamed(node, 'directive').map(cue => cue.value),
      switches: childrenNamed(node, 'switch').map(cue => ({text: cue.value, mode: childValue(cue, 'mode')})),
    }));
    const lexemes = childrenNamed(root, 'lexeme').map(node => ({
      id: node.value, language: childValue(node, 'language'),
      forms: childrenNamed(node, 'form').map(form => ({
        text: form.value, mode: childValue(form, 'mode'),
      })),
    }));
    return {profiles, languages, lexemes};
}

/** A positive integer tier; no floating point, signed or exponential aliases. */
export function frequencyTier(value) {
  if (!/^top-[1-9][0-9]*$/u.test(value)) return null;
  const count = Number(value.slice(4));
  return Number.isSafeInteger(count) ? count : null;
}

/** Validate output options only. This function never validates user input. */
export function outputConfiguration(value = {}, policy = cnlPolicy()) {
  const named = value.output_standard ?? 'cnl-default';
  let density = value.lexical_density ?? 'standard';
  if (typeof named !== 'string' || typeof density !== 'string') throw new TypeError('cnl_option_type');
  const lower = named.toLowerCase();
  const densityMode = ['primes', 'dense', 'max-precision'].includes(lower) || frequencyTier(lower) !== null;
  const profile = densityMode ? policy.profiles.find(item => item.id === 'cnl-default')
    : policy.profiles.find(item => item.id === lower || item.aliases.some(alias => alias.toLowerCase() === lower));
  if (!profile) throw new RangeError('cnl_unknown_standard');
  if (densityMode && value.lexical_density === undefined) density = lower;
  density = density.toLowerCase();
  if (density === 'dense') density = 'max-precision';
  if (!['primes', 'standard', 'max-precision'].includes(density) && frequencyTier(density) === null) {
    throw new RangeError('cnl_unknown_density');
  }
  return {output_standard: profile.id, lexical_density: density};
}

/**
 * Lossless representation. Only declared whole utterances acquire a meaning.
 * Substrings cannot change a quote, identifier, negation or quantifier scope.
 * All other bytes remain opaque. Retention does not count as understanding.
 */
export function formalizeOutput(text, language = 'en', policy = cnlPolicy()) {
  if (typeof text !== 'string') throw new TypeError('cnl_text_type');
  const trimmed = text.trim();
  const matches = policy.lexemes.filter(item => item.language === language
    && item.forms.some(form => form.text.toLowerCase() === trimmed.toLowerCase()));
  const parts = [];
  if (matches.length === 1) {
    const start = text.indexOf(trimmed);
    if (start) parts.push({kind: 'opaque', text: text.slice(0, start)});
    parts.push({kind: 'meaning', id: matches[0].id, surface: trimmed});
    if (start + trimmed.length < text.length) parts.push({kind: 'opaque', text: text.slice(start + trimmed.length)});
  } else if (text) {
    parts.push({kind: 'opaque', text});
  }
  return {language, source: text, parts};
}

/** The canonical lexical identity includes every unresolved byte and its order. */
export function outputMeaning(document) {
  return document.parts.map(part => part.kind === 'meaning' ? ['meaning', part.id] : ['opaque', part.text]);
}

/** Whitespace-delimited languages use Unicode words; CJK has no whitespace promise. */
export function outputWords(text, language) {
  return language === 'zh' ? (text.match(/[\p{Script=Han}]|[\p{L}\p{M}\p{N}]+/gu) ?? [])
    : (text.match(/[\p{L}\p{M}\p{N}]+(?:[-’'][\p{L}\p{M}\p{N}]+)*/gu) ?? []);
}

function capitalizedLike(text, original) {
  if (!text || !original || original[0] === original[0].toLowerCase()) return text;
  return text[0].toUpperCase() + text.slice(1);
}

/** Select a realization without ever deleting an opaque span or a proposition. */
export function projectOutput(document, configuration = {}, policy = cnlPolicy()) {
  const selected = outputConfiguration(configuration, policy);
  const parts = document.parts.map(part => {
    if (part.kind !== 'meaning') return part.text;
    const lexeme = policy.lexemes.find(item => item.id === part.id && item.language === document.language);
    if (!lexeme) return part.surface;
    const desired = selected.lexical_density === 'standard' ? selected.output_standard : selected.lexical_density;
    let forms = lexeme.forms.filter(form => form.mode.split('|').includes(desired));
    if (forms.length === 0) return part.surface;
    // Minimum over the declared equivalent realizations, with a stable tie break.
    if (selected.lexical_density === 'max-precision') forms = [...forms].sort((left, right) =>
      outputWords(left.text, document.language).length - outputWords(right.text, document.language).length
      || [...left.text].length - [...right.text].length || (left.text < right.text ? -1 : left.text > right.text ? 1 : 0));
    return capitalizedLike(forms[0].text, part.surface);
  });
  return parts.join('');
}

/**
 * Constraint evidence, not a certificate. A standard is compliant only when
 * every required independent checker ran and passed. Missing checks stay open.
 */
export function validateOutput(text, language, configuration = {}, policy = cnlPolicy(), checkers = {}) {
  const selected = outputConfiguration(configuration, policy);
  const profile = policy.profiles.find(item => item.id === selected.output_standard);
  const locale = policy.languages.find(item => item.id === language);
  const violations = [];
  const unchecked = [];
  if (!locale) unchecked.push('language');
  const tokens = outputWords(text, language).map(word => word.toLowerCase());
  if (profile.maxWords && text.split(/[.!?。！？]+/u).some(sentence => outputWords(sentence, language).length > profile.maxWords)) {
    violations.push('sentence_length');
  }
  if (selected.output_standard === 'e-prime') {
    if (language !== 'en') unchecked.push('e_prime_language');
    else {
      const words = tokens.map(word => word.replaceAll('’', "'"));
      if (words.some(word => profile.forbidden.includes(word) || /'(?:m|re)$/u.test(word))) violations.push('forbidden_word');
      if (words.some(word => word.endsWith("'s") || word === "ain't")) unchecked.push('e_prime_contraction');
    }
  }
  if (selected.lexical_density === 'primes') {
    if (locale?.primes.length) {
      const allowed = new Set([...locale.primes, ...locale.grammar].flatMap(word => outputWords(word, language).map(token => token.toLowerCase())));
      if (tokens.some(word => !allowed.has(word))) violations.push('non_prime');
    } else unchecked.push('nsm_vocabulary');
    for (const required of ['nsm_syntax', 'nsm_semantics']) {
      const result = checkers[required]?.(text, language);
      if (result === false) violations.push(required);
      else if (result !== true) unchecked.push(required);
    }
  }
  if (selected.lexical_density === 'max-precision') {
    const result = checkers.lexical_minimum?.(text, language);
    if (result === false) violations.push('lexical_minimum');
    else if (result !== true) unchecked.push('lexical_minimum');
  }
  const tier = frequencyTier(selected.lexical_density);
  if (tier !== null) {
    // No fabricated frequency ranks. A corpus-backed word-rank checker is required.
    const result = checkers.frequency?.(text, language, tier);
    if (result === false) violations.push('frequency_vocabulary');
    else if (result !== true) unchecked.push('frequency_vocabulary');
  }
  for (const required of profile.required) {
    const result = checkers[required]?.(text, language);
    if (result === false) violations.push(required);
    else if (result !== true) unchecked.push(required);
  }
  return {...selected, compliant: violations.length === 0 && unchecked.length === 0,
    violations, unchecked};
}

/** Render from the retained document each time; notices never enter its meaning. */
export function renderOutput(document, configuration = {}, policy = cnlPolicy(), checkers = {}) {
  const selected = outputConfiguration(configuration, policy);
  const text = projectOutput(document, selected, policy);
  const report = validateOutput(text, document.language, selected, policy, checkers);
  const notice = selected.lexical_density === 'max-precision'
    ? (policy.languages.find(item => item.id === document.language)?.notice ?? '') : '';
  return {text, notice, content: notice ? notice + '\n\n' + text : text, document,
    links_notation: outputLinksNotation(document), report};
}

/**
 * Recognize only whole user directives or a delimited directive prefix. Text
 * inside a quote/code block and historical assistant messages has no authority.
 */
function directiveConfiguration(mode, policy) {
  const configuration = outputConfiguration({output_standard: mode}, policy);
  if (configuration.lexical_density !== 'standard') return {lexical_density: configuration.lexical_density};
  if (configuration.output_standard === 'cnl-default') return configuration;
  return {output_standard: configuration.output_standard};
}

export function outputDirective(text, policy = cnlPolicy()) {
  if (typeof text !== 'string' || /^\s*[`"'«>]/u.test(text)) return null;
  const trimmed = text.trim();
  for (const language of policy.languages) {
    const [prefix, suffix] = language.frequency.split('{count}');
    const words = trimmed.replace(/[.!。！]+$/u, '');
    if (suffix && words.toLowerCase().startsWith(prefix.toLowerCase()) && words.toLowerCase().endsWith(suffix.toLowerCase())) {
      const density = 'top-' + words.slice(prefix.length, -suffix.length);
      if (frequencyTier(density) !== null) return {config: {lexical_density: density}, rest: '', paraphrase: true};
    }
    for (const item of language.switches) {
      if (trimmed.replace(/[.!。！]+$/u, '').toLowerCase() === item.text.toLowerCase()) {
        return {config: directiveConfiguration(item.mode, policy), rest: '', paraphrase: true};
      }
    }
    for (const cue of language.directives) {
      if (!trimmed.toLowerCase().startsWith(cue.toLowerCase() + ' ')) continue;
      const tail = trimmed.slice(cue.length + 1);
      const split = tail.search(/[.!;。！；\n]/u);
      const name = split < 0 ? tail : tail.slice(0, split);
      try {
        const configuration = directiveConfiguration(name.trim(), policy);
        return {config: configuration, rest: split < 0 ? '' : tail.slice(split + 1).trim(), paraphrase: false};
      } catch { /* A free-form sentence is not a malformed configuration. */ }
    }
  }
  return null;
}

/** Canonical Links Notation; the original surface remains in the document. */
export function outputLinksNotation(document) {
  const rows = ['output-meaning', '  language ' + JSON.stringify(document.language)];
  for (const [kind, value] of outputMeaning(document)) rows.push('  ' + kind + ' ' + JSON.stringify(value));
  return rows.join('\n') + '\n';
}

/** Stateless conversation selection: latest explicit user directive wins. */
export function outputTurn(prompt, history = [], configuration = {}, policy = cnlPolicy()) {
  let selected = outputConfiguration(configuration, policy);
  for (const turn of history) {
    if (turn.role !== 'user') continue;
    const directive = outputDirective(turn.content, policy);
    if (directive) selected = {...selected, ...directive.config};
  }
  const directive = outputDirective(prompt, policy);
  if (directive) selected = {...selected, ...directive.config};
  const previous = [...history].reverse().find(turn => turn.role === 'assistant');
  return {config: outputConfiguration(selected, policy), prompt: directive?.rest || prompt,
    rewrite: directive && !directive.rest ? previous?.content ?? null : null};
}

/** A notice is presentation metadata, never input to a subsequent paraphrase. */
export function outputSource(text, language, policy = cnlPolicy()) {
  const notice = policy.languages.find(item => item.id === language)?.notice;
  return notice && text.startsWith(notice + '\n\n') ? text.slice(notice.length + 2) : text;
}
