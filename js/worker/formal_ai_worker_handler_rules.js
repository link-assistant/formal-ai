// Browser interpreter for data/seed/handler-rules.lino (issue #1085 D1.3).
//
// Mirrors rust/src/rule_interpreter.rs and rust/src/rule_interpreter/parser.rs:
// a `handler` block per precedence name, each `rule` with a `when` tree of
// conditions over seed roles and prompt shape, captured `value`s, `log` steps
// and a `respond` line naming a multilingual seed response. The rules, their
// vocabulary and their wording all live in data/seed; this module only walks
// the links, so the browser answers the rule-backed handlers (clarification,
// punctuation_only_prompt, ill_formed, physical_action_question,
// opinion_question) exactly as the native interpreter does.

/**
 * Tokens of one rule line: whitespace-separated, quotes keep spaces.
 * @param {string} line
 * @returns {string[]}
 */
function handlerRulesTokenize(line) {
  const tokens = [];
  let current = "";
  let quote = "";
  for (const character of Array.from(line)) {
    if (quote !== "" && character === quote) {
      quote = "";
      tokens.push(current);
      current = "";
    } else if (quote === "" && (character === '"' || character === "'")) {
      quote = character;
    } else if (quote === "" && /\s/u.test(character)) {
      if (current !== "") {
        tokens.push(current);
        current = "";
      }
    } else {
      current += character;
    }
  }
  if (current !== "") tokens.push(current);
  return tokens;
}

/**
 * Parse an indented rule document into `{name, args, children}` nodes
 * (Rust `parser::parse_tree`).
 * @param {string} text
 * @returns {{name: string, args: string[], children: object[]}[]}
 */
function handlerRulesParseTree(text) {
  const roots = [];
  const stack = [];
  const attach = (node) => {
    if (stack.length > 0) stack[stack.length - 1].node.children.push(node);
    else roots.push(node);
  };
  for (const raw of String(text || "").split("\n")) {
    const trimmed = raw.replace(/^\s+/u, "");
    if (trimmed === "" || trimmed.startsWith("#")) continue;
    const indent = raw.length - trimmed.length;
    const tokens = handlerRulesTokenize(trimmed.replace(/\s+$/u, ""));
    if (tokens.length === 0) continue;
    const node = { name: tokens[0], args: tokens.slice(1), children: [] };
    while (stack.length > 0 && stack[stack.length - 1].indent >= indent) {
      const finished = stack.pop();
      attach(finished.node);
    }
    stack.push({ indent, node });
  }
  while (stack.length > 0) {
    const finished = stack.pop();
    attach(finished.node);
  }
  return roots;
}

/**
 * @param {{name: string, args: string[]}} node
 * @returns {string}
 */
function handlerRulesFirstArg(node) {
  if (node.args.length === 0) throw new Error("handler_rules:missing_argument");
  return node.args[0];
}

/**
 * Parse one condition node (Rust `parser::parse_condition`).
 * @param {{name: string, args: string[], children: object[]}} node
 * @returns {object}
 */
function handlerRulesParseCondition(node) {
  const options = [];
  let subject = "normalized";
  const rest = node.args.slice(Math.min(1, node.args.length));
  for (let index = 0; index < rest.length; index += 1) {
    if (rest[index] === "of") {
      if (index + 1 < rest.length) subject = rest[index + 1];
      index += 1;
    } else {
      options.push(rest[index]);
    }
  }
  const subjects = ["normalized", "cleaned", "lowercase", "prompt", "trimmed", "padded"];
  if (!subjects.includes(subject)) throw new Error("handler_rules:unknown_subject");
  switch (node.name) {
    case "all":
    case "any":
    case "none":
      return { kind: node.name, children: node.children.map(handlerRulesParseCondition) };
    case "role": {
      const mode = options.length === 0 ? "spelled" : options[0];
      const modes = ["spelled", "raw", "languages", "forms", "separated", "whole"];
      if (!modes.includes(mode)) throw new Error("handler_rules:unknown_role_mode");
      return { kind: "role", value: handlerRulesFirstArg(node), mode, subject };
    }
    case "role_lead":
    case "role_prefix":
    case "role_padded":
    case "word":
    case "substring":
    case "prefix":
    case "cue_set":
    case "only_characters":
    case "route_exact":
    case "history_role":
    case "prior_turn":
    case "evidence":
    case "operation":
      return { kind: node.name, value: handlerRulesFirstArg(node), subject };
    case "unbalanced_parentheses":
      return { kind: node.name, subject };
    case "shape": {
      const shape = handlerRulesFirstArg(node);
      if (!["digit", "time_separator", "url", "path", "quoted"].includes(shape)) {
        throw new Error("handler_rules:unknown_shape");
      }
      return { kind: "shape", value: shape, subject };
    }
    default:
      throw new Error("handler_rules:unknown_condition");
  }
}

