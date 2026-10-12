// Creative composition, part one (issue #1178): brainstorming and advice,
// browser twin (PR #1188 JavaScript-first parity).
//
// Mirrors rust/src/solver_handlers/creative_composition.rs. Recognition
// vocabulary is read from data/seed/meanings-creative-tasks.lino through the
// meaning lexicon; the morphology rules, spelled numbers, stop words, request
// cues, distinctness metric and the evidence-graded advice table come from
// data/seed/creative-composition-rules.lino; the prose renders the English
// templates of data/seed/multilingual-responses-creative-tasks.lino. The seed
// records sit under the file's `creative_composition_rules` root, and a
// `number <word> <value>` pair carries its value inline — this twin reads
// both as written.

const CREATIVE_RULES_FILE = "creative-composition-rules.lino";
const CREATIVE_RULES_ROOT = "creative_composition_rules";
const CREATIVE_ROLE_BRAINSTORMING = "brainstorming_request";
const CREATIVE_ROLE_ADVICE = "advice_request";
const CREATIVE_ROLE_WRITING = "creative_writing_request";
const CREATIVE_ROLE_PLANNING = "planning_request";
/** Name-length cap when no request cue states a shorter one. */
const CREATIVE_DEFAULT_MAX_LENGTH = 18;

let creativeRulesCache = { text: null, records: [] };

/**
 * The records under the rules seed's `creative_composition_rules` root.
 * @returns {Array<object>}
 */
function creativeRulesRecords() {
  const text = seedRawText(SEED_RAW, CREATIVE_RULES_FILE) || "";
  if (creativeRulesCache.text === text) return creativeRulesCache.records;
  const tree = parseLinoTree(text);
  const root = tree.children.find((child) => child.name === CREATIVE_RULES_ROOT);
  const records = root === undefined ? tree.children : root.children;
  creativeRulesCache = { text: text, records: records };
  return records;
}

/**
 * Every rules record named `name`, in seed order.
 * @param {string} name
 * @returns {Array<object>}
 */
function creativeRecords(name) {
  return creativeRulesRecords().filter((record) => record.name === name);
}

/**
 * The non-empty values of a record's children named `name`.
 * @param {object} record
 * @param {string} name
 * @returns {Array<string>}
 */
function creativeChildValues(record, name) {
  return record.children
    .filter((child) => child.name === name)
    .map((child) => child.value)
    .filter((value) => value.length > 0);
}

/**
 * The words of the `request_cue` record named `cue`.
 * @param {string} cue
 * @returns {Array<string>}
 */
function creativeCueWords(cue) {
  const record = creativeRecords("request_cue").find((candidate) => childValue(candidate, "cue") === cue);
  return record === undefined ? [] : creativeChildValues(record, "word");
}

/**
 * The value of child `name` on the `request_cue` record named `cue`.
 * @param {string} cue
 * @param {string} name
 * @returns {string}
 */
function creativeCueField(cue, name) {
  const record = creativeRecords("request_cue").find((candidate) => childValue(candidate, "cue") === cue);
  return record === undefined ? "" : childValue(record, name);
}

/**
 * Parse an unsigned decimal the way Rust's `str::parse::<u32>` does.
 * @param {string} text
 * @returns {number|null}
 */
function creativeParseUnsigned(text) {
  const value = String(text);
  if (!/^\+?[0-9]+$/.test(value)) return null;
  const parsed = Number(value);
  return parsed <= 4294967295 ? parsed : null;
}

/**
 * Parse a decimal fraction, else the fallback.
 * @param {string} text
 * @param {number} fallback
 * @returns {number}
 */
function creativeParseFloat(text, fallback) {
  const value = String(text).trim();
  if (!/^[+-]?([0-9]+\.?[0-9]*|\.[0-9]+)$/.test(value)) return fallback;
  return Number(value);
}

/**
 * Rust's `char::is_alphanumeric`.
 * @param {string} character
 * @returns {boolean}
 */
