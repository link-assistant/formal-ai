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
 * @param {string|null} language Null accepts every registered numeral language.
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
      if (language !== null && lexeme.language !== language) continue;
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
  const found = calendarOffsetStatedCount(normalized, language, units);
  return found
    ? { count: found.count, multiplier: found.unit.multiplier, total: found.count * found.unit.multiplier }
    : null;
}

/**
 * The first stated `<numeral> <unit surface>` pair (Rust
 * `stated_count_with_unit`): the count, the matched unit entry and its
 * surface, or null. Units are tried longest surface first.
 * @param {string} normalized
 * @param {string} language
 * @param {Array<{surface: string}>} units
 * @returns {{count: number, unit: object, surface: string}|null}
 */
function calendarOffsetStatedCount(normalized, language, units) {
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
        return { count, unit, surface: unit.surface };
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
  const sourceLabel = calendarWeekdayDirectionLabel(language, operation, source);
  return template
    .split("{n}").join(String(total))
    .split("{weeks}").join(String(weeks))
    .split("{days}").join(String(days))
    .split("{source}").join(sourceLabel)
    .split("{source_plain}").join(calendarWeekdayLabel(language, source))
    .split("{result}").join(calendarWeekdayLabel(language, result));
}

// Month offsets (issue #1176), mirroring rust/src/solver_handlers/calendar/month.rs:
// "2 months after January" is modulo-12 arithmetic over the
// `calendar_month_name` meanings, and "3 months after Monday" is answered
// with a clarification, since a month is no fixed number of days.

const ROLE_CALENDAR_MONTH_NAME = "calendar_month_name";
const CALENDAR_MONTH_UNIT_SLUG = "calendar_month";
const CALENDAR_MONTH_SLUGS = [
  "january",
  "february",
  "march",
  "april",
  "may",
  "june",
  "july",
  "august",
  "september",
  "october",
  "november",
  "december",
];
const CALENDAR_MONTHS_PER_YEAR = 12;

/**
 * The month number (1-12) a prompt names; the longest surface wins, ties keep
 * the earlier month (Rust `detect_month`).
 * @param {string} normalized
 * @returns {number|null}
 */
function detectCalendarMonth(normalized) {
  const encoder = new TextEncoder();
  let best = null;
  for (const meaning of meaningsWithRole(ROLE_CALENDAR_MONTH_NAME)) {
    const position = CALENDAR_MONTH_SLUGS.indexOf(meaning.slug);
    if (position === -1) continue;
    for (const word of meaning.words) {
      const length = encoder.encode(word).length;
      if ((best === null || length > best.length) && containsCalendarTerm(normalized, word)) {
        best = { length: length, month: position + 1 };
      }
    }
  }
  return best === null ? null : best.month;
}

/**
 * The month's name in the answer language from its seed lexeme.
 * @param {string} slug
 * @param {string} language
 * @returns {string}
 */
function calendarMonthLabel(slug, language) {
  const meaning = findMeaning(slug);
  if (!meaning) return CALENDAR_MONTH_UNIT_SLUG;
  return wordInLanguage(meaning, language) || wordInLanguage(meaning, "en") || CALENDAR_MONTH_UNIT_SLUG;
}

/**
 * Days since 1970-01-01 of a Gregorian date, for years from 2000 on (Rust
 * `days_from_civil`).
 * @param {number} year
 * @param {number} month
 * @param {number} day
 * @returns {number}
 */
function calendarDaysFromCivil(year, month, day) {
  const shiftedYear = month <= 2 ? year - 1 : year;
  const era = Math.floor(shiftedYear / 400);
  const yearOfEra = shiftedYear - era * 400;
  const monthFromMarch = (month + 9) % 12;
  const dayOfYear = Math.floor((153 * monthFromMarch + 2) / 5) + day - 1;
  const dayOfEra = yearOfEra * 365 + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100) + dayOfYear;
  return era * 146097 + dayOfEra - 719468;
}

