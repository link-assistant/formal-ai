// Issue #1174 text-transform family, browser twin (issue #1188 JS parity).
//
// Mirrors rust/src/solver_handlers/summarization_request.rs (free-text
// summarization), the statement pipeline it calls in
// rust/src/summarization/mod.rs (formalize -> summarize -> deformalize), and
// rust/src/solver_handlers/text_rewrite.rs (register rewriting, grammar
// correction, genre writing). Every vocabulary item is read from the same seed
// records the native handlers read: data/seed/register-lexicon.lino,
// data/seed/agreement-rules.lino, data/seed/writing-genre-styleguides.lino,
// the text_summarization_action and summary_classification_cue meaning roles,
// and the multilingual-responses-text-transform.lino templates.

const TEXT_TRANSFORM_REGISTER_FILE = "register-lexicon.lino";
const TEXT_TRANSFORM_AGREEMENT_FILE = "agreement-rules.lino";
const TEXT_TRANSFORM_GENRE_FILE = "writing-genre-styleguides.lino";
const TEXT_TRANSFORM_INTENT_REGISTER = "text_transform_register";
const TEXT_TRANSFORM_INTENT_GRAMMAR = "text_transform_grammar";
const TEXT_TRANSFORM_INTENT_GENRE_PREFIX = "text_transform_genre";
const ROLE_TEXT_SUMMARIZATION_ACTION = "text_summarization_action";
const ROLE_SUMMARY_CLASSIFICATION_CUE = "summary_classification_cue";

// ---------------------------------------------------------------------------
// Shared seed and text helpers
// ---------------------------------------------------------------------------

/**
 * The records of one seed file: every child of every top-level node.
 * @param {string} fileName
 * @returns {Array<{name: string, value: string, children: Array<object>}>}
 */
function textTransformSeedRecords(fileName) {
  const text = seedRawText(SEED_RAW, fileName);
  if (!text) return [];
  const tree = parseLinoTree(text);
  const out = [];
  for (const top of tree.children) {
    for (const record of top.children) out.push(record);
  }
  return out;
}

/**
 * Every non-empty value carried by a repeated child head of one record.
 * @param {{children: Array<{name: string, value: string}>}} record
 * @param {string} head
 * @returns {Array<string>}
 */
function textTransformRecordValues(record, head) {
  return record.children
    .filter((child) => child.name === head)
    .map((child) => child.value)
    .filter((value) => value.length > 0);
}

/**
 * Rust `str::split_whitespace`.
 * @param {string} text
 * @returns {Array<string>}
 */
function textTransformWords(text) {
  return String(text).split(/\s+/u).filter((word) => word.length > 0);
}

/**
 * Rust `char::is_alphanumeric`.
 * @param {string} character
 * @returns {boolean}
 */
function textTransformIsAlphanumeric(character) {
  return /^[\p{Alphabetic}\p{N}]$/u.test(character);
}

/**
 * A localized response template, exactly as the seed declares it for one
 * language, or null when that language has no record.
 * @param {string} intent
 * @param {string} language
 * @returns {string|null}
 */
function textTransformResponseFor(intent, language) {
  const table = MULTILINGUAL_ANSWERS[intent];
  if (!table) return null;
  const entry = table[language];
  if (!entry) return null;
  if (typeof entry === "string") return entry;
  return typeof entry.text === "string" ? entry.text : null;
}

/**
 * Mirrors `seed::localized_response`: the language, then the registry's
 * `unknown` gap record for a known language, then English.
 * @param {string} intent
 * @param {string} language
 * @returns {string|null}
 */
function textTransformLocalizedResponse(intent, language) {
  const exact = textTransformResponseFor(intent, language);
  if (exact !== null) return exact;
  if (isKnownResponseLanguage(language)) {
    const gap = textTransformResponseFor(intent, "unknown");
    if (gap !== null) return gap;
  }
  return textTransformResponseFor(intent, "en");
}

/**
 * Fill `{placeholder}` slots, every occurrence, in order.
 * @param {string} template
 * @param {Array<Array<string>>} values pairs of [name, value]
 * @returns {string}
 */
function textTransformFill(template, values) {
  let out = String(template);
  for (const pair of values) {
    out = out.split(`{${pair[0]}}`).join(pair[1]);
  }
  return out;
}

/**
 * Mirrors `render_line`: the language's template, else English, else "".
 * @param {string} intent
 * @param {string} language
 * @param {Array<Array<string>>} values
 * @returns {string}
 */
