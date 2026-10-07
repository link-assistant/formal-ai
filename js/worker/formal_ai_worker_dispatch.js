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
  if (!hit || (record.hoisted && metaIsImpasseIntent(hit.intent))) return false;
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

function synchronousHandlerCandidates(context, registry = promotedHandlerOrder(browserHandlerPrecedence(), context.prompt)) {
  return registry.map((record) => ({
    name: record.name,
    run: () => {
      const hit = claimRouteRun(record.name, context.prompt, context.normalized, context.history, () =>
        handlerImplementation(record, context)(...(record.arguments || []).map((path) => handlerContextValue(context, path))));
      return handlerResultMatches(record, hit) ? hit : null;
    },
  }));
}

// Claim routing (issue #1175 R3), twin of rust/src/capability_routing/claims.rs: the `claim` rows of
// data/seed/capability-routing.lino name a handler's browser function and the evidence kinds any one of
// which admits it; a handler whose row admits on none is never offered the prompt.
let CLAIM_ROUTE_ROWS = null;
const CLAIM_EVIDENCE = Object.freeze({
  object_phrase_artifact: (prompt) => detectSoftwareObjectPhrase(normalizePrompt(prompt)) !== null,
  approval_of_a_proposal: (prompt) => isSoftwareApprovalPrompt(normalizePrompt(prompt)),
  shell_command_shape: (prompt) => detectTerminalCommand(prompt) !== null,
  semantic_shell_task: (prompt) => detectSemanticShellCommand(prompt) !== null,
  repository_subject: () => false,
  supplied_page: (prompt) => pageSplitSuppliedPage(prompt) !== null,
  javascript_program: (prompt) => extractJavaScriptProgram(prompt) !== null,
  incompatible_unit_pair: (prompt, normalized) => detectIncompatibleUnitPair(normalized) !== null,
  fetch_url: (prompt, normalized) => extractHttpFetchUrl(prompt, normalized) !== null,
  navigation_url: (prompt, normalized) => extractUrlNavigateUrl(prompt, normalized) !== null,
  calendar_date_signal: (prompt, normalized) => mentionsCalendarCreateRequest(normalized),
  code_artifact: (prompt) => codeTaskCodeBlock(prompt) !== null, supplied_text: (prompt) => textTransformFreeTextPayload(prompt) !== null, ...(typeof NUMERIC_CLAIM_EVIDENCE === "object" ? NUMERIC_CLAIM_EVIDENCE : {}),
  ...(typeof CLASS_CLAIM_EVIDENCE === "object" ? CLASS_CLAIM_EVIDENCE : {}), // classes (b), (c), (e): formal_ai_worker_claim_evidence.js
  // Refusal group: the operand each handler's own reader extracts before it composes anything.
  function_under_test: (prompt) => testGenerationFunctionName(prompt) !== null,
  structured_document: (prompt) => formatConversionJsonText(prompt) !== null || formatConversionYamlText(prompt) !== null
    || formatConversionCsvText(prompt) !== null,
});

function claimRouteRows() {
  if (CLAIM_ROUTE_ROWS !== null) return CLAIM_ROUTE_ROWS;
  const text = typeof SEED_RAW === "object" && SEED_RAW ? seedRawText(SEED_RAW, "capability-routing.lino") : "";
  const values = (record, name) => record.children.filter((child) => child.name === name && child.value).map((child) => child.value);
  const records = text ? parseLinoTree(text).children.flatMap((document) => document.children) : [];
  const rows = records.filter((record) => record.name === "claim").map((record) => ({
    handler: values(record, "handler")[0] || "", browserHandler: values(record, "browser_handler")[0] || "", admitsOn: values(record, "admits_on"),
    refusalEvents: values(record, "refusal_event"),
  }));
  if (text) CLAIM_ROUTE_ROWS = rows;
  return rows;
}

// "full" when the row's evidence holds (or there is no row), "refusal" when it does not but the row names a
// refusal event (the handler may only refuse), "denied" otherwise (Rust `claim_admission_in_dialogue`); the
// dialogue kinds read the earlier turns from `history`.
function claimRouteAdmission(browserHandler, prompt, normalized, history = []) {
  const row = claimRouteRows().find((candidate) => candidate.browserHandler === browserHandler);
  if (!row || row.admitsOn.some((kind) => Boolean(CLAIM_EVIDENCE[kind] && CLAIM_EVIDENCE[kind](prompt, normalized ?? normalizePrompt(prompt), history)))) return "full";
  return row.refusalEvents.length > 0 ? "refusal" : "denied";
}

function claimRouteAdmits(browserHandler, prompt, normalized, history = []) {
  return claimRouteAdmission(browserHandler, prompt, normalized, history) !== "denied";
}

// Run a handler through its claim row: null when denied, and in the refusal lane its answer only when it
// recorded the row's refusal event. `run` may return a promise (the asynchronous lookups).
function claimRouteRun(browserHandler, prompt, normalized, history, run) {
  const admission = claimRouteAdmission(browserHandler, prompt, normalized, history);
  if (admission === "denied") return null;
  const keep = (hit) => (admission === "refusal" && !claimRouteRefused(browserHandler, hit) ? null : hit);
  const hit = run();
  return hit && typeof hit.then === "function" ? hit.then(keep) : keep(hit);
}

// Whether a refusal-lane answer may stand: the handler recorded one of its row's refusal events (Rust `refusal_recorded`).
function claimRouteRefused(browserHandler, hit) {
  const row = claimRouteRows().find((candidate) => candidate.browserHandler === browserHandler);
  const evidence = (hit && Array.isArray(hit.evidence)) ? hit.evidence : [];
  return Boolean(row) && evidence.some((entry) => row.refusalEvents.some((kind) => entry === kind || String(entry).startsWith(`${kind}:`)));
}