function creativeIsAlphanumeric(character) {
  return /[\p{Alphabetic}\p{N}]/u.test(character);
}

/**
 * Trim non-alphanumeric characters from both ends of a token.
 * @param {string} token
 * @returns {string}
 */
function creativeTrimToken(token) {
  const characters = Array.from(token);
  let start = 0;
  let end = characters.length;
  while (start < end && !creativeIsAlphanumeric(characters[start])) start += 1;
  while (end > start && !creativeIsAlphanumeric(characters[end - 1])) end -= 1;
  return characters.slice(start, end).join("");
}

/**
 * Whitespace tokens of a string, like Rust's `split_whitespace`.
 * @param {string} text
 * @returns {Array<string>}
 */
function creativeTokens(text) {
  return String(text).split(/\s+/u).filter((token) => token.length > 0);
}

/**
 * Character count in Unicode scalar values.
 * @param {string} text
 * @returns {number}
 */
function creativeLength(text) {
  return Array.from(text).length;
}

/**
 * Compare two strings by code point, as Rust orders `String`s.
 * @param {string} left
 * @param {string} right
 * @returns {number}
 */
function creativeCompare(left, right) {
  const a = Array.from(left);
  const b = Array.from(right);
  const shared = Math.min(a.length, b.length);
  for (let index = 0; index < shared; index += 1) {
    const difference = a[index].codePointAt(0) - b[index].codePointAt(0);
    if (difference !== 0) return difference;
  }
  return a.length - b.length;
}

/**
 * Fill a localized (English) response template's `{placeholder}` slots.
 * @param {string} intent
 * @param {Array<Array<string>>} values
 * @returns {string}
 */
function creativeTemplate(intent, values) {
  return textTransformFill(textTransformLocalizedResponse(intent, "en") || "", values);
}

/**
 * The worker answer shape of `finalize_simple`.
 * @param {string} intent
 * @param {string} handler
 * @param {string} body
 * @param {number} confidence
 * @param {Array<string>} trace
 * @param {string} language
 * @returns {object}
 */
function creativeAnswer(intent, handler, body, confidence, trace, language) {
  return {
    intent: intent,
    content: body,
    confidence: confidence,
    evidence: [`handler:${handler}`].concat(trace, [`response:${intent}`, `language:${language}`]),
    trace: trace,
  };
}

/**
 * The `topic_stop_word` words of one language.
 * @param {string} language
 * @returns {Array<string>}
 */
function creativeStopWords(language) {
  const out = [];
  for (const record of creativeRecords("topic_stop_word")) {
    if (childValue(record, "language") !== language) continue;
    for (const word of creativeChildValues(record, "word")) out.push(word);
  }
  return out;
}

/**
 * The `(word, value)` pairs of one language's `spelled_number` record. The
 * seed writes `number <word> <value>`; a `value` child is read too.
 * @param {string} language
 * @returns {Array<{word: string, value: number}>}
 */
function creativeSpelledNumbers(language) {
  const out = [];
  for (const record of creativeRecords("spelled_number")) {
    if (childValue(record, "language") !== language) continue;
    for (const pair of record.children) {
      if (pair.name !== "number") continue;
      const parts = creativeTokens(pair.value);
      const word = parts.length > 0 ? parts[0] : "";
      const raw = childValue(pair, "value") || (parts.length > 1 ? parts[1] : "");
      const value = creativeParseUnsigned(raw);
      if (word.length > 0 && value !== null && value > 0) out.push({ word: word, value: value });
    }
  }
  return out;
}

/**
 * The CJK runs of `normalized` left once the request vocabulary is removed:
 * the cue surfaces of the four composition roles, plus `removed` (the
 * language's stop words and numerals), longest first.
 * @param {string} normalized
 * @param {Array<string>} removed
 * @returns {Array<string>}
 */
