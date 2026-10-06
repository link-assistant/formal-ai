// Stated weekday offsets for the calendar relation handler (issue #1176).
//
// Mirrors detect_offset / render_offset_answer in
// rust/src/solver_handlers/calendar.rs: "100 days after Monday" shifts by the
// stated 100 days (mod 7), "2 weeks before Friday" by 14, and a bare "the day
// after X" states no offset and keeps the ±1 reading. The unit surfaces come
// from the `calendar_day` / `calendar_week` meanings and spelled numerals from
// the `cardinal_number_word` role, both in data/seed; the derivation prose is
// the `calendar_weekday_offset_*` templates in
// data/seed/multilingual-responses-quantities.lino.

/** Spanish weekday names for the offset templates (Rust `Weekday::es`). */
const CALENDAR_OFFSET_SPANISH_WEEKDAYS = {
  monday: "lunes",
  tuesday: "martes",
  wednesday: "miércoles",
  thursday: "jueves",
  friday: "viernes",
  saturday: "sábado",
  sunday: "domingo",
};

/**
 * The seed-localized response text for an intent: the language's own entry,
 * then the `unknown` slot, then English (Rust `seed::localized_response`).
 * @param {string} intent
 * @param {string} language
 * @returns {string|null}
 */
function calendarOffsetLocalizedResponse(intent, language) {
  const table = MULTILINGUAL_ANSWERS[intent];
  if (!table) return null;
  const raw = table[language] || table.unknown || table.en;
  if (!raw) return null;
  return typeof raw === "string" ? raw : String(raw.text || "") || null;
}

/**
 * Is the character a CJK unified ideograph (Rust `is_cjk_character`)?
 * @param {string} character
 * @returns {boolean}
 */
function calendarOffsetIsCjk(character) {
  return /[一-鿿]/u.test(character);
}

/**
 * Numeral tokens are letters and digits, except CJK unit characters, which
 * start the unit word instead of continuing the numeral.
 * @param {string} character
 * @returns {boolean}
 */
function calendarOffsetIsNumeralCharacter(character) {
  return /[\p{Alphabetic}\p{N}]/u.test(character) && !calendarOffsetIsCjk(character);
}

/**
 * Does `text` begin with `needle` as a standalone term (Rust
 * `starts_with_term`)?
 * @param {string} text
 * @param {string} needle
 * @returns {boolean}
 */
function calendarOffsetStartsWithTerm(text, needle) {
  if (!text.startsWith(needle)) return false;
  if (Array.from(needle).some(calendarOffsetIsCjk)) return true;
  const after = Array.from(text.slice(needle.length))[0] || "";
  return after === "" || !isCalendarWordCharacter(after);
}

/**
 * Offsets in `haystack` where `needle` occurs, token-bounded for alphabetic
 * needles outside unspaced scripts (Rust quantities `matching_offsets`).
 * @param {string} haystack
 * @param {string} needle
 * @returns {number[]}
 */
function calendarOffsetMatchingOffsets(haystack, needle) {
  const out = [];
  if (!needle) return out;
  const tokenBounded = /^[\p{Alphabetic}\s]+$/u.test(needle);
  const unspaced = /[㐀-鿿]/u.test(needle);
  let index = haystack.indexOf(needle);
  while (index !== -1) {
    let keep = true;
    if (tokenBounded && !unspaced) {
      const before = Array.from(haystack.slice(0, index)).pop() || "";
      const after = Array.from(haystack.slice(index + needle.length))[0] || "";
      keep =
        (before === "" || !/[\p{Alphabetic}\p{N}]/u.test(before)) &&
        (after === "" || !/[\p{Alphabetic}\p{N}]/u.test(after));
    }
    if (keep) out.push(index);
    index = haystack.indexOf(needle, index + needle.length);
  }
  return out;
}

/**
 * Every quantity the prompt states — ASCII digit runs plus spelled numerals
 * of `language` from the seed — in order, overlaps removed (Rust
 * `verifiable_task::quantities::extract_quantities`, value and offset only).
 * @param {string} text
 * @param {string} language
 * @returns {{value: string, start: number, end: number}[]}
 */
function calendarOffsetQuantities(text, language) {
  const found = [];
  const digitPattern = /[0-9]+/g;
  let match = digitPattern.exec(text);
  while (match) {
    found.push({ value: match[0], start: match.index, end: match.index + match[0].length });
    match = digitPattern.exec(text);
  }
  const lowered = text.toLowerCase();
  for (const meaning of meaningsWithRole(ROLE_CARDINAL_NUMBER_WORD)) {
    const value = meaning.words.find((word) => /^[0-9]+$/.test(word));
    if (!value) continue;
    for (const lexeme of meaning.lexemes) {
      if (lexeme.language !== language) continue;
      for (const surface of lexeme.words) {
        if (!/\p{Alphabetic}/u.test(surface)) continue;
        const needle = surface.toLowerCase();
        for (const start of calendarOffsetMatchingOffsets(lowered, needle)) {
          found.push({ value, start, end: start + needle.length });
        }
      }
    }
  }
  const sorted = found.slice().sort((left, right) =>
    left.start !== right.start ? left.start - right.start : right.end - left.end,
  );
  const unique = [];
  for (const candidate of sorted) {
    const previous = unique[unique.length - 1];
    if (previous && candidate.start < previous.end) continue;
    unique.push(candidate);
  }
  return unique;
}

