// Issue #1186 user-facing formalization task, browser twin (issue #1188 JS
// parity). Mirrors rust/src/solver_handlers/formalization_task.rs and
// formalization_task_render.rs: cue phrases, the target grammars (FOL, Lean 4,
// Rocq, Links Notation) and the natural-language recognition tables all come
// from data/seed/formal-targets.lino; the prose comes from
// multilingual-responses-formalization.lino. The parse is structural and
// predicate symbols derive from the sentence's own words.
//
// Seed shape notes (where this twin follows the seed and the issue #1186
// tests rather than the native reader): formal-targets.lino wraps every record
// under one `formal_targets` root, so the records are read from that root's
// children; the clause templates are the seed's `clause_conditional` /
// `clause_conjunctive` fields; quantifier and marker surfaces are matched on
// the lowercased sentence; and a deformalize cue wins over the formalize verb
// it contains.
// The browser has no PATH to probe, so the prover sentences always take the
// "not found in PATH" wording: nothing was compiled here either way.

const FORMAL_TARGETS_FILE = "formal-targets.lino";
const FORMALIZATION_INTENT = "formalization";
const FORMALIZATION_DEFAULT_VARIABLE = "x";

/**
 * The records of formal-targets.lino (children of its top-level nodes).
 * @returns {Array<object>}
 */
function formalTargetRecords() {
  const text = seedRawText(SEED_RAW, FORMAL_TARGETS_FILE);
  if (!text) return [];
  const out = [];
  for (const top of parseLinoTree(text).children) {
    for (const record of top.children) out.push(record);
  }
  return out;
}

/**
 * @param {object} node
 * @param {string} name
 * @returns {object|null}
 */
function formalNamedChild(node, name) {
  const found = node.children.find((child) => child.name === name);
  return found === undefined ? null : found;
}

/**
 * Pairs of [child.value, child.text] for every `name` child.
 * @param {object} record
 * @param {string} name
 * @returns {Array<Array<string>>}
 */
function formalKeyedTexts(record, name) {
  return record.children
    .filter((child) => child.name === name)
    .map((child) => [child.value, childValue(child, "text")]);
}

/**
 * Parse the targets seed into grammar tables.
 * @returns {{formal: Array<object>, natural: Array<object>}}
 */
function formalGrammar() {
  const formal = [];
  const natural = [];
  for (const record of formalTargetRecords()) {
    if (record.name === "formal_language") {
      const quantifiers = formalKeyedTexts(record, "quantifier").sort((left, right) =>
        left[0] < right[0] ? -1 : left[0] > right[0] ? 1 : 0,
      );
      formal.push({
        slug: record.value,
        aliases: textTransformRecordValues(record, "alias"),
        atom: childValue(record, "atom"),
        atomWithObject: childValue(record, "atom_with_object"),
        joins: formalKeyedTexts(record, "join"),
        quantifiers: quantifiers,
        clauseConditional: childValue(record, "clause_conditional"),
        clauseConjunctive: childValue(record, "clause_conjunctive"),
        clauseNegative: childValue(record, "clause_negative"),
      });
    } else if (record.name === "natural_language") {
      natural.push({
        language: record.value,
        quantifiers: formalKeyedTexts(record, "quantifier").map((pair) => [pair[1], pair[0]]),
        joins: formalKeyedTexts(record, "join").map((pair) => [pair[1], pair[0]]),
        relMarkers: record.children
          .filter((child) => child.name === "rel_marker")
          .map((child) => childValue(child, "text"))
          .filter((surface) => surface.length > 0),
        objectIntroducers: textTransformRecordValues(record, "object_introducer"),
        copulaDrops: textTransformRecordValues(record, "copula_drop"),
        headFinal: formalNamedChild(record, "head_final") !== null,
        clauseConditional: childValue(record, "clause_conditional"),
        clauseConjunctive: childValue(record, "clause_conjunctive"),
      });
    }
  }
  return { formal: formal, natural: natural };
}

/**
 * @param {Array<Array<string>>} pairs
 * @param {string} key
 * @returns {string|null}
 */
function formalLookup(pairs, key) {
  const found = pairs.find((pair) => pair[0] === key);
  return found === undefined ? null : found[1];
}

/**
 * The response template for one language, else English, else "".
 * @param {string} intent
 * @param {string} language
 * @returns {string}
 */
