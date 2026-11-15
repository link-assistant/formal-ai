import { sourceLexicalRule } from './source_lexical_policy.mjs';
// Manual generic engine bootstrap; no autonomous artifact-authoring credit.
import { observedManifestFacts } from './source_manifest_facts.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { sourceDocumentationPolicy } from './source_documentation_policy.mjs';
const bytes = text => new TextEncoder().encode(text).length;
const gap = reason => ({
  gap: reason
});
const identifier = character => /[\p{L}\p{N}_$@/.-]/u.test(character ?? '');
function mentions(text, name) {
  if (!name) return false;
  let offset = text.indexOf(name);
  while (offset >= 0) {
    const before = [...text.slice(0, offset)].at(-1);
    const after = [...text.slice(offset + name.length)][0];
    if (!identifier(before) && !identifier(after)) return true;
    offset = text.indexOf(name, offset + name.length);
  }
  return false;
}
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    const ordered = Object.create(null);
    for (const key of Object.keys(value).sort()) ordered[key] = canonical(value[key]);
    return ordered;
  }
  return value;
}
function safe(text) {
  return typeof text === 'string' && bytes(text) <= 2048 && !/[`|\p{Cc}\p{Cf}]/u.test(text);
}
// One pass: placeholder-looking source text is never evaluated as a template.
function fill(template, bindings) {
  if (typeof template !== 'string') throw Error('MissingManifestDocumentationForms');
  return template.replace(/\{([a-z]+)\}/gu, (token, key) => {
    if (!Object.hasOwn(bindings, key)) throw Error('UnsupportedManifestDocumentationForm');
    return bindings[key];
  });
}
function ownedBlocks(guide, policy) {
  const formats = [policy.manifestStart, policy.manifestEnd].map(template => template.split('{source}'));
  if (formats.some(parts => parts.length !== 2)) throw Error('InvalidManifestMarkers');
  const blocks = [],
    owners = new Set();
  let pending = null,
    offset = 0;
  for (const raw of guide.split('\n')) {
    const line = raw.endsWith('\r') ? raw.slice(0, -1) : raw;
    for (let kind = 0; kind < formats.length; kind++) {
      const [prefix, suffix] = formats[kind];
      if (!line.includes(prefix)) continue;
      if (!line.startsWith(prefix) || !line.endsWith(suffix)) throw Error('InvalidManifestMarkers');
      const owner = line.slice(prefix.length, line.length - suffix.length);
      if (!/^[A-Za-z0-9_][A-Za-z0-9_./-]*$/u.test(owner) || sourceLexicalRule(owner, 'html-comment-forbidden') !== false || owner.split('/').some(part => !part || part === '.' || part === '..')) throw Error('InvalidManifestMarkers');
      if (kind === 0) {
        if (pending || owners.has(owner)) throw Error('InvalidManifestMarkers');
        pending = {
          owner,
          start: offset
        };
        owners.add(owner);
      } else {
        if (!pending || pending.owner !== owner) throw Error('InvalidManifestMarkers');
        blocks.push({
          ...pending,
          end: offset + line.length
        });
        pending = null;
      }
    }
    offset += raw.length + 1;
  }
  if (pending) throw Error('InvalidManifestMarkers');
  return blocks;
}
export function deriveManifestDocumentation(task, sourcePath, source, guide, forms = null) {
  if ([task, sourcePath, source, guide].some(value => typeof value !== 'string')) return gap('InvalidManifestDocumentationInput');
  if (bytes(source) + bytes(guide) > 262144 || bytes(task) > 262144) return gap('ManifestDocumentationByteLimit');
  const policy = forms ?? sourceDocumentationPolicy();
  if (!policy || ["manifestStart", "manifestEnd", "manifestTitle", "manifestNotice", "manifestHeader", "manifestSeparator", "manifestRow", "manifestAbsentVersion", "manifestUnqualifiedTarget"].some(key => typeof policy[key] !== 'string')) return gap('MissingManifestDocumentationForms');
  let declarations;
  try {
    declarations = observedManifestFacts(sourcePath, source).filter(fact => fact.kind === 'manifest-declaration');
  } catch {
    return gap('UnsupportedManifestDocumentationSource');
  }
  const named = declarations.filter(fact => mentions(task, fact.name) || typeof fact.options?.package === 'string' && mentions(task, fact.options.package));
  const selected = named.length ? named : declarations;
  if (!selected.length) return gap('MissingManifestDeclarations');
  if (selected.length > 32) return gap('ManifestDocumentationRowLimit');
  if (!safe(sourcePath) || sourceLexicalRule(sourcePath, 'html-comment-forbidden') !== false) return gap('UnsupportedManifestDocumentationValue');
  let start, end, paragraph;
  try {
    start = fill(policy.manifestStart, {
      source: sourcePath
    });
    end = fill(policy.manifestEnd, {
      source: sourcePath
    });
    const lines = [start, fill(policy.manifestTitle, {
      source: sourcePath
    }), policy.manifestNotice, policy.manifestHeader, policy.manifestSeparator];
    for (const fact of selected) {
      const row = {
        name: fact.name,
        version: fact.version ?? policy.manifestAbsentVersion,
        section: fact.section,
        target: fact.target ?? policy.manifestUnqualifiedTarget,
        options: JSON.stringify(canonical(fact.options ?? {}))
      };
      if (!row.name || !Object.values(row).every(safe)) return gap('UnsupportedManifestDocumentationValue');
      lines.push(fill(policy.manifestRow, row));
    }
    if (lines.some(line => typeof line !== 'string')) return gap('MissingManifestDocumentationForms');
    paragraph = [...lines, end].join('\n');
  } catch {
    return gap('UnsupportedManifestDocumentationValue');
  }
  if (!start || !end || start === end) return gap('MissingManifestDocumentationForms');
  let blocks;
  try {
    blocks = ownedBlocks(guide, policy);
  } catch {
    return gap('AmbiguousManifestDocumentationBlock');
  }
  const current = blocks.find(block => block.owner === sourcePath);
  let unowned = guide;
  for (const block of [...blocks].reverse()) unowned = unowned.slice(0, block.start) + unowned.slice(block.end);
  for (const line of unowned.split(/\r?\n/u)) {
    if (!line.includes('|')) continue;
    const cells = line.split('|').map(cell => cell.trim().replace(/^`|`$/gu, ''));
    const matching = selected.filter(fact => cells.includes(fact.name));
    if (matching.length && !matching.some(fact => cells.includes(fact.version ?? policy.manifestAbsentVersion) && cells.includes(fact.section))) return gap('ConflictingManifestDocumentation');
  }
  const content = !current ? guide + (guide.endsWith('\n\n') ? '' : guide.endsWith('\n') ? '\n' : '\n\n') + paragraph + '\n' : guide.slice(0, current.start) + paragraph + guide.slice(current.end);
  if (bytes(content) > 262144) return gap('ManifestDocumentationByteLimit');
  return {
    key: 'manifest-declarations',
    value: selected,
    facts: selected,
    paragraph,
    sourceIdentity: sha256Hex(source),
    content
  };
}
