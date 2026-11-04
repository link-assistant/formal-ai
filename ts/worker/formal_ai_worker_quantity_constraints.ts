// The explicit quantity aggregate parser and inclusive-bound comparison.
// Native twin rust/src/quantity_constraint.rs owns the same source frame;
// tryArithmetic also delegates ordinary expressions to tryArithmeticExpression.

// Explicit mathematical aggregates only; labels never imply execution order.
function quantityConstraintRolePattern(role) {
  return wordsForRole(role).slice().sort((a, b) => b.length - a.length)
    .map(word => escapeRegExp(word).replace(/\s+/gu, "\\s+")).join("|");
}

function quantityConstraintMatch(text, role, captures, whole = true) {
  for (const template of wordsForRole(role)) {
    let source = "", valid = true;
    for (const part of template.split(/(\{[a-z_]+\})/gu)) {
      const slot = /^\{([a-z_]+)\}$/u.exec(part);
      if (slot) {
        if (!captures[slot[1]]) { valid = false; break; }
        source += "(?<" + slot[1] + ">" + captures[slot[1]] + ")";
      } else source += escapeRegExp(part).replace(/\s+/gu, "\\s+");
    }
    if (!valid) continue;
    const pattern = whole ? "^\\s*" + source + "\\s*[.?!]?\\s*$"
      : "(?<![\\p{L}\\p{N}_])" + source + "(?=$|[^\\p{L}\\p{N}_])";
    const found = new RegExp(pattern, "idu").exec(text);
    if (found) return found;
  }
  return null;
}

function quantityConstraintSpan(raw, start, end) {
  const encode = new TextEncoder();
  return {start: encode.encode(raw.slice(0, start)).length,
    end: encode.encode(raw.slice(0, end)).length};
}

function quantityConstraintUnit(surface) {
  if (surface !== surface.toLowerCase()) return null;
  const canonical = siUnitNamedBy(surface);
  const candidates = new Set(unitMentions(surface).filter(mention =>
    mention.position === 0 && mention.surface.length === surface.length)
    .map(mention => siUnitNamedBy(mention.slug) || mention.slug));
  return canonical && candidates.size === 1 && candidates.has(canonical) ? canonical : null;
}

// Initial English subset: three fully consumed bound/list/comparison sentences.
// Quantities are nonnegative exact integers in one canonical unit; no scheduling inference.
function quantityConstraintFor(prompt) {
  const raw = String(prompt || "");
  if (raw.length > 8192) return null;
  const prefix = quantityConstraintRolePattern("quantity_constraint_query_prefix");
  const aggregate = quantityConstraintRolePattern("quantity_constraint_aggregate");
  if (!prefix || !aggregate) return null;
  const sentences = [...raw.matchAll(/[^.?!]+(?:[.?!]|$)/gu)];
  const query = sentences[sentences.length - 1];
  if (!query || !new RegExp("^\\s*(?:" + prefix + ")(?=\\s|$)", "iu").test(query[0])) return null;
  const aggregateMatch = new RegExp("(?<![\\p{L}\\p{N}_])(?:" + aggregate + ")(?=$|[^\\p{L}\\p{N}_])", "idu").exec(query[0]);
  if (!aggregateMatch) return null;
  const boundNames = quantityConstraintRolePattern("quantity_constraint_bound_reference");
  const relation = quantityConstraintRolePattern("quantity_constraint_relation_le");
  if (!boundNames || !relation || !quantityConstraintMatch(query[0],
    "quantity_constraint_ownership_frame", {
      relation: "(?:" + relation + ")", query_ref: "(?:" + boundNames + ")"
    }, false)) return null;
  const unknown = reason => ({status: "unknown", reason,
    aggregationProof: {kind: "explicit-sum-cue", surface: aggregateMatch[0],
      span: quantityConstraintSpan(raw, query.index + aggregateMatch.index,
        query.index + aggregateMatch.index + aggregateMatch[0].length)},
    source: raw, sourceUtf8Bytes: new TextEncoder().encode(raw).length});
  if (sentences.length !== 3 || sentences.map(row => row[0]).join("") !== raw) {
    return unknown("incomplete-raw-frame");
  }
  const members = quantityConstraintRolePattern("quantity_constraint_member_reference");
  const conjunction = quantityConstraintRolePattern("quantity_constraint_list_conjunction");
  if (!boundNames || !members || !relation || !conjunction) return unknown("missing-seeded-grammar");
  const patterns = {bound_ref: "(?:" + boundNames + ")", query_ref: "(?:" + boundNames + ")",
    bound_amount: "[+-]?[0-9]+", bound_unit: "[\\p{L}]+", members_ref: "(?:" + members + ")",
    quantities: "[\\s\\S]+?", aggregation: "(?:" + aggregate + ")", relation: "(?:" + relation + ")"};
  const bound = quantityConstraintMatch(sentences[0][0], "quantity_constraint_bound_frame", patterns);
  const collection = quantityConstraintMatch(sentences[1][0], "quantity_constraint_members_frame", patterns);
  const comparison = quantityConstraintMatch(query[0], "quantity_constraint_comparison_frame", patterns);
  if (!bound || !collection || !comparison) return unknown("unsupported-constraint-frame");
  if (bound.groups.bound_ref.toLowerCase() !== comparison.groups.query_ref.toLowerCase()) {
    return unknown("unbound-capacity-reference");
  }
  const boundValue = parseWordProblemInteger(bound.groups.bound_amount);
  const unit = quantityConstraintUnit(bound.groups.bound_unit);
  if (!Number.isSafeInteger(boundValue) || boundValue < 0 || !unit) return unknown("unresolved-bound");
  const captureSpan = (row, field, start) => quantityConstraintSpan(raw,
    start + row.indices.groups[field][0], start + row.indices.groups[field][1]);
  const values = collection.groups.quantities;
  const valueStart = sentences[1].index + collection.indices.groups.quantities[0];
  const token = /\s*(?<amount>[+-]?[0-9]+)\s+(?<unit>[\p{L}]+)\s*/diyu;
  const join = new RegExp("(?:" + conjunction + ")(?=\\s|$)\\s*", "iyu");
  const terms = [];
  let cursor = 0, finalTerm = false;
  while (cursor < values.length && terms.length < 32) {
    token.lastIndex = cursor;
    const current = token.exec(values);
    if (!current) return unknown("unresolved-quantity");
    const value = parseWordProblemInteger(current.groups.amount);
    if (!Number.isSafeInteger(value) || value < 0
      || quantityConstraintUnit(current.groups.unit) !== unit) return unknown("unresolved-quantity-unit");
    terms.push({literal: current.groups.amount, unit, unitSurface: current.groups.unit,
      span: captureSpan(current, "amount", valueStart),
      unitSpan: captureSpan(current, "unit", valueStart)});
    cursor = token.lastIndex;
    if (cursor === values.length) break;
    if (finalTerm) return unknown("ambiguous-collection-tail");
    let comma = false;
    if (values[cursor] === ",") { comma = true; cursor++; }
    while (/\s/u.test(values[cursor] || "") && cursor < values.length) cursor++;
    join.lastIndex = cursor;
    const joined = join.exec(values);
    if (joined) { cursor = join.lastIndex; finalTerm = true; }
    else if (!comma) return unknown("unowned-collection-separator");
    if (cursor >= values.length) return unknown("missing-quantity");
  }
  if (terms.length === 0 || cursor !== values.length) return unknown("unbounded-collection");
  return {status: "supported", aggregation: "sum", relation: "le", terms,
    bound: {literal: bound.groups.bound_amount, unit, unitSurface: bound.groups.bound_unit,
      span: captureSpan(bound, "bound_amount", sentences[0].index),
      unitSpan: captureSpan(bound, "bound_unit", sentences[0].index)},
    aggregationProof: {kind: "explicit-sum-cue", surface: comparison.groups.aggregation,
      span: captureSpan(comparison, "aggregation", query.index)},
    comparisonProof: {role: "quantity_constraint_relation_le", surface: comparison.groups.relation,
      span: captureSpan(comparison, "relation", query.index)},
    source: raw, sourceUtf8Bytes: new TextEncoder().encode(raw).length};
}

