// Keep JSON leaf records readable without changing any non-whitespace token.
// In particular, do not parse and reserialize numbers or duplicate object keys.
export function compactJsonLeaves(source, maxWidth = 100) {
  if (!Number.isSafeInteger(maxWidth) || maxWidth < 1) throw new RangeError('invalid JSON line width');
  JSON.parse(source);
  const tokens = [...source.matchAll(/"(?:\\.|[^"\\])*"|[{}\[\],:]|-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?|true|false|null/gu)];
  const stack = [];
  const replacements = [];
  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index][0];
    if (token === '{' || token === '[') {
      if (stack.length) stack.at(-1).nested = true;
      stack.push({index, nested: false});
    } else if (token === '}' || token === ']') {
      const opened = stack.pop();
      if (opened.nested) continue;
      const start = tokens[opened.index].index;
      const end = tokens[index].index + 1;
      if (!source.slice(start, end).includes('\n')) continue;
      const compact = tokens.slice(opened.index, index + 1)
        .map(([part]) => part === ',' ? ', ' : part === ':' ? ': ' : part).join('');
      const lineStart = source.lastIndexOf('\n', start - 1) + 1;
      const nextLine = source.indexOf('\n', end);
      const suffix = source.slice(end, nextLine < 0 ? source.length : nextLine).replace(/\r$/u, '');
      if (start - lineStart + compact.length + suffix.length <= maxWidth) {
        replacements.push({start, end, compact});
      }
    }
  }
  let result = '';
  let cursor = 0;
  for (const {start, end, compact} of replacements) {
    result += source.slice(cursor, start) + compact;
    cursor = end;
  }
  return result + source.slice(cursor);
}