/**
 * Parse one `rule` node (Rust `parser::parse_rule`).
 * @param {{name: string, args: string[], children: object[]}} node
 * @returns {object}
 */
function handlerRulesParseRule(node) {
  const rule = {
    name: handlerRulesFirstArg(node),
    when: null,
    values: [],
    steps: [],
    response: null,
    intent: "",
    link: "",
    confidence: 1.0,
  };
  for (const child of node.children) {
    switch (child.name) {
      case "when":
        rule.when = { kind: "all", children: child.children.map(handlerRulesParseCondition) };
        break;
      case "value":
        rule.values = rule.values.concat([handlerRulesParseValue(child)]);
        break;
      case "log":
        rule.steps = rule.steps.concat([{
          kind: handlerRulesFirstArg(child),
          value: child.args.length > 1 ? child.args[1] : "",
        }]);
        break;
      case "respond": {
        let fallback = "localized";
        const texts = [];
        for (const option of child.children) {
          switch (option.name) {
            case "fallback":
              fallback = handlerRulesFirstArg(option);
              break;
            case "text":
              if (option.args.length < 2) throw new Error("handler_rules:text_without_body");
              texts.push({ language: option.args[0], text: option.args[1] });
              break;
            default:
              throw new Error("handler_rules:unknown_respond_option");
          }
        }
        rule.response = { kind: "seed", intent: handlerRulesFirstArg(child), fallback, texts };
        break;
      }
      case "respond_unknown":
        rule.response = { kind: "unknown" };
        break;
      case "intent":
        rule.intent = handlerRulesFirstArg(child);
        break;
      case "link":
        rule.link = handlerRulesFirstArg(child);
        break;
      case "confidence":
        rule.confidence = Number(handlerRulesFirstArg(child));
        if (!Number.isFinite(rule.confidence)) {
          throw new Error("handler_rules:confidence_not_a_number");
        }
        break;
      default:
        throw new Error("handler_rules:unknown_rule_field");
    }
  }
  if (!rule.when) throw new Error("handler_rules:rule_without_when");
  if (!rule.response) throw new Error("handler_rules:rule_without_respond");
  return rule;
}

/**
 * Parse one `value` node (Rust `parser::parse_value`).
 * @param {{name: string, args: string[]}} node
 * @returns {{name: string, source: string, text: string, key: string}}
 */
