// Creative composition, part two (issue #1178): constrained verse and
// itinerary planning, browser twin (PR #1188 JavaScript-first parity).
//
// Mirrors rust/src/solver_handlers/creative_writing.rs and its include!-ed
// half creative_writing_planning.rs. The caller supplies cited rhyme classes
// and itinerary observations alongside the request. Composition and
// feasibility checks use the same observations.

/**
 * The `writing_form` records: defaults and the surfaces naming the form.
 * @returns {Array<{form: string, defaultLines: number, defaultScheme: string, syllables: string, surfaces: Array<string>}>}
 */
function creativeWritingForms() {
  const fields = ["form", "default_lines", "default_scheme", "syllables"];
  return creativeRecords("writing_form")
    .map((record) => {
      const lines = creativeParseUnsigned(childValue(record, "default_lines"));
      return {
        form: childValue(record, "form"),
        defaultLines: lines === null ? 4 : lines,
        defaultScheme: childValue(record, "default_scheme"),
        syllables: childValue(record, "syllables"),
        surfaces: record.children
          .filter((child) => !fields.includes(child.name))
          .map((child) => child.value)
          .filter((surface) => surface.length > 0),
      };
    })
    .filter((form) => form.form.length > 0);
}

function suppliedCreativeRhymes(prompt, language) {
  const evidence = creativeEvidence(prompt, "rhyme");
  if (creativeEvidenceText(evidence, "language") !== language) return { scheme: null, classes: [] };
  const candidate = creativeEvidenceText(evidence, "scheme");
  const scheme = candidate && /^[a-z]{1,14}$/i.test(candidate) ? candidate.toLowerCase() : null;
  const classes = [];
  if (!Array.isArray(evidence.classes) || evidence.classes.length > 32) return { scheme, classes };
  for (const record of evidence.classes) {
    const id = creativeEvidenceText(record, "id");
    const sourceUrl = creativeEvidenceSource(record);
    if (!id || !sourceUrl || !Array.isArray(record.words) || record.words.length > 64
      || classes.some((rhyme) => rhyme.id === id)) continue;
    const words = [...new Set(record.words.filter((word) => typeof word === "string")
      .map((word) => word.trim().toLowerCase())
      .filter((word) => word && !/\s/u.test(word) && creativeEndWord(word) === word))];
    if (words.length >= 2 && !classes.some((rhyme) => rhyme.words.some((word) => words.includes(word)))) {
      classes.push({ id, sourceUrl, words });
    }
  }
  return { scheme, classes };
}

/**
 * The `poem_line_skeleton` records of one language.
 * @param {string} language
 * @returns {Array<{text: string, rhymeSlot: boolean}>}
 */
function creativeSkeletons(language) {
  return creativeRecords("poem_line_skeleton")
    .filter((record) => childValue(record, "language") === language)
    .map((record) => ({ text: childValue(record, "text"), rhymeSlot: childValue(record, "rhyme_slot") === "true" }))
    .filter((skeleton) => skeleton.text.length > 0);
}

/**
 * The named verse form, else the plain poem form.
 * @param {string} normalized
 * @returns {{form: string, defaultLines: number, defaultScheme: string, syllables: string, surfaces: Array<string>}}
 */
function creativeDetectForm(normalized) {
  const forms = creativeWritingForms();
  const named = forms.find((form) => form.surfaces.some((surface) => normalized.includes(surface)));
  if (named !== undefined) return named;
  const poem = forms.find((form) => form.form === "poem");
  if (poem !== undefined) return poem;
  return { form: "poem", defaultLines: 4, defaultScheme: "abcb", syllables: "", surfaces: [] };
}

/**
 * Every constraint the request states, defaults filling the rest.
 * @param {string} normalized
 * @param {string} language
 * @returns {{lines: number, scheme: string, topic: string, rhymes: boolean, syllables: string}}
 */
