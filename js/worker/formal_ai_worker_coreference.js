// Seeded conversation references, bounded normalized surfaces and typed resolution events.
// Mirrors `try_coreference_request` in
// `src/solver_handlers/benchmark_prompts.rs` for fact-style follow-ups.
function replaceBoundedToken(text, token, replacement) {
  if (!text || !token || !replacement) return null;
  const pattern = new RegExp(
    `(^|[^\\p{L}\\p{N}_])${escapeRegExp(token)}(?=$|[^\\p{L}\\p{N}_])`,
    "gu",
  );
  let changed = false;
  const rewritten = String(text).replace(pattern, (match, prefix) => {
    changed = true;
    return `${prefix}${replacement}`;
  });
  return changed ? rewritten : null;
}

function matchingCoreferencePronoun(normalized) {
  const pronouns = Array.isArray(COREFERENCE_SEEDS && COREFERENCE_SEEDS.pronouns)
    ? COREFERENCE_SEEDS.pronouns
    : [];
  return pronouns.find((pronoun) => {
    const contexts = Array.isArray(pronoun && pronoun.contexts)
      ? pronoun.contexts
      : [];
    const startsWith = Array.isArray(pronoun && pronoun.startsWith)
      ? pronoun.startsWith
      : [];
    return contexts.some((context) => surfacePresent(normalized, normalizePrompt(context))) ||
      startsWith.some((prefix) => {
        const surface = normalizePrompt(prefix);
        return normalized.startsWith(surface) && surfacePresent(normalized, surface);
      });
  }) || null;
}

function matchingCoreferenceAntecedent(previous) {
  const antecedents = Array.isArray(COREFERENCE_SEEDS && COREFERENCE_SEEDS.antecedents)
    ? COREFERENCE_SEEDS.antecedents
    : [];
  return antecedents.find((antecedent) => {
    const aliases = Array.isArray(antecedent && antecedent.aliases)
      ? antecedent.aliases
      : [];
    return aliases.some((alias) => alias && previous.includes(alias));
  }) || null;
}

function matchingAntecedentFactAlias(record, antecedent, previous) {
  const factAliases = (record && record.subjectAliases) || [];
  const antecedentAliases = (antecedent && antecedent.aliases) || [];
  // Issue #1172: word-boundary match (see tryFactLookup) so the alias "us"
  // counts only when the previous turn literally mentions it.
  return factAliases.find((alias) =>
    alias &&
    surfacePresent(previous, normalizePrompt(alias)) &&
    antecedentAliases.includes(String(alias).toLowerCase()),
  ) || "";
}

// Rust `resolve_coreference_antecedent`: the nearest earlier user turn naming an antecedent of the seed
// (assistant turns name none, and a later unrelated user turn leaves an earlier antecedent visible).
function nearestCoreferenceAntecedent(history) {
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index];
    if (!turn || turn.role !== "user") continue;
    const antecedent = matchingCoreferenceAntecedent(String(turn.content || "").toLowerCase());
    if (antecedent) return { antecedent, previous: normalizePrompt(String(turn.content || "")) };
  }
  return null;
}

function tryCoreferenceFactLookup(prompt, normalized, history) {
  const pronoun = matchingCoreferencePronoun(normalized);
  if (!pronoun || !pronoun.token) return null;

  const resolved = nearestCoreferenceAntecedent(history);
  if (!resolved) return null;
  const { antecedent, previous } = resolved;

  for (const record of FACTS) {
    // Issue #1172: word-boundary keyword prefilter (see tryFactLookup).
    if (!record || !(record.questionKeywords || []).some((keyword) => surfacePresent(normalized, normalizePrompt(keyword)))) continue;

    const alias = matchingAntecedentFactAlias(record, antecedent, previous);
    if (!alias) continue;

    const rewritten = replaceBoundedToken(normalized, pronoun.token, alias);
    if (!rewritten) continue;

    const hit = tryFactLookup(prompt, rewritten);
    if (!hit) continue;

    const subject = antecedent.displayName || record.subjectLabel || alias;
    return Object.assign({}, hit, {
      solverEvents: [
        { kind: "coreference:resolved", payload: `${pronoun.token}=${subject}` },
        ...(antecedent.wikidata ? [{ kind: "wikidata", payload: antecedent.wikidata }] : []),
        { kind: "coreference:rewrite", payload: rewritten },
        ...(Array.isArray(hit.solverEvents) ? hit.solverEvents : []),
      ],
      evidence: [
        `coreference:resolved:${pronoun.token}=${subject}`,
        `coreference:rewrite:${rewritten}`,
        ...(Array.isArray(hit.evidence) ? hit.evidence : []),
      ],
    });
  }

  // Rust `try_coreference_request`: no fact answers the rewritten question, so the antecedent's seeded body does.
  if (!antecedent.body) return null;
  const evidence = [`coreference:resolved:${pronoun.token}=${antecedent.displayName}`];
  if (antecedent.wikidata) evidence.push(`wikidata:${antecedent.wikidata}`);
  const solverEvents = [{ kind: "coreference:resolved", payload: `${pronoun.token}=${antecedent.displayName}` }];
  if (antecedent.wikidata) solverEvents.push({ kind: "wikidata", payload: antecedent.wikidata });
  return { intent: antecedent.intent, content: antecedent.body, confidence: 0.85, solverEvents, evidence: [...evidence, "response:coreference"] };
}
