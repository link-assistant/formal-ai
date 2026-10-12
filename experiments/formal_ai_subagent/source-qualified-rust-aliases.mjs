import { posix } from 'node:path';
import { lex } from '../../scripts/self-translation/lexer.mjs';

// Structural source projection only. The caller must physically guard every
// selected source; this module never authorizes filesystem effects.
function tokens(source) {
  if (typeof source !== 'string') throw new Error('Unsupported Rust lexical shape');
  const quote = String.fromCodePoint(34);
  const apostrophe = String.fromCodePoint(39);
  const escape = String.fromCodePoint(92);
  const newline = String.fromCodePoint(10);
  const characters = [];
  let index = 0;
  while (index < source.length) {
    if (source.startsWith('//', index)) {
      const end = source.indexOf(newline, index + 2);
      index = end < 0 ? source.length : end + 1;
      continue;
    }
    if (source.startsWith('/*', index)) {
      const end = source.indexOf('*/', index + 2);
      if (end < 0 || source.slice(index + 2, end).includes('/*')) {
        throw new Error('Unsupported Rust lexical shape');
      }
      index = end + 2;
      continue;
    }
    if (source[index] === quote) {
      let end = index + 1;
      while (end < source.length && source[end] !== quote) {
        end += source[end] === escape ? 2 : 1;
      }
      if (end >= source.length) throw new Error('Unsupported Rust lexical shape');
      index = end + 1;
      continue;
    }
    if (source[index] === apostrophe) {
      let end = index + 1;
      if (source[end] === escape) {
        if (source[end + 1] === 'u' && source[end + 2] === '{') {
          const close = source.indexOf('}', end + 3);
          const digits = close < 0 ? '' : source.slice(end + 3, close);
          if (!/^[0-9a-fA-F](?:_?[0-9a-fA-F]){0,5}$/u.test(digits)) {
            throw new Error('Unsupported Rust lexical shape');
          }
          const scalar = Number.parseInt(digits.replaceAll('_', ''), 16);
          if (scalar > 0x10ffff || (scalar >= 0xd800 && scalar <= 0xdfff)) {
            throw new Error('Unsupported Rust lexical shape');
          }
          end = close + 1;
        } else if (source[end + 1] === 'x') {
          if (!/^[0-7][0-9a-fA-F]$/u.test(source.slice(end + 2, end + 4))) {
            throw new Error('Unsupported Rust lexical shape');
          }
          end += 4;
        } else if (['n', 'r', 't', '0', quote, apostrophe, escape].includes(source[end + 1])) {
          end += 2;
        } else {
          throw new Error('Unsupported Rust lexical shape');
        }
      } else {
        const scalar = source.codePointAt(end);
        if (scalar !== undefined && scalar !== 10 && scalar !== 13 &&
            !(scalar >= 0xd800 && scalar <= 0xdfff)) end += scalar > 0xffff ? 2 : 1;
      }
      if (source[end] === apostrophe) {
        characters.push({ start: index, end: end + 1 });
        index = end + 1;
        continue;
      }
    }
    const previous = index === 0 ? '' : source[index - 1];
    if ((index === 0 || !(/[A-Za-z0-9_$]/u.test(previous) || previous.charCodeAt(0) >= 128)) &&
        /^(?:r|br|cr)(?=#|")/u.test(source.slice(index))) {
      throw new Error('Unsupported Rust lexical shape');
    }
    index += 1;
  }
  let surface = source;
  for (const span of characters.toReversed()) {
    const opaque = quote + ' '.repeat(span.end - span.start - 2) + quote;
    surface = surface.slice(0, span.start) + opaque + surface.slice(span.end);
  }
  const all = lex(surface, 'Rust');
  return all.filter(token => token.type !== 'comment').map(token => ({
    ...token,
    type: characters.some(span => span.start === token.start && span.end === token.end)
      ? 'char' : token.type,
    text: source.slice(token.start, token.end),
  }));
}
function statements(source) {
  const result = [];
  let depth = 0;
  let start = 0;
  const all = tokens(source);
  for (let index = 0; index < all.length; index += 1) {
    const text = all[index].text;
    if (depth === 0 && all[index].type === 'ident' && text.endsWith('!')) {
      throw new Error('Unsupported top-level macro');
    }
    if (text === '{') depth += 1;
    if (text === '}') {
      depth -= 1;
      if (depth < 0) throw new Error('Unbalanced source');
      if (depth === 0 && all[start]?.text !== 'use') start = index + 1;
    }
    if (text === ';' && depth === 0) {
      result.push(all.slice(start, index + 1));
      start = index + 1;
    }
  }
  if (depth !== 0) throw new Error('Unbalanced source');
  return result;
}
function moduleDeclarations(source, selectedName) {
  const found = new Map();
  for (const statement of statements(source)) {
    const words = statement.map(token => token.text);
    const at = words.indexOf('mod');
    if (at < 0 || words[at + 1] !== selectedName) continue;
    const prefix = words.slice(0, at).join(' ');
    if (!['', 'pub', 'pub ( crate )'].includes(prefix) ||
        statement[at + 1]?.type !== 'ident' ||
        words.length !== at + 3 || words[at + 2] !== ';') {
      throw new Error('Unsupported module declaration');
    }
    const name = words[at + 1];
    if (found.has(name)) throw new Error('Duplicate module binding');
    found.set(name, prefix === 'pub');
  }
  return found;
}
function libraryName(source) {
  if (typeof source !== 'string') throw new Error('Missing library namespace');
  if ((source.match(/^\[lib\]\s*$/gmu) ?? []).length !== 1) {
    throw new Error('Ambiguous library namespace');
  }
  const section = /^\[lib\]\s*\n([^]*?)(?=^\[|$(?![^]))/mu.exec(source);
  const names = section?.[1].match(/^name\s*=\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*$/gmu);
  if (!names || names.length !== 1) throw new Error('Unknown library namespace');
  const pathLines = section[1].split(/\r?\n/u)
    .filter(line => /^[ \t]*path[ \t]*=/u.test(line));
  if (pathLines.length > 1) throw new Error('Ambiguous Cargo library path');
  if (pathLines.length === 1) {
    const field = /^[ \t]*path[ \t]*=[ \t]*"([^"]+)"[ \t]*(?:#.*)?$/u.exec(pathLines[0]);
    if (!field || field[1] !== 'src/lib.rs') {
      throw new Error('Unbound Cargo library source path');
    }
  }
  return /"([^"]+)"/u.exec(names[0])[1];
}
function parentFor(parts, root, sources) {
  let directory = posix.dirname(root);
  let current = root;
  for (const [index, part] of parts.entries()) {
    if (!moduleDeclarations(sources.get(current), part).has(part)) {
      throw new Error('Unbound physical module');
    }
    directory = posix.join(directory, part);
    if (index === parts.length - 1) {
      if (moduleDeclarations(sources.get(current), part).get(part) !== true) {
        throw new Error('Private exported module');
      }
      continue;
    }
    const candidates = [directory + '.rs', posix.join(directory, 'mod.rs')]
      .filter(path => sources.has(path));
    if (candidates.length !== 1) throw new Error('Ambiguous physical module');
    current = candidates[0];
  }
  return directory;
}
export function sourceQualifiedAliasReferences(source, file, moves, sources) {
  if (!(sources instanceof Map)) throw new Error('Missing selected sources');
  if (typeof source !== 'string') throw new Error('Missing source');
  const selectedModuleNames = moves.filter(move => move.from.endsWith('.rs') &&
    move.to.endsWith('.rs')).flatMap(move => [posix.basename(move.from, '.rs'),
    posix.basename(move.to, '.rs')]);
  if (selectedModuleNames.every(name => /^[A-Za-z_][A-Za-z0-9_]*$/u.test(name) &&
      !new RegExp('(^|[^A-Za-z0-9_])' + name + '($|[^A-Za-z0-9_])', 'u').test(source))) {
    return []; // Exact identifier absence: no selected span, no lexical capability claim.
  }
  const declarations = statements(source);
  const found = [];
  for (const [root, rootSource] of sources) {
    if (!root.endsWith('/src/lib.rs')) continue;
    const project = root.slice(0, -'/src/lib.rs'.length);
    const library = libraryName(sources.get(project + '/Cargo.toml'));
    const aliases = new Map();
    const consumerTokens = tokens(source);
    const requestedAliases = new Set(consumerTokens.filter((token, index) =>
      token.text === library && consumerTokens[index + 1]?.text === '::')
      .map(token => consumerTokens[consumerTokens.indexOf(token) + 2]?.text));
    for (const statement of statements(rootSource)) {
      const words = statement.map(token => token.text);
      if (!words.includes('as') || !requestedAliases.has(words[words.indexOf('as') + 1])) continue;
      const at = words.indexOf('as');
      if (words[0] !== 'pub' || words[1] !== 'use' ||
          words.at(-1) !== ';' || at !== words.length - 3 ||
          statement[at + 1]?.type !== 'ident') {
        throw new Error('Unsupported exported alias');
      }
      const path = statement.slice(2, at);
      if (!path.length || path.some((token, index) => index % 2 === 0
        ? token.type !== 'ident' : token.text !== '::')) {
        throw new Error('Unsupported alias path');
      }
      const alias = words[at + 1];
      let bindingDepth = 0;
      for (const token of tokens(rootSource)) {
        if (token.text === '}') bindingDepth -= 1;
        if (bindingDepth === 0 && token.type === 'ident' && token.text === alias &&
            token.start !== statement[at + 1].start) {
          throw new Error('Ambiguous exported alias binding');
        }
        if (token.text === '{') bindingDepth += 1;
      }
      if (aliases.has(alias)) throw new Error('Duplicate exported alias');
      aliases.set(alias, parentFor(path.filter(token => token.type === 'ident')
        .map(token => token.text), root, sources));
    }
    for (const statement of declarations) {
      const words = statement.map(token => token.text);
      if (words[0] !== 'use' || words[1] !== library || words[2] !== '::') continue;
      const parent = aliases.get(words[3]);
      if (!parent) continue;
      if (!moves.some(move => posix.dirname(move.from) === parent &&
          posix.dirname(move.to) === parent)) continue; // No selected namespace rename.
      const pathTokens = statement.slice(1, -1);
      const completePath = words.at(-1) === ';' && pathTokens.length >= 5
        && pathTokens.length % 2 === 1
        && pathTokens.every((token, index) => index % 2 === 0
          ? token.type === 'ident' && token.text !== 'as' : token.text === '::');
      const grouped = words.length >= 11 && words[4] === '::' &&
        statement[5]?.type === 'ident' && words[6] === '::' && words[7] === '{' &&
        words.at(-2) === '}' && words.at(-1) === ';';
      let groupItems = null;
      if (!completePath && grouped) {
        const entries = statement.slice(8, -2);
        if (entries.at(-1)?.text === ',') entries.pop();
        if (entries.length === 0 || entries.length % 2 !== 1 ||
            !entries.every((token, index) => index % 2 === 0
              ? token.type === 'ident' && !['self', 'super', 'crate', 'as'].includes(token.text)
              : token.text === ',')) throw new Error('Unsupported grouped alias consumer');
        groupItems = entries.filter((_, index) => index % 2 === 0).map(token => token.text);
        if (new Set(groupItems).size !== groupItems.length) throw new Error('Duplicate grouped binding');
        const ownerMove = moves.find(move => posix.dirname(move.from) === parent &&
          posix.dirname(move.to) === parent && [posix.basename(move.from, '.rs'), posix.basename(move.to, '.rs')].includes(words[5]));
        if (!ownerMove) continue; // Unchanged nonselected module: no reference or rename authority.
        if (!sources.has(ownerMove.from)) throw new Error('Unowned grouped module');
        const definitionTokens = tokens(sources.get(ownerMove.from));
        if (definitionTokens.some((token, index) => token.text === '#' &&
            definitionTokens[index + 1]?.text === '[' &&
            ['cfg', 'cfg_attr'].includes(definitionTokens[index + 2]?.text))) {
          throw new Error('Conditional grouped item ownership');
        }
        const exports = new Set();
        let definitionDepth = 0;
        for (let index = 0; index < definitionTokens.length; index += 1) {
          const token = definitionTokens[index];
          if (token.text === '}') definitionDepth -= 1;
          if (definitionDepth === 0 && token.text === 'pub' &&
              ['struct', 'enum', 'type', 'fn', 'const', 'static'].includes(definitionTokens[index + 1]?.text) &&
              definitionTokens[index + 2]?.type === 'ident') exports.add(definitionTokens[index + 2].text);
          if (token.text === '{') definitionDepth += 1;
        }
        if (groupItems.some(item => !exports.has(item))) throw new Error('Unowned grouped item');
        for (const other of declarations) {
          if (other === statement) continue;
          const names = other.map(token => token.text);
          if (names[0] === 'use' && other.some(token => groupItems.includes(token.text))) {
            throw new Error('Competing grouped import binding');
          }
        }
        const sourceTokens = tokens(source);
        for (let index = 0; index < sourceTokens.length; index += 1) {
          const keyword = sourceTokens[index].text;
          if (['struct', 'enum', 'type', 'fn', 'const', 'static', 'mod'].includes(keyword) &&
              groupItems.includes(sourceTokens[index + 1]?.text)) throw new Error('Shadowed grouped binding');
          if (keyword === 'let' || keyword === 'for') {
            if (keyword === 'for') {
              let declarationStart = index - 1;
              while (declarationStart >= 0 &&
                  ![';', '{', '}'].includes(sourceTokens[declarationStart].text)) declarationStart -= 1;
              if (sourceTokens.slice(declarationStart + 1, index)
                  .some(token => token.text === 'impl')) continue; // Trait type clause, not a value pattern.
            }
            let patternDepth = 0;
            for (let binding = index + 1; binding < sourceTokens.length; binding += 1) {
              const word = sourceTokens[binding].text;
              if (patternDepth === 0 && (['=', ':', ';'].includes(word) ||
                  (keyword === 'for' && word === 'in'))) break;
              if (groupItems.includes(word)) throw new Error('Shadowed grouped binding pattern');
              if (['(', '{', '['].includes(word)) patternDepth += 1;
              if ([')', '}', ']'].includes(word)) {
                patternDepth -= 1;
                if (patternDepth < 0) throw new Error('Unbalanced grouped binding pattern');
              }
            }
          }
          if (keyword === '=>') {
            let patternDepth = 0;
            for (let binding = index - 1; binding >= 0; binding -= 1) {
              const word = sourceTokens[binding].text;
              if (patternDepth === 0 && [',', '{', ';', '=>'].includes(word)) break;
              if (groupItems.includes(word)) throw new Error('Unsupported grouped arm pattern binding');
              if ([')', '}', ']'].includes(word)) patternDepth += 1;
              if (['(', '{', '['].includes(word)) {
                patternDepth -= 1;
                if (patternDepth < 0) throw new Error('Unbalanced grouped arm pattern');
              }
            }
          }
          if (keyword === 'fn') {
            let parameter = index + 2;
            while (parameter < sourceTokens.length &&
                !['(', '{', ';'].includes(sourceTokens[parameter].text)) parameter += 1;
            if (sourceTokens[parameter]?.text !== '(') continue;
            let parameterDepth = 1;
            let inBinding = true;
            for (parameter += 1; parameter < sourceTokens.length && parameterDepth > 0; parameter += 1) {
              const word = sourceTokens[parameter].text;
              if (parameterDepth === 1 && word === ',') inBinding = true;
              if (parameterDepth === 1 && word === ':') inBinding = false;
              if (inBinding && groupItems.includes(word)) throw new Error('Shadowed grouped parameter pattern');
              if (['(', '{', '['].includes(word)) parameterDepth += 1;
              if ([')', '}', ']'].includes(word)) parameterDepth -= 1;
            }
          }
          if (keyword === '|') {
            for (let parameter = index + 1; parameter < sourceTokens.length &&
                !['|', ';'].includes(sourceTokens[parameter].text); parameter += 1) {
              if (groupItems.includes(sourceTokens[parameter].text)) throw new Error('Unsupported grouped closure binding');
            }
          }
        }
      } else if (!completePath) {
        throw new Error('Unsupported alias consumer');
      }
      for (const move of moves) {
        if (posix.dirname(move.from) !== parent ||
            posix.dirname(move.to) !== parent) continue;
        const oldName = posix.basename(move.from, '.rs');
        const newName = posix.basename(move.to, '.rs');
        if (words[5] === oldName) found.push({
          start: statement[5].start, end: statement[5].end,
          from: oldName, to: newName, move,
        });
      }
    }
    // A qualified use outside the admitted top-level declaration must not
    // silently bypass lexical ownership or shadowing checks.
    const all = tokens(source);
    for (let index = 0; index < all.length; index += 1) {
      if (all[index].text !== library) continue;
      if (all[index + 1]?.text !== '::') {
        throw new Error('Shadowed library namespace');
      }
      if (!aliases.has(all[index + 2]?.text)) continue;
      const selectedParent = aliases.get(all[index + 2].text);
      if (!moves.some(move => posix.dirname(move.from) === selectedParent &&
          posix.dirname(move.to) === selectedParent && all[index + 3]?.text === '::' &&
          [posix.basename(move.from, '.rs'), posix.basename(move.to, '.rs')]
            .includes(all[index + 4]?.text))) continue; // Unchanged nonselected qualification.
      const owned = declarations.some(statement => statement[0]?.text === 'use' &&
        statement[1]?.start === all[index].start);
      if (!owned) throw new Error('Unsupported nested or shadowed alias consumer');
    }
  }
  return found;
}