function textTransformRenderLine(intent, language, values) {
  const exact = textTransformResponseFor(intent, language);
  const template = exact !== null ? exact : textTransformResponseFor(intent, "en");
  if (template === null) return "";
  return textTransformFill(template, values);
}

/**
 * The browser answer shape every text-transform handler returns.
 * @param {string} intent
 * @param {string} body
 * @param {number} confidence
 * @param {string} slug the precedence row that answered
 * @param {Array<string>} trace
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>, trace: Array<string>}}
 */
function textTransformAnswer(intent, body, confidence, slug, trace) {
  return {
    intent: intent,
    content: body,
    confidence: confidence,
    evidence: [`handler:${slug}`, `response:${intent}`],
    trace: trace,
  };
}

// ---------------------------------------------------------------------------
// Command head and free-text payload (text_rewrite.rs)
// ---------------------------------------------------------------------------

/**
 * The colon separating a command head (at most twelve words) from a payload
 * (at least three words); -1 when none. Time, ratio and URL colons are skipped.
 * @param {string} prompt
 * @returns {number}
 */
function textTransformCommandColon(prompt) {
  for (let index = prompt.indexOf(":"); index !== -1; index = prompt.indexOf(":", index + 1)) {
    const before = prompt.slice(0, index);
    const beforeIsDigit = /[0-9]$/.test(before);
    const afterIsDigit = /^[0-9]/.test(prompt.slice(index + 1));
    if (beforeIsDigit && afterIsDigit) continue;
    if (prompt.slice(index + 1, index + 3) === "//") continue;
    if (textTransformWords(before).length > 12) continue;
    if (textTransformWords(prompt.slice(index + 1).trim()).length < 3) continue;
    return index;
  }
  return -1;
}

/**
 * Strip one matched pair of surrounding quotes, any kind.
 * @param {string} text
 * @returns {string}
 */
function textTransformStripOuterQuotes(text) {
  const pairs = [['"', '"'], ["«", "»"], ["'", "'"]];
  for (const pair of pairs) {
    if (text.length >= 2 && text.startsWith(pair[0]) && text.endsWith(pair[1])) {
      const body = text.slice(1, -1);
      if (body.length > 0) return body;
    }
  }
  return text;
}

/**
 * The first double-quoted or guillemet-quoted span, or null.
 * @param {string} text
 * @returns {string|null}
 */
function textTransformDoubleQuoted(text) {
  const pairs = [['"', '"'], ["«", "»"]];
  for (const pair of pairs) {
    const start = text.indexOf(pair[0]);
    if (start === -1) continue;
    const rest = text.slice(start + 1);
    const end = rest.indexOf(pair[1]);
    if (end > 0) return rest.slice(0, end);
  }
  return null;
}

/**
 * Mirrors `free_text_payload`: the text after a command colon, after the
 * first newline, or inside the first double quotes — at least three words.
 * @param {string} prompt
 * @returns {string|null}
 */
function textTransformFreeTextPayload(prompt) {
  const colon = textTransformCommandColon(prompt);
  if (colon !== -1) {
    const tail = textTransformStripOuterQuotes(prompt.slice(colon + 1).trim());
    if (textTransformWords(tail).length >= 3) return tail;
  }
  const newline = prompt.indexOf("\n");
  if (newline !== -1) {
    const rest = textTransformStripOuterQuotes(prompt.slice(newline + 1).trim());
    if (textTransformWords(rest).length >= 3) return rest;
  }
  const quoted = textTransformDoubleQuoted(prompt);
  if (quoted !== null && textTransformWords(quoted).length >= 3) return quoted;
  return null;
}

/**
 * The command head: the text before the command colon, the first newline, or
 * the first quote — whichever comes first.
 * @param {string} prompt
 * @returns {string}
 */
function textTransformCommandHead(prompt) {
  let cut = textTransformCommandColon(prompt);
  const markers = [prompt.indexOf("\n"), prompt.indexOf('"'), prompt.indexOf("«")];
  for (const marker of markers) {
    if (marker > 0) cut = cut === -1 ? marker : Math.min(cut, marker);
  }
  return cut === -1 ? prompt : prompt.slice(0, cut);
}

/**
 * @param {string} normalizedHead
 * @param {string} cue
 * @returns {boolean}
 */
function textTransformHeadCueMatches(normalizedHead, cue) {
  return ` ${normalizedHead} `.includes(` ${normalizePrompt(cue)} `);
}

/**
 * All action-cue phrases of a seed file, in record order.
 * @param {string} fileName
 * @returns {Array<string>}
 */