function handlerRulesParseValue(node) {
  const name = handlerRulesFirstArg(node);
  if (node.args.length < 2) throw new Error("handler_rules:value_without_source");
  const source = node.args[1];
  switch (source) {
    case "backticks":
    case "trimmed_prompt":
    case "quoted":
    case "network_snapshot":
      return { name, source, text: "", key: "" };
    case "literal":
      if (node.args.length < 3) throw new Error("handler_rules:literal_without_text");
      return { name, source, text: node.args[2], key: "" };
    case "agent_info": {
      if (node.args.length < 3) throw new Error("handler_rules:agent_info_without_key");
      const text = node.args[3] === "default" ? node.args.slice(4).join(" ") : "";
      return { name, source, text, key: node.args[2] };
    }
    case "stable_id":
      if (node.args.length < 3) throw new Error("handler_rules:stable_id_without_prefix");
      return { name, source, text: "", key: node.args[2] };
    case "role_slot":
      if (node.args.length < 3) throw new Error("handler_rules:role_slot_without_role");
      return { name, source, text: "", key: node.args[2] };
    case "operand": // #1175 R3: the operand a claim-evidence reader extracts, by index (Rust ValueSource::Operand)
      if (node.args.length < 3) throw new Error("handler_rules:operand_without_kind");
      return { name, source, text: node.args[3] || "0", key: node.args[2] };
    case "table": { // #918: a row of a `table` block, by containment `of <subject>` or by `key <capture>`
      if (node.args.length < 3) throw new Error("handler_rules:table_without_name");
      const mode = node.args[3];
      if ((mode !== "of" && mode !== "key") || node.args.length < 5) throw new Error("handler_rules:table_without_lookup");
      if (mode === "of" && !["normalized", "cleaned", "lowercase", "prompt", "trimmed", "padded"].includes(node.args[4])) {
        throw new Error("handler_rules:unknown_subject");
      }
      return { name, source, text: node.args[4], key: node.args[2], mode };
    }
    case "response": // #918: the seeded response whose intent the template names, in the prompt language
      if (node.args.length < 3) throw new Error("handler_rules:response_without_intent");
      return { name, source, text: "", key: node.args[2] };
    default:
      throw new Error("handler_rules:unknown_value_source");
  }
}

let cachedHandlerRules = null;

/**
 * Every rule set the seed declares, keyed by precedence name.
 * @returns {Object<string, object[]>}
 */
function handlerRulesDocument() {
  if (cachedHandlerRules) return cachedHandlerRules;
  const text = seedRawText(SEED_RAW, "handler-rules.lino");
  if (!text) return {};
  const roots = handlerRulesParseTree(text);
  const root = roots.find((node) => node.name === "handler_rules");
  if (!root) throw new Error("handler_rules:0:missing_root");
  const handlers = {};
  for (const node of root.children) {
    if (node.name !== "handler") continue;
    handlers[handlerRulesFirstArg(node)] = node.children
      .filter((child) => child.name === "rule")
      .map(handlerRulesParseRule);
  }
  cachedHandlerRules = handlers;
  return cachedHandlerRules;
}

let cachedHandlerRuleTables = null;

/**
 * The `table <name>` blocks of the rule document: ordered `row <key> <value>`
 * pairs and an optional `default` (Rust `parser::tables`).
 * @returns {Object<string, {rows: string[][], fallback: (string|null)}>}
 */
function handlerRulesTables() {
  if (cachedHandlerRuleTables) return cachedHandlerRuleTables;
  const root = handlerRulesParseTree(seedRawText(SEED_RAW, "handler-rules.lino"))
    .find((node) => node.name === "handler_rules");
  const tables = {};
  for (const node of root ? root.children : []) {
    if (node.name !== "table" || node.args.length === 0) continue;
    const table = { rows: [], fallback: null };
    for (const child of node.children) {
      if (child.name === "row" && child.args.length >= 2) table.rows.push([child.args[0], child.args[1]]);
      else if (child.name === "default" && child.args.length > 0) table.fallback = child.args[0];
    }
    tables[node.args[0]] = table;
  }
  cachedHandlerRuleTables = tables;
  return tables;
}

/**
 * The value of the first row of `table <name>` whose key occurs in `text`,
 * or null when none does (Rust `rule_interpreter::handler_table_row`): the
 * reader a native-primitive handler takes its vocabulary tables through.
 * @param {string} name
 * @param {string} text
 * @returns {string|null}
 */
function handlerRulesTableRow(name, text) {
  const table = handlerRulesTables()[name];
  const row = table ? table.rows.find((entry) => entry[0] !== "" && String(text).includes(entry[0])) : null;
  return row ? row[1] : null;
}

/**
 * The value a `policy <handler>` block of the rule document declares for
 * `key` (Rust `rule_interpreter::handler_policy`), or null.
 * @param {string} handler
 * @param {string} key
 * @returns {string|null}
 */
