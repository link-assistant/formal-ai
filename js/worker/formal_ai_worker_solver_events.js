// The native solver's event log for one turn (R1013, server parity).
//
// The browser trace (`steps`) narrates the worker's own dispatch. Beside it,
// every answer carries `solverEvents`: the `[{kind, payload}]` events the Rust
// solver appends to its `EventLog` for the same turn, so the JavaScript server
// projects them through js/server/solver-trace.mjs (a port of
// `EventLog::thinking_steps_for_answer`) and both servers narrate one trace.
//
// Mirrors rust/src/solver.rs `solve_*` (the `impulse` / `language` prelude and
// `record_intent_formalization`), rust/src/intent_formalization.rs
// `route_for_prompt` / `route_from_relevants`, rust/src/handler_promotion.rs
// `promoted_relevants`, rust/src/solver_handlers/mod.rs `try_arithmetic` /
// `finalize_simple`, rust/src/meta_reasoner/integration.rs `project`, and the
// `link-calculator` crate's `Calculator::calculate_with_value` for the plain
// numeric subset (literals, `+ - * / %`, `^`, unary minus, parentheses).
//
// Not mirrored yet: the handler-specific events of routes other than the
// calculator (concept lookups, web search, programs...), the coding
// `task_spec` route, and `cue_set` promotion conditions (the worker's rule
// interpreter does not read the cue lexicon). Those turns still project the
// shared prelude and finish, so their traces differ only in the middle.

const SOLVER_EVENT_VALIDATION = "validation";
const SOLVER_VALIDATION_ACCEPTED = "accepted_without_extra_constraints";
const SOLVER_EVENT_SIMPLIFICATION = "trace:simplification";
const SOLVER_SIMPLIFICATION_SMALLEST = "smallest_sufficient";
const SOLVER_RESPONSE_PREFIX = "response:";
const SOLVER_META_RESPONSE_LINK = "response:meta_reasoner";
const SOLVER_ENGINE_LINK_CALCULATOR = "link-calculator";
const SOLVER_PROMOTIONS_FILE = "handler-promotions.lino";
const SOLVER_CACHE_POLICY_FILE = "program-cache-policy.lino";

function solverEvent(kind, payload) {
  return { kind, payload: String(payload === undefined || payload === null ? "" : payload) };
}

// ---------------------------------------------------------------- calculator

// `rust_decimal::Decimal::normalize` as `Display`: no leading zeros on the
// integer part, no trailing zeros (or point) on the fraction.
function linkCalculatorNumberText(text) {
  const [whole, fraction = ""] = String(text).split(".");
  const integer = whole.replace(/^0+(?=\d)/, "") || "0";
  const trimmed = fraction.replace(/0+$/, "");
  return trimmed ? `${integer}.${trimmed}` : integer;
}

function linkCalculatorTokens(expression) {
  const tokens = [];
  const source = String(expression || "");
  let index = 0;
  while (index < source.length) {
    const character = source[index];
    if (/\s/.test(character)) {
      index += 1;
    } else if (/[0-9]/.test(character)) {
      const match = /^[0-9]+(?:\.[0-9]+)?/.exec(source.slice(index));
      tokens.push({ kind: "num", text: match[0] });
      index += match[0].length;
    } else if ("+-*/%^()".includes(character)) {
      tokens.push({ kind: character });
      index += 1;
    } else {
      return null;
    }
  }
  return tokens.length ? tokens : null;
}