function creativeFormConstraints(normalized, language, supplied) {
  const form = creativeDetectForm(normalized);
  const lineWords = creativeCueWords("line_count");
  const counted = creativeRequestedCount(normalized, language);
  const stated = counted !== null && (lineWords.some((word) => normalized.includes(word)) || containsCjk(normalized))
    ? counted
    : null;
  const lines = Math.min(14, Math.max(1, stated === null ? form.defaultLines : stated));
  const statedScheme = creativeCueWords("stated_scheme").find((scheme) => normalized.split(/\s+/u).includes(scheme));
  let scheme = form.defaultScheme;
  if (statedScheme !== undefined || supplied.scheme !== null) {
    scheme = statedScheme ?? supplied.scheme;
  } else if (creativeCueWords("rhyme_request").some((word) => normalized.includes(word))) {
    scheme = creativeCueField("rhyme_request", "scheme") || form.defaultScheme;
  }
  const rhymes = scheme !== "none" && supplied.classes.length > 0;
  const topics = creativeTopicWords(normalized, language);
  return {
    lines: lines,
    scheme: scheme,
    topic: topics.length > 0 ? topics[topics.length - 1] : "",
    rhymes: rhymes,
    syllables: form.syllables,
  };
}

/**
 * The lowercased end word of a verse line.
 * @param {string} line
 * @returns {string}
 */
function creativeEndWord(line) {
  const tokens = creativeTokens(line);
  return tokens.length === 0 ? "" : creativeTrimToken(tokens[tokens.length - 1]).toLowerCase();
}

/**
 * The alphabetic letters of a rhyme scheme.
 * @param {string} scheme
 * @returns {Array<string>}
 */
function creativeSchemeLetters(scheme) {
  return Array.from(scheme).filter((character) => /\p{Alphabetic}/u.test(character));
}

/**
 * Verify a poem against its scheme (`check_scheme`): repeating letters must
 * share a rhyme class, different repeating letters must not.
 * @param {Array<string>} poem
 * @param {string} scheme
 * @param {Array<{id: string, words: Array<string>}>} classes
 * @returns {Array<string>}
 */
function creativeCheckScheme(poem, scheme, classes) {
  const letters = creativeSchemeLetters(scheme);
  if (letters.length !== poem.length) {
    return [`scheme ${scheme} names ${letters.length} positions but the poem has ${poem.length} lines`];
  }
  const classOf = (word) => {
    const found = classes.find((rhyme) => rhyme.words.includes(word));
    return found === undefined ? null : found.id;
  };
  const repeats = (letter) => letters.filter((other) => other === letter).length;
  const failures = [];
  for (let i = 0; i < letters.length; i += 1) {
    if (repeats(letters[i]) < 2) continue;
    for (let j = i + 1; j < letters.length; j += 1) {
      if (repeats(letters[j]) < 2) continue;
      const wordI = creativeEndWord(poem[i]);
      const wordJ = creativeEndWord(poem[j]);
      const classI = classOf(wordI);
      const classJ = classOf(wordJ);
      if (letters[i] === letters[j]) {
        if (classI !== null && classJ !== null && classI !== classJ) {
          failures.push(`positions ${i + 1} and ${j + 1} must rhyme but '${wordI}' (${classI}) and '${wordJ}' (${classJ}) are different classes`);
        } else if (classI === null || classJ === null) {
          failures.push(`position ${i + 1} or ${j + 1} ends in a word outside every rhyme class`);
        }
      } else if (classI !== null && classJ !== null && classI === classJ) {
        failures.push(`positions ${i + 1} and ${j + 1} must NOT rhyme but both end in class ${classI}`);
      }
    }
  }
  return failures;
}

/**
 * Fill a skeleton's `{topic}` and `{rhyme}` slots.
 * @param {string} skeleton
 * @param {string} topic
 * @param {string} rhyme
 * @returns {string}
 */
function creativeFillSkeleton(skeleton, topic, rhyme) {
  return skeleton.split("{topic}").join(topic).split("{rhyme}").join(rhyme);
}

/**
 * One composition attempt over the rhyme classes rotated by `offset`
 * (`compose_poem`).
 * @param {{lines: number, scheme: string, topic: string, rhymes: boolean}} constraints
 * @param {string} language
 * @param {number} offset
 * @returns {Array<string>|null}
 */
