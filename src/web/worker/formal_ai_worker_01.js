// Worker module 2 of 21. Loaded by ../formal_ai_worker.js.
function extractInvertedWhoIs(input, lower) {
  if (!lower.startsWith("who ") || !lower.endsWith(" is")) return null;
  const body = input.slice("who ".length, input.length - " is".length).trim();
  if (!body) return null;
  const normalized = body.toLowerCase();
  if (["is", "was", "are"].includes(normalized)) return null;
  return body;
}

function cleanMechanismFragment(value) {
  return String(value || "")
    .trim()
    .replace(/^[`"'«»<>()\[\]{}]+/u, "")
    .replace(/[`"'«»<>()\[\]{}]+$/u, "")
    .replace(/[?？。.!,,;:]+$/u, "")
    .trim();
}

// Trim optional detail/politeness modifiers from a candidate subject and reject
// it outright when it is a non-referential subject. The modifier tails carry
// ROLE_DETAIL_MODIFIER (suffix surfaces, stripped in declaration order); the
// rejection set carries ROLE_NON_REFERENTIAL_SUBJECT (bare surfaces match the
// whole candidate, prefix surfaces match a candidate that begins with the
// literal before the … slot). Mirrors clean_mechanism_subject in
// src/solver_handler_how.rs — no per-language modifier or pronoun array here.
function cleanMechanismSubject(value) {
  let clean = cleanMechanismFragment(value);
  for (const form of roleWordForms(ROLE_DETAIL_MODIFIER)) {
    const suffix = form.after;
    const lower = clean.toLowerCase();
    if (lower.endsWith(suffix)) {
      clean = cleanMechanismFragment(clean.slice(0, clean.length - suffix.length));
    }
  }
  const lower = clean.toLowerCase();
  const nonReferential = roleWordForms(ROLE_NON_REFERENTIAL_SUBJECT).some((form) => {
    if (form.slot === "bare") return lower === form.text;
    if (form.slot === "prefix") return lower.startsWith(form.before);
    return false;
  });
  if (!clean || nonReferential) {
    return null;
  }
  return clean;
}

// Strip a trailing mechanism predicate so a prefix match such as "how does X
// work" yields the bare subject "X". The predicate tails carry
// ROLE_MECHANISM_PREDICATE (suffix surfaces); they are tried in declaration
// order and the first match wins. Mirrors strip_mechanism_tail in
// src/solver_handler_how.rs — no per-language tail array here.
function stripMechanismTail(subject) {
  let clean = cleanMechanismSubject(subject);
  if (!clean) return null;
  const lower = clean.toLowerCase();
  for (const form of roleWordForms(ROLE_MECHANISM_PREDICATE)) {
    const suffix = form.after;
    if (lower.endsWith(suffix)) {
      clean = cleanMechanismSubject(clean.slice(0, clean.length - suffix.length));
      break;
    }
  }
  return clean;
}

function mechanismSubjectAfterPrefix(original, lower, prefix) {
  if (!lower.startsWith(prefix)) return null;
  return cleanMechanismSubject(original.slice(prefix.length));
}

function mechanismSubjectBeforeSuffix(original, lower, suffix) {
  if (!lower.endsWith(suffix)) return null;
  return cleanMechanismSubject(original.slice(0, -suffix.length));
}

function mechanismSubjectBetween(original, lower, prefix, suffixes) {
  if (!lower.startsWith(prefix)) return null;
  for (const suffix of suffixes) {
    if (!lower.endsWith(suffix)) continue;
    const end = original.length - suffix.length;
    if (end <= prefix.length) return null;
    return cleanMechanismSubject(original.slice(prefix.length, end));
  }
  return null;
}

function extractHowItWorksSubject(input, lowerInput) {
  const original = cleanMechanismFragment(input);
  if (!original) return null;
  const lower = cleanMechanismFragment(lowerInput || original.toLowerCase())
    .toLowerCase();

  // The affixes are the slot-marked surface forms of the mechanism_inquiry
  // meaning (data/seed/meanings-how.lino, loaded into MEANINGS_LINO): the
  // position of the … marker classifies each form, so the matching strategy is
  // derived from the data, not from a hardcoded per-language list. Bucket order
  // — prefix, then circumfix, then suffix — and within-bucket declaration order
  // mirror extract_how_it_works_subject in src/solver_handler_how.rs (#386).
  // Suffix surfaces are end-anchored and script-disjoint across languages, so
  // the cross-language declaration order does not change which one matches.
  const forms = roleWordForms(ROLE_MECHANISM_INQUIRY);

  for (const form of forms) {
    if (form.slot !== "prefix") continue;
    const subject = mechanismSubjectAfterPrefix(original, lower, form.before);
    if (subject) return stripMechanismTail(subject);
  }

  for (const form of forms) {
    if (form.slot !== "circumfix") continue;
    const subject = mechanismSubjectBetween(original, lower, form.before, [
      form.after,
    ]);
    if (subject) return subject;
  }

  for (const form of forms) {
    if (form.slot !== "suffix") continue;
    const subject = mechanismSubjectBeforeSuffix(original, lower, form.after);
    if (subject) return subject;
  }

  return null;
}

function cleanMeaningCandidate(value) {
  const cleaned = String(value || "")
    .trim()
    .replace(/^[«»"“”‘’'`]+|[«»"“”‘’'`]+$/gu, "")
    .trim();
  if (!cleaned) return null;
  if (/^(?:it|that|this|word|the word|mean|means|meaning|i)$/iu.test(cleaned)) {
    return null;
  }
  return cleaned;
}

function extractMeaningQuestionBody(original, lower) {
  for (const prefix of [
    "what is the meaning of ",
    "what's the meaning of ",
    "what is meaning of ",
    "meaning of ",
  ]) {
    if (lower.startsWith(prefix)) {
      return cleanMeaningCandidate(original.slice(prefix.length));
    }
  }

  for (const suffix of [" mean", " means", " meaning"]) {
    if (!lower.endsWith(suffix)) continue;
    const stem = original.slice(0, -suffix.length).trim();
    const stemLower = stem.toLowerCase();
    for (const prefix of [
      "what does the word ",
      "what does ",
      "what do ",
      "what did ",
      "what is the word ",
      "what is ",
      "what's ",
      "what i ",
    ]) {
      if (stemLower.startsWith(prefix)) {
        return cleanMeaningCandidate(stem.slice(prefix.length));
      }
    }
  }

  return null;
}

function extractConceptQuery(prompt) {
  let trimmedRaw = String(prompt || "")
    .trim()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (!trimmedRaw) return null;
  trimmedRaw = stripLeadingRequest(trimmedRaw);
  let outerResponseLanguage = null;
  const outerLanguage = stripTrailingResponseLanguageMarker(
    trimmedRaw,
    trimmedRaw.toLowerCase(),
  );
  if (outerLanguage.language) {
    trimmedRaw = outerLanguage.original;
    outerResponseLanguage = outerLanguage.language;
  }

  const suffixes = conceptPatternsByKind("suffix");
  for (const suffix of suffixes) {
    if (trimmedRaw.endsWith(suffix)) {
      return finalizeConceptBody(
        trimmedRaw.slice(0, -suffix.length).trim(),
        outerResponseLanguage,
      );
    }
  }

  const lower = trimmedRaw.toLowerCase();
  const meaningBody = extractMeaningQuestionBody(trimmedRaw, lower);
  if (meaningBody) return finalizeConceptBody(meaningBody, outerResponseLanguage);

  const invertedWhoBody = extractInvertedWhoIs(trimmedRaw, lower);
  if (invertedWhoBody) {
    return finalizeConceptBody(invertedWhoBody, outerResponseLanguage);
  }

  const howItWorksSubject = extractHowItWorksSubject(trimmedRaw, lower);
  if (howItWorksSubject) {
    return finalizeConceptBody(howItWorksSubject, outerResponseLanguage);
  }

  const prefixes = conceptPatternsByKind("prefix");
  let body = null;
  for (const prefix of prefixes) {
    if (lower.startsWith(prefix)) {
      body = trimmedRaw.slice(prefix.length);
      break;
    }
  }
  if (!body) return null;
  return finalizeConceptBody(body, outerResponseLanguage);
}

function extractConceptTerm(prompt) {
  const query = extractConceptQuery(prompt);
  return query ? query.term : null;
}

function cleanWikipediaArticleQuestionTerm(value) {
  return String(value || "")
    .trim()
    .replace(/^[«»"“”‘’'`「」『』]+|[«»"“”‘’'`「」『』]+$/gu, "")
    .replace(/[?!.。！？।]+$/gu, "")
    .replace(/\s+/g, " ")
    .trim();
}

function hasWikipediaArticleQuestionShape(value) {
  const lower = String(value || "").toLowerCase();
  if (!/(?:wikipedia|wiki|википед|维基百科|維基百科|विकिपीडिया)/u.test(lower)) return false;
  const hasArticleWord = /(?:article|page|стать[ьяеию]|страниц|条目|條目|页面|頁面|文章|लेख|पृष्ठ)/u.test(lower);
  if (!hasArticleWord) return false;
  return /(?:is there|does .*have|exist|available|есть|существ|имеет|найд|назв|有|存在|有没有|是否有|吗|嗎|क्या|है|मौजूद)/u.test(lower);
}

function extractWikipediaArticleQuestionTerm(prompt) {
  const raw = cleanWikipediaArticleQuestionTerm(prompt);
  if (!raw || !hasWikipediaArticleQuestionShape(raw)) return null;

  const dashMatch = raw.match(/^(.+?)\s+[-—–:]\s+(.+)$/u);
  if (dashMatch && hasWikipediaArticleQuestionShape(dashMatch[2])) {
    return cleanWikipediaArticleQuestionTerm(dashMatch[1]);
  }

  for (const pattern of [
    /^(?:is|are)\s+there\s+(?:an?\s+)?(?:wikipedia|wiki)\s+(?:article|page)\s+(?:about|on|for)\s+(.+)$/iu,
    /^does\s+(?:wikipedia|wiki)\s+have\s+(?:an?\s+)?(?:article|page)\s+(?:about|on|for)\s+(.+)$/iu,
    /^(?:есть|существует|имеется)\s+(?:ли\s+)?(?:в\s+)?(?:русскоязычной\s+)?википедии\s+(?:отдельная\s+)?(?:статья|страница)\s+(?:о|об|про|с\s+названием)\s+(.+)$/iu,
    /^(?:есть|существует|имеется)\s+(?:ли\s+)?(?:отдельная\s+)?(?:статья|страница)\s+(?:в\s+)?(?:русскоязычной\s+)?википедии\s+(?:о|об|про|с\s+названием)\s+(.+)$/iu,
    /^(?:维基百科|維基百科)(?:上)?(?:有|存在)(?:关于|關於|名为|名為)?\s*(.+?)\s*(?:的)?(?:条目|條目|文章|页面|頁面)(?:吗|嗎)?$/iu,
    /^(.+?)\s*(?:在)?(?:维基百科|維基百科)(?:上)?(?:有|存在)(?:这样(?:的)?|這樣(?:的)?|一篇)?(?:条目|條目|文章|页面|頁面)(?:吗|嗎)?$/iu,
    /^(?:क्या\s+)?(?:विकिपीडिया|wiki)\s+(?:पर|में)\s+(.+?)\s+(?:के\s+बारे\s+में\s+)?(?:लेख|पृष्ठ)\s+(?:है|मौजूद\s+है)$/iu,
    /^(?:क्या\s+)?(.+?)\s+(?:के\s+बारे\s+में\s+)?(?:विकिपीडिया|wiki)\s+(?:पर|में)\s+(?:ऐसा\s+)?(?:लेख|पृष्ठ)\s+(?:है|मौजूद\s+है)$/iu,
  ]) {
    const match = raw.match(pattern);
    if (match) return cleanWikipediaArticleQuestionTerm(match[1]);
  }

  const trailingRussian = raw.match(/^(.+?)\s+(?:есть|существует|имеется)\s+(?:ли\s+)?(?:такая\s+)?(?:статья|страница)\s+(?:в\s+)?(?:русскоязычной\s+)?википедии$/iu);
  if (trailingRussian) return cleanWikipediaArticleQuestionTerm(trailingRussian[1]);
  const trailingHindi = raw.match(/^(.+?)\s+(?:के\s+बारे\s+में\s+)?(?:विकिपीडिया|wiki)\s+(?:पर|में)\s+(?:ऐसा\s+)?(?:लेख|पृष्ठ)\s+(?:है|मौजूद\s+है)$/iu);
  if (trailingHindi) return cleanWikipediaArticleQuestionTerm(trailingHindi[1]);
  const trailingChinese = raw.match(/^(.+?)\s*(?:在)?(?:维基百科|維基百科)(?:上)?(?:有|存在)(?:这样(?:的)?|這樣(?:的)?|一篇)?(?:条目|條目|文章|页面|頁面)(?:吗|嗎)?$/iu);
  if (trailingChinese) return cleanWikipediaArticleQuestionTerm(trailingChinese[1]);

  return null;
}

function refineWikipediaArticleQuestionLookup(term, language) {
  const exactTerm = cleanWikipediaArticleQuestionTerm(term);
  const query = {
    exactTerm,
    lookupTerm: exactTerm,
    contextOriginal: "",
  };
  const lower = exactTerm.toLowerCase();
  if (
    (language === "ru" || /[а-яё]/iu.test(exactTerm)) &&
    /\s(?:в|на)\s+(?:предложени[еяию]|предложениях|словосочетани[еяию]|словосочетаниях)$/iu.test(lower)
  ) {
    query.lookupTerm = cleanWikipediaArticleQuestionTerm(
      exactTerm.replace(/\s(?:в|на)\s+(?:предложени[еяию]|предложениях|словосочетани[еяию]|словосочетаниях)$/iu, ""),
    );
    query.contextOriginal = "грамматика";
  }
  if (
    (language === "en" || /^[\p{ASCII}\s]+$/u.test(exactTerm)) &&
    /\s+in\s+(?:a\s+)?sentences?$/iu.test(lower)
  ) {
    query.lookupTerm = cleanWikipediaArticleQuestionTerm(
      exactTerm.replace(/\s+in\s+(?:a\s+)?sentences?$/iu, ""),
    );
    query.contextOriginal = "grammar";
  }
  if (language === "hi" || /[\u0900-\u097f]/u.test(exactTerm)) {
    const prefix = exactTerm.match(/^(?:वाक्य|वाक्यों)\s+में\s+(.+)$/u);
    const suffix = exactTerm.match(/^(.+?)\s+(?:वाक्य|वाक्यों)\s+में$/u);
    const match = prefix || suffix;
    if (match) {
      query.lookupTerm = cleanWikipediaArticleQuestionTerm(match[1]);
      query.contextOriginal = "व्याकरण";
    }
  }
  if (language === "zh" || /[\u3400-\u9fff]/u.test(exactTerm)) {
    const prefix = exactTerm.match(/^(?:句子中(?:的)?|句子里(?:的)?|句中的)(.+)$/u);
    const suffix = exactTerm.match(/^(.+?)(?:在)?句子(?:中|里)$/u);
    const match = prefix || suffix;
    if (match) {
      query.lookupTerm = cleanWikipediaArticleQuestionTerm(match[1]);
      query.contextOriginal = "语法";
    }
  }
  return query;
}

// Issue #21: render a percent-encoded URL in its readable IRI form for
// display, while leaving the original encoded form available as the href.
// `decodeURI` keeps reserved URI delimiters (`; / ? : @ & = + $ , #`) intact,
// so query strings are preserved; malformed escapes fall back to the original
// string.
function humanizeUrl(url) {
  if (typeof url !== "string" || url.length === 0) return url;
  if (!url.includes("%")) return url;
  try {
    return decodeURI(url);
  } catch (_error) {
    return url;
  }
}

// Render a source URL as a Markdown link [human](encoded) when humanization
// changes anything, or the bare URL otherwise.
function renderSourceLink(source) {
  const human = humanizeUrl(source);
  return human === source ? source : `[${human}](${source})`;
}

function stripConceptIdiomSuffix(original, lower) {
  for (const suffix of [" mean", " stand for"]) {
    if (lower.endsWith(suffix)) {
      return {
        original: original.slice(0, -suffix.length).trim(),
        lower: lower.slice(0, -suffix.length).trim(),
      };
    }
  }
  return { original, lower };
}

function finalizeConceptBody(body, inheritedResponseLanguage = null) {
  let originalBase = String(body || "")
    .trim()
    .replace(/[?。.!!,,;:]+$/g, "")
    .trim();
  if (!originalBase) return null;
  let original = originalBase;
  let lower = original.toLowerCase();
  let responseLanguage = inheritedResponseLanguage || null;

  let stripped = stripTrailingResponseLanguageMarker(original, lower);
  if (stripped.language) {
    original = stripped.original;
    lower = stripped.lower;
    responseLanguage = responseLanguage || stripped.language;
  }

  const withoutIdiom = stripConceptIdiomSuffix(original, lower);
  original = withoutIdiom.original;
  lower = withoutIdiom.lower;

  stripped = stripTrailingResponseLanguageMarker(original, lower);
  if (stripped.language) {
    original = stripped.original;
    lower = stripped.lower;
    responseLanguage = responseLanguage || stripped.language;
  }

  if (!lower) return null;
  const query = splitTermAndContext(original, lower);
  if (responseLanguage) {
    query.responseLanguage = responseLanguage;
  }
  return query;
}

function tokenizeArithmetic(input) {
  const tokens = [];
  let i = 0;
  while (i < input.length) {
    const ch = input[i];
    if (ch === " " || ch === "\t" || ch === "_" || ch === ",") {
      i += 1;
      continue;
    }
    if (ch === "+") {
      tokens.push({ kind: "+" });
      i += 1;
    } else if (ch === "-" || ch === "−") {
      tokens.push({ kind: "-" });
      i += 1;
    } else if (ch === "*" || ch === "×" || ch === "·") {
      tokens.push({ kind: "*" });
      i += 1;
    } else if (ch === "/" || ch === "÷") {
      tokens.push({ kind: "/" });
      i += 1;
    } else if (ch === "%") {
      tokens.push({ kind: "%" });
      i += 1;
    } else if (ch === "^") {
      tokens.push({ kind: "^" });
      i += 1;
    } else if (ch === "(") {
      tokens.push({ kind: "(" });
      i += 1;
    } else if (ch === ")") {
      tokens.push({ kind: ")" });
      i += 1;
    } else if ((ch >= "0" && ch <= "9") || ch === ".") {
      let j = i;
      while (
        j < input.length &&
        ((input[j] >= "0" && input[j] <= "9") || input[j] === ".")
      ) {
        j += 1;
      }
      const slice = input.slice(i, j);
      const hasDecimal = slice.includes(".");
      const value = hasDecimal ? Number(slice) : BigInt(slice);
      if (hasDecimal && Number.isNaN(value)) throw new Error("unparseable");
      tokens.push({ kind: "num", value });
      i = j;
    } else {
      throw new Error("unparseable");
    }
  }
  return tokens;
}

// Issue #386: the spelled-operator and cardinal-number vocabularies are no
// longer literal arrays here. They live in the seed meanings — the
// arithmetic_operation operators (addition, subtraction, multiplication,
// division, modulo) and the cardinal_number digits (zero, один, 三, …) — and
// are read by role through the lexicon, exactly as the Rust solver does
// (contains_word_operator / contains_spelled_arithmetic in src/calculation.rs).
// These role names mirror the constants in src/seed/roles.rs.
const ROLE_ARITHMETIC_OPERATOR_WORD = "arithmetic_operator_word";
const ROLE_CARDINAL_NUMBER_WORD = "cardinal_number_word";
const ROLE_CALCULATION_REQUEST_CUE = "calculation_request_cue";
const ROLE_CALCULATION_RESULT_QUERY_CUE = "calculation_result_query_cue";
const ROLE_TIME_DURATION_CUE = "time_duration_cue";
const ROLE_POLITENESS_CUE = "politeness_cue";
const ROLE_QUANTITY_CONVERSION_CUE = "quantity_conversion_cue";
const ROLE_CALCULATION_DOMAIN_TERM = "calculation_domain_term";
const ROLE_MATH_FUNCTION_NAME = "math_function_name";
const ROLE_MEASUREMENT_UNIT = "measurement_unit";
const ROLE_PHYSICAL_DIMENSION = "physical_dimension";

// Issue #386: the spelled digit/operator → value normalization tables, derived
// from the seed at runtime exactly as the Rust solver derives them
// (Lexicon::arithmetic_normalization_tables in src/seed/meanings.rs, materialized
// into src/arithmetic_word_tables.rs for the no_std wasm worker). A cardinal or
// operator meaning carries its script-independent value surface as the one word
// form with no alphabetic character — the numeral "2", the symbol "+" — and every
// spelled surface (any language) maps onto it. Multi-word surfaces ("divided by",
// "разделить на") are returned as `phrases`, rewritten before tokenization and
// ordered longest-first so a phrase applies before any shorter phrase it
// contains; single words ("two", "плюс") are returned as `tokens`, mapped after
// the whitespace split. Cached because the lexicon never changes at runtime.
let cachedArithmeticTables = null;
function arithmeticNormalizationTables() {
  if (cachedArithmeticTables) return cachedArithmeticTables;
  const isValueSurface = (word) => !/\p{Alphabetic}/u.test(word);
  const tokens = [];
  const phrases = [];
  for (const role of [ROLE_CARDINAL_NUMBER_WORD, ROLE_ARITHMETIC_OPERATOR_WORD]) {
    for (const meaning of meaningsWithRole(role)) {
      // The value surface is the one word form with no alphabetic character: the
      // numeral for a cardinal, the symbol for an operator. Spelled surfaces in
      // every language map onto it.
      const value = meaning.words.find((word) => isValueSurface(word));
      if (value === undefined) continue;
      for (const word of meaning.words) {
        if (word === value || isValueSurface(word)) continue;
        const entry = [word, value];
        if (/\s/u.test(word)) phrases.push(entry);
        else tokens.push(entry);
      }
    }
  }
  const cmpStr = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
  const dedupe = (pairs) =>
    pairs.filter(
      (pair, index) =>
        index === 0 ||
        pair[0] !== pairs[index - 1][0] ||
        pair[1] !== pairs[index - 1][1],
    );
  // tokens.sort() in Rust orders tuples by surface then value; phrases sort by
  // descending code-point count (longest first), then surface ascending.
  tokens.sort((a, b) => cmpStr(a[0], b[0]) || cmpStr(a[1], b[1]));
  phrases.sort(
    (a, b) => [...b[0]].length - [...a[0]].length || cmpStr(a[0], b[0]),
  );
  cachedArithmeticTables = { tokens: dedupe(tokens), phrases: dedupe(phrases) };
  return cachedArithmeticTables;
}

const PERCENT_OF_CURRENCY_CODES = new Map([
  ["$", "USD"],
  ["€", "EUR"],
  ["¥", "JPY"],
  ["₹", "INR"],
  ["₽", "RUB"],
]);

const DEFAULT_CURRENCY_RATES = new Map([
  ["USD:EUR", 0.92],
  ["USD:GBP", 0.79],
  ["USD:JPY", 148.5],
  ["USD:CHF", 0.88],
  ["USD:CNY", 7.25],
  ["USD:RUB", 89.5],
  ["USD:INR", 86.5],
  ["USD:CLF", 0.022],
  ["USD:VND", 25810.0],
  ["USD:KZT", 470.0],
  ["EUR:USD", 1.087],
  ["EUR:GBP", 0.86],
  ["EUR:JPY", 161.5],
  ["EUR:CHF", 0.96],
  ["GBP:USD", 1.27],
  ["GBP:EUR", 1.16],
]);

const USD_RUB_RATE_EXPRESSION = "1 USD in RUB";

// Issue #386: the canonical ISO 4217 code is the recognizer's output, so it
// stays in code; only the recognition vocabulary lives in the seed. Mirrors the
// role -> code mapping the Rust calculator handlers resolve from the same roles.
function currencyCodeForRole(role) {
  if (role === ROLE_CURRENCY_USD_REFERENCE) return "USD";
  if (role === ROLE_CURRENCY_EUR_REFERENCE) return "EUR";
  if (role === ROLE_CURRENCY_RUB_REFERENCE) return "RUB";
  return "";
}

// Issue #386: currency vocabulary is seed data, not a hardcoded declension list.
// Walk the three currency reference roles (USD, then EUR, then RUB — the
// original recognizer's priority) and return the ISO code of the first role a
// surface matches. The matching strategy follows the surface's script, the same
// split surfacePresent already makes: Latin surfaces (the ISO codes and English
// terms, enumerated singular and plural) and CJK/Devanagari surfaces match the
// whole token exactly, so unrelated words like "rubbish" are rejected just as
// the original exact-match list rejected them; Cyrillic surfaces are stems
// matched by prefix, so every Russian declension (доллар… , руб…) is caught
// from доллар / руб without listing each inflected form. The calculator regexes
// only ever feed this Latin or Cyrillic tokens.
function currencyCodeFromWord(value) {
  const lower = String(value || "").toLowerCase();
  if (!lower) return "";
  for (const role of [
    ROLE_CURRENCY_USD_REFERENCE,
    ROLE_CURRENCY_EUR_REFERENCE,
    ROLE_CURRENCY_RUB_REFERENCE,
  ]) {
    for (const word of wordsForRole(role)) {
      if (!word) continue;
      // Cyrillic block is U+0400-U+04FF; Latin sorts below it and CJK/Devanagari
      // above, so the first codepoint tells us which matching strategy to use.
      const head = word.charCodeAt(0);
      const isCyrillic = head >= 0x0400 && head <= 0x04ff;
      if (isCyrillic ? lower.startsWith(word) : lower === word) {
        return currencyCodeForRole(role);
      }
    }
  }
  return "";
}

function defaultCurrencyRate(from, to) {
  if (from === to) return 1;
  const direct = DEFAULT_CURRENCY_RATES.get(`${from}:${to}`);
  if (direct) return direct;
  const inverse = DEFAULT_CURRENCY_RATES.get(`${to}:${from}`);
  if (inverse) return 1 / inverse;
  if (from !== "USD" && to !== "USD") {
    const fromUsd = defaultCurrencyRate(from, "USD");
    const usdTo = defaultCurrencyRate("USD", to);
    if (fromUsd && usdTo) return fromUsd * usdTo;
  }
  return null;
}

// Issue #386: the trailing currency word in an "N% of M <currency>" expression
// is seed data, not a hardcoded English list. Build the alternation from the
// same three currency reference roles currencyCodeFromWord resolves, so the
// recognizer captures exactly what the resolver understands — the ISO codes,
// the English singular/plural forms, and the Cyrillic/CJK/Devanagari names all
// come straight from the seed instead of the old usd|eur|rub|dollars?… literal.
// Longest-first under the trailing `$` anchor so "dollars" is preferred over
// "dollar"; each surface is regex-escaped. Cached because the seed is immutable
// for the worker's lifetime (matching the lazy, post-init access pattern
// currencyCodeFromWord uses, which keeps the ROLE_* consts out of the TDZ).
let percentOfExpressionRegexCache = null;
function percentOfExpressionRegex() {
  if (percentOfExpressionRegexCache) return percentOfExpressionRegexCache;
  const surfaces = [];
  const seen = new Set();
  for (const role of [
    ROLE_CURRENCY_USD_REFERENCE,
    ROLE_CURRENCY_EUR_REFERENCE,
    ROLE_CURRENCY_RUB_REFERENCE,
  ]) {
    for (const word of wordsForRole(role)) {
      const surface = String(word || "").toLowerCase();
      if (!surface || seen.has(surface)) continue;
      seen.add(surface);
      surfaces.push(surface);
    }
  }
  surfaces.sort((a, b) => b.length - a.length || (a < b ? -1 : a > b ? 1 : 0));
  const alternation = surfaces
    .map((surface) => surface.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    .join("|");
  percentOfExpressionRegexCache = new RegExp(
    `^([+-]?\\d+(?:\\.\\d+)?)\\s*%\\s+of\\s+([$€¥₹₽])?\\s*([+-]?\\d+(?:\\.\\d+)?)(?:\\s*(${alternation}))?$`,
    "i",
  );
  return percentOfExpressionRegexCache;
}

function evaluatePercentOfExpression(expression) {
  const match = String(expression || "")
    .trim()
    .match(percentOfExpressionRegex());
  if (!match) return null;
  const percent = Number(match[1]);
  const amount = Number(match[3]);
  if (!Number.isFinite(percent) || !Number.isFinite(amount)) return null;
  const currency =
    PERCENT_OF_CURRENCY_CODES.get(match[2] || "") ||
    currencyCodeFromWord(match[4]);
  const result = formatArithmeticResult((amount * percent) / 100);
  return currency ? `${result} ${currency}` : result;
}

function evaluateCurrencyConversionExpression(expression) {
  const match = String(expression || "")
    .trim()
    .match(
      /^([+-]?\d+(?:[.,]\d+)?)\s+(.+?)\s+(?:in|as|to|в|во|к)\s+(.+)$/iu,
    );
  if (!match) return null;
  const amount = Number(match[1].replace(",", "."));
  if (!Number.isFinite(amount)) return null;
  const from = currencyCodeFromWord(match[2].trim());
  const to = currencyCodeFromWord(match[3].trim());
  if (!from || !to) return null;
  const rate = defaultCurrencyRate(from, to);
  if (!rate) return null;
  return `${formatArithmeticResult(amount * rate)} ${to}`;
}

function usdRubRateDetail() {
  const rate = defaultCurrencyRate("USD", "RUB");
  if (!rate) return "";
  return `Exchange rate: 1 USD = ${formatArithmeticResult(rate)} RUB (source: default (hardcoded), date: unknown)`;
}

function evaluateUsdRubRateBasis() {
  const wasmResult = wasmEvaluateArithmetic(USD_RUB_RATE_EXPRESSION);
  if (wasmResult && wasmResult.ok) {
    return {
      formatted: wasmResult.value,
      backend: "wasm",
      detail: usdRubRateDetail(),
    };
  }
  const currencyConversionResult = evaluateCurrencyConversionExpression(
    USD_RUB_RATE_EXPRESSION,
  );
  if (currencyConversionResult === null) return null;
  return {
    formatted: currencyConversionResult,
    backend: "js-currency",
    detail: usdRubRateDetail(),
  };
}

// Issue #386: recognise the USD/RUB rate-basis question by *meaning*, not by a
// hardcoded per-language word list. The surface forms live once in
// data/seed/meanings-calculator.lino and are queried by semantic role, matched
// as raw substrings — the JS mirror of mentions_role_raw and of
// asks_for_usd_rate_basis in src/solver_handlers/calculator_rate.rs.
//
// A prompt references US dollars (an exchange_rate between currencies AND the
// us_dollar currency) when both currency roles are present.
function mentionsUsdRate(normalized) {
  return (
    lexiconMentionsRoleSubstring(ROLE_EXCHANGE_RATE_REFERENCE, normalized) &&
    lexiconMentionsRoleSubstring(ROLE_CURRENCY_USD_REFERENCE, normalized)
  );
}

// A prompt asks what the assistant uses as the basis for a calculation when the
// calculation_basis role is present (the question side of "which rate do you
// use for calculations").
function mentionsRateCalculationBasis(normalized) {
  return lexiconMentionsRoleSubstring(ROLE_CALCULATION_BASIS_REFERENCE, normalized);
}

function tryCalculatorRateBasis(normalized, language) {
  if (!mentionsUsdRate(normalized) || !mentionsRateCalculationBasis(normalized)) {
    return null;
  }
  const evaluation = evaluateUsdRubRateBasis();
  if (!evaluation) {
    const content =
      language === "ru"
        ? "Я распознал вопрос о курсе USD/RUB для расчетов, но калькулятор не смог его вычислить."
        : "I recognized this as a question about the USD/RUB rate used for calculations, but the calculator could not evaluate it.";
    return {
      intent: "calculation_error",
      content,
      confidence: 0.3,
      evidence: ["calculation_error:USD/RUB"],
    };
  }
  const calculationBody = `${USD_RUB_RATE_EXPRESSION} = ${evaluation.formatted}`;
  let content;
  if (language === "ru") {
    content = `При расчетах валюты я использую link-calculator. Для USD/RUB он возвращает: ${calculationBody}.`;
  } else if (language === "hi") {
    content = `मुद्रा गणनाओं के लिए मैं link-calculator का उपयोग करता हूं। USD/RUB के लिए वह लौटाता है: ${calculationBody}.`;
  } else if (language === "zh") {
    content = `货币计算时我使用 link-calculator。USD/RUB 返回: ${calculationBody}.`;
  } else {
    content = `For currency calculations I use link-calculator. For USD/RUB it returns: ${calculationBody}.`;
  }
  if (evaluation.detail) {
    let details = "Calculator rate details";
    if (language === "ru") {
      details = "Детали курса от калькулятора";
    } else if (language === "hi") {
      details = "कैलकुलेटर दर विवरण";
    } else if (language === "zh") {
      details = "计算器汇率详情";
    }
    content += `\n\n${details}: ${evaluation.detail}`;
  }
  return {
    intent: "calculation",
    content,
    confidence: 1.0,
    evidence: [
      `calculation:${calculationBody}`,
      `calculation_backend:${evaluation.backend}`,
      "calculation_rate_basis:USD/RUB",
    ],
  };
}

// Rewrite "N% of M" percentage-of phrases into explicit arithmetic the parser
// can evaluate: "8% of 500" -> "( 8 * 500 / 100 )". Mirrors the Rust
// `rewrite_percent_of` helper so the JS fallback agrees with the WASM worker:
// "55 * 8% of 500" evaluates to 2200 (issue #334). A bare `%` not followed by
// "of" is left untouched so it still parses as the modulo operator.
function rewritePercentOf(expression) {
  const isNumber = (token) => token.length > 0 && /^[0-9.]+$/.test(token);
  const tokens = String(expression).split(/\s+/).filter(Boolean);
  const out = [];
  let index = 0;
  while (index < tokens.length) {
    let percent = null;
    let consumed = 0;
    const token = tokens[index];
    if (token.endsWith("%") && isNumber(token.slice(0, -1))) {
      percent = token.slice(0, -1);
      consumed = 1;
    } else if (isNumber(token) && tokens[index + 1] === "%") {
      percent = token;
      consumed = 2;
    }
    const after = index + consumed;
    if (
      percent !== null &&
      tokens[after] === "of" &&
      tokens[after + 1] !== undefined &&
      isNumber(tokens[after + 1])
    ) {
      out.push("(", percent, "*", tokens[after + 1], "/", "100", ")");
      index = after + 2;
      continue;
    }
    out.push(token);
    index += 1;
  }
  return out.join(" ");
}

function normalizeArithmeticWords(expression) {
  const { tokens, phrases } = arithmeticNormalizationTables();
  const lower = String(expression).toLowerCase();
  // Multi-word operator phrases first, longest-first (the table is pre-sorted),
  // each padded with spaces so it only rewrites on a token boundary — exactly as
  // the Rust normalize_expression does before it splits on whitespace.
  let padded = ` ${lower} `;
  for (const [phrase, value] of phrases) {
    padded = padded.replaceAll(` ${phrase} `, ` ${value} `);
  }
  const tokenMap = new Map(tokens);
  const mapped = padded
    .split(/\s+/)
    .filter(Boolean)
    .map((token) => tokenMap.get(token) || token)
    .join(" ");
  return rewritePercentOf(mapped);
}

const EXACT_ARITHMETIC_EXPONENT_LIMIT = 10000n;

function isArithmeticBigInt(value) {
  return typeof value === "bigint";
}

function arithmeticToNumber(value) {
  return isArithmeticBigInt(value) ? Number(value) : value;
}

function arithmeticIsZero(value) {
  return value === 0 || value === 0n;
}

function arithmeticEnsureFinite(value) {
  if (typeof value === "number" && !Number.isFinite(value)) {
    throw new Error("overflow");
  }
  return value;
}

function arithmeticNegate(value) {
  return isArithmeticBigInt(value) ? -value : -value;
}

function arithmeticAdd(left, right) {
  if (isArithmeticBigInt(left) && isArithmeticBigInt(right)) return left + right;
  return arithmeticEnsureFinite(arithmeticToNumber(left) + arithmeticToNumber(right));
}

function arithmeticSub(left, right) {
  if (isArithmeticBigInt(left) && isArithmeticBigInt(right)) return left - right;
  return arithmeticEnsureFinite(arithmeticToNumber(left) - arithmeticToNumber(right));
}

function arithmeticMul(left, right) {
  if (isArithmeticBigInt(left) && isArithmeticBigInt(right)) return left * right;
  return arithmeticEnsureFinite(arithmeticToNumber(left) * arithmeticToNumber(right));
}

function arithmeticDiv(left, right) {
  if (arithmeticIsZero(right)) throw new Error("division by zero");
  if (
    isArithmeticBigInt(left) &&
    isArithmeticBigInt(right) &&
    left % right === 0n
  ) {
    return left / right;
  }
  return arithmeticEnsureFinite(arithmeticToNumber(left) / arithmeticToNumber(right));
}

function arithmeticRem(left, right) {
  if (arithmeticIsZero(right)) throw new Error("division by zero");
  if (isArithmeticBigInt(left) && isArithmeticBigInt(right)) return left % right;
  return arithmeticEnsureFinite(arithmeticToNumber(left) % arithmeticToNumber(right));
}

function arithmeticPow(left, right) {
  if (isArithmeticBigInt(left) && isArithmeticBigInt(right) && right >= 0n) {
    if (right > EXACT_ARITHMETIC_EXPONENT_LIMIT) throw new Error("overflow");
    return left ** right;
  }
  return arithmeticEnsureFinite(
    Math.pow(arithmeticToNumber(left), arithmeticToNumber(right)),
  );
}

function evaluateArithmetic(expression) {
  const normalized = normalizeArithmeticWords(expression);
  const tokens = tokenizeArithmetic(normalized);
  if (tokens.length === 0) {
    throw new Error("empty");
  }
  let cursor = 0;
  const peek = () => tokens[cursor];
  const advance = () => tokens[cursor++];
  function parsePrimary() {
    const tok = advance();
    if (!tok) throw new Error("unparseable");
    if (tok.kind === "num") return tok.value;
    if (tok.kind === "(") {
      const inner = parseAdditive();
      const close = advance();
      if (!close || close.kind !== ")") throw new Error("unbalanced");
      return inner;
    }
    throw new Error("unparseable");
  }
  function parsePower() {
    let left = parsePrimary();
    const tok = peek();
    if (tok && tok.kind === "^") {
      advance();
      left = arithmeticPow(left, parseUnary());
    }
    return left;
  }
  function parseUnary() {
    const tok = peek();
    if (tok && tok.kind === "-") {
      advance();
      return arithmeticNegate(parseUnary());
    }
    if (tok && tok.kind === "+") {
      advance();
      return parseUnary();
    }
    return parsePower();
  }
  function parseMultiplicative() {
    let left = parseUnary();
    while (true) {
      const tok = peek();
      if (!tok || (tok.kind !== "*" && tok.kind !== "/" && tok.kind !== "%")) {
        break;
      }
      const op = tok.kind;
      advance();
      const right = parseUnary();
      if (op === "*") {
        left = arithmeticMul(left, right);
      } else if (op === "/") {
        left = arithmeticDiv(left, right);
      } else {
        left = arithmeticRem(left, right);
      }
    }
    return left;
  }
  function parseAdditive() {
    let left = parseMultiplicative();
    while (true) {
      const tok = peek();
      if (!tok || (tok.kind !== "+" && tok.kind !== "-")) break;
      const isPlus = tok.kind === "+";
      advance();
      const right = parseMultiplicative();
      left = isPlus ? arithmeticAdd(left, right) : arithmeticSub(left, right);
    }
    return left;
  }
  const value = parseAdditive();
  if (cursor !== tokens.length) {
    throw new Error("unparseable");
  }
  return value;
}

function formatArithmeticResult(value) {
  if (isArithmeticBigInt(value)) return value.toString();
  if (!Number.isFinite(value)) return "non-finite";
  if (Math.abs(value % 1) === 0 && Math.abs(value) < 1e15) {
    return value.toFixed(0);
  }
  const rendered = Math.abs(value) >= 1e21 ? String(value) : value.toFixed(10);
  if (/[eE]/.test(rendered)) return rendered;
  const trimmed = rendered.replace(/0+$/, "").replace(/\.$/, "");
  return trimmed === "" || trimmed === "-" ? "0" : trimmed;
}

function escapeCalculationMarkdown(value) {
  return String(value)
    .replace(/\\/g, "\\\\")
    .replace(/\*/g, "\\*")
    .replace(/_/g, "\\_");
}

const EQUATION_EPSILON = 1e-10;

function equationNearlyZero(value) {
  return Math.abs(value) <= EQUATION_EPSILON;
}

function isEquationUnknownPlaceholder(character) {
  return character === "?" || character === "*";
}

function parseEquationNumber(input, state, constant) {
  const start = state.position;
  let hasDigit = false;
  let hasDot = false;
  while (/[0-9.]/.test(input[state.position] || "")) {
    if (input[state.position] === ".") {
      if (hasDot) break;
      hasDot = true;
    } else {
      hasDigit = true;
    }
    state.position += 1;
  }
  if (!hasDigit) throw new Error("expression could not be parsed");
  const value = Number(input.slice(start, state.position));
  if (!Number.isFinite(value)) throw new Error("expression could not be parsed");
  return constant(value);
}

function parseEquationVariable(input, state, onName) {
  const start = state.position;
  if (isEquationUnknownPlaceholder(input[state.position] || "")) {
    state.position += 1;
  } else {
    while (/[\p{L}_]/u.test(input[state.position] || "")) {
      state.position += 1;
    }
  }
  const name = input.slice(start, state.position);
  if (!name) throw new Error("expression could not be parsed");
  onName(name);
  return name;
}

function linearConstant(value) {
  return { terms: Object.create(null), constant: value };
}

function linearVariable(name) {
  const terms = Object.create(null);
  terms[name] = 1;
  return { terms, constant: 0 };
}

function linearEntries(value) {
  return Object.entries(value.terms).filter((entry) => !equationNearlyZero(entry[1]));
}

function linearHasVariable(value) {
  return linearEntries(value).length > 0;
}

function linearAdd(left, right) {
  const terms = Object.create(null);
  for (const [name, coefficient] of linearEntries(left)) {
    terms[name] = (terms[name] || 0) + coefficient;
  }
  for (const [name, coefficient] of linearEntries(right)) {
    terms[name] = (terms[name] || 0) + coefficient;
  }
  return { terms, constant: left.constant + right.constant };
}

function linearSubtract(left, right) {
  const terms = Object.create(null);
  for (const [name, coefficient] of linearEntries(left)) {
    terms[name] = (terms[name] || 0) + coefficient;
  }
  for (const [name, coefficient] of linearEntries(right)) {
    terms[name] = (terms[name] || 0) - coefficient;
  }
  return { terms, constant: left.constant - right.constant };
}

function linearScale(value, scalar) {
  const terms = Object.create(null);
  for (const [name, coefficient] of linearEntries(value)) {
    terms[name] = coefficient * scalar;
  }
  return { terms, constant: value.constant * scalar };
}

function linearMultiply(left, right) {
  if (linearHasVariable(left) && linearHasVariable(right)) {
    throw new Error("non-linear equation");
  }
  if (linearHasVariable(left)) return linearScale(left, right.constant);
  if (linearHasVariable(right)) return linearScale(right, left.constant);
  return linearConstant(left.constant * right.constant);
}

function linearDivide(left, right) {
  if (linearHasVariable(right)) throw new Error("variable denominator");
  if (equationNearlyZero(right.constant)) throw new Error("division by zero");
  return linearScale(left, 1 / right.constant);
}

function parseLinearExpression(input) {
  const state = { position: 0 };
  const variables = [];

  function peek() {
    return input[state.position] || "";
  }

  function skipWhitespace() {
    while (/\s/.test(peek())) state.position += 1;
  }

  function consume(expected) {
    if (peek() === expected) {
      state.position += 1;
      return true;
    }
    return false;
  }

  function rememberVariable(name) {
    if (!variables.includes(name)) variables.push(name);
  }

  function parseExpression() {
    let value = parseTerm();
    while (true) {
      skipWhitespace();
      if (consume("+")) {
        value = linearAdd(value, parseTerm());
      } else if (consume("-") || consume("−")) {
        value = linearSubtract(value, parseTerm());
      } else {
        return value;
      }
    }
  }

  function parseTerm() {
    let value = parseFactor();
    while (true) {
      skipWhitespace();
      if (consume("*") || consume("×") || consume("·")) {
        value = linearMultiply(value, parseFactor());
      } else if (consume("/") || consume("÷")) {
        value = linearDivide(value, parseFactor());
      } else {
        return value;
      }
    }
  }

  function parseFactor() {
    skipWhitespace();
    if (consume("+")) return parseFactor();
    if (consume("-") || consume("−")) return linearScale(parseFactor(), -1);
    if (consume("(")) {
      const value = parseExpression();
      skipWhitespace();
      if (!consume(")")) throw new Error("unbalanced parentheses");
      return value;
    }
    if (/[0-9.]/.test(peek())) return parseEquationNumber(input, state, linearConstant);
    if (/\p{L}/u.test(peek()) || isEquationUnknownPlaceholder(peek())) {
      return linearVariable(parseEquationVariable(input, state, rememberVariable));
    }
    throw new Error("expression could not be parsed");
  }

  const value = parseExpression();
  skipWhitespace();
  if (state.position !== input.length) throw new Error("expression could not be parsed");
  return { value, variables };
}
