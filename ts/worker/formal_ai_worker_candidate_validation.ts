// Validate prime-interval candidates from seeded property meanings and actual divisibility.
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
  const events = [];
  for (let candidate = low; candidate <= high; candidate += 1) {
    events.push({ kind: "candidate", payload: String(candidate) });
    if (candidate < 2) continue;
    let prime = true;
    for (let divisor = 2; divisor <= Math.floor(Math.sqrt(candidate)); divisor += 1) {
      if (candidate % divisor === 0) { prime = false; break; }
    }
    if (prime) {
      events.push({ kind: "validation", payload: "prime_between_" + low + "_and_" + high });
      return { answer: String(candidate), events };
    }
  }
  events.push({ kind: "validation", payload: "no_prime_in_range" });
  return { answer: null, events };
}
