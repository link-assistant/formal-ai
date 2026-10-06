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

/** Mirrors `fn escapes_root`. */
function escapesRoot(repoRelative) {
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
  for (const token of splitWhitespace(folded)) {
    for (const root of esRoots) {
      if (token.startsWith(`${root.directory}/`) && token.endsWith(`.${root.owned_extension}`)) {
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
