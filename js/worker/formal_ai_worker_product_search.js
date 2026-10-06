// Issues #800 and #872 product search, browser twin (issue #1188 JS parity).
// Mirrors rust/src/solver_handlers/product_search.rs: marketplaces, request
// cues, qualifier constraints and product nouns are read from
// data/seed/product-search-cues.lino, and the answer renders the English
// product_search_result template from
// multilingual-responses-product-search.lino exactly as the native handler
// does. Nothing is fetched; the link is the exact query that would run.

const PRODUCT_SEARCH_CUES_FILE = "product-search-cues.lino";
const PRODUCT_SEARCH_INTENT = "product_search";

/**
 * The lowercased, non-empty `phrase` values directly under a record.
 * @param {object} record
 * @returns {Array<string>}
 */
function productSearchPhrases(record) {
  return record.children
    .filter((child) => child.name === "phrase")
    .map((child) => child.value.trim().toLowerCase())
    .filter((phrase) => phrase.length > 0);
}

/**
 * The catalogue from the seed file's `product_search` root.
 * @returns {{marketplaces: Array<object>, constraints: Array<object>, productNouns: Array<object>, cues: Array<string>}}
 */
function productSearchCatalogue() {
  const catalogue = { marketplaces: [], constraints: [], productNouns: [], cues: [] };
  const text = seedRawText(SEED_RAW, PRODUCT_SEARCH_CUES_FILE);
  if (!text) return catalogue;
  const root = parseLinoTree(text).children.find((child) => child.name === "product_search");
  if (root === undefined) return catalogue;
  for (const record of root.children) {
    switch (record.name) {
      case "marketplace":
        if (record.value.length > 0) {
          catalogue.marketplaces.push({
            name: record.value,
            host: childValue(record, "host"),
            linkTemplate: childValue(record, "link"),
            phrases: productSearchPhrases(record),
          });
        }
        break;
      case "constraint": {
        const name = childValue(record, "name");
        if (name.length > 0) {
          catalogue.constraints.push({
            name: name,
            phrases: productSearchPhrases(record),
            advice: childValue(record, "advice"),
          });
        }
        break;
      }
      case "product_noun": {
        const noun = childValue(record, "noun");
        if (noun.length > 0) {
          catalogue.productNouns.push({
            noun: noun,
            phrases: productSearchPhrases(record),
            advice: childValue(record, "advice"),
          });
        }
        break;
      }
      case "intent_cues":
        if (childValue(record, "intent") === PRODUCT_SEARCH_INTENT) {
          catalogue.cues = productSearchPhrases(record);
        }
        break;
      default:
        break;
    }
  }
  return catalogue;
}

/**
 * Percent-encode UTF-8 bytes; unreserved characters stay, space is %20.
 * @param {string} value
 * @returns {string}
 */