function textTransformCueList(fileName) {
  const out = [];
  for (const record of textTransformSeedRecords(fileName)) {
    if (record.name !== "action_cue") continue;
    for (const cue of textTransformRecordValues(record, "cue")) out.push(cue);
  }
  return out;
}

/**
 * Split a token into punctuation prefix, alphanumeric core (apostrophes kept)
 * and punctuation suffix.
 * @param {string} token
 * @returns {{prefix: string, core: string, suffix: string}}
 */
function textTransformTokenCore(token) {
  const characters = Array.from(token);
  const isCore = (character) =>
    textTransformIsAlphanumeric(character) || character === "'" || character === "’";
  let start = -1;
  for (let index = 0; index < characters.length; index += 1) {
    if (isCore(characters[index])) {
      start = index;
      break;
    }
  }
  if (start === -1) return { prefix: "", core: "", suffix: token };
  let end = start;
  while (end < characters.length && isCore(characters[end])) end += 1;
  return {
    prefix: characters.slice(0, start).join(""),
    core: characters.slice(start, end).join(""),
    suffix: characters.slice(end).join(""),
  };
}

/**
 * @param {string} text
 * @returns {string}
 */
function textTransformCapitalizeFirst(text) {
  const characters = Array.from(text);
  if (characters.length === 0) return "";
  return characters[0].toUpperCase() + characters.slice(1).join("");
}

/**
 * Rebuild a replacement with the source core's casing.
 * @param {string} core
 * @param {string} replacement
 * @returns {string}
 */
function textTransformRebuildCore(core, replacement) {
  let alphabetic = false;
  let lowercase = false;
  for (const character of core) {
    if (/\p{Alphabetic}/u.test(character)) {
      alphabetic = true;
      if (/\p{Lowercase}/u.test(character)) lowercase = true;
    }
  }
  if (alphabetic && !lowercase) return replacement.toUpperCase();
  const first = Array.from(core)[0];
  if (first !== undefined && /\p{Uppercase}/u.test(first)) {
    return textTransformCapitalizeFirst(replacement);
  }
  return replacement;
}

// ---------------------------------------------------------------------------
// Register rewriting
// ---------------------------------------------------------------------------

/**
 * Splice one phrase rule into a token list.
 * @param {Array<string>} tokens
 * @param {object} record
 * @param {Array<Array<string>>} substitutions appended to
 * @returns {Array<string>}
 */
function textTransformApplyPhraseRule(tokens, record, substitutions) {
  const pattern = childValue(record, "pattern");
  const replacement = childValue(record, "replacement");
  const words = textTransformWords(pattern);
  if (words.length === 0 || replacement.length === 0) return tokens;
  const out = [];
  let index = 0;
  while (index < tokens.length) {
    if (index + words.length <= tokens.length) {
      const window = tokens.slice(index, index + words.length);
      const windowMatches = window.every(
        (token, offset) => textTransformTokenCore(token).core.toLowerCase() === words[offset],
      );
      if (windowMatches) {
        const leadCore = textTransformTokenCore(window[0]).core;
        const leadFirst = Array.from(leadCore)[0];
        const leadsCapitalized = leadFirst !== undefined && /\p{Uppercase}/u.test(leadFirst);
        const rebuilt = leadsCapitalized ? textTransformCapitalizeFirst(replacement) : replacement;
        substitutions.push([window.join(" "), rebuilt]);
        out.push(rebuilt);
        index += words.length;
        continue;
      }
    }
    out.push(tokens[index]);
    index += 1;
  }
  return out;
}

/**
 * Mirrors `handle_rewrite_register`.
 * @param {string} prompt
 * @returns {object|null}
 */
