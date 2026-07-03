// Relative-meta-logic mirror of `src/relative_meta_logic.rs` and
// `src/statement_verification.rs`. Issue #535 asks us to weigh each statement
// with a relative probability: assume it true, raise it with trusted original
// first sources, ignore reposts. The web worker replays the exact same
// deterministic offline plan into its evidence log so the website matches the
// Rust `FormalAiEngine` byte-for-byte.
//
// This module is the issue #535 surface: it holds both the relative-meta-logic
// statement weigher and the document-verification handler that calls it
// (`tryDocumentOriginalityCheck`, appended below), so worker_19 stays under the
// 1500-line ceiling enforced by scripts/check-file-size.rs. Function
// declarations hoist across the concatenated worker bundle, so worker_20's
// dispatcher reaches the handler regardless of module order.
const RML_ASSUMED_TRUE_PRIOR = 0.6;
const RML_TRUSTED_SOURCE_POLICY = [
  { slug: "original_first_party", weight: 1.0 },
  { slug: "original_journalism", weight: 0.85 },
  { slug: "independent_corroboration", weight: 0.5 },
  { slug: "unoriginal", weight: 0.0 },
];
// Sentence terminators across every script the solver recognises, mirroring
// `SENTENCE_TERMINATORS` in `src/statement_verification.rs`.
const RML_SENTENCE_TERMINATORS = new Set([
  ".",
  "!",
  "?",
  "。",
  "！",
  "？",
  "।",
  "॥",
  "؟",
  "\n",
]);
const RML_MIN_STATEMENT_WORDS = 3;
const RML_MIN_STATEMENT_CHARS = 6;
const MARKET_PRICE_CLAIM_STATUS_CONTRADICTED = "contradicted";
const MARKET_PRICE_CLAIM_STATUS_WITHIN_RANGE = "within_recorded_range";

// Market-price references are data-driven, mirroring `market_price_assets()` in
// `src/seed/market_price_references.rs`. Every asset alias (across every
// language) and every observed price range lives in
// `data/seed/market-price-references.lino`, so issue #493's fact check covers
// the whole class of assets/periods/languages with no per-language phrase list
// baked into JavaScript (issue #386 convention). The registry is parsed once
// from `MARKET_PRICE_REFERENCES_LINO` (hydrated from the seed bundle in
// `formal_ai_worker_00.js`) and flattened to one entry per asset-period so the
// lookup shape matches the Rust `MarketPricePeriod` slice.
let cachedMarketPriceReferences = null;

function marketPriceLinoChildValue(node, name) {
  for (const child of node.children) {
    if (child.name === name) return child.value || "";
  }
  return "";
}

function marketPriceAssetAliases(assetNode) {
  const aliases = [];
  for (const lexeme of assetNode.children) {
    if (lexeme.name !== "lexeme") continue;
    for (const surface of lexeme.children) {
      if (surface.name !== "surface") continue;
      const text = marketPriceLinoChildValue(surface, "text");
      if (text && !aliases.includes(text)) aliases.push(text);
    }
  }
  return aliases;
}

function parseMarketPriceReferences() {
  const references = [];
  if (!MARKET_PRICE_REFERENCES_LINO) return references;
  const root = parseLinoTree(MARKET_PRICE_REFERENCES_LINO);
  const registry =
    root.children.find((child) => child.name === "market_price_references") || root;
  for (const assetNode of registry.children) {
    if (assetNode.name !== "asset") continue;
    const asset = assetNode.value || "";
    const assetLabel = marketPriceLinoChildValue(assetNode, "label");
    const quoteCurrency = marketPriceLinoChildValue(assetNode, "quote-currency");
    const aliases = marketPriceAssetAliases(assetNode);
    for (const reference of assetNode.children) {
      if (reference.name !== "reference") continue;
      references.push({
        asset,
        assetLabel,
        aliases,
        quoteCurrency,
        period: reference.value || "",
        sourceId: marketPriceLinoChildValue(reference, "source-id"),
        sourceLabel: marketPriceLinoChildValue(reference, "source-label"),
        sourceUrl: marketPriceLinoChildValue(reference, "source-url"),
        observedMinPrice:
          Number.parseFloat(marketPriceLinoChildValue(reference, "observed-min-price")) || 0,
        observedMinDate: marketPriceLinoChildValue(reference, "observed-min-date"),
        observedMaxPrice:
          Number.parseFloat(marketPriceLinoChildValue(reference, "observed-max-price")) || 0,
        observedMaxDate: marketPriceLinoChildValue(reference, "observed-max-date"),
      });
    }
  }
  return references;
}

