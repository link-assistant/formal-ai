import assert from 'node:assert/strict';
import {lex} from '../self-translation/lexer.mjs';

const significant = source => lex(source, 'Rust').filter(token => token.type !== 'comment');
function structure(source) {
  const tokens = significant(source), pairs = new Map(), stack = [];
  for (const [index, token] of tokens.entries()) {
    if (['(', '[', '{'].includes(token.text)) stack.push(index);
    if ([')', ']', '}'].includes(token.text)) {
      const start = stack.pop();
      assert(start !== undefined && ({')':'(', ']':'[', '}':'{'})[token.text] === tokens[start].text,
        'GeneratedRustUnbalancedDelimiter');
      pairs.set(start, index); pairs.set(index, start);
    }
  }
  assert.equal(stack.length, 0);
  return {tokens, pairs};
}
function edits(source, changes) {
  const ordered = [...changes].sort((a, b) => b.start - a.start);
  let end = source.length;
  for (const change of ordered) {
    assert(change.end <= end && change.start <= change.end, 'GeneratedRustOverlappingEdit');
    source = source.slice(0, change.start) + change.text + source.slice(change.end);
    end = change.start;
  }
  return source;
}
function functions(source) {
  const {tokens, pairs} = structure(source), result = [];
  for (let index = 0; index < tokens.length; index++) {
    if (tokens[index].text !== 'fn' || tokens[index + 1]?.type !== 'ident') continue;
    let parameters = index + 2;
    while (parameters < tokens.length && tokens[parameters].text !== '(') parameters++;
    const close = pairs.get(parameters);
    assert(close !== undefined, 'GeneratedRustFunctionParameters');
    let body = close + 1;
    while (body < tokens.length && !['{', ';'].includes(tokens[body].text)) body++;
    if (tokens[body]?.text !== '{') continue;
    result.push({name:tokens[index + 1].text, start:tokens[index - 1]?.text === 'pub' ? index - 1 : index,
      parameters, close, body, end:pairs.get(body), tokens, pairs});
  }
  return result;
}
function parameters(fn) {
  const {tokens, pairs} = fn, rows = [];
  let start = fn.parameters + 1;
  for (let index = start; index <= fn.close; index++) {
    if (['(', '[', '{'].includes(tokens[index]?.text)) { index = pairs.get(index); continue; }
    if (tokens[index]?.text !== ',' && index !== fn.close) continue;
    const row = tokens.slice(start, index), colon = row.findIndex(token => token.text === ':');
    if (colon > 0) rows.push({name:row[colon - 1].text, type:row.slice(colon + 1).map(token => token.text)});
    start = index + 1;
  }
  return rows;
}
/** Backward liveness over the emitted structured, immutable owned-value fragment. */
export function eliminateLastOwnedClones(source) {
  const changes = [], observations = [];
  if (/\b(?:unsafe|extern|impl)\b/u.test(source)) return {source, observations};
  for (const fn of functions(source)) {
    const {tokens:t, pairs:p} = fn;
    if (t.slice(fn.start,fn.parameters).some(token => token.text === '<')) continue;
    const owned = new Set(parameters(fn).filter(row => !row.type.includes('&')
      && row.type.some(type => type === 'String' || type === 'Vec' || /^Research/u.test(type)))
      .map(row => row.name));
    const declared = new Map([...owned].map(name => [name, 1]));
    let opaque = false;
    for (let index = fn.body + 1; index < fn.end; index++) {
      if (t[index].text !== 'let') continue;
      let name = index + 1;
      if (t[name]?.text === 'mut') name++;
      if (t[name]?.type !== 'ident' || t[name + 1]?.text !== '=') continue;
      declared.set(t[name].text, (declared.get(t[name].text) ?? 0) + 1);
      let end = name + 2;
      while (end < fn.end && t[end].text !== ';') {
        if (['(', '[', '{'].includes(t[end].text)) end = p.get(end);
        end++;
      }
      const rhs = t.slice(name + 2, end), spelling = rhs.map(token => token.text).join(' ');
      if (rhs[0]?.text === '&' || /\. (?:get|as_str) \(/u.test(spelling)
          && !/\. (?:clone|to_string|to_owned) \(/u.test(spelling)) opaque = true;
      if (/\b(?:String|Vec|format!|vec!)\b/u.test(spelling) || /\. (?:clone|to_string|to_owned) \(/u.test(spelling)
          || rhs[0]?.text === 'crate' || owned.has(rhs[0]?.text)) owned.add(t[name].text);
    }
    const patternDeclarationSites = new Set();
    for (let index = fn.body + 1; index < fn.end; index++) {
      if (t[index].text !== 'match') continue;
      let open = index + 1;
      while (open < fn.end && t[open].text !== '{') { if (['(', '['].includes(t[open].text)) open = p.get(open); open++; }
      if (t.slice(index + 1, open).some(token => token.text === '&')) { opaque = true; continue; }
      if (!t.slice(index + 1, open).some(token => owned.has(token.text))) continue;
      const close = p.get(open);
      for (let cursor = open + 1; cursor < close; cursor++) {
        if (t[cursor].text !== '=>') continue;
        let pattern = cursor - 1;
        if (t[pattern]?.text !== ')') continue;
        const begin = p.get(pattern);
        for (let item = begin + 1; item < pattern; item++) {
          if (t[item].type === 'ident' && ['(', ','].includes(t[item - 1]?.text)
              && [')', ','].includes(t[item + 1]?.text)) {
            owned.add(t[item].text);
            if (!patternDeclarationSites.has(item)) {
              declared.set(t[item].text, (declared.get(t[item].text) ?? 0) + 1);
              patternDeclarationSites.add(item);
            }
          }
        }
      }
    }
    // Inline capture-free predicates are safe; retaining any owned capture is not.
    for (let index = fn.body + 1; index < fn.end; index++) {
      const emptyClosure = t[index].text === '||' && ['=', '(', ',', 'return', '=>', '{', 'move'].includes(t[index - 1]?.text);
      if (t[index].text !== '|' && !emptyClosure) continue;
      let end = emptyClosure ? index : index + 1;
      while (!emptyClosure && end < fn.end && t[end].text !== '|') end++;
      if (end === fn.end) { opaque = true; break; }
      const enclosing = [...p].filter(([a,b]) => a < index && b > end && a < b).sort((a,b) => b[0]-a[0])[0];
      const bodyEnd = t[end + 1]?.text === '{' ? p.get(end + 1) : enclosing?.[1];
      if (bodyEnd === undefined || t.slice(end + 1, bodyEnd).some(token => owned.has(token.text))) opaque = true;
      index = end;
    }
    if (t.slice(fn.body + 1, fn.end).some(token => ['for','while','ref'].includes(token.text))) opaque = true;
    if (opaque) continue;
    const candidates = new Map();
    const union = (...sets) => new Set(sets.flatMap(set => [...set]));
    const equal = (a,b) => a.size === b.size && [...a].every(name => b.has(name));
    function block(open, close, after, loop, record) {
      const segments = [];
      let start = open + 1;
      for (let cursor = start; cursor < close; cursor++) {
        if (['(', '[', '{'].includes(t[cursor].text)) { cursor = p.get(cursor); continue; }
        if (t[cursor].text === ';') { segments.push([start,cursor]); start = cursor + 1; }
      }
      if (start < close) segments.push([start,close]);
      let live = new Set(after);
      for (const [start,end] of segments.reverse()) live = expression(start,end,live,loop,record);
      return live;
    }
    function expression(start, end, after, loop, record) {
      while (start < end && t[start].text === '(' && p.get(start) === end - 1) { start++; end--; }
      if (start >= end) return new Set(after);
      if (t[start].text === 'return') return expression(start + 1,end,new Set(),loop,record);
      if (t[start].text === 'continue') return new Set(loop?.entry ?? after);
      if (t[start].text === 'break') return expression(start + 1,end,loop?.after ?? after,loop,record);
      if (t[start].text === '{' && p.get(start) === end - 1) return block(start,end - 1,after,loop,record);
      if (t[start].text === 'loop' && t[start + 1]?.text === '{' && p.get(start + 1) === end - 1) {
        let entry = new Set();
        for (let iteration = 0; iteration < owned.size + 2; iteration++) {
          const next = block(start + 1,end - 1,after,{entry,after},false);
          if (equal(next,entry)) return block(start + 1,end - 1,after,{entry,after},record);
          entry = union(entry,next);
        }
        throw Error('GeneratedOwnershipLoopDidNotConverge');
      }
      if (t[start].text === 'if') {
        let open = start + 1;
        while (open < end && t[open].text !== '{') { if (['(', '['].includes(t[open].text)) open = p.get(open); open++; }
        const close = p.get(open);
        if (close !== undefined) {
          if (t[close + 1]?.text !== 'else' && close !== end - 1) throw Error('UnsupportedGeneratedOwnershipShape');
          if (t[close + 1]?.text === 'else' && t[close + 2]?.text === '{'
              && p.get(close + 2) !== end - 1) throw Error('UnsupportedGeneratedOwnershipShape');
          const then = block(open,close,after,loop,record);
          const other = t[close + 1]?.text === 'else'
            ? expression(close + 2,end,after,loop,record) : after;
          return expression(start + 1,open,union(then,other),loop,record);
        }
      }
      if (t[start].text === 'match') {
        let open = start + 1;
        while (open < end && t[open].text !== '{') { if (['(', '['].includes(t[open].text)) open = p.get(open); open++; }
        const close = p.get(open), arms = [];
        if (close !== end - 1) throw Error('UnsupportedGeneratedOwnershipShape');
        let pattern = open + 1;
        for (let cursor = pattern; cursor < close; cursor++) {
          if (['(', '[', '{'].includes(t[cursor].text)) { cursor = p.get(cursor); continue; }
          if (t[cursor].text !== '=>') continue;
          const value = cursor + 1;
          let finish = value;
          while (finish < close && t[finish].text !== ',') {
            if (['(', '[', '{'].includes(t[finish].text)) finish = p.get(finish);
            finish++;
          }
          const live = expression(value,finish,after,loop,record);
          for (const token of t.slice(pattern,cursor)) if (owned.has(token.text)) live.delete(token.text);
          arms.push(live); cursor = finish; pattern = finish + 1;
        }
        return expression(start + 1,open,union(...arms),loop,record);
      }
      if (t[start].text === 'let') {
        let name = start + 1;
        if (t[name]?.text === 'mut') name++;
        if (t[name + 1]?.text === '=') {
          const live = new Set(after); live.delete(t[name].text);
          return expression(name + 2,end,live,loop,record);
        }
      }
      let assignment = -1;
      for (let cursor = start; cursor < end; cursor++) {
        if (['(', '[', '{'].includes(t[cursor].text)) { cursor = p.get(cursor); continue; }
        if (t[cursor].text === '=') { assignment = cursor; break; }
      }
      if (assignment >= 0) {
        if (t.slice(start,assignment).some(token => ['.', '[', '*', '&'].includes(token.text))) throw Error('UnsupportedGeneratedOwnershipShape');
        const live = new Set(after);
        for (const token of t.slice(start,assignment)) if (owned.has(token.text)) live.delete(token.text);
        return expression(assignment + 1,end,live,loop,record);
      }
      let live = new Set(after);
      for (let cursor = end - 1; cursor >= start; cursor--) {
        if (t[cursor].text === ')' && t[cursor - 1]?.text === '(' && t[cursor - 2]?.text === 'clone'
            && t[cursor - 3]?.text === '.' && owned.has(t[cursor - 4]?.text)) {
          const name = t[cursor - 4].text;
          if (record && !live.has(name) && (declared.get(name) ?? 0) === 1
              && !['.', '&'].includes(t[cursor - 5]?.text)) {
            candidates.set(cursor,{start:t[cursor - 3].start,end:t[cursor].end,text:'',
              function:fn.name,binding:name,rule:'owned-control-flow-last-use'});
          }
          live.add(name); cursor -= 4; continue;
        }
        if ([')', ']', '}'].includes(t[cursor].text)) {
          const open = p.get(cursor);
          live = t[cursor].text === '}' ? block(open,cursor,live,loop,record)
            : expression(open + 1,cursor,live,loop,record);
          cursor = open; continue;
        }
        if (t[cursor].type === 'ident' && owned.has(t[cursor].text)
            && !['.', '::'].includes(t[cursor - 1]?.text)) live.add(t[cursor].text);
      }
      return live;
    }
    try { block(fn.body,fn.end,new Set(),null,true); } catch (error) { if (error.message === 'UnsupportedGeneratedOwnershipShape') continue; throw error; }
    for (const change of candidates.values()) {
      changes.push(change); observations.push({function:change.function,binding:change.binding,rule:change.rule,offset:change.start});
    }
  }
  return {source:edits(source,changes),observations};
}

function simpleEdits(source) {
  const {tokens, pairs} = structure(source), changes = [], occupied = new Set();
  const add = (start, end, text, rule) => {
    if ([...occupied].some(index => index >= start && index < end)) return;
    changes.push({start:tokens[start].start, end:tokens[end - 1].end, text, rule});
    for (let index = start; index < end; index++) occupied.add(index);
  };
  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index];
    if (token.type === 'number' && /^[0-9]{5,}(?:\.[0-9]+)?(?:f64)?$/u.test(token.text)) {
      const [integer, fraction] = token.text.split('.');
      add(index, index + 1, integer.replace(/\B(?=(?:[0-9]{3})+(?![0-9]))/gu, '_')
        + (fraction === undefined ? '' : '.' + fraction), 'readable-numeric-spelling');
    }
    if (token.type === 'string' && /[^\x00-\x7f]/u.test(token.text)) {
      const escaped = token.text.replace(/[^\x00-\x7f]/gu, character =>
        '\\u{' + character.codePointAt(0).toString(16) + '}');
      add(index, index + 1, escaped, 'exact-unicode-escape');
    }
    if (tokens.slice(index, index + 6).map(row => row.text).join(' ') === 'String :: from ( "" )') {
      add(index, index + 6, 'String::new()', 'empty-owned-string');
    }
    if (token.text === '(' && tokens[index + 1]?.text === '(') {
      const inner = pairs.get(index + 1), outer = pairs.get(index);
      if (inner + 1 === outer) {
        let comma = false;
        for (let cursor = index + 2; cursor < inner; cursor++) {
          if (['(', '[', '{'].includes(tokens[cursor].text)) cursor = pairs.get(cursor);
          else if (tokens[cursor].text === ',') comma = true;
        }
        if (!comma && inner > index + 2) add(index + 1, inner + 1, source.slice(tokens[index + 1].end, tokens[inner].start), 'redundant-parenthesis');
      }
    }
    if (token.text === '.' && ['ends_with', 'starts_with', 'contains'].includes(tokens[index + 1]?.text)
        && tokens[index + 2]?.text === '(' && tokens[index + 3]?.type === 'string'
        && tokens[index + 4]?.text === ')') {
      const literal = tokens[index + 3].text;
      let value;
      try { value = JSON.parse(literal); } catch { continue; }
      if ([...value].length === 1) {
        const character = value === "'" ? "\\'" : literal.slice(1, -1).replace(/\\"/gu, '"');
        add(index + 3, index + 4, "'" + character + "'", 'single-character-pattern');
      }
    }
    if (token.text === 'format!' && tokens[index + 1]?.text === '(' && tokens[index + 2]?.type === 'string') {
      const close = pairs.get(index + 1), inside = tokens.slice(index + 3, close);
      const names = inside.filter((row, cursor) => cursor % 2 === 1 && row.type === 'ident').map(row => row.text);
      if (inside.every((row, cursor) => cursor % 2 === 0 ? row.text === ',' : row.type === 'ident')
          && names.length > 0 && tokens[index + 2].text === '"' + '{}'.repeat(names.length) + '"') {
        add(index + 2, close, '"' + names.map(name => '{' + name + '}').join('') + '"', 'named-format-arguments');
      }
    }
  }
  return {source:edits(source, changes), rules:changes.map(row => row.rule)};
}
function invertOneIf(source) {
  for (const fn of functions(source)) {
    const {tokens, pairs} = fn;
    for (let index = fn.end - 1; index > fn.body; index--) {
      if (tokens[index].text !== 'if') continue;
      let body = index + 1;
      while (body < fn.end && tokens[body].text !== '{') {
        if (['(', '['].includes(tokens[body].text)) body = pairs.get(body);
        body++;
      }
      const close = pairs.get(body);
      if (tokens[close + 1]?.text !== 'else' || tokens[close + 2]?.text !== '{') continue;
      const otherwise = close + 2, end = pairs.get(otherwise);
      let left = index + 1, right = body;
      while (tokens[left]?.text === '(' && pairs.get(left) === right - 1) { left++; right--; }
      let condition;
      if (tokens[left]?.text === '!') {
        let complete = true;
        for (let cursor = left + 1; cursor < right; cursor++) {
          if (['(', '['].includes(tokens[cursor].text)) cursor = pairs.get(cursor);
          else if (['&&', '||', '==', '!=', '<', '>', '<=', '>=', '+', '-', '*', '/'].includes(tokens[cursor].text)) complete = false;
        }
        if (complete) condition = source.slice(tokens[left + 1].start, tokens[right - 1].end);
      }
      else {
        let operator = -1;
        for (let cursor = left; cursor < right; cursor++) {
          if (['(', '['].includes(tokens[cursor].text)) cursor = pairs.get(cursor);
          else if (tokens[cursor].text === '!=') operator = cursor;
          else if (['&&', '||', '==', '<', '>', '<=', '>='].includes(tokens[cursor].text)) { operator = -1; break; }
        }
        if (operator >= 0) condition = source.slice(tokens[left].start, tokens[operator].start)
          + '==' + source.slice(tokens[operator].end, tokens[right - 1].end);
      }
      if (!condition) continue;
      return source.slice(0, tokens[index].start) + 'if ' + condition + ' '
        + source.slice(tokens[otherwise].start, tokens[end].end) + ' else '
        + source.slice(tokens[body].start, tokens[close].end) + source.slice(tokens[end].end);
    }
  }
  return null;
}
function collapseOneElse(source) {
  const {tokens, pairs} = structure(source);
  for (let index = tokens.length - 1; index >= 0; index--) {
    if (tokens[index].text !== 'else' || tokens[index + 1]?.text !== '{' || tokens[index + 2]?.text !== 'if') continue;
    const end = pairs.get(index + 1), nested = index + 2;
    // The complete block must contain exactly this one if expression.
    let body = nested + 1;
    while (body < end && tokens[body].text !== '{') {
      if (['(', '['].includes(tokens[body].text)) body = pairs.get(body);
      body++;
    }
    let cursor = pairs.get(body);
    while (tokens[cursor + 1]?.text === 'else') {
      const next = cursor + 2;
      if (tokens[next]?.text === '{') cursor = pairs.get(next);
      else if (tokens[next]?.text === 'if') {
        let open = next + 1;
        while (tokens[open].text !== '{') { if (['(', '['].includes(tokens[open].text)) open = pairs.get(open); open++; }
        cursor = pairs.get(open);
      } else break;
    }
    if (cursor + 1 !== end) continue;
    if (!/^\s*$/u.test(source.slice(tokens[index].end,tokens[nested].start).replace('{',''))) continue;
    return source.slice(0,tokens[index].end) + ' ' + source.slice(tokens[nested].start,tokens[end].start).trimEnd()
      + source.slice(tokens[end].end);
  }
  return null;
}
function terminalExpression(tokens, pairs, start, end) {
  while (start < end && tokens[start].text === '(' && pairs.get(start) === end - 1) { start++; end--; }
  if (start >= end) return false;
  if (['return', 'continue', 'break'].includes(tokens[start].text)) return true;
  if (tokens[start].text === '{' && pairs.get(start) === end - 1) return terminalBlock(tokens, pairs, start, end - 1);
  if (!['if', 'match'].includes(tokens[start].text)) return false;
  let open = start + 1;
  while (open < end && tokens[open].text !== '{') {
    if (['(', '['].includes(tokens[open].text)) open = pairs.get(open);
    open++;
  }
  const close = pairs.get(open);
  if (close === undefined) return false;
  if (tokens[start].text === 'if') {
    return tokens[close + 1]?.text === 'else' && terminalBlock(tokens, pairs, open, close)
      && terminalExpression(tokens, pairs, close + 2, end);
  }
  if (close !== end - 1) return false;
  let cursor = open + 1;
  let arms = 0;
  while (cursor < close) {
    while (cursor < close && tokens[cursor].text !== '=>') {
      if (['(', '[', '{'].includes(tokens[cursor].text)) cursor = pairs.get(cursor);
      cursor++;
    }
    if (cursor >= close) return false;
    const value = cursor + 1;
    let finish = value;
    if (tokens[value]?.text === '{') finish = pairs.get(value) + 1;
    else {
      while (finish < close && tokens[finish].text !== ',') {
        if (['(', '[', '{'].includes(tokens[finish].text)) finish = pairs.get(finish);
        finish++;
      }
    }
    if (!terminalExpression(tokens, pairs, value, finish)) return false;
    arms++;
    cursor = finish + (tokens[finish]?.text === ',' ? 1 : 0);
  }
  return arms > 0;
}
function terminalBlock(tokens, pairs, open, close) {
  const statements = []; let start = open + 1;
  for (let cursor = start; cursor < close; cursor++) {
    if (['(', '[', '{'].includes(tokens[cursor].text)) { cursor = pairs.get(cursor); continue; }
    if (tokens[cursor].text === ';') { if (start < cursor) statements.push([start,cursor]); start = cursor + 1; }
  }
  if (start < close) statements.push([start,close]);
  const last = statements.at(-1);
  return last !== undefined && terminalExpression(tokens, pairs, last[0], last[1]);
}
function terminalOwnedRebindingClones(source) {
  if (/\b(?:unsafe|extern|impl)\b/u.test(source)) return { source, changes: [] };
  const { tokens, pairs } = structure(source), changes = [];
  for (let open = 0; open < tokens.length; open++) {
    if (tokens[open].text !== '{') continue;
    const close = pairs.get(open), body = tokens.slice(open + 1, close);
    if (body.length !== 11 || body[0].text !== 'let' || body[1].type !== 'ident') continue;
    const name = body[1].text;
    if (body.map(token => token.text).join(' ') !== `let ${name} = * ${name} ; ${name} . clone ( )`) continue;
    changes.push({ start: body[7].start, end: body[10].end, text: '', rule: 'terminal-owned-rebinding' });
  }
  return { source: edits(source, changes), changes };
}
function removeOneRedundantElse(source) {
  const {tokens, pairs} = structure(source);
  for (let index = tokens.length - 1; index >= 0; index--) {
    if (tokens[index].text !== 'if') continue;
    let open = index + 1;
    while (open < tokens.length && tokens[open].text !== '{') {
      if (['(', '['].includes(tokens[open].text)) open = pairs.get(open);
      open++;
    }
    const close = pairs.get(open);
    if (tokens[close + 1]?.text !== 'else' || tokens[close + 2]?.text !== '{') continue;
    if (!terminalBlock(tokens,pairs,open,close)) continue;
    const end = pairs.get(close + 2);
    return source.slice(0,tokens[index].start) + '{ ' + source.slice(tokens[index].start,tokens[close].end)
      + '\n' + source.slice(tokens[close + 2].start,tokens[end].end) + ' }' + source.slice(tokens[end].end);
  }
  return null;
}
function removeOneTailBinding(source) {
  const {tokens, pairs} = structure(source);
  for (let index = 0; index < tokens.length; index++) {
    if (tokens[index].text !== 'let' || tokens[index + 1]?.type !== 'ident'
        || tokens[index + 2]?.text !== '=') continue;
    let end = index + 3;
    while (end < tokens.length && tokens[end].text !== ';') {
      if (['(', '[', '{'].includes(tokens[end].text)) end = pairs.get(end);
      end++;
    }
    if (tokens[end + 1]?.text !== tokens[index + 1].text || tokens[end + 2]?.text !== '}') continue;
    return source.slice(0,tokens[index].start) + source.slice(tokens[index + 3].start,tokens[end].start)
      + source.slice(tokens[end + 1].end);
  }
  return null;
}
function simplifyEmptyComparisons(source) {
  const {tokens:t,pairs:p} = structure(source), changes = [];
  for (let index = 0; index < t.length; index++) {
    if (!['==','!='].includes(t[index].text) || t[index + 1]?.text !== '""') continue;
    function receiverStart(end) {
      if (t[end]?.text === ')') {
        const open = p.get(end);
        if (t[open - 1]?.type === 'ident' && t[open - 2]?.text === '.') return receiverStart(open - 3);
        if (t[open - 1]?.type === 'ident' && !['if','while','match','return','break'].includes(t[open - 1].text)) { let begin = open - 1; while(t[begin - 1]?.text === '::') begin -= 2; return begin; }
        return open;
      }
      return t[end]?.type === 'ident' ? end : null;
    }
    const start = receiverStart(index - 1);
    if (start === null) continue;
    const receiver = source.slice(t[start].start,t[index - 1].end);
    changes.push({start:t[start].start,end:t[index + 1].end,
      text:(t[index].text === '!='?'!':'')+'('+receiver+').is_empty()'});
  }
  return edits(source,changes);
}
function removeComparisonOwnership(source) {
  const {tokens:t,pairs:p} = structure(source), changes = [];
  for (let index = 0; index < t.length; index++) {
    if (t[index].text !== '.' || t[index + 1]?.text !== 'to_string'
        || t[index + 2]?.text !== '(' || t[index + 3]?.text !== ')') continue;
    let next = index + 4;
    while (t[next]?.text === ')') next++;
    if (['==','!='].includes(t[next]?.text)) changes.push({start:t[index].start,end:t[index + 3].end,text:''});
  }
  return edits(source,changes);
}
function simplifyArrayReadMatch(source) {
  const before = 'match index.and_then(|index| values.get(index)) {\n            Some(value) => value.clone(),\n            None => panic!("array index out of range"),\n        }';
  // This is the single pinned runtime ABI helper, not a canonical-task special case.
  return source.replace(before,'index.and_then(|index| values.get(index)).map_or_else(|| panic!("array index out of range"), Clone::clone)');
}

/** A compiler-only lowering: no runtime state, executable ordering or source/IR edits. */
export function lowerGeneratedRust(source, {ownedValueModel, pure} = {}) {
  assert.equal(ownedValueModel, 'pinned-meta-language-owned-values/v1');
  assert.equal(pure, true);
  for (const token of significant(source)) {
    if (token.type === 'ident' && token.text.endsWith('!')) assert(['vec!','format!','panic!'].includes(token.text), 'OpaqueGeneratedRustMacro');
  }
  const original = source, operations = [];
  const terminalMoves = terminalOwnedRebindingClones(source);
  source = terminalMoves.source;
  operations.push(...terminalMoves.changes.map(change => change.rule));
  let clones = eliminateLastOwnedClones(source);
  source = clones.source;
  for (let iteration = 0; iteration < 20; iteration++) {
    const result = simpleEdits(source);
    source = result.source; operations.push(...result.rules);
    if (!result.rules.length) break;
  }
  for (const [rule, transform] of [['positive-condition', invertOneIf], ['collapsed-else-if', collapseOneElse], ['terminal-branch', removeOneRedundantElse], ['tail-binding', removeOneTailBinding]]) {
    for (let iteration = 0; iteration < 200; iteration++) {
      const next = transform(source);
      if (next === null) break;
      source = next; operations.push(rule);
      assert(iteration < 199, 'GeneratedRustTransformationDidNotConverge');
    }
  }
  source = simplifyEmptyComparisons(removeComparisonOwnership(simplifyArrayReadMatch(source)));
  const allowances = [], attributes = [];
  for (const fn of functions(source)) {
    const {tokens} = fn, params = parameters(fn), body = tokens.slice(fn.body + 1, fn.end);
    const spelling = body.map(token => token.text).join(' '), own = params.some(row => !row.type.includes('&')
      && row.type.some(type => type === 'String' || type === 'Vec' || /^Research/u.test(type)));
    const rules = [];
    if (own) rules.push(['needless_pass_by_value', 'Pinned checked IR public ABI owns immutable values']);
    if (params.length > 7) rules.push(['too_many_arguments', 'Pinned checked IR function arity is the canonical source contract']);
    if (params.filter(row => row.type.includes('bool')).length > 3) {
      rules.push(['fn_params_excessive_bools', 'Pinned checked IR boolean parameters preserve the canonical typed ABI']);
    }
    const number = params.some(row => row.type.includes('f64')) || /\bas f64\b|\b[0-9]+f64\b/u.test(spelling);
    if (number && body.some(token => token.text === '==' || token.text === '!=')) {
      rules.push(['float_cmp', 'JavaScript Number equality is exact, including NaN and signed zero']);
    }
    if (/\bas f64\b/u.test(spelling)) rules.push(['cast_precision_loss', 'JavaScript Number conversion deliberately rounds to IEEE binary64']);
    if (/\bas usize\b/u.test(spelling)) {
      const parameter = params.find(row => row.type.length === 1 && row.type[0] === 'f64')?.name;
      const expected = parameter && ('if '+parameter+' >= 0.0 && '+parameter+' . fract ( ) == 0.0 && '+parameter+' < 9_007_199_254_740_992.0 { Some ( '+parameter+' as usize ) } else { None }');
      assert(spelling === expected, 'UnprovenGeneratedIndexCast');
      rules.push(['cast_possible_truncation', 'Guarded nonnegative integral Number index retains pinned runtime conversion'],
        ['cast_sign_loss', 'Guarded nonnegative integral Number index retains pinned runtime conversion']);
    }
    let prefix = tokens.slice(fn.close + 1, fn.body).some(token => token.text === '->') ? '#[must_use]\n' : '';
    for (const [lint, reason] of rules) {
      prefix += '#[allow(clippy::' + lint + ', reason = ' + JSON.stringify(reason) + ')]\n';
      allowances.push({function:fn.name, lint, reason});
    }
    attributes.push({start:tokens[fn.start].start, end:tokens[fn.start].start, text:prefix});
  }
  source = edits(source, attributes);
  return {source, operations, eliminatedClones:clones.observations, allowances,
    model:'pinned-meta-language-owned-values/v1', originalBytes:Buffer.byteLength(original)};
}