// Parse with the crate's grammar (`parse_additive` .. `parse_primary`) and
// return `{lino, steps}`: `Expression::to_lino` and the length of the step list
// `evaluate_with_steps` records, or null outside the numeric subset.
function linkCalculatorTrace(expression) {
  const tokens = linkCalculatorTokens(expression);
  if (!tokens) return null;
  let cursor = 0;
  const peek = () => tokens[cursor];
  const startsOperand = (token) => Boolean(token) && (token.kind === "num" || token.kind === "(");
  function primary() {
    const token = tokens[cursor++];
    if (!token) throw new Error("eof");
    if (token.kind === "num") return { lino: linkCalculatorNumberText(token.text), steps: 1, kind: "num" };
    if (token.kind !== "(") throw new Error("token");
    const inner = additive();
    if (!peek() || peek().kind !== ")") throw new Error("paren");
    cursor += 1;
    const wrapped = inner.lino.startsWith("(") && inner.lino.endsWith(")") ? inner.lino : `(${inner.lino})`;
    return { lino: wrapped, steps: inner.steps + 1, kind: "group" };
  }
  function unary() {
    if (peek() && peek().kind === "-") {
      cursor += 1;
      const inner = unary();
      const lino = inner.kind === "binary" ? `(-(${inner.lino}))` : `(-${inner.lino})`;
      return { lino, steps: inner.steps + 1, kind: "negate" };
    }
    const operand = primary();
    if (peek() && peek().kind === "%" && !startsOperand(tokens[cursor + 1])) throw new Error("percent");
    return operand;
  }
  function power() {
    const base = unary();
    if (peek() && peek().kind === "^") {
      cursor += 1;
      const exponent = power();
      return { lino: `(${base.lino} ^ ${exponent.lino})`, steps: base.steps + exponent.steps + 2, kind: "power" };
    }
    return base;
  }
  function binary(next, operators) {
    let left = next();
    while (peek() && operators.includes(peek().kind)
      && (peek().kind !== "%" || startsOperand(tokens[cursor + 1]))) {
      const operator = tokens[cursor++].kind;
      const right = next();
      left = { lino: `(${left.lino} ${operator} ${right.lino})`, steps: left.steps + right.steps + 2, kind: "binary" };
    }
    return left;
  }
  const multiplicative = () => binary(power, ["*", "/", "%"]);
  const additive = () => binary(multiplicative, ["+", "-"]);
  try {
    const tree = additive();
    if (cursor !== tokens.length) return null;
    return { lino: tree.lino, steps: tree.steps + 2 };
  } catch (_error) {
    return null;
  }
}

// `try_arithmetic`: the calculator trace for one evaluated expression.
function solverCalculationEvents(expression, content, backend) {
  const events = [solverEvent("calculation:request", expression)];
  const trace = /^js$|^wasm$/.test(String(backend)) ? linkCalculatorTrace(expression) : null;
  if (trace) {
    events.push(solverEvent("calculation:engine", SOLVER_ENGINE_LINK_CALCULATOR));
    events.push(solverEvent("calculation:lino", trace.lino));
    events.push(solverEvent("calculation:steps", trace.steps));
  }
  events.push(solverEvent("calculation", content));
  return events;
}

// ---------------------------------------------------------------- routing

let cachedSolverPromotions = null;

// `handler_promotion::promotions`: `{handler, rank, when}` rows in rank order.
function solverHandlerPromotions() {
  if (cachedSolverPromotions) return cachedSolverPromotions;
  const text = typeof SEED_RAW === "object" ? seedRawText(SEED_RAW, SOLVER_PROMOTIONS_FILE) : "";
  const rows = [];
  const root = text ? handlerRulesParseTree(text).find((node) => node.name === "handler_promotions") : null;
  for (const [declaration, node] of (root ? root.children : []).entries()) {
    if (node.name !== "promotion" || !node.args.length) continue;
    const rankNode = node.children.find((child) => child.name === "rank");
    const whenNode = node.children.find((child) => child.name === "when");
    if (!rankNode || !whenNode) continue;
    try {
      rows.push({
        handler: node.args[0],
        rank: Number(rankNode.args[0]),
        declaration,
        when: { kind: "all", children: whenNode.children.map(handlerRulesParseCondition) },
      });
    } catch (_error) {
      // A row the interpreter cannot compile never promotes (Rust skips it too).
    }
  }
  rows.sort((left, right) => left.rank - right.rank || left.declaration - right.declaration);
  cachedSolverPromotions = rows;
  return rows;
}