function handlerRulesPolicy(handler, key) {
  const text = seedRawText(SEED_RAW, "handler-rules.lino");
  const root = handlerRulesParseTree(text).find((node) => node.name === "handler_rules");
  const policy = root && root.children.find((node) => node.name === "policy" && node.args[0] === handler);
  const entry = policy && policy.children.find((node) => node.name === key);
  return entry && entry.args.length > 0 ? entry.args[0] : null;
}

/**
 * The surfaces of every meaning carrying `role`, with language and slot
 * (Rust `LinkStoreSource::role_surfaces`).
 * @param {string} role
 * @returns {{text: string, language: string, slot: string}[]}
 */
function handlerRulesRoleSurfaces(role) {
  const out = [];
  for (const meaning of meaningsWithRole(role)) {
    for (const lexeme of meaning.lexemes) {
      for (const text of lexeme.words) {
        if (!text) continue;
        out.push({ text, language: lexeme.language || "", slot: handlerRulesSlotOf(text) });
      }
    }
  }
  return out;
}

/**
 * @param {string} text
 * @returns {string} "bare" | "prefix" | "suffix" | "circumfix"
 */
function handlerRulesSlotOf(text) {
  const index = text.indexOf("…");
  if (index === -1) return "bare";
  const before = text.slice(0, index);
  const after = text.slice(index + 1);
  if (before !== "" && after !== "") return "circumfix";
  if (before !== "") return "prefix";
  if (after !== "") return "suffix";
  return "bare";
}

/**
 * @param {string} character
 * @returns {boolean}
 */
function handlerRulesIsAlphanumeric(character) {
  return /[\p{Alphabetic}\p{N}]/u.test(character);
}

/**
 * Whether `phrase` occurs in `text` as complete words (Rust
 * `phrase_present_as_words`).
 * @param {string} text
 * @param {string} phrase
 * @returns {boolean}
 */
function handlerRulesPhraseAsWords(text, phrase) {
  if (phrase === "") return false;
  if (containsCjk(phrase)) return text.includes(phrase);
  let search = 0;
  let start = text.indexOf(phrase, search);
  while (start !== -1) {
    const end = start + phrase.length;
    const before = Array.from(text.slice(0, start)).pop() || "";
    const after = Array.from(text.slice(end))[0] || "";
    if ((before === "" || !handlerRulesIsAlphanumeric(before)) &&
        (after === "" || !handlerRulesIsAlphanumeric(after))) {
      return true;
    }
    search = end;
    start = text.indexOf(phrase, search);
  }
  return false;
}

/**
 * Whether a seeded role surface is present, slot-aware (Rust
 * `spelled_surface_present`).
 * @param {string} text
 * @param {{text: string, slot: string}} surface
 * @returns {boolean}
 */
function handlerRulesSpelledPresent(text, surface) {
  const expected = surface.text;
  const index = expected.indexOf("…");
  const before = index === -1 ? expected : expected.slice(0, index).trim();
  const after = index === -1 ? "" : expected.slice(index + 1).trim();
  switch (surface.slot) {
    case "bare":
      return surfacePresent(text, expected);
    case "prefix":
      return handlerRulesPhraseAsWords(text, before);
    case "suffix":
      return text.includes(after);
    case "circumfix": {
      if (before === "" || after === "") return false;
      const start = text.indexOf(before);
      if (start === -1) return false;
      const tail = text.slice(start + before.length);
      const offset = tail.indexOf(after);
      return offset !== -1 && tail.slice(0, offset).trim() !== "";
    }
    default:
      return false;
  }
}

/**
 * @param {string} text
 * @param {string} surface a two-word phrasal verb
 * @returns {boolean}
 */
function handlerRulesSeparatedPresent(text, surface) {
  const parts = surface.split(/\s+/u).filter((part) => part !== "");
  if (parts.length !== 2) return false;
  const words = text.split(/\s+/u).filter((word) => word !== "");
  return words.some((word, index) =>
    word === parts[0] && words.slice(index + 2, index + 2 + 6).includes(parts[1]));
}

