// Captured values of a data-owned handler rule, in the browser worker.
//
// The twin of rust/src/rule_interpreter/values.rs: the subjects a condition
// reads, the `value` sources a rule captures, and the named text transforms a
// `value <name> transform <transform>` source applies to the free-text payload
// of the request (the text after a command colon, a line break or inside
// quotes, as textTransformFreeTextPayload reads it for summarization). The
// rule walk itself stays in formal_ai_worker_handler_rules.js.

/**
 * The texts a rule condition may read `of <subject>` (Rust `Context::new`).
 * `command_head` is the command head: the request before its free-text payload, so a
 * cue inside the pasted text never decides the route.
 * @param {string} source the raw prompt
 * @param {string} normalized the normalized prompt
 * @returns {Object<string, string>}
 */
function handlerRulesSubjects(source, normalized) {
  return {
    normalized,
    cleaned: normalizePrompt(normalized),
    lowercase: source.toLowerCase(),
    prompt: source,
    trimmed: normalized.trim(),
    padded: ` ${normalized} `,
    command_head: normalizePrompt(textTransformCommandHead(source)),
  };
}

/**
 * Every subject name a condition may read: the keys of handlerRulesSubjects.
 */
const HANDLER_RULES_SUBJECT_NAMES = Object.freeze([
  "normalized",
  "cleaned",
  "lowercase",
  "prompt",
  "trimmed",
  "padded",
  "command_head",
]);

/**
 * The requirements the text states, one Markdown list item per line; null when
 * it states none (Rust `values::requirement_list`).
 * @param {string} text
 * @returns {string|null}
 */
function requirementListTransform(text) {
  const requirements = extractRequirements(text);
  if (requirements.length === 0) return null;
  return requirements.map((requirement) => `- ${requirement}`).join("\n");
}

/**
 * The named pure text transforms a rule can apply to the request's free-text
 * payload (Rust `values::TEXT_TRANSFORMS`). Each maps the payload to the
 * captured text, or to null when it has nothing to say about it.
 */
const HANDLER_RULES_TEXT_TRANSFORMS = Object.freeze({
  requirement_list: requirementListTransform,
});

/**
 * The text transform `name` applied to the free-text payload of `prompt`; null
 * when the transform is unknown, the prompt carries no payload, or the
 * transform declines it (Rust `values::transform_payload`).
 * @param {string} name
 * @param {string} prompt
 * @returns {string|null}
 */
function handlerRulesTransformPayload(name, prompt) {
  const transform = HANDLER_RULES_TEXT_TRANSFORMS[name];
  if (typeof transform !== "function") return null;
  const payload = textTransformFreeTextPayload(prompt);
  return payload === null ? null : transform(payload);
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
      case "transform": {
        const transformed = handlerRulesTransformPayload(value.key, context.prompt);
        if (transformed === null) return null;
        text = transformed;
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