function tryQuantityConstraint(prompt) {
  const frame = quantityConstraintFor(prompt);
  if (!frame) return null;
  const language = detectLanguage(prompt);
  const render = intent => answerFor(intent, language) || answerFor(intent, "en");
  const unresolved = reason => ({intent: "quantity_constraint_unknown",
    content: render("quantity_constraint_unknown"), confidence: 0.4,
    evidence: ["quantity_constraint:unknown:" + reason,
      "quantity_constraint:frame:" + JSON.stringify(frame)],
    solverEvents: [solverEvent("quantity_constraint:frame", JSON.stringify(frame)),
      solverEvent("quantity_constraint:unknown", reason)]});
  if (frame.status !== "supported") return unresolved(frame.reason);
  const expression = frame.terms.map(term => term.literal).join(" + ");
  try {
    const total = evaluateArithmetic(expression);
    const difference = evaluateArithmetic("(" + expression + ") - (" + frame.bound.literal + ")");
    const exactInteger = value => typeof value === "bigint"
      || typeof value === "number" && Number.isSafeInteger(value);
    if (!exactInteger(total) || !exactInteger(difference)) return unresolved("inexact-integer");
    const holds = typeof difference === "bigint" ? difference <= 0n : difference <= 0;
    const intent = holds ? "quantity_constraint_within" : "quantity_constraint_exceeds";
    let content = render(intent);
    for (const [key, value] of [["total", formatArithmeticResult(total)],
      ["bound", frame.bound.literal], ["unit", frame.bound.unitSurface]]) {
      content = String(content).split("{" + key + "}").join(value);
    }
    return {intent, content, confidence: 1,
      evidence: ["quantity_constraint:verdict:" + holds,
        "quantity_constraint:frame:" + JSON.stringify(frame),
        "quantity_constraint:expression:" + expression,
        "quantity_constraint:difference:" + formatArithmeticResult(difference), "calculation_backend:js"],
      solverEvents: [
        solverEvent("quantity_constraint:frame", JSON.stringify(frame)),
        solverEvent("quantity_constraint:verdict", String(holds)),
        solverEvent("quantity_constraint:expression", expression),
        solverEvent("quantity_constraint:difference", formatArithmeticResult(difference)),
        ...solverCalculationEvents(expression, formatArithmeticResult(total), "js")
      ]};
  } catch (_error) { return unresolved("arithmetic-unresolved"); }
}


function tryArithmetic(prompt) {
  return tryQuantityConstraint(prompt) || tryArithmeticExpression(prompt);
}