/**
 * Structural shape of a subject text (Rust `Shape::holds`).
 * @param {string} shape
 * @param {string} text
 * @returns {boolean}
 */
function handlerRulesShapeHolds(shape, text) {
  const isUrl = (token) => ["http://", "https://", "www."].some((prefix) => token.startsWith(prefix));
  const tokens = text.split(/\s+/u).filter((token) => token !== "");
  switch (shape) {
    case "digit":
      return /\p{N}/u.test(text);
    case "time_separator":
      return text.includes(":") || text.includes("：");
    case "url":
      return tokens.some((token) => isUrl(token.toLowerCase()));
    case "path":
      return tokens.some((token) =>
        !isUrl(token.toLowerCase()) && (token.includes("/") || token.includes("\\")));
    case "quoted": {
      const pairs = [['"', '"'], ["'", "'"], ["«", "»"], ["“", "”"],
        ["「", "」"], ["‘", "’"]];
      return pairs.some((pair) => {
        if (pair[0] === pair[1]) return text.split(pair[0]).length - 1 >= 2;
        const start = text.indexOf(pair[0]);
        return start !== -1 && text.slice(start + 1).includes(pair[1]);
      });
    }
    default:
      return false;
  }
}

/**
 * Evaluate one condition against the rule context (Rust `Condition::holds`).
 * @param {object} condition
 * @param {object} context
 * @returns {boolean}
 */
function handlerRulesHolds(condition, context) {
  const text = context.subjects[condition.subject] || "";
  switch (condition.kind) {
    case "all":
      return condition.children.every((child) => handlerRulesHolds(child, context));
    case "any":
      return condition.children.some((child) => handlerRulesHolds(child, context));
    case "none":
      return !condition.children.some((child) => handlerRulesHolds(child, context));
    case "role": {
      const surfaces = handlerRulesRoleSurfaces(condition.value);
      switch (condition.mode) {
        case "spelled":
          return surfaces.some((surface) => handlerRulesSpelledPresent(text, surface));
        case "raw":
        case "forms":
          return surfaces.some((surface) => surface.text !== "" && text.includes(surface.text));
        case "languages":
          return surfaces.some((surface) =>
            context.languages.includes(surface.language) && text.includes(surface.text));
        case "separated":
          return surfaces.some((surface) => handlerRulesSeparatedPresent(text, surface.text));
        case "whole":
          return text !== "" && surfaces.some((surface) => surface.text === text);
        default:
          return false;
      }
    }
    case "role_lead":
      return handlerRulesRoleSurfaces(condition.value).some((surface) =>
        surface.slot === "prefix"
          ? text.startsWith(surface.text.split("…")[0])
          : text.includes(surface.text));
    case "role_prefix":
      return handlerRulesRoleSurfaces(condition.value).some((surface) =>
        surface.slot === "prefix" && text.startsWith(surface.text.split("…")[0]));
    case "role_padded": {
      const padded = ` ${text} `;
      return handlerRulesRoleSurfaces(condition.value).some((surface) =>
        surface.text.startsWith(" ") || surface.text.endsWith(" ")
          ? padded.includes(surface.text)
          : text.includes(surface.text));
    }
    case "word":
      return text.split(/\s+/u).includes(condition.value);
    case "substring":
      return text.includes(condition.value);
    case "prefix":
      return text.startsWith(condition.value);
    case "only_characters": {
      const trimmed = context.prompt.trim();
      return trimmed !== "" && Array.from(trimmed).every((character) =>
        condition.value.includes(character));
    }
    case "unbalanced_parentheses": {
      const opens = context.prompt.split("(").length - 1;
      const closes = context.prompt.split(")").length - 1;
      return opens !== closes;
    }
    case "history_role": {
      const surfaces = handlerRulesRoleSurfaces(condition.value);
      return context.history.some((turn) => {
        const payload = String((turn && (turn.content || turn.text)) || "").toLowerCase();
        return surfaces.some((surface) => surface.text !== "" && payload.includes(surface.text));
      });
    }
    case "prior_turn": // #1175 R3: an earlier turn of this role (`any` for either) exists (Rust Condition::PriorTurn)
      return context.history.some((turn) => turn && (turn.content || turn.text) && (condition.value === "any" || turn.role === condition.value));
    case "shape":
      return handlerRulesShapeHolds(condition.value, text);
    case "operation": // #918: the seeded operation vocabulary requests it (Rust Condition::Operation)
      return operationMatchesSlug(condition.value, text);
    case "evidence": {
      // The claim-evidence kind the capability table admits a handler on
      // (Rust `capability_routing::claim_evidence_holds_in_dialogue`), so a
      // rule's refusal lane reads the same reader its admission does.
      const reader = typeof CLAIM_EVIDENCE === "object" ? CLAIM_EVIDENCE[condition.value] : null;
      return typeof reader === "function"
        && Boolean(reader(context.prompt, context.subjects.normalized, context.history));
    }
    case "cue_set": {
      // `cue_lexicon::matches`, read only when the caller carries the cue sets (the server's intent record,
      // data/meta/cue-lexicon.lino); the rule sets this browser module runs keep their dedicated handlers.
      const set = context.cueSets ? context.cueSets[condition.value] : null;
      const token = (cue) => (/[\u3040-\u30ff\u3400-\u9fff\uac00-\ud7af]/u.test(cue) ? text.includes(cue) : text.split(/\s+/).includes(cue));
      return Boolean(set) && set.cues.some((cue) => (set.match === "prefix" ? text.startsWith(cue) : set.match === "substring" ? text.includes(cue) : token(cue)));
    }
    case "route_exact":
      // The exact-route backend is not read by the rule sets this browser module runs.
      return false;
    default:
      return false;
  }
}