function formalResponse(intent, language) {
  const exact = textTransformResponseFor(intent, language);
  if (exact !== null) return exact;
  const english = textTransformResponseFor(intent, "en");
  return english === null ? "" : english;
}

/**
 * Capitalize a leading ASCII lowercase letter.
 * @param {string} word
 * @returns {string}
 */
function formalCapitalize(word) {
  return /^[a-z]/.test(word) ? word[0].toUpperCase() + word.slice(1) : word;
}

/**
 * Lowercase a leading ASCII uppercase letter.
 * @param {string} word
 * @returns {string}
 */
function formalDecapitalize(word) {
  return /^[A-Z]/.test(word) ? word[0].toLowerCase() + word.slice(1) : word;
}

/**
 * @param {Array<string>} words
 * @returns {{name: string, object: string|null}}
 */
function formalPredicateFromWords(words) {
  const name = formalCapitalize(words.length > 0 ? words[0] : "");
  return { name: name, object: words.length > 1 ? words.slice(1).join(" ") : null };
}

/**
 * The comparison key a round trip must preserve.
 * @param {object} clause
 * @returns {string}
 */
function formalStructureKey(clause) {
  const predicates = clause.antecedent.concat([clause.consequent]);
  return JSON.stringify([
    clause.quantifier,
    predicates.map((predicate) => [predicate.name, predicate.object]),
  ]);
}

/**
 * Tokenize the lowercased text on whitespace and punctuation; commas stay as
 * tokens. Word characters are letters, numbers and combining marks, so a
 * Devanagari virama or nukta stays inside its word.
 * @param {string} text
 * @returns {Array<string>}
 */
function formalTokenize(text) {
  const tokens = [];
  let current = "";
  for (const character of text.toLowerCase()) {
    if (/^[\p{Alphabetic}\p{N}\p{M}]$/u.test(character) || character === "-" || character === "'" ||
      character === "’") {
      current += character;
    } else if (character === ",") {
      if (current.length > 0) tokens.push(current);
      current = "";
      tokens.push(",");
    } else {
      if (current.length > 0) tokens.push(current);
      current = "";
    }
  }
  if (current.length > 0) tokens.push(current);
  return tokens;
}

/**
 * Longest-match quantifier at `index`: [kind, span] or null.
 * @param {object} language
 * @param {Array<string>} tokens
 * @param {number} index
 * @returns {Array<string|number>|null}
 */
function formalQuantifierAt(language, tokens, index) {
  if (index + 1 < tokens.length) {
    const bigram = formalLookup(language.quantifiers, `${tokens[index]} ${tokens[index + 1]}`);
    if (bigram !== null) return [bigram, 2];
  }
  const single = formalLookup(language.quantifiers, tokens[index]);
  return single === null ? null : [single, 1];
}

/**
 * @param {object} language
 * @param {Array<string>} tokens
 * @returns {number}
 */
function formalFirstQuantifier(language, tokens) {
  for (let index = 0; index < tokens.length; index += 1) {
    if (formalQuantifierAt(language, tokens, index) !== null) return index;
  }
  return -1;
}

/**
 * Head-final (CJK) parse: {quantifier} {relatives} 的 {head} [都] {main}.
 * @param {string} text
 * @param {object} language
 * @param {Array<string>} joinWords
 * @returns {object|null}
 */
function formalParseHeadFinal(text, language, joinWords) {
  let spaced = text;
  const surfaces = language.quantifiers
    .map((pair) => pair[0])
    .concat(language.relMarkers, joinWords, ["都"]);
  for (const surface of surfaces) {
    spaced = spaced.split(surface).join(` ${surface} `);
  }
  const tokens = formalTokenize(spaced).filter((token) => !language.copulaDrops.includes(token));
  const quantifierIndex = formalFirstQuantifier(language, tokens);
  if (quantifierIndex === -1) return null;
  const found = formalQuantifierAt(language, tokens, quantifierIndex);
  const kind = found[0];
  const span = found[1];
  const markerOffset = tokens
    .slice(quantifierIndex + span)
    .findIndex((token) => language.relMarkers.includes(token));
  if (markerOffset === -1) return null;
  const marker = quantifierIndex + span + markerOffset;
  const relatives = tokens
    .slice(quantifierIndex + span, marker)
    .filter((token) => !joinWords.includes(token));
  const afterMarker = tokens.slice(marker + 1).filter((token) => token !== "都");
  if (afterMarker.length === 0) return null;
  const antecedent = [formalPredicateFromWords([afterMarker[0]])].concat(
    relatives.map((word) => formalPredicateFromWords([word])),
  );
  return {
    quantifier: kind,
    variable: FORMALIZATION_DEFAULT_VARIABLE,
    antecedent: antecedent,
    consequent: formalPredicateFromWords(afterMarker.slice(1)),
  };
}