/**
 * Fewest and most days `months` consecutive months span, from the first of
 * every month over one 400-year Gregorian cycle (Rust `month_span_days`).
 * @param {number} months
 * @returns {{min: number, max: number}}
 */
function calendarMonthSpanDays(months) {
  let min = Infinity;
  let max = -Infinity;
  for (let year = 2000; year < 2400; year += 1) {
    for (let month = 0; month < CALENDAR_MONTHS_PER_YEAR; month += 1) {
      const start = calendarDaysFromCivil(year, month + 1, 1);
      const total = month + months;
      const end = calendarDaysFromCivil(
        year + Math.floor(total / CALENDAR_MONTHS_PER_YEAR),
        (total % CALENDAR_MONTHS_PER_YEAR) + 1,
        1,
      );
      min = Math.min(min, end - start);
      max = Math.max(max, end - start);
    }
  }
  return { min: min, max: max };
}

/**
 * Answer a stated month offset (Rust `try_month_offset`): modulo-12
 * arithmetic from a month name, or a clarification from a weekday; null when
 * the prompt states no month offset, no direction, or no anchor.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryCalendarMonthOffset(prompt, normalized) {
  const meaning = findMeaning(CALENDAR_MONTH_UNIT_SLUG);
  if (!meaning) return null;
  const language = detectLanguage(prompt);
  const found = calendarOffsetStatedCount(
    normalized,
    language,
    meaning.words.map((word) => ({ surface: word })),
  );
  if (!found) return null;
  const operation = detectWeekdayOperation(normalized);
  if (!operation) return null;
  const count = found.count;
  const signed = (operation === "next" ? 1 : -1) * count;
  const signedText = `${signed >= 0 ? "+" : ""}${signed}m`;
  const weekday = detectWeekday(normalized);
  if (weekday) {
    const span = calendarMonthSpanDays(count);
    const template = calendarOffsetLocalizedResponse(
      `calendar_weekday_month_offset_unresolved_${operation}`,
      language,
    );
    const source = calendarWeekdayDirectionLabel(language, operation, weekday);
    const content = template === null
      ? ""
      : template
        .split("{n}").join(String(count))
        .split("{unit}").join(found.surface)
        .split("{source}").join(source)
        .split("{min_days}").join(String(span.min))
        .split("{max_days}").join(String(span.max));
    return {
      intent: "calendar_month_offset_clarification",
      content: content,
      confidence: 1.0,
      evidence: [
        `calendar:subject_weekday:${weekday.slug}`,
        `calendar:operation:${operation}:${weekday.slug}`,
        `calendar:offset:${signedText}`,
        `calendar:offset_span:${span.min}d..${span.max}d`,
        `calendar:offset_unresolved:${CALENDAR_MONTH_UNIT_SLUG}`,
        `language:${language}`,
      ],
    };
  }
  const source = detectCalendarMonth(normalized);
  if (source === null) return null;
  const sum = source + signed;
  const result = ((((sum - 1) % CALENDAR_MONTHS_PER_YEAR) + CALENDAR_MONTHS_PER_YEAR) %
    CALENDAR_MONTHS_PER_YEAR) + 1;
  const sourceSlug = CALENDAR_MONTH_SLUGS[source - 1];
  const resultSlug = CALENDAR_MONTH_SLUGS[result - 1];
  const template = calendarOffsetLocalizedResponse(`calendar_month_offset_${operation}`, language);
  if (template === null) return null;
  const content = template
    .split("{n}").join(String(count))
    .split("{unit}").join(found.surface)
    .split("{source}").join(calendarMonthLabel(sourceSlug, language))
    .split("{source_index}").join(String(source))
    .split("{sum}").join(String(sum))
    .split("{result_index}").join(String(result))
    .split("{result}").join(calendarMonthLabel(resultSlug, language));
  const sign = operation === "next" ? "+" : "-";
  return {
    intent: "calendar_month_relation",
    content: content,
    confidence: 1.0,
    evidence: [
      `calendar:subject_month:${sourceSlug}`,
      `calendar:operation:${operation}:${sourceSlug}`,
      `calendar:offset:${signedText}`,
      `calendar:offset_derivation:${source} ${sign} ${count} = ${sum} ≡ ${result} (mod 12)`,
      `calendar:result_month:${resultSlug}`,
      `language:${language}`,
    ],
  };
}

// The weekday a stated date falls on (R1017), mirroring
// rust/src/solver_handlers/calendar/date_weekday.rs: the question is the
// `calendar_weekday_query` role, the date an ISO-like YYYY-MM-DD read from the
// raw prompt or a seeded month name with a day and a year, and the answer the
// `calendar_date_weekday` template, counting days from 1970-01-01.

const ROLE_CALENDAR_WEEKDAY_QUERY = "calendar_weekday_query";

/**
 * Every run of ASCII digits in `text` with its offsets (Rust `digit_runs`).
 * @param {string} text
 * @returns {Array<{start: number, end: number, value: number}>}
 */
