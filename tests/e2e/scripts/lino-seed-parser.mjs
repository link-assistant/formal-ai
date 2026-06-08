export function decodeLinoValue(rawValue) {
  const value = (rawValue ?? '').trim();
  if (value.length === 0) return '';

  if (value.startsWith('"')) {
    return JSON.parse(value);
  }

  if (value.startsWith("'") && value.endsWith("'")) {
    return value.slice(1, -1);
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
      return value.split('|').filter(Boolean);
    }
  }

  throw new Error('data/seed/agent-info.lino is missing supported_languages');
}
