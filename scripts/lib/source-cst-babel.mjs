// Complete grammar tokens and recursive ownership, projected into the common CST.
export function babelSourceFrontend(parser, filename) {
  const syntaxName = name => name.replace(/([a-z0-9])([A-Z])/gu, '$1_$2').toLowerCase();
  const excluded = new Set(['loc', 'extra', 'tokens', 'comments', 'leadingComments', 'trailingComments', 'innerComments', 'errors']);
  return (source, language) => {
    const plugins = [];
    if (language === 'TypeScript') plugins.push('typescript');
    if (language === 'JavaScript' || filename.endsWith('.tsx')) plugins.push('jsx');
    const ast = parser.parse(source, {
      sourceType: 'unambiguous',
      tokens: true,
      attachComment: false,
      createParenthesizedExpressions: true,
      errorRecovery: false,
      allowUndeclaredExports: true,
      sourceFilename: filename,
      plugins
    });
    const lexical = ast.tokens.filter(token => token.end > token.start),
      tokens = [];
    for (let index = 1; index < lexical.length; index += 1) {
      if (lexical[index].start < lexical[index - 1].end) throw Error('OverlappingGrammarTokens');
    }
    const textNode = (start, end, term, field) => {
      const tokenIndex = tokens.length;
      tokens.push({
        text: source.slice(start, end)
      });
      return {
        term,
        field,
        tokenIndex,
        children: []
      };
    };
    const lexicalRange = (start, end) => {
      const result = [];
      let at = start,
        low = 0,
        high = lexical.length;
      while (low < high) {
        const middle = low + high >>> 1;
        if (lexical[middle].end <= start) low = middle + 1;else high = middle;
      }
      for (let index = low; index < lexical.length && lexical[index].start < end; index += 1) {
        const token = lexical[index],
          begin = Math.max(start, token.start),
          finish = Math.min(end, token.end);
        if (at < begin) {
          if (!/^[\s\uFEFF]*$/u.test(source.slice(at, begin))) throw Error('UnclassifiedGrammarGap');
          result.push(textNode(at, begin, 'whitespace'));
        }
        const kind = typeof token.type === 'string' ? token.type : token.type.label;
        const term = /Comment/u.test(kind) ? 'comment' : kind === 'name' ? 'identifier' : kind;
        result.push(textNode(begin, finish, term));
        at = finish;
      }
      if (at < end) {
        if (!/^[\s\uFEFF]*$/u.test(source.slice(at, end))) throw Error('UnclassifiedGrammarGap');
        result.push(textNode(at, end, 'whitespace'));
      }
      return result;
    };
    const project = (node, field) => {
      const entries = [];
      for (const [name, raw] of Object.entries(node)) {
        if (excluded.has(name)) continue;
        for (const child of Array.isArray(raw) ? raw : [raw]) {
          if (child && typeof child === 'object' && typeof child.type === 'string' && Number.isInteger(child.start) && Number.isInteger(child.end) && child.end > child.start) {
            if (child.start < node.start || child.end > node.end) throw Error('GrammarChildOutsideOwner');
            entries.push({
              node: child,
              field: name === 'id' ? 'name' : name
            });
          }
        }
      }
      entries.sort((left, right) => left.node.start - right.node.start || right.node.end - left.node.end);
      const children = [];
      let at = node.start;
      for (const entry of entries) {
        const previous = children.at(-1);
        if (entry.node.start < at) {
          if (previous?.ownerStart === entry.node.start && previous?.ownerEnd === entry.node.end && previous.term === syntaxName(entry.node.type)) {
            previous.field += '|' + entry.field;
            continue;
          }
          const seek = candidate => {
            if (candidate.ownerStart === entry.node.start && candidate.ownerEnd === entry.node.end && candidate.term === syntaxName(entry.node.type)) return candidate;
            for (const child of candidate.children ?? []) {
              const found = seek(child);
              if (found) return found;
            }
            return null;
          };
          const alias = previous && seek(previous);
          if (alias) {
            alias.field = [alias.field, syntaxName(node.type) + ':' + entry.field].filter(Boolean).join('|');
            continue;
          }
          throw Error('OverlappingGrammarOwners:' + node.type);
        }
        children.push(...lexicalRange(at, entry.node.start), project(entry.node, entry.field));
        at = entry.node.end;
      }
      children.push(...lexicalRange(at, node.end));
      if (!entries.length && children.length === 1 && children[0].tokenIndex !== undefined) {
        const terminal = children[0];
        terminal.term = syntaxName(node.type);
        terminal.field = field;
        terminal.ownerStart = node.start;
        terminal.ownerEnd = node.end;
        return terminal;
      }
      return {
        term: syntaxName(node.type),
        field,
        children,
        ownerStart: node.start,
        ownerEnd: node.end
      };
    };
    const root = {
      ...ast.program,
      start: 0,
      end: source.length
    };
    return {
      tree: project(root),
      tokens
    };
  };
}