// `promoted_relevants`: the handlers a prompt hoists, in rank order; `cue_set` rows hold only with `cueSets`.
function solverPromotedHandlers(prompt, cueSets = null) {
  const source = String(prompt || "");
  const normalized = canonicalizedPrompt(normalizePrompt(source));
  const language = detectLanguage(source);
  const context = {
    prompt: source,
    language,
    languages: language === "en" ? ["en"] : [language, "en"],
    history: [],
    cueSets,
    subjects: {
      normalized,
      cleaned: normalizePrompt(normalized),
      lowercase: source.toLowerCase(),
      prompt: source,
      trimmed: normalized.trim(),
      padded: ` ${normalized} `,
    },
  };
  return solverHandlerPromotions()
    .filter((row) => handlerRulesHolds(row.when, context))
    .map((row) => row.handler);
}

// `ordered_method_names_for_relevants`: the rows a prompt promotes run first, in promotion rank order.
// A hoisted row claims only what it can answer; its impasse waits for the row's own rank. A concrete catalog
// request runs the write-program rows first: solver.rs skips meta dispatch for `is_concrete_write_program`.
function promotedHandlerOrder(registry, prompt) {
  const requested = typeof writeProgramParameters === "function" ? writeProgramParameters(prompt) : null;
  const concrete = (record) => record.contextBinding === "writeProgram" && record.resultIntent === "write_program";
  if (requested && requested.task && requested.language && writeProgramTemplate(requested.task, requested.language) !== null
    && tryPageQueryText(prompt) === null) return [...registry.filter(concrete), ...registry.filter((record) => !concrete(record))];
  const bindings = typeof WORKER_HANDLER_REGISTRY === "object" ? WORKER_HANDLER_REGISTRY.workerHandlers : {};
  const names = [...new Set(solverPromotedHandlers(prompt).map((slug) => bindings[slug]).filter(Boolean))];
  const hoisted = names.flatMap((name) => registry.filter((record) => record.name === name)).map((record) => ({ ...record, hoisted: true }));
  return [...hoisted, ...registry];
}

function solverRouteMatches(normalized, route) {
  const tokens = normalized.split(/\s+/).filter(Boolean);
  const hasToken = (expected) =>
    /[぀-ヿ㐀-鿿가-힯]/.test(expected) ? normalized.includes(expected) : tokens.includes(expected);
  return (route.keywords || []).includes(normalized)
    || (route.phrases || []).includes(normalized)
    || (route.tokens || []).some(hasToken)
    || (route.combos || []).some((combo) => Array.isArray(combo) && combo.length > 0 && combo.every(hasToken));
}

// `route_for_prompt`: the matched `{slug, responseLink}` (write_program, a declared role surface, the table), or null.
function solverRouteForPrompt(prompt) {
  const normalized = normalizePrompt(prompt);
  if (typeof writeProgramParameters === "function" && writeProgramParameters(prompt)) return { slug: "write_program", responseLink: "response:write_program" };
  const intents = INTENT_ROUTING && Array.isArray(INTENT_ROUTING.intents) ? INTENT_ROUTING.intents : [];
  const route = intents.find((row) => Array.isArray(row.roleSurfaces) && row.roleSurfaces.some((role) => wordsForRole(role).includes(normalized)))
    || intents.find((row) => solverRouteMatches(normalized, row));
  return route ? { slug: route.slug, responseLink: route.responseLink } : null;
}

// `route_for_prompt`, then `route_from_relevants` over the promoted handlers.
function solverIntentRoute(prompt) {
  const matched = solverRouteForPrompt(prompt);
  return matched ? matched.slug : solverPromotedHandlers(prompt)[0] || null;
}

// ---------------------------------------------------------------- the log

// The `SelectedRule::WriteProgram` tail `solve` logs after `intent`
// (`program_parameter:*`, `program_parameters`, `legacy_intent`), read back
// from the parameters the program answer already carries as evidence, with
// the `obligation_gap` report of a request carrying work obligations (R1166-4,
// formal_ai_worker_obligations.js).
function solverWriteProgramEvents(answer, prompt) {
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const parameter = (name) => {
    const prefix = `program_parameter:${name}:`;
    const link = evidence.find((item) => String(item).startsWith(prefix));
    return link ? String(link).slice(prefix.length) : "";
  };
  const language = parameter("language");
  const task = parameter("task");
  if (answer.intent !== "write_program" || !language || !task) return [];
  const legacy = task === "hello_world" ? `hello_world_${language}` : `write_program_${task}_${language}`;
  return [
    solverEvent("program_parameter:language", language),
    solverEvent("program_parameter:task", task),
    solverEvent("program_parameters", `write_program(language=${language}, task=${task})`),
    solverEvent("legacy_intent", legacy),
    ...solverObligationGapEvents(prompt),
    ...solverProcedureCacheEvents(prompt, language, task),
    // R1165-6: where each command shown comes from, and what the documented
    // program departs from (`documentation_events` in Rust).
    ...documentationEvents(task, language).map((pair) => solverEvent(pair[0], pair[1])),
  ];
}