function textTransformRewriteRegister(prompt) {
  const headNormalized = normalizePrompt(textTransformCommandHead(prompt));
  const cues = textTransformCueList(TEXT_TRANSFORM_REGISTER_FILE);
  if (!cues.some((cue) => textTransformHeadCueMatches(headNormalized, cue))) return null;
  const payload = textTransformFreeTextPayload(prompt);
  if (payload === null) return null;
  const records = textTransformSeedRecords(TEXT_TRANSFORM_REGISTER_FILE);
  const pairs = records
    .filter((record) => record.name === "word_pair")
    .map((record) => [childValue(record, "informal"), childValue(record, "formal")])
    .filter((pair) => pair[0].length > 0 && pair[1].length > 0);
  const substitutions = [];
  let tokens = [];
  for (const token of textTransformWords(payload)) {
    const parts = textTransformTokenCore(token);
    if (parts.core.length === 0) {
      tokens.push(token);
      continue;
    }
    const lower = parts.core.toLowerCase();
    const pair = pairs.find((candidate) => candidate[0] === lower);
    if (pair === undefined) {
      tokens.push(token);
      continue;
    }
    const replacement = textTransformRebuildCore(parts.core, pair[1]);
    substitutions.push([parts.core, replacement]);
    tokens.push(`${parts.prefix}${replacement}${parts.suffix}`);
  }
  for (const record of records) {
    if (record.name === "phrase_rule") {
      tokens = textTransformApplyPhraseRule(tokens, record, substitutions);
    }
  }
  if (substitutions.length === 0) {
    const language = detectLanguage(prompt);
    const note = textTransformLocalizedResponse("text_transform_rewrite_unchanged", language) || "";
    const body = note.length === 0 ? payload : `${payload}\n\n${note}`;
    return textTransformAnswer(TEXT_TRANSFORM_INTENT_REGISTER, body, 0.8, "text_rewrite", [
      "register_rewrite:no informal tokens found",
    ]);
  }
  const trace = substitutions.map((pair) => `register_rewrite:${pair[0]} -> ${pair[1]}`);
  return textTransformAnswer(TEXT_TRANSFORM_INTENT_REGISTER, tokens.join(" "), 0.8, "text_rewrite", trace);
}

// ---------------------------------------------------------------------------
// Grammar correction
// ---------------------------------------------------------------------------

/**
 * Rust `str::parse::<f64>().is_ok()` over an alphanumeric token core.
 * @param {string} text
 * @returns {boolean}
 */
function textTransformParsesAsFloat(text) {
  return /^(?:inf|infinity|nan|[0-9]+(?:\.[0-9]*)?(?:[eE][+-]?[0-9]+)?|\.[0-9]+(?:[eE][+-]?[0-9]+)?)$/i.test(
    text,
  );
}

/**
 * Mirrors `handle_grammar_correction`.
 * @param {string} prompt
 * @returns {object|null}
 */
function textTransformGrammarCorrection(prompt) {
  const headNormalized = normalizePrompt(textTransformCommandHead(prompt));
  const cues = textTransformCueList(TEXT_TRANSFORM_AGREEMENT_FILE);
  if (!cues.some((cue) => textTransformHeadCueMatches(headNormalized, cue))) return null;
  const payload = textTransformFreeTextPayload(prompt);
  if (payload === null) return null;
  const records = textTransformSeedRecords(TEXT_TRANSFORM_AGREEMENT_FILE);
  const language = detectLanguage(payload);
  const corrections = [];
  const tokens = textTransformWords(payload);

  for (const record of records) {
    if (record.name !== "subject_verb_rule") continue;
    const rule = childValue(record, "name");
    const wrong = childValue(record, "wrong").toLowerCase();
    const right = childValue(record, "right");
    const subjects = textTransformRecordValues(record, "subject").map((subject) => subject.toLowerCase());
    if (rule.length === 0 || wrong.length === 0 || right.length === 0 || subjects.length === 0) continue;
    for (let index = 0; index + 1 < tokens.length; index += 1) {
      const subjectCore = textTransformTokenCore(tokens[index]).core.toLowerCase();
      if (!subjects.includes(subjectCore)) continue;
      const verb = textTransformTokenCore(tokens[index + 1]);
      if (verb.core.toLowerCase() !== wrong) continue;
      const replacement = textTransformRebuildCore(verb.core, right);
      corrections.push([verb.core, replacement, rule]);
      tokens[index + 1] = `${verb.prefix}${replacement}${verb.suffix}`;
    }
  }

  const exceptions = records
    .filter((record) => record.name === "plural_exception")
    .map((record) => [childValue(record, "singular").toLowerCase(), childValue(record, "plural")])
    .filter((pair) => pair[0].length > 0 && pair[1].length > 0);
  const uncountables = records
    .filter((record) => record.name === "uncountable_noun")
    .map((record) => childValue(record, "noun").toLowerCase())
    .filter((noun) => noun.length > 0);
  for (const record of records) {
    if (record.name !== "numeral_noun_rule") continue;
    const rule = childValue(record, "name");
    const suffix = childValue(record, "suffix");
    const numerals = textTransformRecordValues(record, "numeral").map((numeral) => numeral.toLowerCase());
    if (rule.length === 0 || suffix.length === 0 || numerals.length === 0) continue;
    for (let index = 0; index + 1 < tokens.length; index += 1) {
      const numeralLower = textTransformTokenCore(tokens[index]).core.toLowerCase();
      const namesANumeral =
        numerals.includes(numeralLower) ||
        (textTransformParsesAsFloat(numeralLower) && numeralLower !== "1");
      if (!namesANumeral) continue;
      const noun = textTransformTokenCore(tokens[index + 1]);
      const nounLower = noun.core.toLowerCase();
      if (nounLower.length === 0 || nounLower.endsWith(suffix.toLowerCase())) continue;
      const exception = exceptions.find((pair) => pair[0] === nounLower);
      if (exception !== undefined) {
        const replacement = textTransformRebuildCore(noun.core, exception[1]);
        corrections.push([noun.core, replacement, rule]);
        tokens[index + 1] = `${noun.prefix}${replacement}${noun.suffix}`;
        continue;
      }
      if (uncountables.includes(nounLower)) continue;
      const replacement = `${noun.core}${suffix}`;
      corrections.push([noun.core, replacement, rule]);
      tokens[index + 1] = `${noun.prefix}${replacement}${noun.suffix}`;
    }
  }

  if (corrections.length === 0) {
    const note = textTransformLocalizedResponse("text_transform_grammar_clean", language) || "";
    return textTransformAnswer(TEXT_TRANSFORM_INTENT_GRAMMAR, note, 0.8, "text_rewrite", []);
  }
  const trace = corrections.map((item) => `grammar_correction:${item[0]} -> ${item[1]} (${item[2]})`);
  const corrected = tokens.join(" ");
  const lines = corrections
    .map((item) =>
      textTransformRenderLine("text_transform_grammar_line", language, [
        ["wrong", item[0]],
        ["right", item[1]],
        ["rule", item[2]],
      ]),
    )
    .join("\n");
  const rendered = textTransformRenderLine("text_transform_grammar_result", language, [
    ["corrected", corrected],
    ["corrections", lines],
  ]);
  const body = rendered.length === 0 ? `${corrected}\n\n${lines}` : rendered;
  return textTransformAnswer(TEXT_TRANSFORM_INTENT_GRAMMAR, body, 0.8, "text_rewrite", trace);
}

