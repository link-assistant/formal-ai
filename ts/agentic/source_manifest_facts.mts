import { sourceLexicalRule } from './source_lexical_policy.mjs';
// Finite, source-only manifest facts. No solver prose, filesystem access or
// package-name lookup can establish a declaration or a runtime integration.
import { sha256Hex } from './crate/source_fetch.mjs';
const scalarText = text => {
  if (/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/u.test(text)) throw Error('ManifestUnicodeScalarRequired');
  return text;
};
const fail = reason => {
  throw Error(reason);
};
const packageSections = new Set(['dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies']);
const cargoSections = new Set(['dependencies', 'dev-dependencies', 'build-dependencies']);
const cargoOptions = new Set(['version', 'package', 'path', 'git', 'branch', 'tag', 'rev', 'registry', 'default-features', 'optional', 'features', 'workspace']);
const own = (value, key) => Object.hasOwn(value, key);
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);

/** JSON.parse accepts duplicate keys. A separate complete token walk rejects
 * those before any facts are taken from the parsed object. */
function checkedJson(source) {
  let cursor = 0;
  const space = () => {
    while (/\s/u.test(source[cursor] ?? '') && cursor < source.length) cursor++;
  };
  const string = () => {
    const start = cursor;
    if (source[cursor++] !== '"') fail('ManifestStringRequired');
    while (cursor < source.length) {
      const character = source[cursor++];
      if (character === '"') return scalarText(JSON.parse(source.slice(start, cursor)));
      if (character === '\\') cursor++;
    }
    fail('ManifestUnterminatedString');
  };
  const value = depth => {
    if (depth > 64) fail('ManifestDepthLimit');
    space();
    if (source[cursor] === '{') {
      cursor++;
      space();
      const keys = new Set();
      if (source[cursor] === '}') {
        cursor++;
        return;
      }
      for (;;) {
        space();
        const key = string();
        if (keys.has(key)) fail('ManifestDuplicateKey');
        keys.add(key);
        space();
        if (source[cursor++] !== ':') fail('ManifestColonRequired');
        value(depth + 1);
        space();
        const separator = source[cursor++];
        if (separator === '}') return;
        if (separator !== ',') fail('ManifestSeparatorRequired');
      }
    }
    if (source[cursor] === '[') {
      cursor++;
      space();
      if (source[cursor] === ']') {
        cursor++;
        return;
      }
      for (;;) {
        value(depth + 1);
        space();
        const separator = source[cursor++];
        if (separator === ']') return;
        if (separator !== ',') fail('ManifestSeparatorRequired');
      }
    }
    if (source[cursor] === '"') {
      string();
      return;
    }
    const scalar = /^(?:true|false|null|-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)/u.exec(source.slice(cursor));
    if (!scalar) fail('ManifestScalarRequired');
    if(!['true','false','null'].includes(scalar[0])){
      const number=Number(scalar[0]);
      if(!Number.isFinite(number)||Number.isInteger(number)&&!Number.isSafeInteger(number))fail('ManifestNumberRangeUnsupported');
    }
    cursor += scalar[0].length;
  };
  value(0);
  space();
  if (cursor !== source.length) fail('ManifestTrailingContent');
  return JSON.parse(source);
}

/** Parse complete observed JSON with duplicate/depth/range checks. A source
 * value is evidence only; this function grants no edit or truth authority. */
export function observedSourceJson(source) {
  if(typeof source!=='string'||new TextEncoder().encode(source).length>262144)fail('ManifestSourceBudgetExceeded');
  return checkedJson(source);
}

/** This is a declared TOML subset, not a regex search for version-looking
 * text. It consumes every token, including unrelated tables, before emitting
 * facts. Unsupported syntax is a gap; comments never supply declarations. */