function marketPriceReferences() {
  if (!cachedMarketPriceReferences) {
    cachedMarketPriceReferences = parseMarketPriceReferences();
  }
  return cachedMarketPriceReferences;
}

function rmlDecimal(value) {
  // Match Rust's fixed 6-decimal grid so identical inputs serialise identically.
  return Number(value).toFixed(6);
}

function marketPriceDecimal(value) {
  return Number(value).toFixed(2);
}

function documentOriginalityTextSamplePrefixValue(line) {
  for (const prefix of ["Text excerpt:", "Text sample:", "OCR text:"]) {
    if (line.startsWith(prefix)) return line.slice(prefix.length);
  }
  return null;
}

function isDocumentOriginalityAttachmentFileLine(line) {
  return /^\d+\.\s+.+?\s+\([^)]*\)$/u.test(line);
}

function isDocumentOriginalityContextBoundary(line) {
  return (
    /^attached files:$/iu.test(line) ||
    isDocumentOriginalityAttachmentFileLine(line) ||
    line.startsWith("Text omitted:") ||
    line.startsWith("Text unavailable:") ||
    line.startsWith("OCR unavailable:")
  );
}

function documentOriginalityFullTextSamples(prompt) {
  const samples = [];
  let current = null;
  const pushCurrent = () => {
    if (typeof current !== "string") return;
    const sample = current.trim();
    if (sample) samples.push(sample);
    current = null;
  };
  for (const rawLine of String(prompt || "").split(/\r?\n/u)) {
    const line = rawLine.trim();
    const prefixValue = documentOriginalityTextSamplePrefixValue(line);
    if (prefixValue !== null) {
      pushCurrent();
      current = prefixValue.trim();
      continue;
    }
    if (typeof current !== "string") continue;
    if (!line || isDocumentOriginalityContextBoundary(line)) {
      pushCurrent();
      continue;
    }
    current = current ? `${current}\n${line}` : line;
  }
  pushCurrent();
  return samples;
}

function documentOriginalityFullTextSample(prompt) {
  return documentOriginalityFullTextSamples(prompt).join("\n\n");
}

function extractVerificationStatements(sample) {
  const statements = [];
  let current = "";
  const pushStatement = () => {
    const trimmed = current.trim();
    current = "";
    if (!trimmed) return;
    const wordCount = trimmed.split(/\s+/u).filter(Boolean).length;
    const charCount = Array.from(trimmed).filter(
      (character) => !/\s/u.test(character),
    ).length;
    if (wordCount < RML_MIN_STATEMENT_WORDS && charCount < RML_MIN_STATEMENT_CHARS) {
      return;
    }
    statements.push(trimmed);
  };
  for (const character of Array.from(String(sample || ""))) {
    if (RML_SENTENCE_TERMINATORS.has(character)) {
      pushStatement();
    } else {
      current += character;
    }
  }
  pushStatement();
  return statements;
}

function verificationGroundingQuery(statement) {
  const condensed = String(statement || "")
    .split(/\s+/u)
    .filter(Boolean)
    .join(" ");
  return `"${condensed}" fact check source`;
}

// The deterministic offline assessment of a single statement: assumed-true
// prior with no evidence collected yet. Mirrors `StatementAssessment::assess`
// with an empty evidence slice.
function verificationAssessmentTrace() {
  const prior = RML_ASSUMED_TRUE_PRIOR;
  const support = 0;
  const contradiction = 0;
  return verificationAssessmentTraceWithMasses(support, contradiction);
}

function verificationAssessmentTraceWithMasses(support, contradiction) {
  const prior = RML_ASSUMED_TRUE_PRIOR;
  const raised = 1 - (1 - prior) * (1 - support);
  const posterior = raised * (1 - contradiction);
  return (
    `prior=${rmlDecimal(prior)} support=${rmlDecimal(support)} ` +
    `contradiction=${rmlDecimal(contradiction)} posterior=${rmlDecimal(posterior)} ignored=0`
  );
}