function creativeCjkRemainder(normalized, removed) {
  const vocabulary = removed.slice();
  for (const role of [CREATIVE_ROLE_BRAINSTORMING, CREATIVE_ROLE_ADVICE, CREATIVE_ROLE_WRITING, CREATIVE_ROLE_PLANNING]) {
    for (const meaning of meaningsWithRole(role)) {
      for (const word of meaning.words) vocabulary.push(word);
    }
  }
  vocabulary.sort((left, right) => creativeLength(right) - creativeLength(left));
  let text = normalized;
  for (const word of vocabulary) {
    if (word.length > 0 && containsCjk(word)) text = text.split(word).join(" ");
  }
  return creativeTokens(text).filter((run) => containsCjk(run));
}

/**
 * The topic words of a composition request (`topic_words`). Space-delimited
 * scripts keep every alphabetic token that is neither a stop word nor a
 * spelled number; CJK scripts take the meaning-lexicon surfaces present in
 * the prompt, minus the same stop words and numerals, and when the lexicon
 * names none, the runs the request vocabulary leaves behind.
 * @param {string} normalized
 * @param {string} language
 * @returns {Array<string>}
 */
function creativeTopicWords(normalized, language) {
  const stops = creativeStopWords(language);
  const numbers = creativeSpelledNumbers(language).map((pair) => pair.word);
  const skipped = (word) => stops.includes(word) || numbers.includes(word);
  const words = [];
  if (containsCjk(normalized)) {
    for (const meaning of meaningLexicon()) {
      if (
        meaning.roles.includes(CREATIVE_ROLE_BRAINSTORMING) ||
        meaning.roles.includes(CREATIVE_ROLE_ADVICE) ||
        meaning.roles.includes(CREATIVE_ROLE_WRITING) ||
        meaning.roles.includes(CREATIVE_ROLE_PLANNING)
      ) {
        continue;
      }
      const found = meaning.words.find((word) => word.length > 0 && !skipped(word) && normalized.includes(word));
      if (found !== undefined) words.push(found);
    }
    if (words.length === 0) {
      for (const run of creativeCjkRemainder(normalized, stops.concat(numbers))) words.push(run);
    }
  } else {
    for (const raw of creativeTokens(normalized)) {
      const token = creativeTrimToken(raw);
      if (creativeLength(token) < 2 || !/\p{Alphabetic}/u.test(token)) continue;
      if (skipped(token)) continue;
      words.push(token);
    }
  }
  words.sort(creativeCompare);
  return words.filter((word, index) => index === 0 || word !== words[index - 1]);
}

/**
 * The count a request states (`requested_count`): a spelled number (CJK
 * numerals by substring once the language's stop words are removed, the
 * longest numeral winning; other scripts as whole tokens), else the first
 * standalone decimal when it lies in 1..=12.
 * @param {string} normalized
 * @param {string} language
 * @returns {number|null}
 */
function creativeRequestedCount(normalized, language) {
  let stripped = normalized;
  for (const stop of creativeStopWords(language)) {
    if (containsCjk(stop)) stripped = stripped.split(stop).join(" ");
  }
  let best = null;
  for (const pair of creativeSpelledNumbers(language)) {
    if (containsCjk(pair.word)) {
      if (stripped.includes(pair.word) && (best === null || creativeLength(pair.word) > creativeLength(best.word))) {
        best = pair;
      }
    } else if (normalized.includes(` ${pair.word} `) || normalized.startsWith(`${pair.word} `)) {
      return pair.value;
    }
  }
  if (best !== null) return best.value;
  for (const token of creativeTokens(normalized)) {
    const value = creativeParseUnsigned(token);
    if (value !== null) return value > 0 && value <= 12 ? value : null;
  }
  return null;
}

/**
 * The `brainstorm_rule` records: a kind and its pieces.
 * @returns {Array<{kind: string, pieces: Array<string>}>}
 */
function creativeBrainstormRules() {
  return creativeRecords("brainstorm_rule")
    .map((record) => ({ kind: childValue(record, "kind"), pieces: creativeChildValues(record, "piece") }))
    .filter((rule) => rule.kind.length > 0);
}

/**
 * The stated distinctness metric.
 * @returns {{floor: number, statement: string, shareRule: string}}
 */
