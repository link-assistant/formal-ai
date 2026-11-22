// Source-owned callable body contracts and structural signature binding.
// Recurrence cache matching, expression rendering and evaluation remain shared.
function readFormalParameterPositions(expression, formalParameters) {
  if (!Array.isArray(formalParameters) || formalParameters.some((name) => typeof name !== 'string' || !name) || new Set(formalParameters).size !== formalParameters.length) return null;
  const positions = new Map(formalParameters.map((name, position) => [name, position]));
  const reads = new Set();
  const visit = (node) => {
    if (!node || typeof node.name !== 'string' || !Array.isArray(node.children)) return false;
    if (node.name === 'parameter') {
      if (node.children.length !== 0 || !positions.has(node.value)) return false;
      reads.add(positions.get(node.value));
      return true;
    }
    if (node.name === 'literal') return node.children.length === 0;
    return node.children.every(visit);
  };
  return visit(expression) ? [...reads].sort((left, right) => left - right) : null;
}

function bindCallableReadCoverage(contract, taskSignature) {
  if (!contract || !taskSignature || !['provisional', 'declared', 'observed'].includes(taskSignature.origin)) return null;
  const readPositions = readFormalParameterPositions(contract.expression, contract.parameters);
  if (readPositions === null || typeof contract.name !== 'string' || !contract.name) return null;
  const declared = taskSignature.origin === 'declared';
  const parameters = declared ? taskSignature.parameters : contract.parameters;
  const name = taskSignature.origin === 'provisional' ? contract.name : taskSignature.name;
  if (taskSignature.origin === 'observed' && taskSignature.arity !== contract.parameters.length) return null;
  if (!Array.isArray(parameters) || parameters.length !== contract.parameters.length || parameters.some((parameter) => typeof parameter !== 'string' || !parameter) || new Set(parameters).size !== parameters.length || typeof name !== 'string' || !name) return null;
  const callsSelf = (node) => node.name === 'recur' || node.children.some(callsSelf);
  if (parameters.includes(name) && callsSelf(contract.expression)) return null;
  const names = new Map(contract.parameters.map((parameter, position) => [parameter, parameters[position]]));
  const bind = (node) => ({ ...node, value: node.name === 'parameter' ? names.get(node.value) : node.value, children: node.children.map(bind) });
  return { name, parameters: [...parameters], expression: bind(contract.expression), readParameterPositions: readPositions, referencedParameters: readPositions.length };
}

function sourceCallableRequestSignature(prompt) {
  const text = String(prompt || "");
  const name = extractPythonFunctionName(text);
  if (!name) return { origin: "provisional" };
  const declaration = new RegExp("(?:^|\\n)\\s*def\\s+" + name + "\\s*\\(([^()\\n]*)\\)\\s*(?:->\\s*([^:\\n]+))?", "u").exec(text);
  const inline = declaration || new RegExp("\\b" + name + "\\s*\\(([^()\\n]*)\\)", "u").exec(text);
  if (!inline) return null;
  const parts = [];
  let start = 0;
  let depth = 0;
  for (let position = 0; position < inline[1].length; position += 1) {
    const character = inline[1][position];
    if ("[{".includes(character)) depth += 1;
    if ("]}".includes(character)) depth -= 1;
    if (character === "," && depth === 0) {
      parts.push(inline[1].slice(start, position).trim());
      start = position + 1;
    }
  }
  if (inline[1].trim()) parts.push(inline[1].slice(start).trim());
  if (!declaration && new RegExp("\\bassert\\s+" + name + "\\s*\\(", "u").test(text)) {
    return { origin: "observed", name, arity: parts.length };
  }
  const parameters = parts.map(part => part.split(":")[0].trim());
  if (depth !== 0 || parts.some(part => part.includes("=") || part.startsWith("*") || part === "/") || parameters.some(parameter => !/^[A-Za-z_][A-Za-z0-9_]*$/u.test(parameter))) return null;
  const suffix = declaration && declaration[2] ? " -> " + declaration[2].trim() : "";
  return { origin: "declared", name, parameters, declaration: "def " + name + "(" + inline[1] + ")" + suffix + ":" };
}

function trySourceRecurrenceSynthesis(prompt, record) {
  if (!record || !verifiedRecurrenceRecord(record)) return null;
  const signature = sourceCallableRequestSignature(prompt);
  const contract = bindCallableReadCoverage({
    name: codingChildValue(record, "identifier"),
    parameters: [codingChildValue(record, "parameter")],
    expression: recurrenceExpressionNode(record),
  }, signature);
  if (!contract) return null;
  const functionName = contract.name;
  const parameter = contract.parameters[0];
  const root = contract.expression;
  const expression = renderRecurrenceExpression(root, functionName);
  if (!expression) return null;
  const observation = recurrenceObservation(prompt);
  if (observation === null) return null;
  const result = evaluateRecurrenceExpression(
    root,
    root,
    parameter,
    BigInt(observation),
    0,
  );
  if (result === null) return null;
  const declaration = signature.origin === "declared" ? signature.declaration : `def ${functionName}(${parameter}):`;
  const imports = String(prompt || "").split("\n").map(line => line.trim()).filter(line => /^(?:import\s+|from\s+\S+\s+import\s+)/u.test(line));
  const code = `${imports.length ? imports.join("\n") + "\n\n" : ""}${declaration}\n    return ${expression}\n\nprint(${functionName}(${observation}))`;
  const sourceUrl = codingChildValue(record, "source_url");
  const sources = [sourceUrl];
  const provenance = [];
  for (const name of ["source", "definition", "source_test", "search", "concept_source"]) {
    const urls = record.children.filter(node => node.name === name + "_url");
    const digests = record.children.filter(node => node.name === name + "_sha256");
    for (const [index, url] of urls.entries()) {
      if (url.value && !sources.includes(url.value)) sources.push(url.value);
      if (url.value && digests[index]?.value) provenance.push(
        "source_capture:" + url.value + "#sha256=" + digests[index].value);
    }
  }
  const tests = record.children.filter((child) => child.name === "source_test");
  return {
    intent: "write_program",
    content: [
      `Discovered and source-validated recurrence for \`${functionName}\`.`,
      "",
      "```python",
      code,
      "```",
      "",
      `Result for ${functionName}(${observation}): ${result}`,
      "",
      `How it works: the publisher abstract definition was formalized into a boundary condition and a transition whose ${codingChildValue(record, "termination_measure")} measure strictly decreases on every recursive call.`,
      `Validated against ${tests.length} source-provided tests. Sources: ${sources.join("; ")}.`,
    ].join("\n"),
    confidence: 1.0,
    evidence: [
      `synthesis:source_recurrence:${record.value}`,
      `synthesis:callable_binding_origin:${signature.origin}`,
      `synthesis:formal_parameter_reads:${contract.readParameterPositions.join(",")}`,
      `synthesis:termination_offsets:${record.children
        .filter((child) => child.name === "predecessor_offset")
        .map((child) => child.value)
        .join(",")}`,
      `synthesis:source_tests:passed=${tests.length}`,
      ...sources.map(url => `source:${url}`),
      ...provenance,
      "execution_environment:pure browser evaluation of source-derived recurrence AST",
      `execution_result:${result}`,
    ],
    trace: [
      `synthesis:source_recurrence:${record.value}`,
      `synthesis:source_tests:passed=${tests.length}`,
      `execution_result:${result}`,
    ],
  };
}