function isAsciiAlnum(character) {
  return /^[0-9a-z]$/iu.test(character || "");
}

function aliasOccursAt(lower, alias, position) {
  const before = position > 0 ? lower.charAt(position - 1) : "";
  const after = lower.charAt(position + alias.length);
  const first = alias.charAt(0);
  const last = alias.charAt(alias.length - 1);
  const startsWithWord = isAsciiAlnum(first);
  const endsWithWord = isAsciiAlnum(last);
  return (
    (!startsWithWord || !isAsciiAlnum(before)) &&
    (!endsWithWord || !isAsciiAlnum(after))
  );
}

function aliasOccurs(fragment, alias) {
  if (/^[\x00-\x7F]*$/u.test(alias)) {
    const lower = String(fragment || "").toLocaleLowerCase("en-US");
    const needle = alias.toLocaleLowerCase("en-US");
    let position = lower.indexOf(needle);
    while (position >= 0) {
      if (aliasOccursAt(lower, needle, position)) return true;
      position = lower.indexOf(needle, position + 1);
    }
    return false;
  }
  return String(fragment || "").includes(alias);
}

function assetPositions(line) {
  // Mirror `asset_positions` in `src/statement_verification.rs`: scan every
  // alias of every asset so a line naming two assets splits into one fragment
  // per asset. ASCII aliases match case-insensitively with a word boundary;
  // non-ASCII aliases match as substrings against the original line.
  const original = String(line || "");
  const lower = original.toLocaleLowerCase("en-US");
  const positions = [];
  for (const reference of marketPriceReferences()) {
    for (const alias of reference.aliases) {
      if (/^[\x00-\x7F]*$/u.test(alias)) {
        const needle = alias.toLocaleLowerCase("en-US");
        let position = lower.indexOf(needle);
        while (position >= 0) {
          if (aliasOccursAt(lower, needle, position)) positions.push(position);
          position = lower.indexOf(needle, position + 1);
        }
      } else {
        let position = original.indexOf(alias);
        while (position >= 0) {
          positions.push(position);
          position = original.indexOf(alias, position + 1);
        }
      }
    }
  }
  return Array.from(new Set(positions)).sort((left, right) => left - right);
}

function marketPriceFragments(sample) {
  const fragments = [];
  for (const rawLine of String(sample || "").split(/\r?\n/u)) {
    const line = rawLine.split(/\s+/u).filter(Boolean).join(" ");
    if (!line) continue;
    const positions = assetPositions(line);
    if (positions.length <= 1) {
      fragments.push(line);
      continue;
    }
    for (let index = 0; index < positions.length; index += 1) {
      const start = positions[index];
      const end = index + 1 < positions.length ? positions[index + 1] : line.length;
      const fragment = line.slice(start, end).trim();
      if (fragment) fragments.push(fragment);
    }
  }
  return fragments;
}

function extractMarketPriceYear(fragment) {
  const matches = String(fragment || "").match(/[0-9]+/gu) || [];
  for (const candidate of matches) {
    if (candidate.length !== 4) continue;
    const year = Number.parseInt(candidate, 10);
    if (year >= 1900 && year <= 2100) return candidate;
  }
  return "";
}

function parseNumberFromStart(value) {
  let normalized = "";
  let sawDigit = false;
  for (const character of Array.from(String(value || "").trimStart())) {
    if (/[0-9]/u.test(character)) {
      sawDigit = true;
      normalized += character;
    } else if (character === ".") {
      normalized += character;
    } else if (character === "," || character === "_" || character === " " || character === "\u00a0") {
      continue;
    } else {
      break;
    }
  }
  if (!sawDigit) return null;
  const parsed = Number.parseFloat(normalized);
  return Number.isFinite(parsed) ? parsed : null;
}

function parseNumberBefore(value) {
  const characters = Array.from(String(value || "").trimEnd()).reverse();
  const kept = [];
  for (const character of characters) {
    if (
      /[0-9]/u.test(character) ||
      character === "," ||
      character === "." ||
      character === "_" ||
      character === " " ||
      character === "\u00a0"
    ) {
      kept.push(character);
    } else {
      break;
    }
  }
  return parseNumberFromStart(kept.reverse().join(""));
}