/**
 * Read the offset a prompt states as `<numeral> days/weeks` (Rust
 * `detect_offset`). Returns null when the prompt states no offset.
 * @param {string} normalized
 * @param {string} language
 * @returns {{count: number, multiplier: number, total: number}|null}
 */
function detectCalendarOffset(normalized, language) {
  const units = [];
  const unitSlugs = [
    { slug: "calendar_day", multiplier: 1 },
    { slug: "calendar_week", multiplier: 7 },
  ];
  for (const unit of unitSlugs) {
    const meaning = meaningLexicon().find((candidate) => candidate.slug === unit.slug);
    if (!meaning) continue;
    for (const word of meaning.words) {
      units.push({ multiplier: unit.multiplier, surface: word });
    }
  }
  // Longest surfaces first so "weeks" is not read as the tail of "week"
  // (stable, like Rust's sort_by on byte length).
  const encoder = new TextEncoder();
  const ordered = units
    .map((unit, index) => ({ unit, index, bytes: encoder.encode(unit.surface).length }))
    .sort((left, right) => right.bytes - left.bytes || left.index - right.index)
    .map((entry) => entry.unit);
  for (const quantity of calendarOffsetQuantities(normalized, language)) {
    if (!/^[0-9]+$/.test(quantity.value)) continue;
    // A numeral far past any calendar horizon is not an offset.
    if (quantity.value.replace(/^0+/, "").length > 12) continue;
    const count = Number(quantity.value);
    let tokenEnd = normalized.length;
    const chars = Array.from(normalized.slice(quantity.start));
    let cursor = quantity.start;
    for (const character of chars) {
      if (!calendarOffsetIsNumeralCharacter(character)) {
        tokenEnd = cursor;
        break;
      }
      cursor += character.length;
    }
    const tail = normalized.slice(tokenEnd).trimStart();
    for (const unit of ordered) {
      if (calendarOffsetStartsWithTerm(tail, unit.surface)) {
        return { count, multiplier: unit.multiplier, total: count * unit.multiplier };
      }
    }
  }
  return null;
}

/**
 * Shift a weekday by a signed day count, wrapping in the seven-day cycle.
 * @param {object} weekday a WEEKDAY_CYCLE entry
 * @param {number} days
 * @returns {object}
 */
function shiftWeekdayBy(weekday, days) {
  const length = WEEKDAY_CYCLE.length;
  const index = WEEKDAY_CYCLE.indexOf(weekday);
  return WEEKDAY_CYCLE[(((index + days) % length) + length) % length];
}

/**
 * The weekday's plain name for answer prose (Rust `weekday_label`).
 * @param {string} language
 * @param {object} weekday
 * @returns {string}
 */
function calendarOffsetWeekdayLabel(language, weekday) {
  switch (language) {
    case "ru":
      return weekday.ru;
    case "hi":
      return weekday.hi;
    case "zh":
      return weekday.zh;
    case "es":
      return CALENDAR_OFFSET_SPANISH_WEEKDAYS[weekday.slug] || weekday.en;
    default:
      return weekday.en;
  }
}

/**
 * Render the stated-offset answer from the seed templates (Rust
 * `render_offset_answer`); null when no template exists.
 * @param {string} language
 * @param {string} operation "next" or "previous"
 * @param {object} source
 * @param {object} result
 * @param {{total: number}} offset
 * @returns {string|null}
 */
function renderCalendarOffsetAnswer(language, operation, source, result, offset) {
  const total = Math.abs(offset.total);
  const weeks = Math.floor(total / 7);
  const days = total % 7;
  let intent = "";
  if (days === 0) {
    const one = weeks === 1 ? "one_" : "";
    intent = `calendar_weekday_offset_exact_${one}${operation}`;
  } else {
    intent = `calendar_weekday_offset_split_${operation}`;
  }
  const template = calendarOffsetLocalizedResponse(intent, language);
  if (!template) return null;
  let sourceLabel = calendarOffsetWeekdayLabel(language, source);
  if (language === "ru") {
    sourceLabel = operation === "next" ? source.ruGenitive : source.ruInstrumental;
  }
  return template
    .split("{n}").join(String(total))
    .split("{weeks}").join(String(weeks))
    .split("{days}").join(String(days))
    .split("{source}").join(sourceLabel)
    .split("{source_plain}").join(calendarOffsetWeekdayLabel(language, source))
    .split("{result}").join(calendarOffsetWeekdayLabel(language, result));
}
