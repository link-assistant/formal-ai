// The `network_snapshot` value of the `network_query` rules of
// data/seed/handler-rules.lino (issue #918; first ported in PR #1188 for the
// issue #1175 routing probes). The native engine renders its whole knowledge
// network (knowledge_links_notation in rust/src/engine.rs); the browser
// renders, in the same Links Notation record shape, only what it has actually
// loaded: the agent-info header, a concept index and one rule record per
// seeded rule intent whose response the worker carries. The rule intents are
// the `network_snapshot_record` cue records of data/seed/code-task-cues.lino.
// The request cues, the concept introspection, the user filter and their
// wording are the rules and their seeded responses, walked by the shared
// rule interpreter exactly as the native one walks them.

/**
 * Quote one value the way format_lino_value does: sanitize line breaks and
 * backslashes, then pick a delimiter the value does not carry (a quote is
 * doubled when it carries both).
 * @param {string} value
 * @returns {string}
 */
function networkSnapshotValue(value) {
  const flat = String(value)
    .replace(/\\/g, "\\\\")
    .replace(/\r/g, "\\r")
    .replace(/\n/g, "\\n")
    .replace(/\t/g, "\\t");
  const hasSingle = flat.includes("'");
  const hasDouble = flat.includes('"');
  if (hasDouble && !hasSingle) return `'${flat}'`;
  if (hasSingle && !hasDouble) return `"${flat}"`;
  if (hasSingle && hasDouble) return `'${flat.replace(/'/g, "''")}'`;
  return `"${flat}"`;
}

/**
 * One flat two-level record, as format_lino_record writes it.
 * @param {string} id
 * @param {Array<[string, string]>} pairs
 * @returns {string}
 */
function networkSnapshotRecord(id, pairs) {
  return [id, ...pairs.map(([key, value]) => `  ${key} ${networkSnapshotValue(value)}`)].join("\n");
}

/**
 * The seed response file that declares `intent`, or "".
 * @param {string} intent
 * @returns {string}
 */
function networkSnapshotSeedFile(intent) {
  const declaration = new RegExp(`^    intent ${intent}$`, "m");
  const files = Object.entries(SEED_RAW || {})
    .map(([path, text]) => [seedFileBaseName(path), String(text || "")])
    .filter(([name]) => name.startsWith("multilingual-responses"))
    .sort(([left], [right]) => left.localeCompare(right));
  const found = files.find(([, text]) => declaration.test(text));
  return found ? found[0] : "";
}

/**
 * The browser's loaded knowledge network as Links Notation records, in the
 * order and shape of knowledge_links_notation: header, concept index, rules.
 * A rule intent with no seeded response of its own is left out, so the
 * snapshot never claims a record the browser does not hold.
 * @returns {string}
 */
function networkSnapshotLinksNotation() {
  const intents = codeTaskCuePhrases("network_snapshot_record", "rule")
    .filter((intent) => Boolean(MULTILINGUAL_ANSWERS[intent]));
  const header = [
    ["model", AGENT_INFO.name],
    ["version", AGENT_INFO.version],
    ["policy", AGENT_INFO.tagline],
  ].filter(([, value]) => value);
  header.push(["rule_count", String(intents.length)]);
  const records = [
    networkSnapshotRecord("formal_ai_knowledge", header),
    networkSnapshotRecord("concept_index", intents.map((intent) => [intent, `intent: ${intent}`])),
  ];
  for (const intent of intents) {
    const pairs = [
      ["intent", intent],
      ["response_link", `response:${intent}`],
      ["answer", answerFor(intent, "en")],
    ];
    const source = networkSnapshotSeedFile(intent);
    if (source) pairs.push(["source", source]);
    records.push(networkSnapshotRecord(`rule_${intent}`, pairs));
  }
  return records.join("\n\n");
}

/**
 * `network_query` precedence row: the `network_query` rules over the
 * lowercased prompt, the subject the native dispatcher hands every rule.
 * @param {string} prompt
 * @returns {object|null}
 */
function tryNetworkSnapshot(prompt) {
  return runHandlerRuleSet("network_query", prompt, String(prompt || "").toLowerCase(), []);
}