// ---------------------------------------------------------------------------
// Genre writing
// ---------------------------------------------------------------------------

/**
 * @param {Array<string>} requires
 * @param {Array<boolean>} unsatisfied
 * @returns {Array<string>}
 */
function textTransformUnsatisfiedRequirements(requires, unsatisfied) {
  return requires.filter((requirement, index) => unsatisfied[index] === true);
}

/**
 * @param {string} intent
 * @param {string} language
 * @param {string} genre
 * @param {Array<string>} missing
 * @returns {object}
 */
function textTransformGenreIncomplete(intent, language, genre, missing) {
  const body = textTransformRenderLine("text_transform_genre_incomplete", language, [
    ["genre", genre],
    ["missing", missing.join(", ")],
  ]);
  return textTransformAnswer(intent, body, 0.5, "text_rewrite", [`genre_incomplete:${missing.join(", ")}`]);
}

/**
 * @param {object} guide
 * @param {string} payload
 * @param {string} intent
 * @param {string} language
 * @param {Array<string>} requires
 * @returns {object}
 */
function textTransformCommitMessage(guide, payload, intent, language, requires) {
  const tokens = textTransformWords(payload);
  const slots = guide.children
    .filter((child) => child.name === "slot")
    .map((slot) => [childValue(slot, "cue"), childValue(slot, "value")])
    .filter((pair) => pair[0].length > 0 && pair[1].length > 0)
    .map((pair) => [pair[0].toLowerCase(), pair[1]]);
  const articles = textTransformRecordValues(guide, "article").map((article) => article.toLowerCase());
  const isArticle = (core) => articles.includes(core.toLowerCase());
  const preposition = childValue(guide, "component_preposition").toLowerCase();
  const verbCore = tokens.length > 0 ? textTransformTokenCore(tokens[0]).core.toLowerCase() : null;
  const slot = verbCore === null ? undefined : slots.find((pair) => pair[0] === verbCore);
  const commitType = slot === undefined ? null : slot[1];
  let prepositionIndex = -1;
  if (preposition.length > 0) {
    for (let index = tokens.length - 1; index >= 0; index -= 1) {
      if (textTransformTokenCore(tokens[index]).core.toLowerCase() === preposition) {
        prepositionIndex = index;
        break;
      }
    }
  }
  let effectRange = [];
  if (prepositionIndex === -1) effectRange = tokens.slice(1);
  else if (prepositionIndex >= 1) effectRange = tokens.slice(1, prepositionIndex);
  let skipping = true;
  const effectTokens = [];
  for (const token of effectRange) {
    if (skipping && isArticle(textTransformTokenCore(token).core)) continue;
    skipping = false;
    effectTokens.push(token);
  }
  let component = null;
  if (prepositionIndex !== -1) {
    for (const token of tokens.slice(prepositionIndex + 1)) {
      const core = textTransformTokenCore(token).core;
      if (core.length > 0 && !isArticle(core)) {
        component = core;
        break;
      }
    }
  }
  const missing = textTransformUnsatisfiedRequirements(requires, [
    commitType === null,
    effectTokens.length === 0,
  ]);
  if (missing.length > 0) {
    return textTransformGenreIncomplete(intent, language, "commit_message", missing);
  }
  const imperative = verbCore === null ? "" : verbCore;
  const effect = effectTokens.join(" ");
  const body = component !== null
    ? textTransformFill(childValue(guide, "template"), [
      ["type", commitType],
      ["component", component],
      ["imperative", imperative],
      ["effect", effect],
    ])
    : textTransformFill(childValue(guide, "template_no_scope"), [
      ["type", commitType],
      ["imperative", imperative],
      ["effect", effect],
    ]);
  return textTransformAnswer(intent, body, 0.8, "text_rewrite", [`genre_composition:${body}`]);
}