function tomlDeclarations(source) {
  let cursor = 0,
    section = [],
    arrayTable = false;
  const rows = [],
    identities = new Map(),
    tables = new Map();
  const prefix = (left, right) => left.length <= right.length && left.every((part, index) => part === right[index]);
  const selected = parts => cargoSections.has(parts[0]) || parts[0] === 'features' || parts[0] === 'target' && cargoSections.has(parts[2]);
  const horizontal = () => {
    while (source[cursor] === ' ' || source[cursor] === '\t' || source[cursor] === '\r') cursor++;
  };
  const comment = () => {
    if (source[cursor] === '#') while (cursor < source.length && source[cursor] !== '\n') cursor++;
  };
  const layout = () => {
    for (;;) {
      horizontal();
      comment();
      if (source[cursor] !== '\n') return;
      cursor++;
    }
  };
  const string = () => {
    const quote = source[cursor++],
      start = cursor;
    if (sourceLexicalRule(source.slice(cursor - 1, cursor + 2), 'toml-multiline-opening') !== false) fail('ManifestMultilineStringUnsupported');
    while (cursor < source.length) {
      const character = source[cursor++];
      if (character === quote) {
        const raw = source.slice(start, cursor - 1);
        if (quote === "'") return scalarText(raw);
        try {
          return scalarText(JSON.parse('"' + raw + '"'));
        } catch {
          fail('ManifestStringEscapeUnsupported');
        }
      }
      if (character === '\n' || character === '\r' || character < ' ') fail('ManifestStringControlUnsupported');
      if (character === '\\' && quote === '"') {
        if (!/["\\bfnrtu]/u.test(source[cursor] ?? '')) fail('ManifestStringEscapeUnsupported');
        cursor++;
      }
    }
    fail('ManifestUnterminatedString');
  };
  const key = () => {
    horizontal();
    const parts = [];
    for (;;) {
      if (source[cursor] === '"' || source[cursor] === "'") parts.push(string());else {
        const match = /^[A-Za-z0-9_-]+/u.exec(source.slice(cursor));
        if (!match) fail('ManifestKeyRequired');
        parts.push(match[0]);
        cursor += match[0].length;
      }
      horizontal();
      if (source[cursor] !== '.') return parts;
      cursor++;
      horizontal();
    }
  };
  const value = depth => {
    if (depth > 64) fail('ManifestDepthLimit');
    horizontal();
    const character = source[cursor];
    if (character === '"' || character === "'") return string();
    if (character === '[') {
      cursor++;
      layout();
      const result = [];
      if (source[cursor] === ']') {
        cursor++;
        return result;
      }
      for (;;) {
        result.push(value(depth + 1));
        layout();
        if (source[cursor] === ']') {
          cursor++;
          return result;
        }
        if (source[cursor++] !== ',') fail('ManifestArraySeparatorRequired');
        layout();
        if (source[cursor] === ']') {
          cursor++;
          return result;
        }
      }
    }
    if (character === '{') {
      cursor++;
      horizontal();
      const result = Object.create(null);
      if (source[cursor] === '}') {
        cursor++;
        return result;
      }
      for (;;) {
        const parts = key();
        if (parts.length !== 1) fail('ManifestInlineDottedKeyUnsupported');
        if (own(result, parts[0])) fail('ManifestDuplicateKey');
        if (source[cursor++] !== '=') fail('ManifestAssignmentRequired');
        result[parts[0]] = value(depth + 1);
        horizontal();
        if (source[cursor] === '}') {
          cursor++;
          return result;
        }
        if (source[cursor++] !== ',') fail('ManifestInlineSeparatorRequired');
        horizontal();
        if (source[cursor] === '}') fail('ManifestInlineTrailingComma');
      }
    }
    const scalar = /^(?:true|false|[+-]?(?:0|[1-9](?:_?[0-9])*)(?:\.[0-9](?:_?[0-9])*)?(?:[eE][+-]?[0-9](?:_?[0-9])*)?)(?=$|[\s,#\]}])/u.exec(source.slice(cursor));
    if (!scalar) fail('ManifestScalarUnsupported');
    cursor += scalar[0].length;
    if (scalar[0] === 'true') return true;
    if (scalar[0] === 'false') return false;
    const number = Number(scalar[0].replaceAll('_', ''));
    if (!Number.isFinite(number)) fail('ManifestFiniteNumberRequired');
    return number;
  };
  while (cursor < source.length) {
    if (rows.length >= 1024) fail('ManifestDeclarationBudgetExceeded');
    layout();
    if (cursor === source.length) break;
    if (source[cursor] === '[') {
      cursor++;
      arrayTable = source[cursor] === '[';
      if (arrayTable) cursor++;
      section = key();
      if (source[cursor++] !== ']' || arrayTable && source[cursor++] !== ']') fail('ManifestTableTerminatorRequired');
      const identity = JSON.stringify(section);
      if (arrayTable && selected(section)) fail('ManifestDependencyTableUnsupported');
      if (!arrayTable) {
        if (tables.has(identity)) fail('ManifestDuplicateTable');
        if ([...identities.values()].some(parts => prefix(parts, section))) fail('ManifestTableKeyConflict');
        tables.set(identity, [...section]);
      }
      rows.push({
        header: true,
        section: [...section],
        arrayTable
      });
    } else {
      const parts = key();
      if (source[cursor++] !== '=') fail('ManifestAssignmentRequired');
      const parsed = value(0),
        identity = JSON.stringify([...section, ...parts]);
      if (!arrayTable) {
        const full = [...section, ...parts];
        if ([...identities.values()].some(previous => prefix(previous, full) || prefix(full, previous)) || [...tables.values()].some(table => prefix(full, table))) fail('ManifestDuplicateKey');
        identities.set(identity, full);
      }
      rows.push({
        section: [...section],
        key: parts,
        value: parsed,
        arrayTable
      });
    }
    horizontal();
    comment();
    if (cursor < source.length && source[cursor] !== '\n') fail('ManifestTrailingContent');
    if (source[cursor] === '\n') cursor++;
  }
  return rows;
}
function dependency(value) {
  if (typeof value === 'string') return {
    version: value
  };
  if (!object(value)) fail('ManifestDependencyLiteralRequired');
  for (const [key, item] of Object.entries(value)) {
    if (!cargoOptions.has(key)) fail('ManifestDependencyOptionUnsupported');
    const valid = ['default-features', 'optional', 'workspace'].includes(key) ? typeof item === 'boolean' : key === 'features' ? Array.isArray(item) && item.every(entry => typeof entry === 'string') : typeof item === 'string';
    if (!valid) fail('ManifestDependencyOptionType');
  }
  const options = Object.fromEntries(Object.entries(value).filter(([key]) => key !== 'version'));
  if (!own(value, 'version') && !['path', 'git'].some(key => own(value, key)) && value.workspace !== true) fail('ManifestDependencyOriginRequired');
  return {
    version: value.version ?? null,
    ...(Object.keys(options).length ? {
      options
    } : {})
  };
}
export function observedManifestFacts(owner, source) {
  if (typeof source !== 'string' || new TextEncoder().encode(source).length > 262144) fail('ManifestSourceBudgetExceeded');
  if (typeof owner !== 'string' || !/^[A-Za-z0-9_][A-Za-z0-9_./-]*$/u.test(owner) || owner.split('/').some(part => !part || part === '.' || part === '..')) fail('ManifestOwnerRequired');
  const base = {
      manifest: owner,
      sourceIdentity: sha256Hex(source)
    },
    facts = [];
  if (owner.endsWith('.json')) {
    const document = observedSourceJson(source);
    if (!object(document)) fail('ManifestObjectRequired');
    for (const section of Object.keys(document).filter(key => packageSections.has(key)).sort()) {
      if (!object(document[section])) fail('ManifestDependencySectionRequired');
      for (const name of Object.keys(document[section]).sort()) {
        const version = document[section][name];
        if (typeof version !== 'string') fail('ManifestDependencyLiteralRequired');
        facts.push({
          kind: 'manifest-declaration',
          ...base,
          section,
          name,
          version
        });
      }
    }
    return sortedFacts(facts);
  }
  if (!owner.endsWith('.toml')) fail('ManifestLanguageUnsupported');
  const declarations = tomlDeclarations(source),
    groups = new Map();
  for (const row of declarations) {
    if (row.header) {
      const section = row.section,
        category = section[0] === 'target' ? section[2] : section[0],
        offset = section[0] === 'target' ? 3 : 1;
      if (cargoSections.has(category)) {
        if (row.arrayTable || section.length > offset + 1) fail('ManifestDependencyTableUnsupported');
        if (section.length === offset + 1) groups.set(JSON.stringify(section), {
          section: category,
          target: section[0] === 'target' ? section[1] : null,
          name: section[offset],
          value: Object.create(null)
        });
      }
      if (section[0] === 'features' && section.length !== 1) fail('ManifestFeatureArrayRequired');
      continue;
    }
    const section = row.section,
      category = section[0] === 'target' && section.length >= 3 ? section[2] : section[0];
    if (section.length === 0 && (cargoSections.has(row.key[0]) || ['features', 'target'].includes(row.key[0]))) fail('ManifestDependencySectionRequired');
    if (section.length === 1 && section[0] === 'features') {
      if (row.arrayTable || row.key.length !== 1 || !Array.isArray(row.value) || row.value.some(item => typeof item !== 'string')) fail('ManifestFeatureArrayRequired');
      facts.push({
        kind: 'cargo-feature',
        ...base,
        section: 'features',
        name: row.key[0],
        enables: row.value
      });
      continue;
    }
    if (!cargoSections.has(category)) continue;
    const target = section[0] === 'target' ? section[1] : null,
      offset = target === null ? 1 : 3;
    if (row.arrayTable || section.length > offset + 1) fail('ManifestDependencyTableUnsupported');
    if (section.length === offset) {
      if (row.key.length !== 1) fail('ManifestDependencyDottedKeyUnsupported');
      facts.push({
        kind: 'manifest-declaration',
        ...base,
        section: category,
        ...(target !== null ? {
          target
        } : {}),
        name: row.key[0],
        ...dependency(row.value)
      });
    } else {
      if (row.key.length !== 1) fail('ManifestDependencyDottedKeyUnsupported');
      const identity = JSON.stringify(section),
        group = groups.get(identity) ?? {
          section: category,
          target,
          name: section[offset],
          value: Object.create(null)
        };
      if (own(group.value, row.key[0])) fail('ManifestDuplicateKey');
      group.value[row.key[0]] = row.value;
      groups.set(identity, group);
    }
  }
  for (const group of groups.values()) facts.push({
    kind: 'manifest-declaration',
    ...base,
    section: group.section,
    ...(group.target !== null ? {
      target: group.target
    } : {}),
    name: group.name,
    ...dependency(group.value)
  });
  const identities = new Set();
  for (const fact of facts) {
    const identity = JSON.stringify([fact.kind, fact.section, fact.target ?? null, fact.name]);
    if (identities.has(identity)) fail('ManifestDuplicateDeclaration');
    identities.add(identity);
  }
  return sortedFacts(facts);
}
function sortedFacts(facts) {
  if (facts.length > 1024) fail('ManifestFactBudgetExceeded');
  const bytes = value => new TextEncoder().encode(JSON.stringify([value.kind, value.section, value.target ?? '', value.name]));
  const compare = (left, right) => {
    const a = bytes(left),
      b = bytes(right);
    for (let index = 0; index < Math.min(a.length, b.length); index++) if (a[index] !== b[index]) return a[index] - b[index];
    return a.length - b.length;
  };
  return facts.sort(compare);
}

/** Candidate Cargo lib/bin search scopes from complete observed TOML only.
 * Convention does not establish any file, target, import or runtime use. */
export function observedCargoSourceDirectories(owner, source) {
  const manifest = typeof owner === 'string' ? owner : null;
  let sourceIdentity = null;
  const result = (directories, facts, gaps) => ({
    schema: 'observed-cargo-source-directories/v1', manifest, sourceIdentity,
    directories, facts, gaps, complete: gaps.length === 0,
    scope: 'cargo-lib-bin-candidate-search-scopes-only; no existence/target/runtime claim'
  });
  const gap = reason => result([], [], [{ reason }]);
  if (typeof source !== 'string' || new TextEncoder().encode(source).length > 262144) return gap('CargoSourceBudgetExceeded');
  if (typeof owner !== 'string' || !/^[A-Za-z0-9_][A-Za-z0-9_./-]*$/u.test(owner)
      || owner.split('/').some(part => !part || part === '.' || part === '..' || part.startsWith('.'))
      || owner.split('/').at(-1) !== 'Cargo.toml') return gap('CargoSourceOwnerRequired');
  sourceIdentity = sha256Hex(source);
  let rows;
  try { rows = tomlDeclarations(source); } catch { return gap('CargoSourceLayoutUnsupported'); }
  const parent = owner.split('/').slice(0, -1), directories = new Set(), facts = [], identities = new Set();
  const add = (directory, basis, sourcePath = null) => {
    if (directory.split('/').length <= parent.length || !directory.startsWith(parent.length ? parent.join('/') + '/' : '')) fail('CargoSourcePathUnsupported');
    directories.add(directory);
    if (directories.size > 8) fail('CargoSourceDirectoryBudgetExceeded');
    const identity = JSON.stringify([directory, basis, sourcePath]);
    if (identities.has(identity)) return;
    identities.add(identity);
    facts.push({ kind: 'cargo-source-directory-candidate', manifest, sourceIdentity,
      directory, basis, ...(sourcePath === null ? {} : { sourcePath }) });
  };
  const explicit = (path, section) => {
    if (typeof path !== 'string' || !/^[A-Za-z0-9_][A-Za-z0-9_./-]*\.rs$/u.test(path)
        || path.split('/').some(part => !part || part === '.' || part === '..' || part.startsWith('.'))
        || path.split('/').length < 2) fail('CargoSourcePathUnsupported');
    add([...parent, ...path.split('/').slice(0, -1)].join('/'), 'explicit-' + section + '-path', path);
  };
  try {
    add([...parent, 'src'].join('/'), 'conventional-src');
    let selected = null;
    for (const row of rows) {
      if (row.header) {
        selected = null;
        if (!['lib', 'bin'].includes(row.section[0])) continue;
        if (row.section.length !== 1 || row.arrayTable !== (row.section[0] === 'bin')) fail('CargoSourceLayoutUnsupported');
        selected = { section: row.section[0], keys: new Set() };
        continue;
      }
      if (row.section.length === 0 && ['lib', 'bin'].includes(row.key[0])) fail('CargoSourceLayoutUnsupported');
      if (!['lib', 'bin'].includes(row.section[0])) continue;
      if (!selected || row.section.length !== 1 || row.section[0] !== selected.section || row.key.length !== 1
          || selected.keys.has(row.key[0])) fail('CargoSourceLayoutUnsupported');
      selected.keys.add(row.key[0]);
      if (row.key[0] === 'path') explicit(row.value, selected.section);
    }
  } catch (error) {
    return gap(['CargoSourcePathUnsupported', 'CargoSourceDirectoryBudgetExceeded'].includes(error.message)
      ? error.message : 'CargoSourceLayoutUnsupported');
  }
  facts.sort((left, right) => {
    const first = JSON.stringify([left.directory, left.basis, left.sourcePath ?? '']);
    const second = JSON.stringify([right.directory, right.basis, right.sourcePath ?? '']);
    return first < second ? -1 : first > second ? 1 : 0;
  });
  return result([...directories].sort(), facts, []);
}