function extractCurrencyAmount(fragment, period) {
  const text = String(fragment || "");
  let searchStart = 0;
  while (searchStart < text.length) {
    const dollar = text.indexOf("$", searchStart);
    if (dollar < 0) break;
    const price = parseNumberFromStart(text.slice(dollar + 1));
    if (price !== null && Math.abs(price - Number(period)) > Number.EPSILON) {
      return price;
    }
    searchStart = dollar + 1;
  }
  const lower = text.toLocaleLowerCase();
  for (const marker of ["usd", "usdt", "доллар", "美元", "डॉलर"]) {
    const position = lower.indexOf(marker);
    if (position < 0) continue;
    const after = parseNumberFromStart(text.slice(position + marker.length));
    if (after !== null) return after;
    const before = parseNumberBefore(text.slice(0, position));
    if (before !== null) return before;
  }
  return null;
}

function extractMarketPriceClaims(sample) {
  const claims = [];
  for (const fragment of marketPriceFragments(sample)) {
    const reference = marketPriceReferences().find((item) =>
      item.aliases.some((alias) => aliasOccurs(fragment, alias)),
    );
    if (!reference) continue;
    const period = extractMarketPriceYear(fragment);
    if (!period) continue;
    const claimedPrice = extractCurrencyAmount(fragment, period);
    if (claimedPrice === null) continue;
    const claim = {
      asset: reference.asset,
      assetLabel: reference.assetLabel,
      period,
      claimedPrice,
      currency: "USD",
      statement: fragment.trim(),
    };
    if (
      !claims.some(
        (existing) =>
          existing.asset === claim.asset &&
          existing.period === claim.period &&
          Math.abs(existing.claimedPrice - claim.claimedPrice) < Number.EPSILON &&
          existing.statement === claim.statement,
      )
    ) {
      claims.push(claim);
    }
  }
  return claims;
}

function assessMarketPriceClaims(claims) {
  return claims
    .map((claim) => {
      const reference = marketPriceReferences().find(
        (item) => item.asset === claim.asset && item.period === claim.period,
      );
      if (!reference) return null;
      const status =
        claim.claimedPrice < reference.observedMinPrice ||
        claim.claimedPrice > reference.observedMaxPrice
          ? MARKET_PRICE_CLAIM_STATUS_CONTRADICTED
          : MARKET_PRICE_CLAIM_STATUS_WITHIN_RANGE;
      const contradiction = status === MARKET_PRICE_CLAIM_STATUS_CONTRADICTED ? 0.95 : 0;
      const support = status === MARKET_PRICE_CLAIM_STATUS_WITHIN_RANGE ? 0.95 : 0;
      return {
        claim,
        status,
        sourceId: reference.sourceId,
        sourceLabel: reference.sourceLabel,
        sourceUrl: reference.sourceUrl,
        quoteCurrency: reference.quoteCurrency,
        observedMinPrice: reference.observedMinPrice,
        observedMinDate: reference.observedMinDate,
        observedMaxPrice: reference.observedMaxPrice,
        observedMaxDate: reference.observedMaxDate,
        posterior: verificationAssessmentTraceWithMasses(support, contradiction)
          .match(/posterior=([0-9.]+)/u)?.[1] || rmlDecimal(0),
      };
    })
    .filter(Boolean);
}

function marketPriceAssessmentTrace(assessment) {
  return (
    `asset=${assessment.claim.asset} period=${assessment.claim.period} ` +
    `claimed=${marketPriceDecimal(assessment.claim.claimedPrice)} ` +
    `status=${assessment.status} source=${assessment.sourceId} ` +
    `min=${marketPriceDecimal(assessment.observedMinPrice)} ` +
    `min_date=${assessment.observedMinDate} ` +
    `max=${marketPriceDecimal(assessment.observedMaxPrice)} ` +
    `max_date=${assessment.observedMaxDate} posterior=${assessment.posterior}`
  );
}

function marketPriceSummarySentence(assessment) {
  if (assessment.status === MARKET_PRICE_CLAIM_STATUS_CONTRADICTED) {
    return (
      `${assessment.claim.statement} is contradicted: ${assessment.sourceLabel} reports ` +
      `${assessment.claim.asset} ${assessment.quoteCurrency} daily candles in ` +
      `${assessment.claim.period} stayed between ` +
      `$${marketPriceDecimal(assessment.observedMinPrice)} on ${assessment.observedMinDate} ` +
      `and $${marketPriceDecimal(assessment.observedMaxPrice)} on ${assessment.observedMaxDate}.`
    );
  }
  return (
    `${assessment.claim.statement} is within the recorded ` +
    `${assessment.claim.asset} ${assessment.quoteCurrency} daily candle range for ` +
    `${assessment.claim.period} ($${marketPriceDecimal(assessment.observedMinPrice)} ` +
    `to $${marketPriceDecimal(assessment.observedMaxPrice)}).`
  );
}