/**
 * @param {object} guide
 * @param {string} head
 * @param {string} payload
 * @param {string} intent
 * @param {string} language
 * @param {Array<string>} requires
 * @returns {object}
 */
function textTransformEmail(guide, head, payload, intent, language, requires) {
  const tokens = textTransformWords(payload);
  const coreLower = (token) => textTransformTokenCore(token).core.toLowerCase();
  const preposition = childValue(guide, "recipient_preposition").toLowerCase();
  const possessives = textTransformRecordValues(guide, "possessive").map((value) => value.toLowerCase());
  const headTokens = textTransformWords(head);
  const prepositionIndex = preposition.length === 0
    ? -1
    : headTokens.findIndex((token) => coreLower(token) === preposition);
  let recipient = null;
  if (prepositionIndex !== -1) {
    for (const token of headTokens.slice(prepositionIndex + 1)) {
      const core = textTransformTokenCore(token).core;
      if (core.length > 0 && !possessives.includes(core.toLowerCase())) {
        recipient = core;
        break;
      }
    }
  }
  const reasonCue = childValue(guide, "reason_cue").toLowerCase();
  const reasonIndex = reasonCue.length === 0
    ? -1
    : tokens.findIndex((token) => coreLower(token) === reasonCue);
  const intentText = (reasonIndex === -1 ? tokens : tokens.slice(0, reasonIndex)).join(" ");
  if (intentText.length === 0) {
    const missing = textTransformUnsatisfiedRequirements(requires, [true]);
    return textTransformGenreIncomplete(intent, language, "email", missing);
  }
  const reason = reasonIndex === -1 ? null : tokens.slice(reasonIndex + 1).join(" ");
  const weekdayRole = childValue(guide, "weekday_role");
  let weekday = null;
  if (weekdayRole.length > 0) {
    for (const candidate of wordsForRoleInLanguages(weekdayRole, [language])) {
      const token = tokens.find((item) => coreLower(item) === candidate);
      if (token !== undefined) {
        weekday = textTransformCapitalizeFirst(textTransformTokenCore(token).core);
        break;
      }
    }
  }
  const parts = [];
  for (const part of textTransformRecordValues(guide, "part")) {
    switch (part) {
      case "greeting":
        parts.push(
          recipient !== null
            ? textTransformFill(childValue(guide, "greeting"), [["recipient", recipient]])
            : childValue(guide, "greeting_default"),
        );
        break;
      case "intent_frame":
        parts.push(textTransformFill(childValue(guide, "intent_frame"), [["intent", intentText]]));
        break;
      case "reason_frame":
        if (reason !== null) {
          parts.push(textTransformFill(childValue(guide, "reason_frame"), [["reason", reason]]));
        }
        break;
      case "date_frame":
        if (weekday !== null) {
          parts.push(textTransformFill(childValue(guide, "date_frame"), [["date", weekday]]));
        }
        break;
      case "closing":
        parts.push(childValue(guide, "closing"));
        break;
      default:
        break;
    }
  }
  const body = parts.join("\n\n");
  return textTransformAnswer(intent, body, 0.8, "text_rewrite", [`genre_composition:${body}`]);
}

/**
 * Mirrors `handle_genre_writing`.
 * @param {string} prompt
 * @returns {object|null}
 */
