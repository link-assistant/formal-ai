// Translation renderers split from worker 05 so each worker fragment stays
// within the repository's reviewable line budget.

function renderTranslationGap(surface, source, target) {
  const trimmed = String(surface || "").trim();
  if (!trimmed) {
    return `I could not identify a source phrase to translate from ${source} to ${target}.`;
  }
  return `I could not translate "${trimmed}" from ${source} to ${target} with the available formalization data. I recorded this as a translation gap for follow-up.`;
}

// Issue #890: formalize an interval proof once, independently from both the
// natural language of the request and the programming language used to present
// it. This mirrors `proof_program::FormalProof` in the native engine.
function formalizeIntegerIntervalProof(statement) {
  const tokens = String(statement || "").trim().split(/\s+/u);
  if (tokens.length < 9 || tokens[3] !== "and" || tokens[0] !== tokens[4]) return null;
  if (!/^[A-Za-z]+$/u.test(tokens[0])) return null;
  if (![">", ">="].includes(tokens[1]) || !["<", "<="].includes(tokens[5])) {
    return null;
  }
  let lower;
  let upper;
  try {
    lower = BigInt(tokens[2]);
    upper = BigInt(tokens[6]);
  } catch (_error) {
    return null;
  }
  const i64Min = -9223372036854775808n;
  const i64Max = 9223372036854775807n;
  if (lower < i64Min || lower > i64Max || upper < i64Min || upper > i64Max) return null;
  const suffix = tokens.slice(7).join(" ");
  if (
    suffix !== "is satisfiable" &&
    suffix !== "is satisfiable over integers" &&
    suffix !== "is unsatisfiable over integers"
  ) {
    return null;
  }
  const first = tokens[1] === ">=" ? lower : lower + 1n;
  const last = tokens[5] === "<=" ? upper : upper - 1n;
  const witness = first <= last ? first : null;
  const expectedSatisfiable = !suffix.startsWith("is unsatisfiable");
  if (Boolean(witness !== null) !== expectedSatisfiable) return null;
  return {
    variable: tokens[0],
    lower,
    lowerOperator: tokens[1],
    upper,
    upperOperator: tokens[5],
    first,
    last,
    witness,
  };
}

function formalProofSlug(proof) {
  return [
    "proof",
    "integer_interval",
    proof.variable,
    proof.lowerOperator,
    String(proof.lower),
    proof.upperOperator,
    String(proof.upper),
    proof.witness === null ? "unsatisfiable" : "satisfiable",
  ].join(":");
}

let cachedProofProgramTemplates;
function proofProgramTemplates() {
  if (cachedProofProgramTemplates === undefined) {
    const tree = parseLinoTree(PROOF_PROGRAM_TEMPLATES_LINO);
    cachedProofProgramTemplates =
      tree.children.find((child) => child.name === "proof_program_templates") || null;
  }
  return cachedProofProgramTemplates;
}

function expandProofProgramTemplate(template, proof) {
  const bindings = {
    variable: proof.variable,
    lower_operator: proof.lowerOperator,
    lower_value: String(proof.lower),
    upper_operator: proof.upperOperator,
    upper_value: String(proof.upper),
    witness: proof.witness === null ? "" : String(proof.witness),
    first: String(proof.first),
    last: String(proof.last),
  };
  let rendered = template;
  for (const [name, value] of Object.entries(bindings)) {
    rendered = rendered.split(`{${name}}`).join(value);
  }
  return rendered;
}

function renderFormalProofProgram(proof, target) {
  const catalog = proofProgramTemplates();
  if (!catalog) return null;
  const language = catalog.children.find(
    (child) => child.name === "language" && child.value === target,
  );
  if (!language) return null;
  const state = proof.witness === null ? "unsatisfiable" : "satisfiable";
  const template = childValue(language, state);
  return template ? expandProofProgramTemplate(template, proof) : null;
}

function extractBacktickedFormalProof(prompt) {
  const match = /`([^`\r\n]+)`/u.exec(String(prompt || ""));
  if (!match) return { source: null, proof: null };
  return { source: match[1], proof: formalizeIntegerIntervalProof(match[1]) };
}

function formalProofProgramTranslation(prompt, normalized) {
  const formalProof = extractBacktickedFormalProof(prompt);
  if (!formalProof.proof) return null;
  const target = programLanguageFromPrompt(normalized);
  if (!target) return null;
  const rendered = renderFormalProofProgram(formalProof.proof, target);
  const program = rendered ||
    `${target === "python" || target === "ruby" ? "#" : "//"} translation gap for \`formal proof\` from proof to ${target}`;
  const meaningId = stableBehaviorRuleId("meaning", formalProofSlug(formalProof.proof));
  return {
    target,
    answer: {
      intent: `translate_proof_to_${target}`,
      content: `Translated \`${formalProof.source}\` from proof to ${target}:\n\n\`\`\`${target}\n${program}\n\`\`\``,
      confidence: 1.0,
      evidence: [
        "handler:translation",
        "language_from:proof",
        `language_to:${target}`,
        `meaning:${meaningId}`,
      ],
    },
  };
}