/**
 * Parse a natural sentence into a quantified clause (`parse_quantified_clause`).
 * @param {string} text
 * @param {object} language
 * @returns {object|null}
 */
function formalParseQuantifiedClause(text, language) {
  const tokens = formalTokenize(text).filter((token) => !language.copulaDrops.includes(token));
  const joinWords = language.joins.map((pair) => pair[0]);
  if (language.headFinal) return formalParseHeadFinal(text, language, joinWords);

  const quantifierIndex = formalFirstQuantifier(language, tokens);
  if (quantifierIndex === -1) return null;
  const found = formalQuantifierAt(language, tokens, quantifierIndex);
  const kind = found[0];
  const headIndex = quantifierIndex + found[1];
  if (headIndex >= tokens.length) return null;
  const head = tokens[headIndex];
  const afterHead = tokens.slice(headIndex + 1);
  const markerOffset = afterHead.findIndex((token) => language.relMarkers.includes(token));
  if (markerOffset === -1) return null;
  const spanTokens = afterHead.slice(markerOffset + 1).filter((token) => token !== ",");
  let relativeWords = [];
  let mainWords = [];
  if (language.objectIntroducers.length > 0) {
    const introducer = spanTokens.findIndex((token) => language.objectIntroducers.includes(token));
    if (introducer !== -1) {
      if (introducer === 0) return null;
      const before = spanTokens.slice(0, introducer);
      mainWords = [before[before.length - 1]].concat(spanTokens.slice(introducer + 1));
      relativeWords = before.slice(0, before.length - 1);
    } else {
      if (spanTokens.length === 0) return null;
      mainWords = [spanTokens[spanTokens.length - 1]];
      relativeWords = spanTokens.slice(0, spanTokens.length - 1);
    }
  } else {
    const tail = afterHead.slice(markerOffset + 1);
    const lastComma = tail.lastIndexOf(",");
    if (lastComma !== -1) {
      relativeWords = tail.slice(0, lastComma).filter((token) => token !== ",");
      mainWords = tail.slice(lastComma + 1).filter((token) => token !== ",");
      if (mainWords.length === 0) return null;
    } else {
      if (spanTokens.length === 0) return null;
      mainWords = [spanTokens[spanTokens.length - 1]];
      relativeWords = spanTokens.slice(0, spanTokens.length - 1);
    }
  }
  const antecedent = [formalPredicateFromWords([head])].concat(
    relativeWords
      .filter((word) => !joinWords.includes(word))
      .map((word) => formalPredicateFromWords([word])),
  );
  return {
    quantifier: kind,
    variable: FORMALIZATION_DEFAULT_VARIABLE,
    antecedent: antecedent,
    consequent: formalPredicateFromWords(mainWords),
  };
}

/**
 * @param {object} language a formal grammar
 * @param {{name: string, object: string|null}} predicate
 * @param {string} variable
 * @returns {string}
 */
function formalRenderAtom(language, predicate, variable) {
  const template = predicate.object !== null ? language.atomWithObject : language.atom;
  return textTransformFill(template, [
    ["predicate", predicate.name],
    ["variable", variable],
    ["object", predicate.object === null ? "" : predicate.object],
  ]);
}

/**
 * Render the clause in one target formal language (`render_clause`).
 * @param {object} grammar
 * @param {object} clause
 * @param {string} slug
 * @returns {string|null}
 */
function formalRenderClause(grammar, clause, slug) {
  const language = grammar.formal.find((item) => item.slug === slug);
  if (language === undefined) return null;
  const andJoin = formalLookup(language.joins, "and");
  const and = andJoin === null ? "" : andJoin;
  const variable = clause.variable;
  const antecedent = clause.antecedent
    .map((predicate) => formalRenderAtom(language, predicate, variable))
    .join(and);
  const consequent = formalRenderAtom(language, clause.consequent, variable);
  const negative = clause.quantifier === "no" && language.clauseNegative.length > 0;
  const shape = negative ? language.clauseNegative : language.clauseConjunctive;
  const template = clause.quantifier === "forall" ? language.clauseConditional : shape;
  const symbol = formalLookup(language.quantifiers, clause.quantifier);
  const rendered = textTransformFill(template, [
    ["quantifier", symbol === null ? "" : symbol],
    ["variable", variable],
    ["antecedent", antecedent],
    ["consequent", consequent],
    ["joined", [antecedent, consequent].join(and)],
    ["quantifier_word", clause.quantifier],
  ]);
  return rendered.trimEnd();
}

