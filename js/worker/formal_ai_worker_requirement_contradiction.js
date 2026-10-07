// Requirement contradiction warnings (issue #661, R384 of issue #538).
//
// Browser twin of rust/src/requirement_contradiction.rs over the slice of
// rust/src/statement_audit/ that a conversation reaches. Each user turn and the
// current prompt is a prose document (`conversation/<index>.md`,
// `conversation/current.md`); each of its lines is a statement; a line that
// opens with a required or forbidden surface of
// data/seed/statement-audit-registry.lino ("always ...", "never ...", "всегда
// ...", "永远不要 ...") becomes an exclusive `requirement_state` claim about the
// rest of the line. Two claims about one subject with different values are a
// contradiction; when the current prompt's first statement is in one, the
// answer names both statements, their softmax weights and the append-only
// retraction protocol, from the `requirement_contradiction` template, before
// any contextual handler (a response-language replay) can act on the second
// directive. The weights are the native ones: every conversational statement
// carries the no-evidence assumed-true posterior, ranked by the solver
// temperature.
//
// Not ported, because no conversational statement reaches them: path claims
// (their only value is "true", so they never conflict), evidence captures,
// and the pronoun-antecedent context ceiling (it lowers `relative_weight`
// before the exclusive ranking overwrites it).

/**
 * The required and forbidden surfaces of the statement-audit registry, in file
 * order. Mirrors `registry()` in rust/src/statement_audit/extract.rs.
 * @returns {{required: Array<string>, forbidden: Array<string>}}
 */
function contradictionRegistry() {
  const registry = { required: [], forbidden: [] };
  const text = typeof SEED_RAW === "object" ? seedRawText(SEED_RAW, "statement-audit-registry.lino") : "";
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    const space = line.indexOf(" ");
    if (space < 0) continue;
    const key = line.slice(0, space);
    const value = line.slice(space + 1).trim().replace(/^"+|"+$/g, "");
    if (key === "requirement_required") registry.required.push(value);
    if (key === "requirement_forbidden") registry.forbidden.push(value);
  }
  return registry;
}

/**
 * Trim a statement and its trailing `.,;:`. Mirrors `trim_statement`.
 * @param {string} text
 * @returns {string}
 */
function contradictionTrimStatement(text) {
  return text.trim().replace(/[.,;:]+$/u, "").trim();
}

/**
 * The subject after `surface` when `text` opens with it as a whole word.
 * Mirrors `strip_surface`.
 * @param {string} text
 * @param {string} surface
 * @returns {string|null}
 */
function contradictionStripSurface(text, surface) {
  if (!text.startsWith(surface)) return null;
  const remainder = text.slice(surface.length);
  const alphanumeric = /^[\p{Alphabetic}\p{N}]$/u;
  const last = Array.from(surface).pop() || "";
  const first = Array.from(remainder)[0] || "";
  if (/^[\x00-\x7f]*$/.test(surface) && remainder.length > 0 && !/^\s/u.test(remainder) &&
      alphanumeric.test(last) && alphanumeric.test(first)) {
    return null;
  }
  const subject = contradictionTrimStatement(remainder);
  return subject ? subject : null;
}

/**
 * The exclusive claim a directive makes, or null. Forbidden surfaces are tried
 * first, so "must not" is never read as "must". Mirrors `requirement_claim`.
 * @param {string} text
 * @param {{required: Array<string>, forbidden: Array<string>}} registry
 * @returns {{subject: string, value: string}|null}
 */
function contradictionClaim(text, registry) {
  const normalized = contradictionTrimStatement(text).toLowerCase();
  for (const entry of [[registry.forbidden, "forbidden"], [registry.required, "required"]]) {
    for (const surface of entry[0]) {
      const subject = contradictionStripSurface(normalized, surface);
      if (subject !== null) return { subject: subject, value: entry[1] };
    }
  }
  return null;
}

/**
 * Whether a sentence of `text` that names a response language forbids it:
 * "never answer in Russian" names Russian only to rule it out, while "Never use
 * slang. Answer in Russian." still asks for Russian. Mirrors
 * `forbids_response_language` in rust/src/translation/language_markers.rs.
 * @param {string} text
 * @returns {boolean}
 */
function responseLanguageForbidden(text) {
  const registry = contradictionRegistry();
  return String(text || "").split(/[.!?\n。！？।]/u).some((sentence) => {
    const claim = contradictionClaim(sentence, registry);
    return claim !== null && claim.value === "forbidden" && Boolean(detectResponseLanguage(sentence.toLowerCase()));
  });
}

/**
 * The prose statements of one document: every non-empty line outside a code
 * fence that is not a heading or table row, list markers stripped. Mirrors
 * `extract_prose` plus `push_statement`.
 * @param {string} path
 * @param {string} content
 * @param {{required: Array<string>, forbidden: Array<string>}} registry
 * @returns {Array<{id: string, path: string, line: number, text: string, claim: {subject: string, value: string}|null}>}
 */
