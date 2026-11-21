// Dated publisher quotes, twin of the native calculator and finance handlers.
function currencyQuoteDecimal(text, fractionLimit) {
  const match = /^([+-]?)([0-9]{1,12})(?:[.,]([0-9]*))?$/.exec(String(text));
  const fraction = match?.[3] ?? "";
  if (!match || fraction.length > fractionLimit) return null;
  return { mantissa: BigInt((match[1] === "-" ? "-" : "") + match[2] + fraction),
    divisor: 10n ** BigInt(fraction.length) };
}

function statedCurrencyQuoteDate(text) {
  return String(text).match(/(?:^|[^\d-])(\d{4}-\d{2}-\d{2})(?![\d-])/u)?.[1];
}

function currencyQuoteDateMilliseconds(date) {
  if (typeof date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(date) || date.startsWith("0000")) return null;
  const result = Date.parse(`${date}T00:00:00.000Z`);
  return Number.isFinite(result) && new Date(result).toISOString().startsWith(date) ? result : null;
}

function currencyQuoteRequest(expression) {
  expression = expression.trim();
  const cue = wordsForRole("quantity_conversion_cue").find(word => calendarOffsetStartsWithTerm(expression.toLowerCase(), word));
  if (cue) expression = expression.slice(cue.length).trimStart();
  const date = /(?:\s+on)?\s+(\d{4}-\d{2}-\d{2})$/u.exec(expression);
  if (date) expression = expression.slice(0, date.index);
  const leading = /^([$€¥₹₽])\s*([+-]?[0-9.,]+)\s+/u.exec(expression);
  if (leading) expression = `${leading[2]} ${leading[1]} ${expression.slice(leading[0].length)}`;
  const tokens = expression.trim().split(/\s+/u);
  if (tokens.length < 4 || !calendarDurationWords("currency-conversion-connector")
    .includes(tokens.slice(2, -1).join(" ").toLowerCase())) return null;
  const from = currencyCodeFromWord(tokens[1]);
  const to = currencyCodeFromWord(tokens.at(-1));
  const amount = tokens[0].replace(",", ".");
  return from && to && currencyQuoteDecimal(amount, 6) ? { amount, from, to, date: date?.[1] } : null;
}

async function readCurrencyQuote(from, to, date, preferences = {}) {
  const today = new Date().toISOString().slice(0, 10);
  const requested = date ?? today;
  const requestedTime = currencyQuoteDateMilliseconds(requested);
  if (requestedTime === null || requested > today) return null;
  if (from === to) return { rate: "1", date: "", source: "" };
  const provider = code => handlerRulesTableValue("currency-quote-provider", code);
  const endpoint = provider(from) || provider(to) || provider("default");
  const url = `${endpoint}/${from.toLowerCase()}/${to.toLowerCase()}?date=${requested}`;
  const capture = await sourceWalkFetchCapture(url, { online: preferences.offline !== true, bounded: true, maxAgeSeconds: 3600 });
  if (!capture.ok) return null;
  try {
    const quote = JSON.parse(capture.text);
    const effectiveTime = currencyQuoteDateMilliseconds(quote.date);
    if (effectiveTime === null || effectiveTime > requestedTime
      || requestedTime - effectiveTime > 7 * 86400000
      || quote.base !== from || quote.quote !== to || typeof quote.rate !== "number") return null;
    let rate = String(quote.rate);
    if (rate.includes("e")) rate = quote.rate.toFixed(12).replace(/0+$/, "");
    if (!(currencyQuoteDecimal(rate, 12)?.mantissa > 0n)) return null;
    return { rate, date: quote.date, source: url, sha256: capture.sha256 };
  } catch {
    return null;
  }
}

function convertCurrencyQuote(amount, rate) {
  const source = currencyQuoteDecimal(amount, 6);
  const quote = currencyQuoteDecimal(rate, 12);
  if (!source || !quote) return null;
  const numerator = source.mantissa * quote.mantissa * 100n;
  const denominator = source.divisor * quote.divisor;
  const absolute = numerator < 0n ? -numerator : numerator;
  const cents = absolute / denominator + (absolute % denominator >= (denominator + 1n) / 2n ? 1n : 0n);
  const digits = cents.toString().padStart(3, "0");
  const sign = numerator < 0n && cents !== 0n ? "-" : "";
  return `${sign}${digits.slice(0, -2)}.${digits.slice(-2)}`;
}

function currencyQuoteResponse(intent, language, values = {}) {
  let content = calendarOffsetLocalizedResponse(intent, language);
  for (const [key, value] of Object.entries(values)) content = content.replaceAll(`{${key}}`, () => value);
  return content;
}

function currencyQuoteEvidence(quote) {
  return quote.source ? [
    { kind: "currency:source", payload: quote.source },
    { kind: "currency:date", payload: quote.date },
    { kind: "source:sha256", payload: quote.sha256 },
  ] : [];
}

async function answerCurrencyQuote(request, language, preferences) {
  const { amount, from, to, date } = request;
  const quote = await readCurrencyQuote(from, to, date, preferences);
  const result = quote && convertCurrencyQuote(amount, quote.rate);
  if (result === null) return {
    intent: "calculation_error",
    content: currencyQuoteResponse("currency-quote-unavailable", language, { from, to }),
    confidence: 0.3,
    evidence: [`currency:source_error:${from}/${to}`],
  };
  const derivation = `${amount} × ${quote.rate} ≈ ${result} ${to}`;
  const template = from === to ? "currency-quote-identity" : "currency-conversion-publisher";
  const content = currencyQuoteResponse(template, language, { amount, from, to, ...quote, result, derivation });
  const solverEvents = [
    ...currencyQuoteEvidence(quote),
    { kind: "currency:rate_basis", payload: `${from}:${quote.rate}${to}` },
    { kind: "currency:derivation", payload: derivation },
    { kind: "currency:result", payload: `${result} ${to}` },
    { kind: "calculation:engine", payload: "evidenced-currency" },
    { kind: "calculation", payload: content },
  ];
  return { intent: "calculation", content, confidence: 1,
    evidence: solverEvents.map(({ kind, payload }) => `${kind}:${payload}`), solverEvents };
}

function currencyMentions(text) {
  return [...text.matchAll(/[\p{L}\p{M}]+|[$€¥₹₽]/gu)].flatMap(match => {
    const code = currencyCodeFromWord(match[0]);
    return code ? [{ code, token: match[0], at: match.index }] : [];
  });
}

function asksForRateBasis(normalized) {
  return lexiconMentionsRoleSubstring(ROLE_EXCHANGE_RATE_REFERENCE, normalized)
    && lexiconMentionsRoleSubstring(ROLE_CALCULATION_BASIS_REFERENCE, normalized);
}

function tryCalculatorRateBasis(prompt, normalized, language, preferences) {
  if (!asksForRateBasis(normalized)) return null;
  const currencies = currencyMentions(prompt).map(({ code }) => code);
  const [from, to] = new Set(currencies);
  if (!to) return { intent: "clarification", content: currencyQuoteResponse("currency-quote-pair-needed", language),
    confidence: 0.8, evidence: ["clarification:currency_pair"] };
  return answerCurrencyQuote({ amount: "1", from, to, date: statedCurrencyQuoteDate(prompt) }, language, preferences);
}
