// Lossless projection of a complete concrete grammar tree; positions remain construction-only.
export function rustSourceFrontend(parser) {
  return (source, language) => {
    if (language !== 'Rust') throw Error('RustGrammarLanguageRequired');
    const tree = parser.parse(source),
      tokens = [];
    try {
      if (!tree) throw Error('RustGrammarTreeUnavailable');
      const flag = (node, name) => typeof node[name] === 'function' ? node[name]() : node[name];
      const project = (node, field) => {
        if (flag(node, 'hasError') || flag(node, 'isError') || flag(node, 'isMissing')) throw Error('UnrepresentedSourceSyntax');
        const start = node.startIndex,
          end = node.endIndex;
        if (!Number.isInteger(start) || !Number.isInteger(end) || start < 0 || end < start || end > source.length) throw Error('InvalidGrammarSourceRange');
        const children = node.children;
        if (children.length === 0) {
          const tokenIndex = tokens.length;
          tokens.push({
            text: source.slice(start, end)
          });
          return {
            term: node.type,
            field,
            tokenIndex,
            children: []
          };
        }
        const projected = [];
        let at = start;
        const gap = finish => {
          if (at >= finish) return;
          const text = source.slice(at, finish);
          const leading = /^[\s\uFEFF]*/u.exec(text)[0].length;
          const trailing = leading === text.length ? 0 : /[\s\uFEFF]*$/u.exec(text)[0].length;
          for (const [begin, end, term] of [[0, leading, 'whitespace'], [leading, text.length - trailing, 'hidden_text'], [text.length - trailing, text.length, 'whitespace']]) {
            if (begin < end) {
              const tokenIndex = tokens.length;
              tokens.push({
                text: text.slice(begin, end)
              });
              projected.push({
                term,
                tokenIndex,
                children: []
              });
            }
          }
        };
        for (let index = 0; index < children.length; index += 1) {
          const child = children[index];
          if (child.startIndex < at || child.endIndex > end) throw Error('OverlappingRustGrammarChildren');
          gap(child.startIndex);
          projected.push(project(child, node.fieldNameForChild(index)));
          at = child.endIndex;
        }
        gap(end);
        return {
          term: node.type,
          field,
          children: projected
        };
      };
      const root = tree.rootNode,
        leading = [],
        trailing = [];
      const outside = (start, end, target) => {
        if (start < end) {
          const text = source.slice(start, end);
          if (!/^[\s\uFEFF]*$/u.test(text)) throw Error('UnclassifiedRustGrammarOutside');
          const tokenIndex = tokens.length;
          tokens.push({
            text
          });
          target.push({
            term: 'whitespace',
            tokenIndex,
            children: []
          });
        }
      };
      outside(0, root.startIndex, leading);
      const projected = project(root);
      outside(root.endIndex, source.length, trailing);
      return {
        tree: projected,
        tokens,
        leading,
        trailing
      };
    } finally {
      tree?.delete();
    }
  };
}