function creativeDistinctnessMetric() {
  const record = creativeRecords("distinctness_metric")[0];
  if (record === undefined) {
    return { floor: 0.5, statement: "pairwise normalized levenshtein ratio", shareRule: "" };
  }
  return {
    floor: creativeParseFloat(childValue(record, "floor"), 0.5),
    statement: childValue(record, "statement"),
    shareRule: childValue(record, "share_rule"),
  };
}

/**
 * Levenshtein distance over Unicode scalar values.
 * @param {string} left
 * @param {string} right
 * @returns {number}
 */
function creativeLevenshtein(left, right) {
  const a = Array.from(left);
  const b = Array.from(right);
  let previous = [];
  for (let j = 0; j <= b.length; j += 1) previous.push(j);
  for (let i = 0; i < a.length; i += 1) {
    const current = [i + 1];
    for (let j = 0; j < b.length; j += 1) {
      const cost = a[i] === b[j] ? 0 : 1;
      current.push(Math.min(previous[j] + cost, previous[j + 1] + 1, current[j] + 1));
    }
    previous = current;
  }
  return previous[b.length];
}

/**
 * Distinct enough: distance at least `floor` times the longer length.
 * @param {string} left
 * @param {string} right
 * @param {number} floor
 * @returns {boolean}
 */
function creativeDistinctEnough(left, right, floor) {
  const longer = Math.max(creativeLength(left), creativeLength(right), 1);
  return creativeLevenshtein(left, right) / longer >= floor;
}

/**
 * Pronounceable: no ASCII digit, and a vowel unless the script is CJK or
 * Devanagari (whose syllabaries carry their own vowels).
 * @param {string} candidate
 * @returns {boolean}
 */
function creativePronounceable(candidate) {
  if (/[0-9]/.test(candidate)) return false;
  if (containsCjk(candidate) || containsDevanagari(candidate)) return true;
  return /[aeiouyAEIOUYаеёиоуыэюяАЕЁИОУЫЭЮЯ]/.test(candidate);
}

/**
 * The words named after the exclusion cue: candidates containing one are
 * dropped before ranking.
 * @param {string} normalized
 * @returns {Array<string>}
 */
function creativeExclusions(normalized) {
  const markers = creativeCueWords("exclusion");
  const tokens = creativeTokens(normalized);
  const at = tokens.findIndex((token) => markers.includes(token));
  if (at === -1) return [];
  return tokens
    .slice(at + 1)
    .map((token) => creativeTrimToken(token).toLowerCase())
    .filter((token) => creativeLength(token) >= 2);
}

/**
 * Uppercase the first character.
 * @param {string} word
 * @returns {string}
 */
function creativeCapitalize(word) {
  const characters = Array.from(word);
  if (characters.length === 0) return "";
  return characters[0].toUpperCase() + characters.slice(1).join("");
}

/**
 * The raw candidate pool: every rule over every ordered topic pair.
 * @param {Array<string>} topics
 * @param {Array<{kind: string, pieces: Array<string>}>} rules
 * @returns {Array<string>}
 */
function creativeComposeCandidates(topics, rules) {
  const out = [];
  for (const rule of rules) {
    switch (rule.kind) {
      case "compound":
        for (const a of topics) {
          for (const b of topics) {
            if (a !== b) out.push(creativeCapitalize(a) + creativeCapitalize(b));
          }
        }
        break;
      case "blend":
        for (const a of topics) {
          for (const b of topics) {
            if (a === b) continue;
            const aChars = Array.from(a);
            const bChars = Array.from(b);
            const front = Math.floor((aChars.length + 1) / 2);
            const backStart = Math.floor(bChars.length / 2);
            if (front >= 2 && bChars.length - backStart >= 2) {
              out.push(creativeCapitalize(aChars.slice(0, front).concat(bChars.slice(backStart)).join("")));
            }
          }
        }
        break;
      case "suffix":
        for (const a of topics) {
          for (const piece of rule.pieces) out.push(creativeCapitalize(a) + piece);
        }
        break;
      case "definite":
        for (const a of topics) {
          for (const b of topics) {
            if (a !== b) out.push(`The ${creativeCapitalize(a)} ${creativeCapitalize(b)}`);
          }
        }
        break;
      default:
        break;
    }
  }
  return out;
}