function textTransformGenreWriting(prompt) {
  const head = textTransformCommandHead(prompt);
  const headNormalized = normalizePrompt(head);
  const records = textTransformSeedRecords(TEXT_TRANSFORM_GENRE_FILE);
  let genre = null;
  for (const record of records) {
    if (record.name !== "action_cue" || genre !== null) continue;
    const recordGenre = childValue(record, "genre");
    for (const cue of textTransformRecordValues(record, "cue")) {
      if (textTransformHeadCueMatches(headNormalized, cue)) {
        genre = recordGenre;
        break;
      }
    }
  }
  if (genre === null) return null;
  const payload = textTransformFreeTextPayload(prompt);
  if (payload === null) return null;
  const guide = records.find(
    (record) => record.name === "genre" && childValue(record, "name") === genre,
  );
  if (guide === undefined) return null;
  const language = detectLanguage(payload);
  const intent = `${TEXT_TRANSFORM_INTENT_GENRE_PREFIX}_${genre}`;
  const requires = textTransformRecordValues(guide, "requires");
  switch (genre) {
    case "commit_message":
      return textTransformCommitMessage(guide, payload, intent, language, requires);
    case "email":
      return textTransformEmail(guide, head, payload, intent, language, requires);
    default:
      return null;
  }
}

/**
 * The text-transform umbrella (`handle_text_rewrite`): register rewriting,
 * then grammar correction, then genre writing.
 * @param {string} prompt
 * @returns {object|null}
 */
function tryTextRewrite(prompt) {
  const text = String(prompt || "");
  return (
    textTransformRewriteRegister(text) ||
    textTransformGrammarCorrection(text) ||
    textTransformGenreWriting(text)
  );
}

// ---------------------------------------------------------------------------
// Free-text summarization (summarization_request.rs + summarization/mod.rs)
// ---------------------------------------------------------------------------

const SUMMARY_SENTENCE_TERMINATORS = [".", "!", "?", "。", "…", "।", "॥", "\n"];
const SUMMARY_TERMINAL_PUNCTUATION = [".", "!", "?", "。", "…", "।", "॥", "」", '"'];

/**
 * @param {string} slug a summary_classification_cue meaning slug
 * @returns {string} the statement kind
 */
function summaryKindFromSlug(slug) {
  switch (slug) {
    case "summary_kind_install":
      return "install";
    case "summary_kind_example":
      return "example";
    case "summary_kind_language":
      return "language";
    case "summary_kind_stars":
      return "stars";
    case "summary_kind_purpose":
      return "purpose";
    case "summary_kind_use_case":
      return "use_case";
    case "summary_kind_feature":
      return "feature";
    default:
      return "misc";
  }
}

/**
 * @param {string} kind
 * @returns {number}
 */
function summaryWeightForKind(kind) {
  switch (kind) {
    case "purpose":
      return 100;
    case "identity":
      return 90;
    case "language":
      return 60;
    case "stars":
      return 55;
    case "feature":
      return 70;
    case "use_case":
      return 65;
    case "install":
      return 10;
    case "example":
      return 15;
    default:
      return 30;
  }
}

/**
 * Mirrors `classify_sentence`: the first seeded cue meaning whose surface
 * occurs in the lowercased sentence; long sentences skip the language kind.
 * @param {string} sentence
 * @returns {string}
 */
function summaryClassifySentence(sentence) {
  const lower = sentence.toLowerCase();
  const wordCount = textTransformWords(lower).length;
  for (const meaning of meaningsWithRole(ROLE_SUMMARY_CLASSIFICATION_CUE)) {
    if (!meaning.words.some((cue) => cue.length > 0 && lower.includes(cue))) continue;
    const kind = summaryKindFromSlug(meaning.slug);
    if (kind === "language" && wordCount > 12) continue;
    return kind;
  }
  return "misc";
}

/**
 * Whether a full stop belongs to the token being written (decimals, dotted
 * names, initialisms) rather than closing a sentence.
 * @param {string} buffer
 * @param {string|null} next
 * @returns {boolean}
 */
function summaryPeriodBelongsToToken(buffer, next) {
  const bufferCharacters = Array.from(buffer);
  const previous = bufferCharacters.length > 0 ? bufferCharacters[bufferCharacters.length - 1] : null;
  if (previous !== null && next !== null && textTransformIsAlphanumeric(previous) &&
    textTransformIsAlphanumeric(next)) {
    return true;
  }
  const words = textTransformWords(buffer);
  const last = words.length > 0 ? words[words.length - 1] : "";
  const keep = (character) => /\p{Alphabetic}/u.test(character) || character === ".";
  const characters = Array.from(last);
  let start = 0;
  let end = characters.length;
  while (start < end && !keep(characters[start])) start += 1;
  while (end > start && !keep(characters[end - 1])) end -= 1;
  const token = characters.slice(start, end).join("");
  const segments = token.split(".");
  const single = (segment) => {
    const parts = Array.from(segment);
    return parts.length === 1 && /\p{Alphabetic}/u.test(parts[0]);
  };
  if (!single(segments[0])) return false;
  for (const segment of segments.slice(1)) {
    if (!single(segment)) return false;
  }
  return segments.length >= 2;
}

