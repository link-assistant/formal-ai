// Validate number-property interval candidates from seeded property meanings and actual divisibility.
// Safe browser arithmetic uses the existing primality ceiling; unsupported bounds stay unknown.
function solverPrimeIntervalValidation(prompt) {
  const source = String(prompt || "").toLowerCase();
  if (!quantityMeaningWords(NUMBER_PROPERTY_PRIME_SLUG).some((word) => quantityContainsTerm(source, word))) return null;
  const normalized = normalizePrompt(prompt);
  if (["code_request", "function", "implement"].some((slug) => operationMatchesSlug(slug, normalized))) return null;
  const tokens = source.match(/[0-9]+/gu) || [];
  if (tokens.length !== 2) return null;
  const [low, high] = tokens.map(Number);
  if (![low, high].every(Number.isSafeInteger) || low > high || high > NUMBER_PROPERTY_MAX_TESTED) return null;
  // Issue #1550: every number is a candidate and every executed test is
  // recorded with its verdict (`number_property_prime(15)=false`), the Rust
  // `record_validation`; the first passing number is chosen.
  const events = [];
  for (let candidate = low; candidate <= high; candidate += 1) {
    let passed = candidate >= 2;
    for (let divisor = 2; passed && divisor <= Math.floor(Math.sqrt(candidate)); divisor += 1) {
      if (candidate % divisor === 0) passed = false;
    }
    events.push({ kind: "candidate", payload: String(candidate) });
    events.push({ kind: "validation", payload: `${NUMBER_PROPERTY_PRIME_SLUG}(${candidate})=${passed}` });
    if (passed) return { answer: String(candidate), events };
  }
  return { answer: null, events };
}