function productSearchPercentEncode(value) {
  const bytes = new TextEncoder().encode(value);
  let out = "";
  for (const byte of bytes) {
    const character = String.fromCharCode(byte);
    if (/[A-Za-z0-9\-_.~]/.test(character)) {
      out += character;
    } else if (byte === 32) {
      out += "%20";
    } else {
      out += `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
    }
  }
  return out;
}

/**
 * Trim characters matching `drop` from both ends.
 * @param {string} token
 * @param {function(string): boolean} drop
 * @returns {string}
 */
function productSearchTrim(token, drop) {
  const characters = Array.from(token);
  let start = 0;
  let end = characters.length;
  while (start < end && drop(characters[start])) start += 1;
  while (end > start && drop(characters[end - 1])) end -= 1;
  return characters.slice(start, end).join("");
}

/**
 * A model code: >= 4 uppercase ASCII letters/digits/hyphens with a digit and
 * a letter, like `A325-45`.
 * @param {string} token
 * @returns {boolean}
 */
function productSearchIsModelCode(token) {
  const kept = Array.from(token).filter((character) => /[A-Za-z0-9-]/.test(character)).join("");
  return kept.length >= 4 && /[0-9]/.test(kept) && /[A-Za-z]/.test(kept) && /^[A-Z0-9-]+$/.test(kept);
}

/**
 * The matched product nouns plus model codes and the brand words before them.
 * @param {string} prompt
 * @param {Array<object>} nouns
 * @returns {Array<string>}
 */
function productSearchTerms(prompt, nouns) {
  const lower = prompt.toLowerCase();
  const terms = [];
  for (const noun of nouns) {
    if (noun.phrases.some((phrase) => lower.includes(phrase))) terms.push(noun.noun);
  }
  const tokens = textTransformWords(prompt);
  const modelTrim = (token) => productSearchTrim(token, (character) => !/[A-Za-z0-9-]/.test(character));
  for (const token of tokens) {
    const trimmed = modelTrim(token);
    if (productSearchIsModelCode(trimmed) && !terms.includes(trimmed)) terms.push(trimmed);
  }
  for (let index = 0; index < tokens.length; index += 1) {
    if (!productSearchIsModelCode(modelTrim(tokens[index]))) continue;
    for (let lookback = index - 1; lookback >= Math.max(0, index - 3); lookback -= 1) {
      const candidate = productSearchTrim(tokens[lookback], (character) => !/[A-Za-z0-9]/.test(character));
      const characters = Array.from(candidate);
      if (
        characters.every((character) => /\p{Alphabetic}/u.test(character)) &&
        new TextEncoder().encode(candidate).length >= 3 &&
        characters.length > 0 && /\p{Uppercase}/u.test(characters[0]) &&
        !terms.some((term) => term.toLowerCase() === candidate.toLowerCase())
      ) {
        terms.push(candidate);
      }
    }
  }
  return terms;
}

/**
 * Compose the site-scoped search a shopping request describes
 * (`handle_product_search`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryProductSearch(prompt, normalized) {
  const text = String(prompt || "");
  const normal = String(normalized || "");
  const catalogue = productSearchCatalogue();
  const lower = text.toLowerCase();
  const mentions = (phrase) => normal.includes(phrase) || lower.includes(phrase);
  const marketplace = catalogue.marketplaces.find((item) => item.phrases.some(mentions));
  if (marketplace === undefined) return null;
  let nounSurface = null;
  for (const noun of catalogue.productNouns) {
    const phrase = noun.phrases.find((candidate) => lower.includes(candidate));
    if (phrase !== undefined) {
      nounSurface = phrase;
      break;
    }
  }
  const cued = catalogue.cues.some(mentions);
  if (!cued && nounSurface === null) return null;
  const terms = productSearchTerms(text, catalogue.productNouns);
  const matched = catalogue.productNouns.find((noun) => terms.includes(noun.noun));
  const matchedNoun = matched === undefined ? null : matched;
  const constraints = catalogue.constraints.filter((constraint) => constraint.phrases.some(mentions));

  const trace = [`product_search:marketplace:${marketplace.name}`]
    .concat(terms.map((term) => `product_search:term:${term}`))
    .concat(constraints.map((constraint) => `product_search:constraint:${constraint.name}`));

  const queryTerms = (nounSurface === null ? [] : [nounSurface]).concat(
    terms.filter((term) => matchedNoun === null || term !== matchedNoun.noun),
  );
  const link = marketplace.linkTemplate
    .split("{query}")
    .join(productSearchPercentEncode(queryTerms.join(" ")));
  const constraintsText = constraints.length === 0
    ? "(none stated)"
    : constraints.map((constraint) => constraint.name).join(", ");
  let advice = matchedNoun === null
    ? "confirm the exact model and seller region before ordering"
    : matchedNoun.advice;
  for (const constraint of constraints) {
    if (constraint.advice.length > 0) advice = `${advice} ${constraint.advice}`;
  }
  let product = text.trim();
  if (nounSurface !== null) product = nounSurface;
  else if (terms.length > 0) product = terms[0];
  const template = textTransformLocalizedResponse("product_search_result", "en") || "";
  const body = textTransformFill(template, [
    ["product", product],
    ["marketplace", marketplace.name.split("_").join(" ")],
    ["link", link],
    ["constraints", constraintsText],
    ["advice", advice],
  ]);
  return {
    intent: PRODUCT_SEARCH_INTENT,
    content: body,
    confidence: 0.75,
    evidence: ["handler:product_search", "response:product_search_result", `marketplace:${marketplace.name}`],
    trace: trace,
  };
}
