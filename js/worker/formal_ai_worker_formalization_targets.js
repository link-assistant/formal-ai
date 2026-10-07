// Issue #1186 R5 browser twin: deformalization from every rendered target, not
// only from first-order logic. Mirrors
// rust/src/solver_handlers/formalization_task_targets.rs: each target's own
// seed templates (data/seed/formal-targets.lino) run in reverse. A binder-style
// target (Lean 4, Rocq) is matched against the clause template its quantifier
// would have selected, with {variable}, {antecedent}, {consequent} and {joined}
// as capture slots, and every captured atom against the target's atom
// templates. A line-shaped target (Links Notation) is read field by field, the
// field names coming from the same templates. No surface is spelled here.

/**
 * Collapse every whitespace run to one space.
 * @param {string} text
 * @returns {string}
 */
function formalCollapseWhitespace(text) {
  return String(text).split(/\s+/u).filter((word) => word.length > 0).join(" ");
}

/**
 * Split a template into literal and slot pieces.
 * @param {string} template
 * @returns {Array<{slot: boolean, text: string}>}
 */
function formalTemplatePieces(template) {
  const pieces = [];
  let rest = template;
  for (;;) {
    const open = rest.indexOf("{");
    if (open === -1) break;
    const close = rest.indexOf("}", open);
    if (close === -1) break;
    if (open > 0) pieces.push({ slot: false, text: rest.slice(0, open) });
    pieces.push({ slot: true, text: rest.slice(open + 1, close) });
    rest = rest.slice(close + 1);
  }
  if (rest.length > 0) pieces.push({ slot: false, text: rest });
  return pieces;
}

/**
 * Match text against a template (`match_template`): the first literal may sit
 * anywhere, a literal after a slot is found at its first occurrence, a literal
 * after a literal must follow directly; with `anchored`, nothing may trail.
 * @param {string} template
 * @param {string} source
 * @param {boolean} anchored
 * @returns {Object<string, string>|null}
 */
function formalMatchTemplate(template, source, anchored) {
  const text = formalCollapseWhitespace(source);
  const captures = {};
  let cursor = 0;
  let pending = null;
  let first = true;
  for (const piece of formalTemplatePieces(formalCollapseWhitespace(template))) {
    if (piece.slot) {
      pending = piece.text;
    } else {
      let at = -1;
      if (pending !== null || first) {
        at = text.indexOf(piece.text, cursor);
        if (at === -1) return null;
      } else if (text.startsWith(piece.text, cursor)) {
        at = cursor;
      } else {
        return null;
      }
      if (pending !== null) {
        const value = text.slice(cursor, at).trim();
        if (value.length === 0) return null;
        captures[pending] = value;
        pending = null;
      }
      cursor = at + piece.text.length;
    }
    first = false;
  }
  const rest = text.slice(cursor).trim();
  if (pending !== null) {
    if (rest.length === 0) return null;
    captures[pending] = rest;
  } else if (anchored && rest.length > 0) {
    return null;
  }
  return captures;
}

/**
 * Read one atom back through the target's atom templates.
 * @param {object} language
 * @param {string} text
 * @param {string} variable
 * @returns {{name: string, object: string|null}|null}
 */
function formalParseTargetAtom(language, text, variable) {
  const bind = (template) => textTransformFill(template, [["variable", variable]]);
  const withObject = formalMatchTemplate(bind(language.atomWithObject), text, true);
  if (withObject !== null && withObject.predicate && withObject.object) {
    return { name: formalCapitalize(withObject.predicate), object: withObject.object };
  }
  const bare = formalMatchTemplate(bind(language.atom), text, true);
  if (bare === null || !bare.predicate) return null;
  return { name: formalCapitalize(bare.predicate), object: null };
}

/**
 * Split a captured body on the target's `and` join and read every atom.
 * @param {object} language
 * @param {string} body
 * @param {string} variable
 * @returns {Array<object>|null}
 */
function formalParseTargetAtoms(language, body, variable) {
  const join = formalLookup(language.joins, "and");
  if (join === null || join.trim().length === 0) return null;
  const atoms = body.split(join.trim()).map((atom) => formalParseTargetAtom(language, atom, variable));
  return atoms.some((atom) => atom === null) ? null : atoms;
}

/**
 * The clause template `formalRenderClause` picks for a quantifier kind.
 * @param {object} language
 * @param {string} kind
 * @returns {string}
 */
function formalClauseTemplateFor(language, kind) {
  if (kind === "forall") return language.clauseConditional;
  if (kind === "no" && language.clauseNegative.length > 0) return language.clauseNegative;
  return language.clauseConjunctive;
}

/**
 * Reverse a binder-style rendering (Lean 4, Rocq) into a clause.
 * @param {object} language
 * @param {string} text
 * @returns {object|null}
 */
function formalParseBinderClause(language, text) {
  // `no` before `exists`: the negative surface embeds the existential one.
  for (const kind of ["forall", "no", "exists"]) {
    const symbol = formalLookup(language.quantifiers, kind);
    const template = textTransformFill(formalClauseTemplateFor(language, kind), [
      ["quantifier", symbol === null ? "" : symbol],
      ["quantifier_word", kind],
    ]);
    const captures = formalMatchTemplate(template, text, false);
    if (captures === null || !captures.variable) continue;
    const variable = captures.variable;
    let antecedent = null;
    let consequent = null;
    if (captures.antecedent && captures.consequent) {
      antecedent = formalParseTargetAtoms(language, captures.antecedent, variable);
      consequent = formalParseTargetAtom(language, captures.consequent, variable);
    } else if (captures.joined) {
      const atoms = formalParseTargetAtoms(language, captures.joined, variable);
      if (atoms !== null && atoms.length > 0) {
        consequent = atoms[atoms.length - 1];
        antecedent = atoms.slice(0, atoms.length - 1);
      }
    }
    if (antecedent === null || consequent === null || antecedent.length === 0) return null;
    return { quantifier: kind, variable: variable, antecedent: antecedent, consequent: consequent };
  }
  return null;
}