/**
 * Split on a separator at parenthesis depth zero.
 * @param {string} text
 * @param {string} separator
 * @returns {Array<string>}
 */
function formalSplitTopLevel(text, separator) {
  let depth = 0;
  const parts = [];
  let current = "";
  for (const character of text) {
    if (character === "(") {
      depth += 1;
      current += character;
    } else if (character === ")") {
      depth = Math.max(0, depth - 1);
      current += character;
    } else if (character === separator && depth === 0) {
      parts.push(current.trim());
      current = "";
    } else {
      current += character;
    }
  }
  if (current.trim().length > 0) parts.push(current.trim());
  return parts;
}

/**
 * Parse one atom `Name(x)` / `Name(x, object)`.
 * @param {string} text
 * @param {string} variable
 * @returns {{name: string, object: string|null}|null}
 */
function formalParseAtom(text, variable) {
  const trimmed = text.trim();
  const open = trimmed.indexOf("(");
  const close = trimmed.lastIndexOf(")");
  if (open === -1 || close === -1 || close < open + 1) return null;
  const name = formalCapitalize(trimmed.slice(0, open).trim());
  const args = trimmed.slice(open + 1, close).split(",").map((argument) => argument.trim());
  const other = args.find((argument) => argument !== variable);
  return { name: name, object: other === undefined ? null : other.replace(/^"+|"+$/g, "") };
}

/**
 * Parse FOL (or Rocq ASCII) text back into a clause (`parse_fol_clause`).
 * @param {object} grammar
 * @param {string} text
 * @returns {object|null}
 */
function formalParseFolClause(grammar, text) {
  const source = text.trim();
  const fol = grammar.formal.find((item) => item.slug === "fol");
  if (fol === undefined) return null;
  let quantifier = "";
  let rest = source;
  const words = [["no", "~ exists"], ["forall", "forall"], ["exists", "exists"]];
  for (const pair of words) {
    if (source.startsWith(pair[1])) {
      quantifier = pair[0];
      rest = source.slice(pair[1].length);
      break;
    }
  }
  if (quantifier.length === 0) {
    for (const pair of fol.quantifiers) {
      if (source.startsWith(pair[1])) {
        quantifier = pair[0];
        rest = source.slice(pair[1].length);
        break;
      }
    }
  }
  if (quantifier.length === 0) return null;
  rest = rest.trim();
  const variableMatch = /^[\p{Alphabetic}\p{N}_]*/u.exec(rest);
  const variable = variableMatch === null ? "" : variableMatch[0];
  if (variable.length === 0) return null;
  const open = rest.indexOf("(");
  const close = rest.lastIndexOf(")");
  if (open === -1 || close === -1 || close < open + 1) return null;
  const body = rest.slice(open + 1, close).trim().split("->").join("→");
  const conditionalParts = formalSplitTopLevel(body, "→");
  const parseAll = (part) =>
    formalSplitTopLevel(part, "∧")
      .map((atom) => formalParseAtom(atom, variable))
      .filter((atom) => atom !== null);
  const predicates = conditionalParts.length >= 2
    ? parseAll(conditionalParts[0]).concat(parseAll(conditionalParts[1]))
    : parseAll(body);
  if (predicates.length < 2) return null;
  if (conditionalParts.length >= 2) {
    const consequent = formalParseAtom(conditionalParts[1], variable);
    if (consequent === null) return null;
    return {
      quantifier: quantifier,
      variable: variable,
      antecedent: parseAll(conditionalParts[0]),
      consequent: consequent,
    };
  }
  return {
    quantifier: quantifier,
    variable: variable,
    antecedent: predicates.slice(0, predicates.length - 1),
    consequent: predicates[predicates.length - 1],
  };
}

/**
 * Render the clause as a natural sentence (`render_clause_natural`).
 * @param {object} grammar
 * @param {object} clause
 * @param {string} language
 * @returns {string|null}
 */