/**
 * The seed response text for exactly `language` (Rust `seed::response_for`).
 * @param {string} intent
 * @param {string} language
 * @returns {string|null}
 */
function handlerRulesResponseFor(intent, language) {
  const table = MULTILINGUAL_ANSWERS[intent];
  if (!table || !table[language]) return null;
  const raw = table[language];
  return typeof raw === "string" ? raw : (raw.text || null);
}

/**
 * @param {string} text
 * @param {{name: string, value: string}[]} values
 * @returns {string}
 */
function handlerRulesSubstitute(text, values) {
  let rendered = text;
  for (const value of values) rendered = rendered.split(`{${value.name}}`).join(value.value);
  return rendered;
}

/**
 * Render a rule's response (Rust `Response::render`).
 * @param {object} response
 * @param {object} context
 * @param {{name: string, value: string}[]} values
 * @returns {string|null}
 */
function handlerRulesRender(response, context, values) {
  if (response.kind === "unknown") {
    return unknownAnswerWithVariation(context.prompt, context.language);
  }
  const inline = (language) => {
    const entry = response.texts.find((candidate) => candidate.language === language);
    return entry ? handlerRulesSubstitute(entry.text, values) : null;
  };
  const exact = (language) => {
    const text = handlerRulesResponseFor(response.intent, language);
    return text === null ? null : handlerRulesSubstitute(text, values);
  };
  let rendered = inline(context.language);
  if (rendered === null) {
    switch (response.fallback) {
      case "exact":
        rendered = exact(context.language);
        break;
      case "localized":
        rendered = exact(context.language);
        if (rendered === null) rendered = exact("unknown");
        if (rendered === null) rendered = exact("en");
        break;
      default:
        rendered = exact(context.language);
        if (rendered === null) rendered = exact(response.fallback);
        break;
    }
  }
  if (rendered === null) rendered = inline("en");
  return rendered;
}

/**
 * Resolve a rule's captured values; null when a capture is missing.
 * @param {object} rule
 * @param {object} context
 * @returns {{name: string, value: string}[]|null}
 */