/**
 * @param {string} buffer
 * @returns {{text: string, kind: string, weight: number}|null}
 */
function summaryStatementFrom(buffer) {
  const sentence = buffer.split("\n").join("").trim();
  if (sentence.length === 0) return null;
  const kind = summaryClassifySentence(sentence);
  return { text: sentence, kind: kind, weight: summaryWeightForKind(kind) };
}

/**
 * Mirrors `summarization::formalize`: one statement per sentence.
 * @param {string} text
 * @returns {Array<{text: string, kind: string, weight: number}>}
 */
function summaryFormalize(text) {
  const characters = Array.from(text);
  const out = [];
  let buffer = "";
  for (let index = 0; index < characters.length; index += 1) {
    const character = characters[index];
    const next = index + 1 < characters.length ? characters[index + 1] : null;
    const internalPeriod = character === "." && summaryPeriodBelongsToToken(buffer, next);
    buffer += character;
    if (!internalPeriod && SUMMARY_SENTENCE_TERMINATORS.includes(character)) {
      const statement = summaryStatementFrom(buffer);
      if (statement !== null) out.push(statement);
      buffer = "";
    }
  }
  const tail = summaryStatementFrom(buffer);
  if (tail !== null) out.push(tail);
  return out;
}

/**
 * Mirrors `summarize` in Standard mode with an explicit cap: boilerplate
 * dropped, stable descending weight order, truncated to the effective cap.
 * @param {Array<{text: string, kind: string, weight: number}>} statements
 * @param {number} maxStatements
 * @returns {Array<{text: string, kind: string, weight: number}>}
 */
function summarySelect(statements, maxStatements) {
  if (statements.length === 0) return [];
  const filtered = statements
    .filter((statement) => statement.kind !== "install" && statement.kind !== "example")
    .map((statement, index) => ({ statement: statement, index: index }))
    .sort((left, right) => right.statement.weight - left.statement.weight || left.index - right.index)
    .map((entry) => entry.statement);
  if (filtered.length === 0) return [];
  const ratioTarget = Math.max(1, Math.floor((filtered.length * 50 + 50) / 100));
  const cap = Math.max(1, Math.min(maxStatements, ratioTarget));
  return filtered.slice(0, cap);
}

/**
 * Mirrors `deformalize`: re-punctuate and join with single spaces.
 * @param {Array<{text: string}>} statements
 * @returns {string}
 */
function summaryDeformalize(statements) {
  return statements
    .map((statement) => {
      const trimmed = statement.text.trim();
      if (trimmed.length === 0) return "";
      const characters = Array.from(trimmed);
      const last = characters[characters.length - 1];
      return SUMMARY_TERMINAL_PUNCTUATION.includes(last) ? trimmed : `${trimmed}.`;
    })
    .filter((text) => text.length > 0)
    .join(" ");
}

/**
 * Free-text summarization (`handle_summarization_request`): a request that
 * carries its own text is answered with a ~30% weight-ranked selection,
 * floored at two statements and always one short of the input.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function trySummarizationText(prompt, normalized) {
  const text = String(prompt || "");
  if (!lexiconMentionsRole(ROLE_TEXT_SUMMARIZATION_ACTION, String(normalized || ""))) return null;
  const payload = textTransformFreeTextPayload(text);
  if (payload === null) return null;
  const statements = summaryFormalize(payload);
  if (statements.length < 2) return null;
  const trace = statements.map(
    (statement) => `summarization_statement:${statement.kind} weight ${statement.weight}`,
  );
  const bound = Math.max(
    1,
    Math.min(Math.max(2, Math.floor((statements.length * 3 + 5) / 10)), statements.length - 1),
  );
  trace.push(`summarization_bound:${bound}/${statements.length}`);
  const selected = summarySelect(statements, bound);
  if (selected.length === 0) return null;
  trace.push(`summarization_selected:${selected.map((statement) => statement.text).join(" | ")}`);
  return textTransformAnswer(
    "summarization_free_text",
    summaryDeformalize(selected),
    0.8,
    "summarization_text",
    trace,
  );
}