/**
 * The first word of the template line carrying a slot, or of the line before.
 * @param {string} template
 * @param {string} slot
 * @param {boolean} lineBefore
 * @returns {string|null}
 */
function formalLineKey(template, slot, lineBefore) {
  const lines = template.split("\n");
  const index = lines.findIndex((line) => line.includes(`{${slot}}`));
  if (index === -1) return null;
  const line = lineBefore ? lines[index - 1] : lines[index];
  if (line === undefined) return null;
  const words = line.trim().split(/\s+/u);
  return words[0] || null;
}

/**
 * Reverse a line-shaped rendering (Links Notation) into a clause.
 * @param {object} language
 * @param {string} text
 * @returns {object|null}
 */
function formalParseLineClause(language, text) {
  const head = language.clauseConditional.split("\n")[0].trim();
  const keys = {
    quantifier: formalLineKey(language.clauseConjunctive, "quantifier_word", false),
    variable: formalLineKey(language.clauseConditional, "variable", false),
    antecedent: formalLineKey(language.clauseConditional, "antecedent", true),
    consequent: formalLineKey(language.clauseConditional, "consequent", true),
    conjuncts: formalLineKey(language.clauseConjunctive, "joined", true),
    predicate: formalLineKey(language.atom, "predicate", false),
    object: formalLineKey(language.atomWithObject, "object", false),
  };
  if (Object.values(keys).some((key) => key === null)) return null;
  const lines = String(text).split("\n").map((line) => line.trim());
  const start = lines.indexOf(head);
  if (start === -1) return null;
  let kind = "";
  let variable = FORMALIZATION_DEFAULT_VARIABLE;
  const sections = {};
  let section = "";
  for (const line of lines.slice(start + 1)) {
    const space = line.indexOf(" ");
    const key = space === -1 ? line : line.slice(0, space);
    const value = space === -1 ? "" : line.slice(space + 1).trim().replace(/^"+|"+$/gu, "");
    if (key === keys.quantifier) {
      const found = language.quantifiers.find((pair) => pair[1] === value);
      if (found === undefined) return null;
      kind = found[0];
    } else if (key === keys.variable && value.length > 0) {
      variable = value;
    } else if (key === keys.antecedent || key === keys.consequent || key === keys.conjuncts) {
      section = key;
    } else if (key === keys.predicate && value.length > 0) {
      if (sections[section] === undefined) sections[section] = [];
      sections[section].push({ name: formalCapitalize(value), object: null });
    } else if (key === keys.object && value.length > 0) {
      const items = sections[section];
      if (items === undefined || items.length === 0) return null;
      items[items.length - 1].object = value;
    }
  }
  if (kind.length === 0) return null;
  const consequents = sections[keys.consequent] || [];
  let antecedent = [];
  let consequent = null;
  if (consequents.length === 0) {
    const conjuncts = sections[keys.conjuncts] || [];
    if (conjuncts.length === 0) return null;
    consequent = conjuncts[conjuncts.length - 1];
    antecedent = conjuncts.slice(0, conjuncts.length - 1);
  } else {
    consequent = consequents[consequents.length - 1];
    antecedent = sections[keys.antecedent] || [];
  }
  if (antecedent.length === 0) return null;
  return { quantifier: kind, variable: variable, antecedent: antecedent, consequent: consequent };
}

/**
 * Parse the rendering of any non-FOL target, naming the target it matched.
 * @param {object} grammar
 * @param {string} text
 * @returns {Array<string|object>|null} [slug, clause]
 */
function formalParseTargetClause(grammar, text) {
  for (const language of grammar.formal) {
    if (language.slug === "fol") continue;
    const lineShaped = language.atom.trimEnd().split("\n").length > 1;
    const clause = lineShaped ? formalParseLineClause(language, text) : formalParseBinderClause(language, text);
    if (clause !== null) return [language.slug, clause];
  }
  return null;
}

/**
 * Parse a formal rendering in any declared target, FOL first.
 * @param {object} grammar
 * @param {string} prompt
 * @returns {Array<string|object>|null} [slug, clause]
 */
function formalParseAnyFormalClause(grammar, prompt) {
  const fol = formalParseFolClause(grammar, formalSpan(prompt));
  if (fol !== null) return ["fol", fol];
  return formalParseTargetClause(grammar, prompt);
}

/**
 * Whether re-rendering the clause into its source target and parsing that
 * text again reproduces the clause structure.
 * @param {object} grammar
 * @param {object} clause
 * @param {string} slug
 * @returns {boolean}
 */
function formalTargetRoundTripHolds(grammar, clause, slug) {
  const rendered = formalRenderClause(grammar, clause, slug);
  if (rendered === null) return false;
  const again = formalParseAnyFormalClause(grammar, rendered);
  return again !== null && formalStructureKey(again[1]) === formalStructureKey(clause);
}