function handlerRulesResolveValues(rule, context) {
  const resolved = [];
  for (const value of rule.values) {
    let text = "";
    switch (value.source) {
      case "backticks": {
        const parts = context.prompt.split("`");
        const term = parts.length > 1 ? parts[1].trim() : "";
        if (term === "") return null;
        text = term;
        break;
      }
      case "agent_info":
        text = AGENT_INFO[value.key] !== undefined ? String(AGENT_INFO[value.key]) : value.text;
        break;
      case "literal":
        text = value.text;
        break;
      case "stable_id":
        text = stableBehaviorRuleId(value.key, context.prompt);
        break;
      case "role_slot": {
        const slot = handlerRulesRoleSlot(value.key, context);
        if (slot === null) return null;
        text = slot;
        break;
      }
      case "quoted": {
        const quoted = handlerRulesQuotedPhrase(context.prompt);
        if (quoted === null) return null;
        text = quoted;
        break;
      }
      case "network_snapshot":
        text = networkSnapshotLinksNotation();
        break;
      case "operand": {
        const reader = typeof CLAIM_OPERANDS === "object" ? CLAIM_OPERANDS[value.key] : null;
        const operands = typeof reader === "function" ? reader(context.prompt) : [];
        const operand = operands[Number(value.text)];
        if (operand === undefined) return null;
        text = operand;
        break;
      }
      case "table": { // Rust `values::lookup_table`
        const table = handlerRulesTables()[value.key];
        if (!table) return null;
        let row;
        if (value.mode === "of") {
          const subject = context.subjects[value.text] || "";
          row = table.rows.find((entry) => entry[0] !== "" && subject.includes(entry[0]));
        } else {
          const capture = resolved.find((entry) => entry.name === value.text);
          if (!capture) return null;
          row = table.rows.find((entry) => entry[0] === capture.value);
        }
        if (row) text = row[1];
        else if (table.fallback !== null) text = table.fallback;
        else return null;
        break;
      }
      case "response": { // Rust `seed::localized_response` under the templated intent
        const intent = handlerRulesSubstitute(value.key, resolved);
        let found = handlerRulesResponseFor(intent, context.language);
        if (found === null) found = handlerRulesResponseFor(intent, "unknown");
        if (found === null) found = handlerRulesResponseFor(intent, "en");
        if (found === null) return null;
        text = found;
        break;
      }
      default:
        text = context.prompt.trim();
        break;
    }
    resolved.push({ name: value.name, value: text });
  }
  return resolved;
}

/**
 * The first quoted phrase of the prompt, trying the quote pairs in the order
 * the native `extract_quoted_phrase` does; null without one.
 * @param {string} prompt
 * @returns {string|null}
 */
function handlerRulesQuotedPhrase(prompt) {
  for (const [open, close] of [["'", "'"], ['"', '"'], ["`", "`"], ["«", "»"]]) {
    const start = prompt.indexOf(open);
    if (start === -1) continue;
    const end = prompt.indexOf(close, start + open.length);
    if (end !== -1) return prompt.slice(start + open.length, end);
  }
  return null;
}

/**
 * Marks a captured slot sheds at its edges besides whitespace: the quotation
 * and sentence punctuation a request wraps its object in (Rust
 * `SLOT_EDGE_MARKS`).
 */
const HANDLER_RULES_SLOT_EDGE_MARKS = "`\"':-_.,?!";

/**
 * The text filling the open slot of a role's prefix surface (Rust
 * `values::role_slot`): the first surface whose lead opens the normalized
 * subject, else the first `scan` surface whose lead occurs anywhere in the
 * lowercased prompt. Null when no surface opens the subject or the slot is
 * empty once its edge marks are shed.
 * @param {string} role
 * @param {object} context
 * @returns {string|null}
 */
function handlerRulesRoleSlot(role, context) {
  const forms = roleWordForms(role);
  const opening = context.subjects.normalized;
  const lowercase = context.subjects.lowercase;
  let slot = null;
  const opener = forms.find((form) => opening.startsWith(form.before));
  if (opener) {
    slot = opening.slice(opener.before.length);
  } else {
    for (const form of forms) {
      if (form.action !== "scan") continue;
      const index = lowercase.indexOf(form.before);
      if (index >= 0) {
        slot = lowercase.slice(index + form.before.length);
        break;
      }
    }
  }
  if (slot === null) return null;
  const characters = Array.from(slot);
  const shed = (character) => /\s/u.test(character) || HANDLER_RULES_SLOT_EDGE_MARKS.includes(character);
  while (characters.length > 0 && shed(characters[0])) characters.shift();
  while (characters.length > 0 && shed(characters[characters.length - 1])) characters.pop();
  return characters.length > 0 ? characters.join("") : null;
}

