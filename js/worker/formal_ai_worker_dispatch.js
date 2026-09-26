// Synchronous browser-handler bindings used by the universal dispatcher.
//
// Membership, arguments and first-match-wins order live in
// data/seed/browser-handler-precedence.lino. The JavaScript side only resolves
// those seed records to executable functions. This is the same separation the
// native dispatcher uses: data owns behaviour; code supplies implementations.
let BROWSER_HANDLER_PRECEDENCE = [];

function handlerContextValue(context, path) {
  return String(path || "").split(".").reduce((value, part) => {
    return value == null ? undefined : value[part];
  }, context);
}

function handlerImplementation(record, context) {
  if (record.contextBinding) {
    const contextual = handlerContextValue(context, record.contextBinding);
    if (typeof contextual !== "function") {
      throw new Error(`browser handler ${record.name} needs context binding ${record.contextBinding}`);
    }
    return contextual;
  }
  const implementation = self[record.name];
  if (typeof implementation !== "function") {
    throw new Error(`browser handler ${record.name} has no executable implementation`);
  }
  return implementation;
}

function handlerResultMatches(record, hit) {
  if (!hit) return false;
  if (record.resultIntent && hit.intent !== record.resultIntent) return false;
  if (!record.evidenceKind) return true;
  return Array.isArray(hit.evidence) && hit.evidence.some((link) => {
    return link === record.evidenceKind || link.startsWith(`${record.evidenceKind}:`);
  });
}

function validateBrowserHandlerPrecedence(registry) {
  if (!Array.isArray(registry) || registry.length === 0) {
    throw new Error("browser handler precedence seed is missing or empty");
  }
  const names = new Set();
  for (const record of registry) {
    if (!record || !record.name || names.has(record.name)) {
      throw new Error(`browser handler precedence has an invalid or duplicate name: ${record && record.name}`);
    }
    names.add(record.name);
    if (!record.contextBinding && typeof self[record.name] !== "function") {
      throw new Error(`browser handler ${record.name} has no executable implementation`);
    }
  }
}

function installBrowserHandlerPrecedence(registry) {
  validateBrowserHandlerPrecedence(registry);
  BROWSER_HANDLER_PRECEDENCE = registry.map((record) => Object.freeze({
    name: record.name,
    arguments: Object.freeze(Array.isArray(record.arguments) ? record.arguments.slice() : []),
    contextBinding: record.contextBinding || "",
    resultIntent: record.resultIntent || "",
    evidenceKind: record.evidenceKind || "",
  }));
}

function browserHandlerPrecedence() {
  return BROWSER_HANDLER_PRECEDENCE.slice();
}

function synchronousHandlerCandidates(context, registry = browserHandlerPrecedence()) {
  return registry.map((record) => ({
    name: record.name,
    run: () => {
      const implementation = handlerImplementation(record, context);
      const args = (record.arguments || []).map((path) => handlerContextValue(context, path));
      const hit = implementation(...args);
      return handlerResultMatches(record, hit) ? hit : null;
    },
  }));
}
