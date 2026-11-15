// Independent acceptance of observed documentation (issue #1483).
// Native twin: rust/src/agentic_coding/documentation_acceptance.rs.
//
// The derivation that wrote a documentation candidate never accepts it. These
// checks read only the task text, the raw observed sources and the readback
// artifact: a version the task claims must be the declared one, every derived
// fact must be anchored in its source and present in the artifact, and the
// guide's earlier lines must survive the edit. Requirements are verified one
// by one from the code spans they name; anything else stays unverified.

const CODE_SPAN = /`([^`\n]{1,200})`/gu;
const CLAIM = /^\s*"?([@A-Za-z0-9_./-]+)"?\s*[=:]\s*"([^"\n]+)"\s*$/u;
const WORD = /[\p{L}\p{N}']+/gu;
const URL = /https?:\/\/\S+/gu;
const COMPOUND = /[@A-Za-z0-9_][@A-Za-z0-9_./:-]*[A-Za-z0-9_]/gu;
const MAXIMUM_DIFF_LINES = 400;

/** Code spans of `text`, in order. */
export function codeSpans(text) {
  return [...String(text).matchAll(CODE_SPAN)].map(match => match[1].trim()).filter(span => span !== '');
}

/** Version claims written as `name = "version"` (or `"name": "version"`) in the task. */
export function documentationClaims(task) {
  const claims = [];
  for (const reference of codeSpans(task)) {
    const match = CLAIM.exec(reference);
    if (match && !claims.some(claim => claim.reference === reference)) claims.push({ reference, name: match[1], version: match[2] });
  }
  return claims;
}

const declarations = facts => (Array.isArray(facts) ? facts : []).filter(fact => fact?.kind === 'manifest-declaration' && typeof fact.name === 'string');

/**
 * Each claim against the declarations: `confirmed` when a declaration has the
 * name and the version, `contradicted` when declarations of the name exist
 * but none has the version, `unobserved` when no declaration has the name.
 */
export function judgedClaims(claims, facts) {
  return claims.map(claim => {
    const named = declarations(facts).filter(fact => fact.name === claim.name);
    const observed = [...new Set(named.map(fact => fact.version).filter(version => typeof version === 'string'))];
    const status = !named.length ? 'unobserved' : observed.includes(claim.version) ? 'confirmed' : 'contradicted';
    return { ...claim, status, observed };
  });
}

const cells = line => line.trim().startsWith('|') ? line.split('|').map(cell => cell.trim().replace(/^`+|`+$/gu, '').trim()) : [];

/**
 * Every derived fact is anchored in its raw source and appears in the
 * artifact. A fact of a kind this check cannot read makes it unavailable.
 */
export function sourceAnchoredFacts(facts, sources, artifact) {
  const sourceOf = path => sources.find(row => row.path === path)?.source ?? null;
  const rows = String(artifact).split('\n').map(cells).filter(row => row.length);
  const missing = [];
  let unavailable = false;
  for (const fact of Array.isArray(facts) ? facts : []) {
    if (fact?.kind === 'manifest-declaration') {
      // A declaration without a literal version (a path or workspace
      // dependency) is anchored by its name; its row still needs the section.
      const source = sourceOf(fact.manifest);
      const anchors = [fact.name, fact.version].filter(value => typeof value === 'string');
      const values = [...anchors, fact.section];
      if (source === null || !anchors.every(value => source.includes(value))) missing.push({ kind: fact.kind, path: fact.manifest, name: fact.name, place: 'source' });
      else if (!rows.some(row => values.every(value => row.includes(value)))) missing.push({ kind: fact.kind, path: fact.manifest, name: fact.name, place: 'artifact' });
    } else if (fact?.kind === 'observed-source-usage') {
      const source = sourceOf(fact.owner);
      for (const call of Array.isArray(fact.calls) ? fact.calls : []) {
        if (source === null || typeof call?.source !== 'string' || !source.includes(call.source)) missing.push({ kind: fact.kind, path: fact.owner, name: call?.imported ?? null, place: 'source' });
        else if (!String(artifact).includes(call.source)) missing.push({ kind: fact.kind, path: fact.owner, name: call.imported, place: 'artifact' });
      }
    } else {
      unavailable = true;
    }
  }
  const checked = (Array.isArray(facts) ? facts : []).length;
  return { kind: 'source-anchored-facts', status: missing.length ? 'failed' : unavailable || !checked ? 'unavailable' : 'passed', checked, missing };
}

const OWNED_START = /^<!-- formal-ai-([a-z-]+)-start: (\S+) -->$/u;

/**
 * Non-blank preimage lines in order inside the postimage; the rest are the
 * added lines. Lines of a block the guide marks as generated
 * (`<!-- formal-ai-…-start: … -->` to its end marker) are owned by the
 * derivation and may be regenerated.
 */
export function preservedPreimage(preimage, postimage) {
  const before = [];
  let owner = null,
    owned = 0;
  for (const line of String(preimage).split('\n')) {
    const start = owner === null ? OWNED_START.exec(line) : null;
    if (start) owner = '<!-- formal-ai-' + start[1] + '-end: ' + start[2] + ' -->';
    if (owner !== null) {
      owned += 1;
      if (line === owner) owner = null;
    } else if (line.trim() !== '') before.push(line);
  }
  const added = [];
  let next = 0;
  for (const line of String(postimage).split('\n')) {
    if (next < before.length && line === before[next]) next += 1;
    else if (line.trim() !== '') added.push(line);
  }
  return { check: { kind: 'preimage-preserved', status: next === before.length && owner === null ? 'passed' : 'failed', kept: next, expected: before.length, owned }, added };
}

/** The bounded added-line diff of a candidate edit. */
export function addedLinesDiff(added) {
  const shown = added.slice(0, MAXIMUM_DIFF_LINES).map(line => '+' + line);
  return { added: added.length, lines: shown, truncated: added.length > shown.length };
}

/**
 * The terms a requirement names: its code spans, or, when it has none, its
 * compound tokens (`ribbon-core`, `docs/guide.md`, `Path::item`) outside links.
 */
export function requirementTerms(text) {
  const spans = codeSpans(text).map(span => span.replace(/\(\)$/u, ''));
  if (spans.length) return spans;
  const terms = [];
  for (const match of String(text).replace(URL, ' ').matchAll(COMPOUND)) {
    const token = match[0];
    if (/[A-Za-z]/u.test(token) && /[-_/.]|::/u.test(token) && !terms.includes(token)) terms.push(token);
  }
  return terms;
}

/** Terms a passed fact check establishes: names, owners, manifests, modules and imported bindings. */
function establishedTerms(facts, claims, target) {
  const terms = new Set(claims.filter(claim => claim.status === 'confirmed').map(claim => claim.reference));
  if (typeof target === 'string') terms.add(target);
  for (const fact of Array.isArray(facts) ? facts : []) {
    for (const value of [fact?.name, fact?.manifest, fact?.owner]) if (typeof value === 'string') terms.add(value);
    for (const entry of Array.isArray(fact?.imports) ? fact.imports : []) {
      if (typeof entry?.module === 'string') terms.add(entry.module);
      for (const binding of Array.isArray(entry?.bindings) ? entry.bindings : []) {
        for (const value of [binding?.imported, binding?.local]) if (typeof value === 'string') terms.add(value);
      }
    }
  }
  return terms;
}

/** Whether `text` asks to keep existing content: a preserve cue not directly negated. */
export function preserveCue(text, preserve, negations) {
  const words = (String(text).toLowerCase().match(WORD) ?? []);
  return words.some((word, index) => preserve.includes(word) && !negations.includes(words[index - 1] ?? ''));
}

/**
 * The status of each requirement. A requirement is verified when it names at
 * least one term and every term is established by the passed fact check (a
 * fact's name, manifest, owner, module or binding, a confirmed claim, or the
 * accepted target), or names none and asks to keep existing content; a
 * preserve cue always needs the preservation check. A requirement that
 * names a contradicted claim is contradicted; any other stays unverified.
 */
export function judgedRequirements(requirements, { facts, claims, target, factsPassed, preservation, preserve, negations }) {
  const terms = factsPassed ? establishedTerms(facts, claims, target) : new Set();
  const contradicted = new Set(claims.filter(claim => claim.status === 'contradicted').map(claim => claim.reference));
  return requirements.map(requirement => {
    const spans = requirementTerms(requirement.text);
    if (spans.some(span => contradicted.has(span))) return { ...requirement, status: 'contradicted' };
    const cue = preserveCue(requirement.text, preserve, negations);
    const established = spans.length ? spans.every(span => terms.has(span)) : cue;
    const kept = !cue || preservation === 'passed';
    return established && kept ? { ...requirement, status: 'verified', evidence: spans.length ? spans : ['preimage-preserved'] } : requirement;
  });
}