function contradictionStatements(path, content, registry) {
  const statements = [];
  let inFence = false;
  const lines = String(content).split("\n");
  for (let index = 0; index < lines.length; index += 1) {
    const trimmed = lines[index].replace(/\r$/, "").trim();
    if (trimmed.startsWith("~~~") || trimmed.startsWith("```")) {
      inFence = !inFence;
      continue;
    }
    if (inFence || trimmed.startsWith("#") || trimmed.startsWith("|")) continue;
    const marker = ["- ", "* ", "+ "].find((prefix) => trimmed.startsWith(prefix));
    const text = (marker ? trimmed.slice(marker.length) : trimmed).trim();
    if (!text) continue;
    statements.push({
      id: stableBehaviorRuleId("audited_statement", `${path}:${index + 1}:${text}`),
      path: path,
      line: index + 1,
      text: text,
      claim: contradictionClaim(text, registry),
    });
  }
  return statements;
}

/**
 * Softmax over f32 scores at `temperature`, clamped to [0, 1]; at zero the
 * last highest score takes all the mass. Mirrors `softmax_scores` in
 * rust/src/probability.rs.
 * @param {Array<number>} scores
 * @param {number} temperature
 * @returns {Array<number>}
 */
function contradictionSoftmax(scores, temperature) {
  const clamped = Number.isFinite(temperature) ? Math.min(Math.max(temperature, 0), 1) : 0;
  if (clamped <= 1.1920929e-7) {
    let best = 0;
    scores.forEach((score, index) => {
      if (score >= scores[best]) best = index;
    });
    return scores.map((_, index) => (index === best ? 1 : 0));
  }
  const max = Math.max(...scores);
  const weights = scores.map((score) => Math.fround(Math.exp(Math.fround(Math.fround(score - max) / Math.fround(clamped)))));
  let total = 0;
  for (const weight of weights) total = Math.fround(total + weight);
  if (!Number.isFinite(total) || total <= 1.1920929e-7) return scores.map(() => Math.fround(1 / scores.length));
  return weights.map((weight) => Math.fround(weight / total));
}

/**
 * Warn when the prompt contradicts a requirement an earlier user turn stated,
 * or null. Mirrors `detect_and_report`.
 * @param {string} prompt
 * @param {string} language
 * @param {Array<{role: string, content: string}>} history
 * @param {number} temperature
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>}|null}
 */
function tryRequirementContradiction(prompt, language, history, temperature) {
  const registry = contradictionRegistry();
  if (registry.required.length === 0 && registry.forbidden.length === 0) return null;
  const statements = [];
  (history || []).forEach((turn, index) => {
    if (turn && turn.role === "user") {
      for (const statement of contradictionStatements(`conversation/${index}.md`, turn.content || "", registry)) {
        statements.push(statement);
      }
    }
  });
  const currentPath = "conversation/current.md";
  for (const statement of contradictionStatements(currentPath, prompt, registry)) statements.push(statement);
  statements.sort((left, right) => (left.path < right.path ? -1 : left.path > right.path ? 1 : left.line - right.line));
  const current = statements.find((statement) => statement.path === currentPath);
  if (!current || !current.claim) return null;

  const group = statements.filter((statement) => statement.claim && statement.claim.subject === current.claim.subject);
  if (new Set(group.map((statement) => statement.claim.value)).size < 2) return null;
  const prior = group.find((statement) => statement.id !== current.id);
  // Every conversational statement carries the no-evidence posterior
  // (`assess_assumed_true` over no captures), narrowed to f32.
  const posterior = Math.fround(1 - (1 - RML_ASSUMED_TRUE_PRIOR) * (1 - 0));
  const weights = contradictionSoftmax(group.map(() => posterior), Math.max(temperature, 1.1920929e-7));
  const weightOf = (statement) => weights[group.indexOf(statement)].toFixed(6);

  const template = answerFor("requirement_contradiction", language === "unknown" ? "en" : language);
  if (!template) return null;
  const body = [
    ["{prior}", prior.text],
    ["{current}", current.text],
    ["{subject}", current.claim.subject],
    ["{prior_weight}", weightOf(prior)],
    ["{current_weight}", weightOf(current)],
  ].reduce((rendered, pair) => rendered.split(pair[0]).join(pair[1]), String(template));
  return {
    intent: "requirement_contradiction",
    content: body,
    confidence: 0.3,
    evidence: [
      `requirement_contradiction:subject=${current.claim.subject} statement_a=${prior.id} weight_a=${weightOf(prior)} statement_b=${current.id} weight_b=${weightOf(current)}`,
      "policy:add_only_history",
      "response:requirement_contradiction",
    ],
  };
}
