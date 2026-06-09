// Single-pass unescape shared by every quote delimiter. The migration only ever
// emits `\\`, `\n` and `\r` inside a quoted body (the chosen delimiter is always
// absent from the text, so an escaped delimiter is never produced), but we accept
// the full escape set so the four parsers decode identically.
function unescapeQuoted(body) {
  let out = '';
  for (let i = 0; i < body.length; i += 1) {
    const ch = body[i];
    if (ch !== '\\') {
      out += ch;
      continue;
    }
    const next = body[i + 1];
    if (next === undefined) {
      out += '\\';
      break;
    }
    i += 1;
    switch (next) {
      case 'n':
        out += '\n';
        break;
      case 'r':
        out += '\r';
        break;
      case 't':
        out += '\t';
        break;
      case '\\':
        out += '\\';
        break;
      case '"':
        out += '"';
        break;
      case "'":
        out += "'";
        break;
      case '`':
        out += '`';
        break;
      case 'x':
        if (body[i + 1] === '2' && body[i + 2] === '7') {
          out += "'";
          i += 2;
        } else {
          out += '\\x';
        }
        break;
      default:
        out += '\\' + next;
        break;
    }
  }
  return out;
}

// Find the index of the unescaped closing delimiter, skipping `\` escape pairs.
function findClosingDelimiter(body, delimiter) {
  for (let i = 0; i < body.length; i += 1) {
    if (body[i] === '\\') {
      i += 1;
      continue;
    }
    if (body[i] === delimiter) {
      return i;
    }
  }
  return -1;
}

export function decodeLinoValue(rawValue) {
  const value = (rawValue ?? '').trim();
  if (value.length === 0) return '';

  const delimiter = value[0];
  if (delimiter === '"' || delimiter === "'" || delimiter === '`') {
    const rest = value.slice(1);
    const close = findClosingDelimiter(rest, delimiter);
    if (close >= 0 && rest.slice(close + 1).trim().length === 0) {
      return unescapeQuoted(rest.slice(0, close));
    }
  }

  if (value === 'unformalized-raw' || value === 'codepoints') {
    return '';
  }

  const codepointPrefix = value.startsWith('unformalized-raw ')
    ? 'unformalized-raw '
    : value.startsWith('codepoints ')
      ? 'codepoints '
      : '';

  if (codepointPrefix) {
    const codePoints = value
      .slice(codepointPrefix.length)
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map((entry) => {
        const codePoint = Number(entry);
        if (!Number.isInteger(codePoint)) {
          throw new Error(`Invalid unformalized-raw code point: ${entry}`);
        }
        return codePoint;
      });
    return String.fromCodePoint(...codePoints);
  }

  return value;
}

// Tokenize a `("a" "b c" d)` reference list into its individual items. Each
// item is either a quoted scalar (which may contain spaces) or a bare
// whitespace-delimited token. This is the canonical multi-value form that
// replaced the legacy `"a|b|c"` pipe packing (issue #398, defect #4).
export function splitReferenceList(value) {
  const trimmed = (value ?? '').trim();
  if (!trimmed.startsWith('(') || !trimmed.endsWith(')')) {
    return trimmed.length === 0 ? [] : [trimmed];
  }
  const body = trimmed.slice(1, -1);
  const tokens = [];
  let i = 0;
  while (i < body.length) {
    const character = body[i];
    if (/\s/.test(character)) {
      i += 1;
      continue;
    }
    if (character === '"' || character === "'" || character === '`') {
      const quote = character;
      i += 1;
      let item = '';
      while (i < body.length) {
        if ((quote === '"' || quote === '`') && body[i] === '\\') {
          item += body[i + 1] ?? '';
          i += 2;
          continue;
        }
        if (body[i] === quote) {
          i += 1;
          break;
        }
        item += body[i];
        i += 1;
      }
      tokens.push(item);
    } else {
      let item = '';
      while (i < body.length && !/\s/.test(body[i])) {
        item += body[i];
        i += 1;
      }
      if (item.length > 0) tokens.push(item);
    }
  }
  return tokens;
}

export function parseLinoEntry(line, indent, keyword) {
  const pattern = new RegExp(`^ {${indent}}${keyword}(?:\\s+(.+))?$`);
  const match = line.match(pattern);
  if (!match) return null;
  return decodeLinoValue(match[1] ?? '');
}

export function parseLinoField(line, indent) {
  const pattern = new RegExp(`^ {${indent}}([a-z_][a-z0-9_]*)(?:\\s+(.+))?$`);
  const match = line.match(pattern);
  if (!match) return null;
  return { key: match[1], value: decodeLinoValue(match[2] ?? '') };
}

export function parseSupportedLanguagesFromAgentInfo(text) {
  let inSupportedLanguagesField = false;

  for (const line of text.split(/\r?\n/)) {
    const field = parseLinoEntry(line, 2, 'field');
    if (field !== null) {
      inSupportedLanguagesField = field === 'supported_languages';
      continue;
    }

    if (!inSupportedLanguagesField) continue;

    const value = parseLinoEntry(line, 4, 'value');
    if (value !== null) {
      return splitReferenceList(value).filter(Boolean);
    }
  }

  throw new Error('data/seed/agent-info.lino is missing supported_languages');
}