/**
 * The topic words occurring in a candidate.
 * @param {string} candidate
 * @param {Array<string>} topics
 * @returns {Array<string>}
 */
function creativeContentWords(candidate, topics) {
  const lower = candidate.toLowerCase();
  return topics.map((topic) => topic.toLowerCase()).filter((topic) => lower.includes(topic));
}

/**
 * Filter the pool by the explicit constraints, then the distinctness floor
 * and the content-share rule.
 * @param {Array<string>} pool
 * @param {Array<string>} topics
 * @param {number} count
 * @param {number} maxLength
 * @param {Array<string>} excluded
 * @param {{floor: number}} metric
 * @returns {Array<string>}
 */
function creativeSelectCandidates(pool, topics, count, maxLength, excluded, metric) {
  const kept = [];
  for (const candidate of pool) {
    if (kept.length >= count) break;
    if (creativeLength(candidate) > maxLength) continue;
    if (!creativePronounceable(candidate)) continue;
    const lower = candidate.toLowerCase();
    if (excluded.some((word) => lower.includes(word))) continue;
    if (kept.some((other) => !creativeDistinctEnough(candidate, other, metric.floor))) continue;
    const words = creativeContentWords(candidate, topics);
    if (words.length >= 2) {
      const clash = kept.some((other) => {
        const otherWords = creativeContentWords(other, topics);
        return otherWords.length >= 2 && words.every((word) => otherWords.includes(word));
      });
      if (clash) continue;
    }
    kept.push(candidate);
  }
  return kept;
}

/**
 * Compose ranked name candidates for a topic-bearing brainstorming request
 * (`handle_brainstorm_request`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryBrainstormComposition(prompt, normalized) {
  const normal = String(normalized || "");
  if (!lexiconMentionsRole(CREATIVE_ROLE_BRAINSTORMING, normal)) return null;
  const language = detectLanguage(String(prompt || ""));
  const trace = [];
  const refusal = (reason) => {
    trace.push(`brainstorming:refusal:${reason}`);
    return creativeAnswer("brainstorming", "brainstorm_composition",
      creativeTemplate("brainstorming_refusal", []), 0.4, trace, language);
  };
  const topics = creativeTopicWords(normal, language);
  if (topics.length === 0) return refusal("no topic words");
  for (const topic of topics) trace.push(`brainstorming:topic:${topic}`);
  const requested = creativeRequestedCount(normal, language);
  const count = requested === null ? 5 : requested;
  trace.push(`brainstorming:count:${count}`);
  const shortCued = creativeCueWords("short_names").some((word) => normal.includes(word));
  const shortCap = creativeParseUnsigned(creativeCueField("short_names", "max_length"));
  const maxLength = shortCued && shortCap !== null ? shortCap : CREATIVE_DEFAULT_MAX_LENGTH;
  const excluded = creativeExclusions(normal);
  for (const word of excluded) trace.push(`brainstorming:exclusion:${word}`);
  const rules = creativeBrainstormRules();
  const metric = creativeDistinctnessMetric();
  const pool = creativeComposeCandidates(topics, rules);
  trace.push(`brainstorming:pool:${pool.length}`);
  const candidates = creativeSelectCandidates(pool, topics, count, maxLength, excluded, metric);
  if (candidates.length === 0) return refusal("no candidate passed");
  trace.push(`brainstorming:candidates:${candidates.join(", ")}`, ...candidates.map((candidate) => `candidate:brainstorming:${candidate}`));
  const listed = candidates.map((candidate, index) => `${index + 1}. ${candidate}`).join("\n");
  const pronounce = `at most ${maxLength} characters, pronounceable (carries a vowel, no digits)`;
  const constraints = excluded.length === 0
    ? pronounce
    : `${pronounce}, excluding words containing ${excluded.join(", ")}`;
  const body = creativeTemplate("brainstorming_candidates", [
    ["count", String(candidates.length)],
    ["concepts", topics.join(", ")],
    ["candidates", listed],
    ["metric", metric.statement],
    ["constraints", constraints],
    ["rules", rules.map((rule) => rule.kind).join(", ")],
  ]);
  return creativeAnswer("brainstorming", "brainstorm_composition", body, 0.6, trace, language);
}

/**
 * The `advice_evidence_grade` scale.
 * @returns {Array<{grade: string, weight: number, label: string}>}
 */