function creativeComposePoem(constraints, language, classes, offset) {
  const all = creativeSkeletons(language);
  if (all.length === 0 || constraints.topic.length === 0) return null;
  const free = all.filter((skeleton) => !skeleton.rhymeSlot);
  if (!constraints.rhymes) {
    if (free.length === 0) return null;
    const unrhymed = [];
    for (let index = 0; index < constraints.lines; index += 1) {
      unrhymed.push(index < free.length
        ? creativeFillSkeleton(free[index].text, constraints.topic, "")
        : constraints.topic);
    }
    return unrhymed;
  }
  const topic = constraints.topic.toLowerCase();
  const rhymed = all.filter((skeleton) => skeleton.rhymeSlot);
  const letters = creativeSchemeLetters(constraints.scheme);
  const counts = new Map();
  for (const letter of letters) counts.set(letter, (counts.get(letter) || 0) + 1);
  const groups = [...counts].filter(([, count]) => count > 1)
    .sort(([left, a], [right, b]) => b - a || left.localeCompare(right));
  const start = offset % classes.length;
  const available = [...classes.slice(start), ...classes.slice(0, start)];
  available.sort((left, right) => Number(right.words.includes(topic)) - Number(left.words.includes(topic)));
  const endings = new Map();
  for (const [letter, count] of groups) {
    const index = available.findIndex((rhyme) => rhyme.words.length >= count);
    if (index < 0) return null;
    const words = available.splice(index, 1)[0].words.slice();
    words.sort((left, right) => Number(right === topic) - Number(left === topic));
    endings.set(letter, words.reverse());
  }
  const poem = [];
  for (const [index, letter] of letters.entries()) {
    const word = endings.get(letter)?.pop() || "";
    if (word && rhymed.length) {
      poem.push(creativeFillSkeleton(rhymed[index % rhymed.length].text, constraints.topic, word));
    } else {
      if (!free.length) return null;
      const line = creativeFillSkeleton(free[index % free.length].text, constraints.topic, "");
      poem.push(word ? `${line} — ${word}` : line);
    }
  }
  return poem;
}

/**
 * Check a rendered poem against every stated constraint (`check_poem`).
 * @param {Array<string>} poem
 * @param {{lines: number, scheme: string, topic: string, rhymes: boolean}} constraints
 * @returns {Array<string>}
 */
function creativeCheckPoem(poem, constraints, classes) {
  const failures = [];
  if (poem.length !== constraints.lines) {
    failures.push(`line count ${poem.length} but ${constraints.lines} wanted`);
  }
  if (poem.some((line) => line.trim().length === 0)) failures.push("an empty line");
  const topicLower = constraints.topic.toLowerCase();
  if (constraints.topic.length > 0 && !poem.some((line) => line.toLowerCase().includes(topicLower))) {
    failures.push(`topic word '${constraints.topic}' absent`);
  }
  if (constraints.rhymes && constraints.scheme !== "none") {
    for (const failure of creativeCheckScheme(poem, constraints.scheme, classes)) {
      failures.push(failure);
    }
  }
  return failures;
}