// Append the relative-meta-logic verification plan into `evidence`, mirroring
// `log_statement_verification` in `src/solver_handlers/document_originality.rs`.
function pushStatementVerificationEvidence(prompt, language, evidence) {
  evidence.push(
    `relative_meta_logic:assumed_prior:${rmlDecimal(RML_ASSUMED_TRUE_PRIOR)}`,
  );
  for (const tier of RML_TRUSTED_SOURCE_POLICY) {
    evidence.push(
      `relative_meta_logic:trusted_source_tier:${tier.slug}:weight=${rmlDecimal(tier.weight)}`,
    );
  }
  evidence.push("relative_meta_logic:ignored_source_tier:unoriginal");

  const sample = documentOriginalityFullTextSample(prompt);
  if (!sample) return [];
  const statements = extractVerificationStatements(sample);
  evidence.push(`statement_verification:statement_count:${statements.length}`);
  for (const statement of statements) {
    const query = verificationGroundingQuery(statement);
    evidence.push(`statement_verification:statement:${statement}`);
    evidence.push(`statement_verification:query:${query}`);
    evidence.push(`web_search:request:${query}`);
    evidence.push("web_search:query_kind:document_originality_check");
    evidence.push(`statement_verification:assessment:${verificationAssessmentTrace()}`);
  }
  const claims = extractMarketPriceClaims(sample);
  const marketAssessments = assessMarketPriceClaims(claims);
  if (claims.length > 0) {
    evidence.push(`market_price_claim:claim_count:${claims.length}`);
  }
  for (const claim of claims) {
    evidence.push(`market_price_claim:claim:${claim.statement}`);
    evidence.push(`market_price_claim:asset:${claim.asset} (${claim.assetLabel})`);
    evidence.push(`market_price_claim:period:${claim.period}`);
    evidence.push(
      `market_price_claim:claimed_price:${claim.currency} ${marketPriceDecimal(claim.claimedPrice)}`,
    );
  }
  for (const assessment of marketAssessments) {
    evidence.push(
      `market_price_claim:source:${assessment.sourceId} ${assessment.sourceUrl}`,
    );
    evidence.push(
      `market_price_claim:range:asset=${assessment.claim.asset} ` +
        `period=${assessment.claim.period} source=${assessment.sourceId} ` +
        `min=${marketPriceDecimal(assessment.observedMinPrice)} ` +
        `min_date=${assessment.observedMinDate} ` +
        `max=${marketPriceDecimal(assessment.observedMaxPrice)} ` +
        `max_date=${assessment.observedMaxDate}`,
    );
    evidence.push(`market_price_claim:assessment:${marketPriceAssessmentTrace(assessment)}`);
  }
  return marketAssessments;
}

// --- Document-verification handler (issue #535) -----------------------------
// The originality/authenticity/fact-check handler lives here alongside the
// relative-meta-logic statement weigher it calls, so worker_19 stays under the
// 1500-line worker ceiling enforced by scripts/check-file-size.rs. Function
// declarations hoist across the concatenated worker bundle, so worker_20's
// dispatcher reaches tryDocumentOriginalityCheck regardless of module order.
function extractDocumentOriginalityAttachmentNames(prompt) {
  const names = [];
  let inSection = false;
  for (const rawLine of String(prompt || "").split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (/^attached files:$/iu.test(line)) {
      inSection = true;
      continue;
    }
    if (!inSection || !line) continue;
    if (
      line.startsWith("OCR text:") ||
      line.startsWith("Text excerpt:") ||
      line.startsWith("Text sample:") ||
      line.startsWith("Text omitted:")
    ) {
      continue;
    }
    const match = line.match(/^\d+\.\s+(.+?)\s+\([^)]*\)$/u);
    if (match && match[1]) names.push(match[1].trim());
  }
  return names.filter(Boolean);
}

