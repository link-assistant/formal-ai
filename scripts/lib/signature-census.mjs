// Translation of rust/src/self_ast_census.rs signature tier and Rust stable_id.
// Lexical behavior intentionally mirrors that implementation, including its limitations.
// Full AST census is explicitly unsupported; this is not a Rust parser or native proof.
const rustWhitespace = /[\u0009-\u000d\u0020\u0085\u00a0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000]+/u;

function rustLines(sourceText) {
  if (sourceText.length === 0) return [];
  const sourceLines = sourceText.split('\n');
  const terminated = sourceText.endsWith('\n');
  if (terminated) sourceLines.pop();
  return sourceLines.map((line, position) =>
    (terminated || position < sourceLines.length - 1) && line.endsWith('\r')
      ? line.slice(0, -1) : line);
}

export function stableSourceIdentifier(sourceText) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(sourceText)) {
    hash = BigInt.asUintN(64, (hash ^ BigInt(byte)) * 0x100000001b3n);
  }
  return 'source_module_' + hash.toString(16).padStart(16, '0');
}

function stripNoncode(line, commentState) {
  const characters = [...line];
  let visibleText = '';
  let position = 0;
  while (position < characters.length) {
    const current = characters[position];
    const following = characters[position + 1];
    if (commentState.inBlockComment) {
      if (current === '*' && following === '/') {
        commentState.inBlockComment = false;
        position += 2;
      } else position += 1;
      continue;
    }
    if (current === '/' && following === '/') break;
    if (current === '/' && following === '*') {
      commentState.inBlockComment = true;
      position += 2;
    } else if (current === '"') {
      visibleText += '"';
      position += 1;
      while (position < characters.length) {
        if (characters[position] === '\\') {
          position += 2;
          continue;
        }
        if (characters[position] === '"') break;
        position += 1;
      }
      position += 1;
      visibleText += '"';
    } else if (current === "'") {
      const characterLiteral = characters[position + 1] === '\\'
        || characters[position + 2] === "'";
      if (characterLiteral) {
        let nextPosition = position + 1;
        if (characters[nextPosition] === '\\') nextPosition += 1;
        while (nextPosition < characters.length && characters[nextPosition] !== "'") {
          nextPosition += 1;
        }
        position = nextPosition + 1;
        visibleText += "''";
      } else {
        visibleText += current;
        position += 1;
      }
    } else {
      visibleText += current;
      position += 1;
    }
  }
  return visibleText;
}

function declaredItem(line) {
  const tokens = line.split(rustWhitespace).filter(Boolean);
  let position = 0;
  let first = tokens[position];
  if (first === undefined) return null;
  if (first.startsWith('pub')) {
    let visibility = tokens[position++];
    while (visibility.includes('(') && !visibility.includes(')')) {
      if (tokens[position] === undefined) return null;
      visibility += tokens[position++];
    }
    first = tokens[position];
    if (first === undefined) return null;
  }
  while (['default', 'async', 'unsafe', 'extern'].includes(first)) {
    const modifier = tokens[position++];
    if (modifier === 'extern' && tokens[position]?.startsWith('"')) position += 1;
    first = tokens[position];
    if (first === undefined) return null;
  }
  if (first === 'const' && tokens[position + 1] === 'fn') {
    position += 1;
    first = 'fn';
  }
  const kinds = {
    fn: 'function', struct: 'struct', enum: 'enum', trait: 'trait',
    type: 'type', mod: 'module', const: 'const', static: 'static', union: 'union',
  };
  const kind = Object.hasOwn(kinds, first) ? kinds[first]
    : first.startsWith('macro_rules!') ? 'macro' : null;
  if (kind === null) return null;
  position += 1;
  if (kind === 'static' && tokens[position] === 'mut') position += 1;
  let rawIdentifier;
  if (kind === 'macro') {
    let inlineIdentifier = first;
    while (inlineIdentifier.startsWith('macro_rules!')) {
      inlineIdentifier = inlineIdentifier.slice('macro_rules!'.length);
    }
    rawIdentifier = inlineIdentifier.length ? inlineIdentifier : tokens[position];
  } else rawIdentifier = tokens[position];
  if (rawIdentifier === undefined) return null;
  let identifier = '';
  for (const character of rawIdentifier) {
    if (character !== '_' && !/[\p{Alphabetic}\p{Number}]/u.test(character)) break;
    identifier += character;
  }
  if (!identifier.length || /^[0-9]/u.test(identifier) || identifier === '_') return null;
  return {kind, name: identifier};
}

function itemEndLine(codeLines, start) {
  let braceDepth = 0;
  let opened = false;
  for (let position = start; position < codeLines.length; position += 1) {
    for (const character of codeLines[position]) {
      if (character === '{') {
        braceDepth += 1;
        opened = true;
      } else if (character === '}') {
        braceDepth = Math.max(0, braceDepth - 1);
        if (opened && braceDepth === 0) return position + 1;
      } else if (character === ';' && !opened && braceDepth === 0) {
        return position + 1;
      }
    }
  }
  return Math.max(codeLines.length, start + 1);
}

export function scanSignatureSymbols(sourceText) {
  if (typeof sourceText !== 'string' || !sourceText.isWellFormed()) {
    throw Error('well_formed_source_text_required');
  }
  const commentState = {inBlockComment: false};
  const codeLines = rustLines(sourceText).map(line => stripNoncode(line, commentState));
  const symbols = [];
  for (const [position, line] of codeLines.entries()) {
    const declaration = declaredItem(line);
    if (declaration !== null) {
      symbols.push({...declaration, startLine: position + 1,
        endLine: itemEndLine(codeLines, position)});
    }
  }
  return symbols;
}

export function signatureCensus(modulePath, sourceText, {fidelity = 'signature'} = {}) {
  if (typeof modulePath !== 'string' || !/^src\/[A-Za-z0-9_/-]+\.rs$/u.test(modulePath)) {
    throw Error('owned_rust_module_path_required');
  }
  if (fidelity !== 'signature' || modulePath.startsWith('src/agentic_coding/')) {
    throw Error('full_ast_tier_unsupported');
  }
  const symbols = scanSignatureSymbols(sourceText);
  const lines = ['self_ast_census', '  target ' + modulePath, '  language rust',
    '  fidelity signature', '  byte_len ' + new TextEncoder().encode(sourceText).length,
    '  line_count ' + rustLines(sourceText).length,
    '  content_id ' + stableSourceIdentifier(sourceText), '  symbol_count ' + symbols.length];
  if (symbols.length) {
    lines.push('  symbols');
    for (const symbol of symbols) {
      lines.push('    ' + [symbol.kind, symbol.name, symbol.startLine, symbol.endLine].join(' '));
    }
  }
  return lines.join('\n') + '\n';
}
