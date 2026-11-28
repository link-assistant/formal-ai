// WikiLambda supplies function labels, abstract implementations, and executable
// examples. Decode its integer expression subset without running publisher code.
const RECURRENCE_OPERANDS = Object.freeze({
  conditional: 3, less_equal: 2, equal: 2, add: 2, multiply: 2, subtract: 2, subtract_one: 1,
});

function recurrenceSourceReference(value) {
  return typeof value === "string" && /^Z[1-9]\d{0,9}$/u.test(value);
}

function recurrenceInteger(value) {
  const token = typeof value === "string" ? value : value?.Z13518K1 ?? value?.Z6K1;
  return typeof token === "string" && /^-?\d{1,16}$/u.test(token)
    && Number.isSafeInteger(Number(token)) ? Number(token) : null;
}

function recurrenceNode(name, value = "", children = []) {
  return { name, value: String(value), children };
}

function recurrenceSourceExpression(source, target, key, operatorIds, descriptors, depth = 0, budget = { nodes: 128 }) {
  if (depth >= 24 || --budget.nodes < 0) return null;
  if (source?.Z1K1 === "Z18") return source.Z18K1 === key ? recurrenceNode("parameter", "n") : null;
  if (source?.Z1K1 !== "Z7") {
    const integer = recurrenceInteger(source);
    return integer === null ? null : recurrenceNode("literal", integer);
  }
  const zid = source.Z7K1;
  if (!recurrenceSourceReference(zid)) return null;
  operatorIds.add(zid);
  const descriptor = descriptors?.get(zid);
  if (descriptors && zid !== target && !descriptor) return null;
  const operation = zid === target ? "recur" : descriptor?.operation;
  const keys = zid === target ? [key]
    : descriptor?.keys ?? Object.keys(source).filter(name => name !== "Z1K1" && name !== "Z7K1");
  if (keys.length > 3 || (descriptors && keys.length !== (zid === target ? 1 : RECURRENCE_OPERANDS[operation]))) return null;
  const children = keys.map(argument => recurrenceSourceExpression(
    source[argument], target, key, operatorIds, descriptors, depth + 1, budget));
  return children.includes(null) ? null : recurrenceNode(operation ?? "", "", children);
}

function recurrenceOperatorDescriptor(object) {
  const labels = object?.Z2K3?.Z12K1;
  if (!Array.isArray(labels)) return null;
  const forms = labels.map(label => normalizePrompt(label?.Z11K2 || ""));
  const matched = meaningsWithRole("coding_recurrence_operator").find(meaning =>
    forms.some(label => meaningEvidencedIn(meaning, label)));
  const keys = recurrenceArgumentKeys(object?.Z2K2);
  const operation = matched?.wordForms.find(form => Object.hasOwn(RECURRENCE_OPERANDS, form.action))?.action;
  return operation && keys.length === RECURRENCE_OPERANDS[operation] ? { keys, operation } : null;
}

function recurrenceArgumentKeys(body) {
  return (Array.isArray(body?.Z8K1) ? body.Z8K1 : [])
    .map(argument => argument?.Z17K2).filter(key => typeof key === "string");
}

function recurrencePredecessors(root) {
  if (root?.name !== "conditional") return null;
  const [condition, boundary, transition] = root.children;
  const [left, right] = condition.children;
  if (!["less_equal", "equal"].includes(condition.name) || left?.name !== "parameter"
      || right?.name !== "literal") return null;
  const lastBoundary = Number(right.value);
  if (lastBoundary < 0 || (condition.name === "equal" && lastBoundary !== 0)) return null;
  const scan = (node, offsets) => {
    if (node.name !== "recur") return node.children.every(child => scan(child, offsets));
    if (!offsets) return false;
    const argument = node.children[0];
    const [parameter, constant] = argument.children;
    if (parameter?.name !== "parameter") return false;
    const offset = argument.name === "subtract_one" ? 1
      : argument.name === "subtract" && constant?.name === "literal" ? Number(constant.value) : 0;
    if (offset <= 0 || offset > lastBoundary + 1) return false;
    offsets.add(offset);
    return true;
  };
  const offsets = new Set();
  return scan(boundary, null) && scan(transition, offsets) && offsets.size
    ? Array.from(offsets).sort((left, right) => left - right) : null;
}

async function recurrenceCapture(url, preferences) {
  const capture = await sourceWalkFetchCapture(url, { online: preferences.offline !== true, bounded: true });
  if (!capture.ok) return null;
  try {
    return { capture, value: JSON.parse(capture.text) };
  } catch {
    return null;
  }
}