function calendarDigitRuns(text) {
  const runs = [];
  for (const match of String(text || "").matchAll(/[0-9]+/gu)) {
    const value = match[0].length > 18 ? -1 : Number(match[0]);
    runs.push({ start: match.index, end: match.index + match[0].length, value: value });
  }
  return runs;
}

/**
 * The stated date as {year, month, day}, validated, or null (Rust `stated_date`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {{year: number, month: number, day: number}|null}
 */
function calendarStatedDate(prompt, normalized) {
  const text = String(prompt || "");
  const runs = calendarDigitRuns(text);
  const width = (run) => run.end - run.start;
  let date = null;
  if (runs.length === 3) {
    const [year, month, day] = runs;
    const first = text.slice(year.end, month.start);
    const second = text.slice(month.end, day.start);
    if (width(year) === 4 && width(month) <= 2 && width(day) <= 2 && first === second &&
      ["-", "/", "."].includes(first)) {
      date = { year: year.value, month: month.value, day: day.value };
    }
  } else if (runs.length === 2) {
    const month = detectCalendarMonth(normalized);
    const [first, second] = runs;
    if (month !== null && width(first) === 4 && width(second) <= 2) {
      date = { year: first.value, month: month, day: second.value };
    } else if (month !== null && width(first) <= 2 && width(second) === 4) {
      date = { year: second.value, month: month, day: first.value };
    }
  }
  if (date === null || date.year < 1 || date.year > 9999 || date.month < 1 || date.month > 12) {
    return null;
  }
  const next = date.month === 12 ? [date.year + 1, 1] : [date.year, date.month + 1];
  const length = calendarDaysFromCivil(next[0], next[1], 1) - calendarDaysFromCivil(date.year, date.month, 1);
  return date.day >= 1 && date.day <= length ? date : null;
}