function formalRenderClauseNatural(grammar, clause, language) {
  const natural = grammar.natural.find((item) => item.language === language);
  if (natural === undefined) return null;
  const quantifier = natural.quantifiers.find((pair) => pair[1] === clause.quantifier);
  if (quantifier === undefined) return null;
  const join = natural.joins.find((pair) => pair[1] === "and");
  const joinWord = join === undefined ? "" : join[0];
  const relMarker = natural.relMarkers.length > 0 ? natural.relMarkers[0] : "";
  const template = clause.quantifier === "forall" ? natural.clauseConditional : natural.clauseConjunctive;
  const introducer = natural.objectIntroducers.length > 0 ? natural.objectIntroducers[0] : null;
  const wordsOf = (predicate) => {
    const head = formalDecapitalize(predicate.name);
    if (predicate.object === null) return head;
    return introducer !== null ? `${head} ${introducer} ${predicate.object}` : `${head} ${predicate.object}`;
  };
  if (clause.antecedent.length === 0) return null;
  const relatives = clause.antecedent
    .slice(1)
    .map((predicate) => formalDecapitalize(predicate.name))
    .join(` ${joinWord} `);
  return textTransformFill(template, [
    ["quantifier", quantifier[0]],
    ["head", formalDecapitalize(clause.antecedent[0].name)],
    ["rel_marker", relMarker],
    ["relatives", relatives],
    ["main", wordsOf(clause.consequent)],
  ]);
}

/**
 * Which cue role families matched, as [family, language] pairs.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {Array<Array<string>>}
 */
function formalMatchedRoles(prompt, normalized) {
  const lower = prompt.toLowerCase();
  const out = [];
  for (const record of formalTargetRecords()) {
    if (record.name !== "cues") continue;
    const intentNode = formalNamedChild(record, "intent");
    if (intentNode === null) continue;
    for (const roleNode of intentNode.children) {
      if (roleNode.name !== "role") continue;
      const role = roleNode.value;
      const hit = textTransformRecordValues(roleNode, "phrase").some(
        (phrase) => normalized.includes(phrase) || lower.includes(phrase),
      );
      if (!hit) continue;
      const pieces = role.split("_");
      out.push([pieces[0], pieces[pieces.length - 1]]);
    }
  }
  return out;
}

/**
 * The formal target an alias in the prompt names, or null.
 * @param {object} grammar
 * @param {string} prompt
 * @param {string} normalized
 * @returns {string|null}
 */
function formalNamedTarget(grammar, prompt, normalized) {
  const lower = prompt.toLowerCase();
  const found = grammar.formal.find((language) =>
    language.aliases.some((alias) => normalized.includes(alias) || lower.includes(alias.toLowerCase())),
  );
  return found === undefined ? null : found.slug;
}

/**
 * The text after a colon, else the trimmed prompt.
 * @param {string} prompt
 * @returns {string}
 */
function formalSentenceUnderDiscussion(prompt) {
  const colon = prompt.indexOf(":");
  if (colon !== -1) {
    const trimmed = prompt.slice(colon + 1).trim();
    if (trimmed.length > 0) return trimmed.replace(/[.。।]+$/u, "");
  }
  return prompt.trim();
}

/**
 * The honesty block: no prover runs in the browser and there is no PATH.
 * @param {string} language
 * @returns {string}
 */
function formalHonesty(language) {
  return textTransformFill(formalResponse("formalization_honesty", language), [
    ["lean_check", "lean was not found in PATH, so the lean text was not compiled."],
    ["rocq_check", "coqc was not found in PATH, so the coqc text was not compiled."],
  ]);
}

/**
 * @param {object} grammar
 * @param {string} prompt
 * @param {string} language
 * @param {Array<string>} trace
 * @returns {Array<string|number>} [body, confidence]
 */