async function recurrenceObjects(api, ids, preferences) {
  if (!ids.length || !ids.every(recurrenceSourceReference)) return null;
  const result = await recurrenceCapture(
    api.replace("{zids}", sourceWalkPercentEncode(ids.join("|")))
      .replace("{language}", "en") + "&origin=*", preferences);
  if (!result) return null;
  const objects = new Map();
  for (const identifier of ids) {
    const encoded = result.value?.[identifier]?.wikilambda_fetch;
    if (typeof encoded !== "string") continue;
    try {
      const object = JSON.parse(encoded);
      if ((object?.Z2K1?.Z6K1 ?? object?.Z2K1) === identifier) objects.set(identifier, object);
    } catch {
      continue;
    }
  }
  return { capture: result.capture, objects };
}

function recurrenceSourceExamples(testers, zid, key) {
  const examples = [];
  for (const object of testers.objects.values()) {
    const body = object?.Z2K2;
    const call = body?.Z20K2;
    if (body?.Z20K1 !== zid || call?.Z7K1 !== zid) continue;
    const argument = recurrenceInteger(call[key]);
    const expected = recurrenceInteger(body?.Z20K3?.Z13522K2);
    if (argument !== null && argument >= 0 && argument <= 20 && expected !== null) {
      examples.push(recurrenceNode("source_test", "", [
        recurrenceNode("argument", argument), recurrenceNode("expected", expected),
      ]));
    }
  }
  return examples.slice(0, 4);
}

async function recurrenceReadAbstract(api, candidate, preferences) {
  const definition = await recurrenceObjects(api, [candidate.page_title], preferences);
  const zid = candidate.page_title;
  const object = definition?.objects.get(zid);
  const body = object?.Z2K2;
  const keys = recurrenceArgumentKeys(body);
  if (body?.Z1K1 !== "Z8" || keys.length !== 1
      || !body.Z8K1.some(argument => argument?.Z17K2 === keys[0] && argument.Z17K1 === "Z13518")
      || body.Z8K2 !== "Z13518" || !Array.isArray(body.Z8K4) || !Array.isArray(body.Z8K3)) return null;
  const [key] = keys;
  const implementations = await recurrenceObjects(api,
    body.Z8K4.filter(identifier => identifier !== "Z14" && recurrenceSourceReference(identifier)).slice(0, 8), preferences);
  if (!implementations) return null;
  for (const [identifier, implementation] of implementations.objects) {
    const raw = implementation?.Z2K2;
    if (raw?.Z14K1 !== zid || !raw.Z14K2) continue;
    const operators = new Set();
    if (!recurrenceSourceExpression(raw.Z14K2, zid, key, operators) || operators.size > 8) continue;
    operators.delete(zid);
    const definitions = await recurrenceObjects(api, [...operators].sort(), preferences);
    if (!definitions) continue;
    const descriptors = new Map(Array.from(definitions.objects, ([reference, node]) => [reference, recurrenceOperatorDescriptor(node)]));
    const root = recurrenceSourceExpression(raw.Z14K2, zid, key, operators, descriptors);
    const offsets = recurrencePredecessors(root);
    if (!offsets) continue;
    const testers = await recurrenceObjects(api,
      body.Z8K3.filter(identifier => identifier !== "Z20" && recurrenceSourceReference(identifier)).slice(0, 8), preferences);
    if (!testers) continue;
    const examples = recurrenceSourceExamples(testers, zid, key);
    if (!examples.length) continue;
    const label = object?.Z2K3?.Z12K1?.find(item => typeof item?.Z11K2 === "string")?.Z11K2 ?? candidate.label;
    const name = (label.split(/[^\p{L}\p{N}]+/u).find(word => /^\p{Lu}/u.test(word)) ?? label)
      .toLowerCase().replace(/[^a-z0-9]+/gu, "_").replace(/^_+|_+$/gu, "");
    if (!/^[a-z_][a-z0-9_]*$/u.test(name)) continue;
    return recurrenceNode("recurrence", zid, [
      recurrenceNode("identifier", name), recurrenceNode("parameter", "n"),
      recurrenceNode("expression", "", [root]), recurrenceNode("termination_measure", "n"),
      recurrenceNode("source_url", implementations.capture.url),
      recurrenceNode("source_sha256", implementations.capture.sha256),
      recurrenceNode("source_fetched_at", implementations.capture.fetchedAt),
      recurrenceNode("source_test_url", testers.capture.url),
      recurrenceNode("source_test_sha256", testers.capture.sha256),
      recurrenceNode("definition_url", definition.capture.url),
      recurrenceNode("definition_sha256", definition.capture.sha256),
      recurrenceNode("search_url", candidate.search.url),
      recurrenceNode("search_sha256", candidate.search.sha256),
      ...candidate.bridge.flatMap(capture => [recurrenceNode("concept_source_url", capture.url),
        recurrenceNode("concept_source_sha256", capture.sha256)]),
      recurrenceNode("implementation", identifier),
      ...offsets.map(offset => recurrenceNode("predecessor_offset", offset)),
      ...examples,
    ]);
  }
  return null;
}