/**
 * Answer "which weekday is <date>?" (Rust `try_date_weekday`), or null.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryCalendarDateWeekday(prompt, normalized) {
  const asks = wordsForRole(ROLE_CALENDAR_WEEKDAY_QUERY).some((word) => containsCalendarTerm(normalized, word));
  const direction = [ROLE_CALENDAR_DIRECTION_NEXT, ROLE_CALENDAR_DIRECTION_PREVIOUS].some((role) =>
    wordsForRole(role).some((marker) => normalized.includes(marker)));
  if (!asks || direction) return null;
  const date = calendarStatedDate(prompt, normalized);
  if (date === null) return null;
  const days = calendarDaysFromCivil(date.year, date.month, date.day);
  const weekdayAt = (count) => WEEKDAY_CYCLE[(((count + 3) % 7) + 7) % 7];
  const weekday = weekdayAt(days);
  const iso = `${String(date.year).padStart(4, "0")}-${String(date.month).padStart(2, "0")}-${String(date.day).padStart(2, "0")}`;
  const weeks = Math.floor(days / 7);
  const rest = days - weeks * 7;
  const language = detectLanguage(prompt);
  const template = calendarOffsetLocalizedResponse("calendar_date_weekday", language);
  if (template === null) return null;
  const content = template
    .split("{date}").join(iso)
    .split("{epoch_weekday}").join(calendarWeekdayLabel(language, weekdayAt(0)))
    .split("{epoch}").join("1970-01-01")
    .split("{weekday}").join(calendarWeekdayLabel(language, weekday))
    .split("{days}").join(String(days))
    .split("{weeks}").join(String(weeks))
    .split("{rest}").join(String(rest));
  return {
    intent: "calendar_date_weekday",
    content: content,
    confidence: 1.0,
    evidence: [
      `calendar:date:${iso}`,
      "calendar:epoch:1970-01-01",
      `calendar:days_since_epoch:${days}`,
      `calendar:offset_derivation:${days} = 7 × ${weeks} + ${rest}`,
      `calendar:result_weekday:${weekday.slug}`,
      `language:${language}`,
      "response:calendar_date_weekday",
    ],
  };
}

// Mirrors calendar/date_offset.rs: a day quantity and its date are bound
// on opposite sides of an explicit direction inside a date question.
function tryCalendarDateOffset(prompt, normalized) {
  if (!wordsForRole("calendar-date-offset").some(word => containsCalendarTerm(normalized, word))) return null;
  const lowered = prompt.toLowerCase();
  const language = detectLanguage(prompt);
  const candidates = [];
  for (const [role, sign] of [[ROLE_CALENDAR_DIRECTION_NEXT, 1], [ROLE_CALENDAR_DIRECTION_PREVIOUS, -1]]) {
    for (const word of wordsForRole(role)) {
      const at = lowered.indexOf(word);
      if (at < 0 || !containsCalendarTerm(lowered, word)) continue;
      const prefix = lowered.slice(0, at);
      const anchor = lowered.slice(at + word.length);
      const offset = detectCalendarOffset(prefix, language);
      const date = calendarStatedDate(anchor, anchor);
      if (offset === null || date === null || calendarDigitRuns(prefix).length !== 1) continue;
      const days = calendarDaysFromCivil(date.year, date.month, date.day);
      const signed = sign * offset.total;
      const resultDays = days + signed;
      if (!Number.isSafeInteger(resultDays) || resultDays < calendarDaysFromCivil(1, 1, 1) ||
          resultDays >= calendarDaysFromCivil(10000, 1, 1)) continue;
      const result = new Date(resultDays * 86400000);
      const iso = result.toISOString().slice(0, 10);
      const source = String(date.year).padStart(4, "0") + "-" + String(date.month).padStart(2, "0") + "-" + String(date.day).padStart(2, "0");
      const monthSlug = CALENDAR_MONTH_SLUGS[result.getUTCMonth()];
      const month = handlerRulesTableValue("calendar-date-month-label", monthSlug + "." + language) || calendarMonthLabel(monthSlug, language);
      const template = calendarOffsetLocalizedResponse("calendar-date-offset", language);
      if (template === null) continue;
      const values = {iso, source, signed: (signed >= 0 ? "+" : "") + signed,
        day: result.getUTCDate(), month, year: result.getUTCFullYear()};
      let content = template;
      for (const [key, value] of Object.entries(values)) content = content.split("{" + key + "}").join(String(value));
      candidates.push({intent: "calendar_date_offset", content, confidence: 1.0,
        evidence: ["calendar:date:" + source, "calendar:offset:" + values.signed + "d",
          "calendar:offset_derivation:" + source + " " + values.signed + "d = " + iso,
          "calendar:result_date:" + iso, "response:calendar-date-offset"]});
    }
  }
  const distinct = [...new Map(candidates.map(row => [row.evidence.join("|"), row])).values()];
  return distinct.length === 1 ? distinct[0] : null;
}

// Calendar durations (issue #1589), twin of calendar/month.rs::duration.
// Seed vocabulary binds quantities to calendar units; months never use an
// average number of days. Full periods use completed day-of-month anniversaries.
function calendarDurationWords(role) {
  return (handlerRulesTables()[role]?.rows || []).map((row) => row[1]).sort((a, b) => b.length - a.length);
}
function calendarDurationUnit(text) {
  const units = [];
  for (const [slug, factor] of [["calendar_month", 1], ["calendar_year", 12]]) {
    const meaning = meaningLexicon().find((entry) => entry.slug === slug);
    for (const word of slug === "calendar_year" ? calendarDurationWords("calendar-duration-year") : meaning ? meaning.words : []) units.push({word, factor});
  }
  return units.sort((a, b) => b.word.length - a.word.length)
    .find((unit) => calendarOffsetStartsWithTerm(text, unit.word)) || null;
}
function calendarDurationTrim(text) {
  return text.trim().replace(/[.?!]+$/u, "").trim();
}
function calendarDurationDate(text) {
  const lowered = calendarDurationTrim(text);
  const runs = calendarDigitRuns(lowered);
  const month = detectCalendarMonth(lowered);
  // Reject multiple named months even when one surface is longer than another.
  const months = new Set();
  for (const entry of meaningsWithRole(ROLE_CALENDAR_MONTH_NAME)) {
    if (entry.words.some((word) => containsCalendarTerm(lowered, word))) months.add(entry.slug);
  }
  if (months.size > 1) return null;
  if (runs.length === 1 && runs[0].end - runs[0].start === 4 &&
      runs[0].value >= 1 && runs[0].value <= 9999 && month !== null) {
    return {year:runs[0].value, month, day:1, precision:"month"};
  }
  const date = calendarStatedDate(lowered, lowered);
  return date ? {...date, precision:"day"} : null;
}
function calendarDurationRequest(prompt) {
  const text = String(prompt).toLowerCase().trim();
  const sumCue = calendarDurationWords("calendar-duration-addition-cue")
    .find((word) => calendarOffsetStartsWithTerm(text, word));
  if (sumCue) {
    const body = text.slice(sumCue.length).trim();
    for (const marker of calendarDurationWords("calendar-duration-output")) {
      for (const position of calendarOffsetMatchingOffsets(body, marker)) {
        const targetText = calendarDurationTrim(body.slice(position + marker.length));
        const target = calendarDurationUnit(targetText);
        if (!target || target.word !== targetText) continue;
        let rest = calendarDurationTrim(body.slice(0, position));
        const terms = [];
        let total = 0;
        while (rest && terms.length < 32) {
          const number = /^([+-]?[0-9]{1,12})\s+/u.exec(rest);
          if (!number) break;
          const tail = rest.slice(number[0].length);
          const unit = calendarDurationUnit(tail);
          if (!unit) break;
          const count = Number(number[1]);
          total += count * unit.factor;
          terms.push(count + " × " + unit.factor);
          rest = tail.slice(unit.word.length).trim();
          if (!rest) break;
          let separated = false;
          if (rest.startsWith(",")) {
            separated = true;
            rest = rest.slice(1).trimStart();
          }
          const join = calendarDurationWords("calendar-duration-join")
            .find((word) => calendarOffsetStartsWithTerm(rest, word));
          if (join) { separated = true; rest = rest.slice(join.length).trimStart(); }
          if (!separated || !rest) { rest = "!"; break; }
        }
        if (rest || terms.length < 2 || !Number.isSafeInteger(total)) continue;
        const divisor = target.factor;
        const gcd = (a, b) => b ? gcd(b, a % b) : a;
        const common = gcd(Math.abs(total), divisor);
        const result = divisor === common ? String(total / divisor)
          : (total / common) + "/" + (divisor / common);
        return {kind:"calendar-duration-sum", result, factor:target.factor,
          derivation:terms.join(" + ") + " = " + total + " months; " + total + " / " + divisor + " = " + result};
      }
    }
    return null;
  }
  if (!calendarDurationWords("calendar-full-query").some((word) => calendarOffsetStartsWithTerm(text, word))) return null;
  const factor = calendarDurationWords("calendar-full-years").some((word) => containsCalendarTerm(text, word)) ? 12
    : calendarDurationWords("calendar-full-months").some((word) => containsCalendarTerm(text, word)) ? 1 : null;
  if (factor === null) return null;
  const candidates = [];
  for (const start of calendarDurationWords("calendar-interval-start")) {
    for (const at of calendarOffsetMatchingOffsets(text, start)) {
      const tail = text.slice(at + start.length);
      for (const end of calendarDurationWords("calendar-interval-end")) {
        for (const split of calendarOffsetMatchingOffsets(tail, end)) {
          const left = calendarDurationDate(tail.slice(0, split));
          const right = calendarDurationDate(tail.slice(split + end.length));
          if (!left || !right || left.precision !== right.precision) continue;
          const ordinal = (date) => calendarDaysFromCivil(date.year, date.month, date.day);
          const [first, last] = ordinal(left) <= ordinal(right) ? [left, right] : [right, left];
          const raw = (last.year - first.year) * 12 + last.month - first.month;
          const incomplete = last.day < first.day ? 1 : 0;
          const months = raw - incomplete;
          const result = String(Math.floor(months / factor));
          candidates.push({kind:"calendar-full-periods", result, factor,
            derivation:calendarDurationIso(first) + " → " + calendarDurationIso(last) +
              ": " + raw + " - " + incomplete + " = " + months + " months; ⌊" + months + " / " + factor + "⌋ = " + result});
        }
      }
    }
  }
  return candidates.length === 1 ? candidates[0] : null;
}
function calendarDurationIso(date) {
  return String(date.year).padStart(4, "0") + "-" + String(date.month).padStart(2, "0") + "-" + String(date.day).padStart(2, "0");
}
function calendarDurationLabel(result, factor, language) {
  const number = result.includes("/") ? NaN : Math.abs(Number(result));
  const kind = !Number.isInteger(number) ? "fraction"
    : (language === "ru" ? number % 10 === 1 && number % 100 !== 11 : number === 1) ? "one"
    : number % 10 >= 2 && number % 10 <= 4 && (number % 100 < 12 || number % 100 > 14) ? "few" : "other";
  const key = (factor === 12 ? "year" : "month") + "." + language + "." + kind;
  return handlerRulesTableValue("calendar-duration-label", key) ||
    handlerRulesTableValue("calendar-duration-label", (factor === 12 ? "year" : "month") + ".en." + kind);
}
function tryCalendarDuration(prompt) {
  const request = calendarDurationRequest(prompt);
  if (!request) return null;
  const role = request.kind === "calendar-duration-sum" ? "calendar-duration-addition-cue" : "calendar-full-query";
  const text = String(prompt).toLowerCase().trim();
  const language = (handlerRulesTables()[role]?.rows || [])
    .find(row => calendarOffsetStartsWithTerm(text, row[1]))?.[0] || detectLanguage(prompt);
  const template = calendarOffsetLocalizedResponse(request.kind, language);
  if (!template) return null;
  const content = template.replaceAll("{result}", request.result)
    .replaceAll("{unit}", calendarDurationLabel(request.result, request.factor, language))
    .replaceAll("{derivation}", request.derivation);
  return {intent:request.kind, content, confidence:1,
    evidence:["calendar:duration_derivation:" + request.derivation,
      "calendar:duration_result:" + request.result],
    solverEvents:[{kind:"calendar:duration_derivation", payload:request.derivation},
      {kind:"calendar:duration_result", payload:request.result}]};
}