/**
 * Compose verse under the request's form constraints and return it only
 * after it passes the constraint check (`handle_creative_writing_request`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryCreativeWritingRequest(prompt, normalized) {
  const request = creativeRequestText(prompt);
  const normal = normalizePrompt(request);
  if (!lexiconMentionsRole(CREATIVE_ROLE_WRITING, normal)) return null;
  const language = detectLanguage(request);
  const supplied = suppliedCreativeRhymes(prompt, language);
  const constraints = creativeFormConstraints(normal, language, supplied);
  const trace = [
    `creative_writing:constraints:lines ${constraints.lines} scheme ${constraints.scheme} topic ${constraints.topic} rhymes ${constraints.rhymes} syllables ${constraints.syllables}`,
  ];
  const answer = (body, confidence) =>
    creativeAnswer("creative_writing", "creative_writing", body, confidence, trace, language);
  const refusal = (topic) => answer(creativeTemplate("creative_writing_refusal", [
    ["lines_want", String(constraints.lines)],
    ["scheme", constraints.scheme],
    ["topic", topic],
  ]), 0.4);
  if (constraints.topic.length === 0) {
    trace.push("creative_writing:refusal:no topic");
    return refusal("none stated");
  }
  let poem = null;
  let failures = [];
  for (let offset = 0; offset < 2; offset += 1) {
    const candidate = creativeComposePoem(constraints, language, supplied.classes, offset);
    if (candidate === null) continue;
    const candidateFailures = creativeCheckPoem(candidate, constraints, supplied.classes);
    if (candidateFailures.length === 0) {
      poem = candidate;
      failures = [];
      break;
    }
    if (offset === 0) failures = candidateFailures;
  }
  if (poem === null) {
    trace.push(`creative_writing:refusal:constraint check failed: ${failures.join("; ")}`);
    return refusal(constraints.topic);
  }
  trace.push(`creative_writing:composed:${poem.length}`);
  const rendered = poem.join("\n");
  const counts = [
    ["poem", rendered],
    ["lines_got", String(poem.length)],
    ["lines_want", String(constraints.lines)],
    ["lines_check", "holds"],
  ];
  if (constraints.rhymes) {
    const ends = new Set(poem.map(creativeEndWord));
    const sources = supplied.classes.filter((rhyme) => rhyme.words.some((word) => ends.has(word)))
      .map((rhyme) => `${rhyme.id} (${rhyme.sourceUrl})`).join(", ");
    return answer(creativeTemplate("creative_writing_poem", counts.concat([
      ["scheme", constraints.scheme],
      ["rhyme_check", "holds — every rhyming pair ends in one rhyme class"],
      ["topic_check", "yes"],
      ["rhyme_sources", sources],
    ])), 0.6);
  }
  if (constraints.scheme === "none") {
    return answer(creativeTemplate("creative_writing_unrhymed", counts.concat([
      ["topic_check", "yes"],
      ["syllables", constraints.syllables.length === 0 ? "none stated" : constraints.syllables],
    ])), 0.6);
  }
  return answer(creativeTemplate("creative_writing_gap", counts.concat([
    ["topic_check", "yes"],
  ])), 0.6);
}

// ---------------------------------------------------------------------------
// Planning
// ---------------------------------------------------------------------------

/**
 * Parse "HH:MM" into minutes past midnight.
 * @param {string} text
 * @returns {number|null}
 */
function creativeParseClock(text) {
  const colon = String(text).indexOf(":");
  if (colon === -1) return null;
  const hours = creativeParseUnsigned(text.slice(0, colon).trim());
  const minutes = creativeParseUnsigned(text.slice(colon + 1).trim());
  if (hours === null || minutes === null) return null;
  return (hours < 24 && minutes < 60) || (hours === 24 && minutes === 0) ? hours * 60 + minutes : null;
}

/**
 * Render minutes past midnight as "HH:MM".
 * @param {number} minutes
 * @returns {string}
 */
function creativeFormatClock(minutes) {
  const hours = String(Math.floor(minutes / 60)).padStart(2, "0");
  return `${hours}:${String(minutes % 60).padStart(2, "0")}`;
}

/**
 * The opening and closing minutes of an "HH:MM-HH:MM" window, with
 * fallbacks for unparsable ends ("always open").
 * @param {string} hours
 * @param {number} open
 * @param {number} close
 * @returns {{open: number, close: number}}
 */
function creativeWindow(hours, open, close) {
  const parts = String(hours).split("-");
  const first = creativeParseClock(parts[0]);
  const last = creativeParseClock(parts[parts.length - 1]);
  return { open: first === null ? open : first, close: last === null ? close : last };
}