/**
 * Whether any rule behind `name` accepts the prompt, without answering (Rust
 * `rule_interpreter::handler_claims`): the claim-evidence probe a migrated
 * handler's recognition answers through.
 * @param {string} name
 * @param {string} prompt
 * @param {string} normalized
 * @returns {boolean}
 */
function handlerRuleSetClaims(name, prompt, normalized) {
  return runHandlerRuleSet(name, prompt, normalized, []) !== null;
}

/**
 * Run the seed rule set behind a precedence name the way the native
 * interpreter does (Rust `rule_interpreter::run_handler`).
 * @param {string} name
 * @param {string} prompt
 * @param {string} normalized
 * @param {object[]} history
 * @returns {object|null}
 */
function runHandlerRuleSet(name, prompt, normalized, history) {
  const rules = handlerRulesDocument()[name];
  if (!Array.isArray(rules)) return null;
  const source = String(prompt || "");
  const normalizedText = String(normalized || "");
  const language = detectLanguage(source);
  const context = {
    prompt: source,
    language,
    languages: language === "en" ? ["en"] : [language, "en"],
    history: Array.isArray(history) ? history : [],
    subjects: {
      normalized: normalizedText,
      cleaned: normalizePrompt(normalizedText),
      lowercase: source.toLowerCase(),
      prompt: source,
      trimmed: normalizedText.trim(),
      padded: ` ${normalizedText} `,
    },
  };
  for (const rule of rules) {
    if (!handlerRulesHolds(rule.when, context)) continue;
    const values = handlerRulesResolveValues(rule, context);
    if (values === null) continue;
    const evidence = [`handler:${name}`, `rule_interpreter:rule:${rule.name}`];
    for (const step of rule.steps) {
      let payload = step.value;
      if (step.value === "$prompt") payload = source;
      else if (step.value === "$trimmed") payload = source.trim();
      else if (step.value.startsWith("$")) {
        const capture = values.find((value) => value.name === step.value.slice(1));
        if (!capture) return null;
        payload = capture.value;
      }
      evidence.push(`${step.kind}:${payload}`);
    }
    const content = handlerRulesRender(rule.response, context, values);
    if (content === null) return null;
    // An intent may name a captured value (`concept_introspection_{concept}`).
    const intent = handlerRulesSubstitute(rule.intent || rule.name, values);
    evidence.push(rule.link || `response:${intent}`);
    evidence.push(`language:${language}`);
    return { intent, content, confidence: rule.confidence, evidence };
  }
  return null;
}

/**
 * `clarification` precedence row: the seeded clarification-request role.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryClarification(prompt, normalized, history = []) {
  return runHandlerRuleSet("clarification", prompt, normalized, history);
}

/**
 * `punctuation_only_prompt` precedence row.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryPunctuationOnlyPrompt(prompt, normalized) {
  return runHandlerRuleSet("punctuation_only_prompt", prompt, normalized, []);
}

/**
 * `ill_formed` precedence row: an unbalanced "teach this fact" link.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryIllFormed(prompt, normalized) {
  return runHandlerRuleSet("ill_formed", prompt, normalized, []);
}

/**
 * `physical_action_question` precedence row.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryPhysicalActionQuestion(prompt, normalized) {
  return runHandlerRuleSet("physical_action_question", prompt, normalized, []);
}

/**
 * `opinion_question` precedence row: the no-opinion policy.
 * @param {string} prompt
 * @param {string} normalized
 * @returns {object|null}
 */
function tryOpinionQuestion(prompt, normalized) {
  return runHandlerRuleSet("opinion_question", prompt, normalized, []);
}