function creativeEvidenceGrades() {
  return creativeRecords("advice_evidence_grade")
    .map((record) => ({
      grade: childValue(record, "grade"),
      weight: creativeParseFloat(childValue(record, "weight"), 0.3),
      label: childValue(record, "label"),
    }))
    .filter((grade) => grade.grade.length > 0);
}

/**
 * The `advice_recommendation` rows of one topic.
 * @param {string} topic
 * @returns {Array<{text: string, grade: string, source: string, sourceUrl: string}>}
 */
function creativeRecommendations(topic) {
  return creativeRecords("advice_recommendation")
    .filter((record) => childValue(record, "topic") === topic)
    .map((record) => ({
      text: childValue(record, "text"),
      grade: childValue(record, "grade"),
      source: childValue(record, "source"),
      sourceUrl: childValue(record, "source_url"),
    }))
    .filter((record) => record.text.length > 0);
}

/**
 * Render evidence-graded, cited recommendations for an advice request
 * (`handle_advice_request`). Uncited items are never returned.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryAdviceRequest(prompt, normalized) {
  const normal = String(normalized || "");
  if (!lexiconMentionsRole(CREATIVE_ROLE_ADVICE, normal)) return null;
  const language = detectLanguage(String(prompt || ""));
  const trace = [];
  const matched = creativeRecords("advice_topic").find((record) =>
    childValue(record, "topic").length > 0 &&
    creativeChildValues(record, "surface").some((surface) => normal.includes(surface)));
  if (matched === undefined) {
    const fallback = creativeTopicWords(normal, language);
    trace.push("advice:refusal:topic not in table");
    return creativeAnswer("advice", "advice_request",
      creativeTemplate("advice_refusal", [["topic", fallback.length > 0 ? fallback[0] : ""]]),
      0.4, trace, language);
  }
  const topic = childValue(matched, "topic");
  trace.push(`advice:topic:${topic}`);
  const grades = creativeEvidenceGrades();
  const items = [];
  for (const record of creativeRecommendations(topic)) {
    const grade = grades.find((candidate) => candidate.grade === record.grade && record.sourceUrl.length > 0);
    if (grade !== undefined) items.push({ weight: grade.weight, item: record });
  }
  items.sort((left, right) => right.weight - left.weight);
  for (const entry of items) {
    trace.push(`advice:item:weight ${entry.weight.toFixed(2)} grade ${entry.item.grade}`);
  }
  const listed = items.map((entry, index) => {
    const grade = grades.find((candidate) => candidate.grade === entry.item.grade);
    const label = grade === undefined ? "ungraded" : grade.label;
    return `${index + 1}. ${entry.item.text} [${label} — ${entry.item.source}] (${entry.item.sourceUrl})`;
  }).join("\n");
  const weighting = grades.map((grade) => `${grade.grade} ${grade.weight.toFixed(2)}`).join(" > ");
  const forwardNote = "Competing recommendations are ordered by grade weight; where two disagree, the higher grade wins here, and full relative probabilities are the relative-meta-logic work of issue #1179.";
  const body = creativeTemplate("advice_guidance", [
    ["topic", topic],
    ["items", listed],
    ["weighting", weighting],
    ["forward_note", forwardNote],
  ]);
  return creativeAnswer("advice", "advice_request", body, 0.6, trace, language);
}