// The `procedure_cache` event of the native `WriteProgram` branch (R1165-10).
// The worker reads no cache file, so an unmodified catalog request either
// rediscovers its program from the documentation captures (R1165-1,
// `outcome=discovered`) or is the miss Rust logs, naming the `miss_route`
// inputs a solve does not carry and why captured pages yielded nothing.
function solverProcedureCacheEvents(prompt, language, task) {
  const template = writeProgramTemplate(task, language);
  if (!template || applyInlineHelloWorldOutputReplacement(prompt, task, template) !== template) return [];
  const { recipe, rejected } = documentedProgram(task, language);
  if (recipe) {
    const fields = `rediscovery_source=${recipe.rediscovery_source} content_id=${recipe.content_id}`;
    return [solverEvent("procedure_cache", `outcome=discovered language=${language} task=${task} ${fields}`)];
  }
  const missing = solverMissResearchMissing().join(",") + (rejected ? ` documentation_rejected=${rejected}` : "");
  return [solverEvent("procedure_cache", `outcome=miss language=${language} task=${task} research_missing=${missing}`)];
}

// Mirrors `discovery_production::miss_research_missing`.
function solverMissResearchMissing() {
  const text = typeof SEED_RAW === "object" ? seedRawText(SEED_RAW, SOLVER_CACHE_POLICY_FILE) : "";
  const route = (parseLinoTree(text).children[0]?.children || []).find((node) => node.name === "miss_route");
  const values = (name) => (route?.children || []).filter((node) => node.name === name).map((node) => node.value);
  const carried = values("solve_carries");
  return values("research_requires").filter((input) => !carried.includes(input));
}

// Marks a meta-reasoner answer, which `project` finishes without validation.
function solverMetaProjection(answer) {
  if (answer) answer.solverMetaProjection = true;
  return answer;
}

// The whole log for a finished answer: the solver prelude, the route, the
// handler's own events, then `finalize_simple` (or the meta reasoner's
// `project`, which records no validation). Not recorded yet: the formalization,
// intent-formalization and meta-core records, and `finalize_simple`'s `candidate`.
function solverEventLog(prompt, answer) {
  const events = [solverEvent("impulse", prompt), solverEvent("language", detectLanguage(prompt))];
  const route = solverIntentRoute(prompt);
  if (route) events.push(solverEvent("intent_formalization:route", route));
  const handlerEvents = Array.isArray(answer.solverEvents) ? answer.solverEvents : [];
  for (const event of handlerEvents) events.push(event);
  events.push(solverEvent("intent", answer.intent));
  for (const event of solverWriteProgramEvents(answer, prompt)) events.push(event);
  if (answer.solverMetaProjection) {
    events.push(solverEvent("response", SOLVER_META_RESPONSE_LINK));
    events.push(solverEvent("trace", answer.intent));
    return events;
  }
  if (!events.some((event) => event.kind === SOLVER_EVENT_VALIDATION)) {
    events.push(solverEvent(SOLVER_EVENT_VALIDATION, SOLVER_VALIDATION_ACCEPTED));
  }
  events.push(solverEvent("response", `${SOLVER_RESPONSE_PREFIX}${answer.intent}`));
  // The `finalize_simple` tail; the `trace` link is the Telegram `/trace` footer.
  events.push(solverEvent(SOLVER_EVENT_SIMPLIFICATION, SOLVER_SIMPLIFICATION_SMALLEST));
  events.push(solverEvent("trace", answer.intent));
  return events;
}
