// The source-tree translation request recognizer (rust/src/meta_translate.rs
// `SourceRoot`, `SourceTreeRequest`, `source_tree_request`). The translation
// engine itself (`translate`, the CST legs) is native-only and not ported:
// the planner only needs the structural gate and the request's roots/path.

/** Mirrors `enum SourceRoot`: `{name, directory, owned_extension}` per root. */
export const SourceRoot = Object.freeze({
  Rust: Object.freeze({ name: 'rust', directory: 'rust', owned_extension: null }),
  JavaScript: Object.freeze({ name: 'js', directory: 'js', owned_extension: 'js' }),
  TypeScript: Object.freeze({ name: 'ts', directory: 'ts', owned_extension: 'ts' }),
  Meta: Object.freeze({ name: 'meta', directory: 'meta', owned_extension: null }),
});

/**
 * Mirrors `SourceRoot::parse` in rust/src/meta_translate.rs.
 * @param {string} name
 */
export function parseSourceRoot(name) {
  switch (name.trim().replace(/[A-Z]/g, (letter) => letter.toLowerCase())) {
    case 'rust': case 'rs': return SourceRoot.Rust;
    case 'js': case 'javascript': return SourceRoot.JavaScript;
    case 'ts': case 'typescript': return SourceRoot.TypeScript;
    case 'meta': case 'lino': return SourceRoot.Meta;
    default: return null;
  }
}

/** Mirrors `fn escapes_root`. @param {string} repoRelative */
export function escapesRoot(repoRelative) {
  return repoRelative.split('/').some((segment) => segment === '..');
}

const asciiLower = (text) => text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
const splitWhitespace = (text) => text.split(/\p{White_Space}+/u).filter(Boolean);

/**
 * Mirrors `fn source_tree_request` in rust/src/meta_translate.rs:
 * `{from, to, path}` (roots are `SourceRoot` values) or null.
 * @param {string} prompt
 */
export function sourceTreeRequest(prompt) {
  const folded = asciiLower(prompt);
  const esRoots = [SourceRoot.JavaScript, SourceRoot.TypeScript];
  let pathToken = null;
  for (const token of splitWhitespace(prompt)) {
    const foldedToken = asciiLower(token);
    for (const root of esRoots) {
      if (foldedToken.startsWith(`${root.directory}/`) && foldedToken.endsWith(`.${root.owned_extension}`)) {
        pathToken = [root, token];
        break;
      }
    }
    if (pathToken) break;
  }
  if (!pathToken) return null;
  const [from, token] = pathToken;
  const keep = (character) => /^[0-9A-Za-z]$/.test(character) || character === '/' || character === '.';
  const chars = Array.from(token);
  let start = 0;
  let end = chars.length;
  while (start < end && !keep(chars[start])) start += 1;
  while (end > start && !keep(chars[end - 1])) end -= 1;
  const path = chars.slice(start, end).join('');
  if (escapesRoot(path)) return null;
  const words = splitWhitespace(folded);
  const toAlias = ['typescript', 'javascript'].find((alias) =>
    words.some((word, index) => index + 1 < words.length && word === 'to' && words[index + 1] === alias));
  if (toAlias === undefined) return null;
  const to = parseSourceRoot(toAlias);
  if (from === to) return null;
  return { from, to, path };
}

import { childrenNamed, parseLino, readText } from '../host.mjs';
import { wordsForRole } from './seed_meanings.mjs';

/** Mirrors `fn owned_source_tree_request` in rust/src/meta_translate.rs. */
export function ownedSourceTreeRequest(prompt) {
  if (/[\uD800-\uDFFF]/u.test(prompt)) return null;
  if (Array.from(prompt).some(character => /[\p{White_Space}\uFEFF]/u.test(character) && !/[ \t\n\r\v\f]/u.test(character))) return null;
  const root = parseLino(readText('data/seed/meanings-translate-cycle.lino'));
  const contract = childrenNamed(root, 'source-tree-translation-contract')[0];
  if (!contract) return null;
  for (const form of childrenNamed(contract, 'form')) {
    const pattern = childrenNamed(form, 'pattern')[0]?.value;
    const writes = childrenNamed(form, 'writes')[0]?.value;
    if (!pattern || !['true', 'false'].includes(writes)) continue;
    let missing = false;
    const expression = pattern.replace(/\{([a-z]+(?:-[a-z]+)*)\}/gu, (_, role) => {
      if (role === 'source' || role === 'target') return `(?<${role}>[^ \t\n\r\v\f]+)`;
      const surfaces = wordsForRole(role.replaceAll('-', '_'));
      if (surfaces.length === 0) { missing = true; return ''; }
      return '(?:'+surfaces.map(surface => surface.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|')+')';
    });
    if (missing) continue;
    const match = new RegExp(expression, 'diu').exec(prompt);
    if (!match || match[0].length !== prompt.length) continue;
    const path = match.groups.source;
    const from = [SourceRoot.JavaScript, SourceRoot.TypeScript].find(root =>
      path.startsWith(root.directory+'/') && path.endsWith('.'+root.owned_extension));
    const to = parseSourceRoot(match.groups.target);
    if (!from || ![SourceRoot.JavaScript,SourceRoot.TypeScript].includes(to) || from === to
      || escapesRoot(path) || /["'`\\\x00-\x20]/u.test(path)) continue;
    const bytes = new TextEncoder();
    const [start,end] = match.indices.groups.source;
    return { request:{from,to,path}, write:writes === 'true', source_unit:'utf8',
      source_span:[bytes.encode(prompt.slice(0,start)).length,bytes.encode(prompt.slice(0,end)).length],
      request_span:[0,bytes.encode(prompt).length] };
  }
  return null;
}
