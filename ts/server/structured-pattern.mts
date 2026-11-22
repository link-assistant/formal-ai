// Anchored ASCII patterns with one level of noncapturing groups. Compile
// repetitions into an NFA so matching never backtracks over the input.
const ATOM = String.raw`(?:\[(?:\\[-.]|[A-Za-z0-9 _'+@-])+\]|\\\.|[A-Za-z0-9 _'@-])`;
const QUANTIFIER = String.raw`(?:[?*+]|\{[1-8],\})`;
const TERMS = new RegExp(`(${ATOM})(${QUANTIFIER})?`, 'gu');
const SEQUENCE = new RegExp(
  String.raw`\(\?:((?:${ATOM}${QUANTIFIER}?)+)\)([?*+])|${TERMS.source}`, 'gu',
);

function state() {
  return { empty: [], character: null };
}

function repeat(start, end, quantifier) {
  if (quantifier === '?' || quantifier === '*') start.empty.push(end);
  if (quantifier === '+' || quantifier === '*') end.empty.push(start);
}

function appendAtom(previous, atom, quantifier) {
  let test;
  try { test = new RegExp(`^${atom}$`, 'u'); } catch { return null; }
  const start = state();
  previous.empty.push(start);
  const minimum = quantifier?.startsWith('{') ? Number(quantifier[1]) : 0;
  let from = start;
  let to = state();
  for (let index = 1; index < minimum; index++) {
    from.character = { test, to };
    from = to;
    to = state();
  }
  from.character = { test, to };
  if (minimum) to.empty.push(from);
  else repeat(start, to, quantifier);
  return to;
}

function close(states) {
  for (const from of states) {
    for (const to of from.empty) states.add(to);
  }
  return states;
}

export function anchoredPattern(pattern) {
  if (!pattern.startsWith('^') || !pattern.endsWith('$')) return null;
  const source = pattern.slice(1, -1);
  const start = state();
  let end = start;
  let position = 0;
  for (const match of source.matchAll(SEQUENCE)) {
    if (match.index !== position) return null;
    position += match[0].length;
    const [, group, repetition, atom, quantifier] = match;
    if (group === undefined) {
      end = appendAtom(end, atom, quantifier);
      if (!end) return null;
    } else {
      const entry = state();
      end.empty.push(entry);
      end = entry;
      for (const [, part, count] of group.matchAll(TERMS)) {
        end = appendAtom(end, part, count);
        if (!end) return null;
      }
      repeat(entry, end, repetition);
    }
  }
  if (position !== source.length) return null;
  return {
    test(text) {
      let active = close(new Set([start]));
      for (const character of text) {
        const next = new Set();
        for (const { character: edge } of active) {
          if (edge?.test.test(character)) next.add(edge.to);
        }
        if (next.size === 0) return false;
        active = close(next);
      }
      return active.has(end);
    },
  };
}