function suppliedCreativePlaces(prompt) {
  const evidence = creativeEvidence(prompt, "planning");
  const destination = creativeEvidenceText(evidence, "destination");
  if (!destination || !Array.isArray(evidence.places) || evidence.places.length > 64) return null;
  const places = evidence.places.flatMap((record) => {
    if (!record || Object.hasOwn(record, "closed_days")) return [];
    const name = creativeEvidenceText(record, "name");
    const district = creativeEvidenceText(record, "district");
    const category = creativeEvidenceText(record, "category");
    const hours = creativeEvidenceText(record, "hours");
    const window = hours?.split("-").map(creativeParseClock);
    if (window?.length !== 2 || window.some((time) => time === null)) return [];
    const [open, close] = window;
    const visitMinutes = record.visit_minutes;
    const sourceUrl = creativeEvidenceSource(record);
    return name && district && category && sourceUrl && Number.isInteger(visitMinutes)
      && visitMinutes > 0 && visitMinutes <= close - open
      ? [{ name, district, category, hours, open, close, visitMinutes, sourceUrl }] : [];
  });
  return places.length ? { destination, places } : null;
}

/**
 * The `plan_constraint` record: day and lunch windows, travel times, cap.
 * @returns {object}
 */
function creativePlanConstraints() {
  const limits = {
    travelIntra: 20,
    travelCross: 40,
    lunchStart: 13 * 60,
    lunchEnd: 14 * 60,
    dayStart: 9 * 60,
    dayEnd: 19 * 60,
    maxItems: 4,
    note: "",
  };
  const record = creativeRecords("plan_constraint")[0];
  if (record === undefined) return limits;
  const number = (name, fallback) => {
    const parsed = creativeParseUnsigned(childValue(record, name));
    return parsed === null ? fallback : parsed;
  };
  const day = creativeWindow(childValue(record, "day_window"), limits.dayStart, limits.dayEnd);
  const lunch = creativeWindow(childValue(record, "lunch_window"), limits.lunchStart, limits.lunchEnd);
  return {
    travelIntra: number("travel_intra_district_minutes", limits.travelIntra),
    travelCross: number("travel_cross_district_minutes", limits.travelCross),
    lunchStart: lunch.open,
    lunchEnd: lunch.close,
    dayStart: day.open,
    dayEnd: day.close,
    maxItems: number("max_items_per_day", limits.maxItems),
    note: childValue(record, "note"),
  };
}

/**
 * Schedule places across `days` days (`schedule_days`): whole districts
 * round-robin in input order, early-closing and long visits first, travel,
 * one lunch window, the daily cap and the category spread enforced.
 * @param {Array<object>} available
 * @param {number} days
 * @param {object} limits
 * @returns {{schedule: Array<Array<object>>, skips: Array<string>}}
 */
function creativeScheduleDays(available, days, limits) {
  const skips = [];
  const districts = [];
  for (const place of available) {
    if (!districts.includes(place.district)) districts.push(place.district);
  }
  const plans = [];
  for (let day = 0; day < Math.max(days, 1); day += 1) plans.push([]);
  districts.forEach((district, index) => {
    const bucket = available.filter((place) => place.district === district);
    bucket.sort((left, right) => {
      return left.close - right.close || right.visitMinutes - left.visitMinutes;
    });
    bucket.forEach((place, slot) => {
      plans[(index + slot) % plans.length].push(place);
    });
  });
  const schedule = [];
  for (const planned of plans) {
    const dayPlaces = planned.slice(0, limits.maxItems);
    for (const place of planned.slice(limits.maxItems)) skips.push(`${place.name} (over the daily cap)`);
    const items = [];
    let cursor = limits.dayStart;
    let previousDistrict = null;
    for (const place of dayPlaces) {
      let travel = 0;
      if (previousDistrict !== null) {
        if (items.length > 0 && items[items.length - 1].place.category === place.category) {
          skips.push(`${place.name} (would repeat the ${place.category} category consecutively)`);
          continue;
        }
        travel = previousDistrict === place.district ? limits.travelIntra : limits.travelCross;
      }
      let start = Math.max(cursor + travel, place.open);
      if (start < limits.lunchEnd && start + place.visitMinutes > limits.lunchStart) {
        start = limits.lunchEnd;
      }
      const end = start + place.visitMinutes;
      if (end > Math.min(place.close, limits.dayEnd)) {
        skips.push(`${place.name} (would run past its ${place.hours} closing)`);
        continue;
      }
      items.push({ start: start, end: end, place: place });
      cursor = end;
      previousDistrict = place.district;
    }
    schedule.push(items);
  }
  return { schedule: schedule, skips: skips };
}