function hasDocumentOriginalityTextSample(prompt) {
  return String(prompt || "")
    .split(/\r?\n/u)
    .some((rawLine) => {
      const line = rawLine.trim();
      return (
        line.startsWith("OCR text:") ||
        line.startsWith("Text excerpt:") ||
        line.startsWith("Text sample:")
      );
    });
}

function documentOriginalityTextSample(prompt) {
  return documentOriginalityFullTextSample(prompt)
    .split(/\s+/u)
    .filter(Boolean)
    .slice(0, 14)
    .join(" ");
}

function documentOriginalityQuery(prompt, attachments) {
  const sample = documentOriginalityTextSample(prompt);
  if (sample) return `"${sample}" plagiarism originality`;
  if (attachments.length > 0) {
    return `${attachments[0]} plagiarism originality uniqueness`;
  }
  return "document plagiarism originality uniqueness";
}

function documentOriginalityContent(language, attachments, samplePresent, marketAssessments) {
  const target = attachments.length > 0 ? attachments.join(", ") : "provided text";
  const templateIntent = samplePresent
    ? "document_originality_check_sample_present"
    : "document_originality_check_sample_missing";
  let body = answerFor(templateIntent, language).split("{target}").join(target);
  const contradicted = marketAssessments.filter(
    (assessment) => assessment.status === MARKET_PRICE_CLAIM_STATUS_CONTRADICTED,
  );
  if (contradicted.length > 0) {
    const heading =
      language === "ru"
        ? "Проверка ценовых утверждений"
        : language === "hi"
          ? "मूल्य दावों की जांच"
          : language === "zh"
            ? "价格声明核查"
            : "Price claim check";
    const summaries = contradicted
      .map((assessment) => `- ${marketPriceSummarySentence(assessment)}`)
      .join("\n");
    body = `${body}\n\n${heading}:\n${summaries}`;
  }
  return body;
}

function tryDocumentOriginalityCheck(prompt, language) {
  const normalized = normalizePrompt(prompt);
  const attachments = extractDocumentOriginalityAttachmentNames(prompt);
  const hasAction = lexiconMentionsRole(
    ROLE_DOCUMENT_ORIGINALITY_CHECK_ACTION,
    normalized,
  ) || lexiconMentionsRoleSubstring(
    ROLE_DOCUMENT_ORIGINALITY_CHECK_ACTION,
    normalized,
  );
  const hasSubject =
    lexiconMentionsRole(ROLE_DOCUMENT_ORIGINALITY_SUBJECT, normalized) ||
    lexiconMentionsRoleSubstring(
      ROLE_DOCUMENT_ORIGINALITY_SUBJECT,
      normalized,
    );
  const hasDocument =
    attachments.length > 0 ||
    lexiconMentionsRole(ROLE_DOCUMENT_ORIGINALITY_DOCUMENT, normalized) ||
    lexiconMentionsRoleSubstring(ROLE_DOCUMENT_ORIGINALITY_DOCUMENT, normalized);

  if (!(hasAction && hasSubject && hasDocument)) return null;

  const query = documentOriginalityQuery(prompt, attachments);
  const samplePresent = hasDocumentOriginalityTextSample(prompt);
  const evidence = [
    `language:${language || ""}`,
    `document_originality_check:request:${query}`,
  ];
  for (const attachment of attachments) {
    evidence.push(`document_originality_check:attachment:${attachment}`);
    evidence.push(`read_local_file:request:${attachment}`);
  }
  if (samplePresent) evidence.push("document_originality_check:text_sample:present");
  evidence.push(`web_search:request:${query}`);
  if (language) evidence.push(`web_search:language:${language}`);
  for (const provider of WEB_SEARCH_PROVIDERS) {
    evidence.push(`web_search:provider:${provider.id}`);
  }
  evidence.push(`web_search:combined:rrf:k=${webSearchRrfK()}`);
  evidence.push("web_search:query_kind:document_originality_check");

  const marketAssessments = pushStatementVerificationEvidence(prompt, language, evidence);

  return {
    intent: "document_originality_check",
    content: documentOriginalityContent(
      language,
      attachments,
      samplePresent,
      marketAssessments,
    ),
    confidence: 0.84,
    evidence,
    query,
    attachments,
    formalizedObject: attachments.length > 0 ? attachments.join(", ") : "provided text",
  };
}