function formalFormalizeAnswer(grammar, prompt, language, trace) {
  const sentence = formalSentenceUnderDiscussion(prompt);
  let natural = grammar.natural.find((item) => item.language === language);
  if (natural === undefined) natural = grammar.natural.find((item) => item.language === "en");
  const clause = natural === undefined ? null : formalParseQuantifiedClause(sentence, natural);
  if (clause === null) {
    trace.push("formalization:clause:no quantified clause recognized");
    return [
      textTransformFill(formalResponse("formalization_unparsed", language), [
        ["reason", "no quantifier word or symbol matched the seed grammar"],
      ]),
      0.4,
    ];
  }
  trace.push(
    `formalization:clause:${clause.quantifier} ${clause.variable} antecedents ${clause.antecedent.length} consequent ${clause.consequent.name}`,
  );
  const all = grammar.formal.map((item) => item.slug);
  const named = formalNamedTarget(grammar, prompt, sentence.toLowerCase());
  const slugs = named === null
    ? all
    : all.filter((slug) => slug === named).concat(all.filter((slug) => slug !== named));
  const blocks = [];
  for (const slug of slugs) {
    const rendered = formalRenderClause(grammar, clause, slug);
    if (rendered !== null) blocks.push(`\`\`\`${slug}\n${rendered}\n\`\`\``);
  }
  const derivation = [
    `sentence: ${sentence}`,
    `parsed clause: ${clause.quantifier} ${clause.variable} (${clause.antecedent.map((p) => p.name).join(", ")})`,
    `consequent: ${clause.consequent.name}`,
    `templates: ${slugs.join(", ")}`,
  ].join("\n");
  return [
    textTransformFill(formalResponse("formalization_result", language), [
      ["statement_block", blocks.join("\n\n")],
      ["honesty", formalHonesty(language)],
      ["derivation", derivation],
    ]),
    0.7,
  ];
}

/**
 * The formal span: from the first quantifier symbol to the end.
 * @param {string} prompt
 * @returns {string}
 */
function formalSpan(prompt) {
  const starts = ["∀", "∃", "¬∃"].map((symbol) => prompt.indexOf(symbol)).filter((index) => index !== -1);
  const start = starts.length === 0 ? 0 : Math.min.apply(null, starts);
  return prompt.slice(start).trim();
}

/**
 * @param {object} grammar
 * @param {string} prompt
 * @param {string} language
 * @param {Array<string>} trace
 * @returns {Array<string|number>} [body, confidence]
 */
function formalDeformalizeAnswer(grammar, prompt, language, trace) {
  const clause = formalParseFolClause(grammar, formalSpan(prompt));
  if (clause === null) {
    trace.push("formalization:clause:no formal clause recognized");
    return [
      textTransformFill(formalResponse("formalization_unparsed", language), [
        ["reason", "no formal quantifier surface matched the seed grammar"],
      ]),
      0.4,
    ];
  }
  trace.push(`formalization:clause:${clause.quantifier} ${clause.variable} from formal text`);
  const natural = grammar.natural.find((item) => item.language === language);
  const rendered = formalRenderClauseNatural(grammar, clause, language);
  const sentence = rendered === null ? "" : rendered;
  const reparsed = natural === undefined ? null : formalParseQuantifiedClause(sentence, natural);
  let roundTrip = "re-parse failed (stated honestly)";
  if (reparsed !== null) {
    roundTrip = formalStructureKey(reparsed) === formalStructureKey(clause)
      ? "structure preserved (re-parse matches)"
      : "structure drifted (re-parse differs; stated honestly)";
  }
  trace.push(`formalization:round_trip:${roundTrip}`);
  return [
    textTransformFill(formalResponse("formalization_deformalized", language), [
      ["sentence", sentence],
      ["round_trip", roundTrip],
      ["honesty", formalHonesty(language)],
    ]),
    0.7,
  ];
}

/**
 * Recognize and answer a formalization or deformalization request
 * (`handle_formalization_request`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryFormalizationRequest(prompt, normalized) {
  const text = String(prompt || "");
  const roles = formalMatchedRoles(text, String(normalized || ""));
  // Every deformalize verb contains its formalize verb ("deformalize" /
  // "деформализуй"), so a matched direction cue decides the direction; the
  // native `formalize || …` precedence would read every deformalize request
  // as a formalize one (see the parity test for the probes this pins).
  const direction = roles.some((pair) => pair[0] === "direction");
  const formalize = !direction && roles.some((pair) => pair[0] === "command");
  const deformalize = direction ||
    (!formalize && ["∀", "∃", "¬∃"].some((symbol) => text.includes(symbol)));
  if (!formalize && !deformalize) return null;
  const language = roles.length > 0 ? roles[0][1] : "en";
  const trace = [
    `formalization:direction:${formalize ? "formalize" : "deformalize"}`,
    `formalization:language:${language}`,
  ];
  const grammar = formalGrammar();
  const result = formalize
    ? formalFormalizeAnswer(grammar, text, language, trace)
    : formalDeformalizeAnswer(grammar, text, language, trace);
  return {
    intent: FORMALIZATION_INTENT,
    content: result[0],
    confidence: result[1],
    evidence: ["handler:formalization_request", "response:formalization", `language:${language}`],
    trace: trace,
  };
}