/**
 * Verify a schedule (`check_schedule`): cap, travel, category spread, hours.
 * @param {Array<Array<object>>} schedule
 * @param {object} limits
 * @returns {Array<string>}
 */
function creativeCheckSchedule(schedule, limits) {
  const failures = [];
  schedule.forEach((items, dayIndex) => {
    const day = dayIndex + 1;
    if (items.length > limits.maxItems) {
      failures.push(`day ${day} carries ${items.length} items, over the cap of ${limits.maxItems}`);
    }
    for (let index = 0; index + 1 < items.length; index += 1) {
      const first = items[index];
      const second = items[index + 1];
      const travel = first.place.district === second.place.district ? limits.travelIntra : limits.travelCross;
      if (second.start < first.end + travel) {
        failures.push(`day ${day}: ${second.place.name} starts before ${first.place.name} ends plus travel`);
      }
      if (first.place.category === second.place.category) {
        failures.push(`day ${day}: two consecutive ${first.place.category} items`);
      }
    }
    for (const item of items) {
      if (item.start < item.place.open || item.end > item.place.close) {
        failures.push(`${item.place.name} falls outside ${item.place.hours}`);
      }
    }
  });
  return failures;
}

/**
 * Answer a planning request with a cited, feasibility-checked schedule
 * (`handle_planning_request`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryPlanningRequest(prompt, normalized) {
  const request = creativeRequestText(prompt);
  const normal = normalizePrompt(request);
  if (!lexiconMentionsRole(CREATIVE_ROLE_PLANNING, normal)) return null;
  const language = detectLanguage(request);
  const trace = [];
  const answer = (body, confidence) => creativeAnswer("planning", "planning_request", body, confidence, trace, language);
  const supplied = suppliedCreativePlaces(prompt);
  if (supplied === null) {
    trace.push("planning:refusal:missing_places");
    return answer(creativeTemplate("planning_refusal", []), 0.4);
  }
  const { destination, places: available } = supplied;
  trace.push(`planning:destination:${destination}`);
  const requested = creativeRequestedCount(normal, language);
  const days = Math.min(7, Math.max(1, requested === null ? 3 : requested));
  trace.push(`planning:days:${days}`);
  const limits = creativePlanConstraints();
  for (const place of available) {
    trace.push(`planning:place:${place.name} (${place.visitMinutes} min, ${place.hours})`);
  }
  const planned = creativeScheduleDays(available, days, limits);
  for (const skip of planned.skips) trace.push(`planning:skipped:${skip}`);
  if (planned.schedule.every((day) => day.length === 0)) {
    trace.push("planning:refusal:no_feasible_visit");
    return answer(creativeTemplate("planning_no_fit", []), 0.4);
  }
  const failures = creativeCheckSchedule(planned.schedule, limits);
  if (failures.length > 0) {
    trace.push(`planning:infeasible:${failures.join("; ")}`);
    return answer(creativeTemplate("planning_infeasible", [
      ["days", String(days)],
      ["destination", destination],
      ["failures", failures.join("; ")],
    ]), 0.4);
  }
  const rendered = planned.schedule.map((items, index) => {
    const lines = items.map((item) =>
      `  ${creativeFormatClock(item.start)}-${creativeFormatClock(item.end)} ${item.place.name} (${item.place.category}, ${item.place.district}) — ${item.place.sourceUrl}`);
    return `Day ${index + 1}\n${lines.join("\n")}`;
  }).join("\n\n");
  const feasibility = `no overlapping items; travel time accounted between consecutive items (${limits.travelIntra} min intra-district, ${limits.travelCross} min cross-district); every item inside its opening hours; at most ${limits.maxItems} items a day`;
  return answer(creativeTemplate("planning_itinerary", [
    ["days", String(days)],
    ["destination", destination],
    ["schedule", rendered],
    ["feasibility", feasibility],
    ["input_note", limits.note],
    ["skips", planned.skips.join("; ")],
  ]), 0.6);
}
