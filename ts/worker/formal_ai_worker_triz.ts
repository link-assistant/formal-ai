// Derive a TRIZ worksheet from the caller's demands, as in rust/src/triz_solver.rs.

let cachedTrizRules = null;

function trizChildValue(record, name) {
  return record.children.find((child) => child.name === name)?.value || "";
}

function trizRules() {
  if (cachedTrizRules) return cachedTrizRules;
  const records = parseLinoTree(seedRawText(SEED_RAW, "triz-principles.lino")).children;
  const families = records
    .filter((row) => trizChildValue(row, "record_type") === "triz_resolution_family")
    .map((row) => ({
      methodId: trizChildValue(row, "method_id").trim(),
      name: trizChildValue(row, "name"),
      mechanism: trizChildValue(row, "mechanism"),
    }))
    .filter((family) => family.methodId !== "");
  const grammar = records.find((row) => row.name === "triz_cues");
  const phrases = (name) => (grammar?.children ?? [])
    .filter((child) => child.name === name && child.value.trim())
    .map((child) => child.value);
  const alternation = (values) => values.map((value) =>
    value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")).join("|");
  const cues = phrases("phrase");
  cachedTrizRules = {
    families,
    cues,
    cue: new RegExp(alternation(cues), "iu"),
    pair: new RegExp(alternation(phrases("separator")), "iu"),
    prefix: new RegExp(`^(?:${alternation(phrases("prefix"))})`, "iu"),
    balancedPrefix: new RegExp(`^(?:${alternation(phrases("balanced_prefix"))})`, "iu"),
    balancedSeparator: new RegExp(alternation(phrases("balanced_separator")), "iu"),
  };
  return cachedTrizRules;
}

function trizFamilies() {
  return trizRules().families;
}

function trizDemand(text) {
  const trimmed = text.replace(/^[\s:：?？.。!！]+|[\s:：?？.。!！]+$/gu, "");
  return /[\p{Alphabetic}\p{N}]/u.test(trimmed) ? trimmed : null;
}

function trizDemands(prompt) {
  const rules = trizRules();
  const cue = rules.cue.exec(prompt);
  if (!cue) return null;
  const tail = prompt.slice(cue.index + cue[0].length).trimStart();
  const tailFrame = tail.replace(/^[:：]\s*/u, "").replace(rules.prefix, "");
  const balanced = rules.balancedPrefix.exec(tailFrame);
  const frame = balanced ? tailFrame.slice(balanced[0].length) : tailFrame;
  const separator = (balanced ? rules.balancedSeparator : rules.pair).exec(frame);
  if (!separator) return null;
  const left = trizDemand(frame.slice(0, separator.index));
  const right = trizDemand(frame.slice(separator.index + separator[0].length));
  if (!left || !right || left.toLowerCase() === right.toLowerCase()) return null;
  return [left, right];
}

function tryTrizResolution(prompt, normalized) {
  const rules = trizRules();
  if (!rules.cues.length || !(rules.cue.test(prompt) || rules.cue.test(normalized))) return null;
  const language = detectLanguage(prompt);
  const evidence = ["handler:triz_resolution", "triz_solver:cued:contradiction question"];
  const demands = trizDemands(prompt);
  if (!demands) {
    evidence.push("triz_resolution:refusal:two demands required", "response:triz_resolution_missing_demands");
    return {
      intent: "triz_resolution",
      content: answerFor("triz_resolution_missing_demands", language),
      confidence: 0.45,
      evidence,
    };
  }
  const [left, right] = demands;
  const families = trizFamilies().map((family) => `- ${family.name}: ${family.mechanism}`).join("\n");
  const values = { left, right, families };
  const content = answerFor("triz_resolution_map", language)
    .replace(/\{(left|right|families)\}/gu, (_, name) => values[name]);
  evidence.push(`triz_solver:demands:${left} / ${right}`, "response:triz_resolution_map");
  return { intent: "triz_resolution", content, confidence: 0.7, evidence };
}
