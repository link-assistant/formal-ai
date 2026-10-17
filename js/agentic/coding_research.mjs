import { tapVerificationDecision } from './portable/coding_research_kernel.mjs';
// Bounded request-local research after ordinary module synthesis has missed.
// Captured code is a proposal; only the existing execution route observes effects.
import { Capability } from './capability.mjs';
import { agenticMessage } from './messages.mjs';
import { toolFor } from './capability_router.mjs';
import { Progress } from './progress.mjs';
import { realm, validateProgram } from './host.mjs';
import { moduleFunctionRequest } from './module_function.mjs';
import { observeSourceCallables } from './module_function/callable_catalog.mjs';
import { tokenizeWithSpans, isKeyword } from './crate/es_tokenizer.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { urlsIn } from './web_research.mjs';
import { planSymbolicCommandReroute } from './command_reroute.mjs';
import { FinalDisposition, resolvedFinalAnswer, planOne, jsonText, fetchArguments } from './plan.mjs';
import { readArguments } from './workspace_change.mjs';
import { commandArgument, incompleteReceipt, normalizedPayload, stepOutcome, StepOutcome } from './tool_result.mjs';

const MAXIMUM_PAGES = 3;
const MAXIMUM_PAGE_BYTES = 131072;
const safeName = name => typeof name === 'string' && /^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(name)
  && !isKeyword(name) && !['eval', 'arguments'].includes(name);
const gap = (reason, evidence = {}) => {
  const discovery = { reason, authored: false, verified: false, ...evidence };
  const plan = resolvedFinalAnswer(agenticMessage('coding-research-gap', { reason }),
    FinalDisposition.Gap, 'coding-research-gap');
  plan.result.discovery = discovery;
  return plan;
};

/** Query needs come from the typed declaration and bounded unquoted specification. */
export function codingResearchQuery(request) {
  let clause = request.clause;
  for (const quoted of quotedSegmentSpans(clause).reverse()) {
    clause = clause.slice(0, quoted.start) + ' ' + clause.slice(quoted.end);
  }
  clause = clause.replace(/https?:\/\/\S+|\S*[/\\.]\S*|[A-Za-z_$][\w$]*\s*\([^()]*\)/gu, ' ');
  const operands = new Set([request.name, ...request.parameters]);
  const terms = [...new Set((clause.match(/[A-Za-z][A-Za-z-]{2,}/gu) ?? [])
    .map(word => word.toLowerCase()).filter(word => !operands.has(word)))].slice(0, 8);
  return [request.language, 'function', ...terms].join(' ');
}

function leaves(trees) {
  return trees.flatMap(tree => tree.$ === 'group' ? leaves(tree.trees) : tree.$ === 'leaf' ? [tree] : []);
}

/** Rebind declaration and parameter identities at observed token spans, never string contents. */
export function deriveCodingProcedure(request, page) {
  const proposals = [];
  for (const block of page.codeBlocks) {
    if (block.language !== request.language) continue;
    const source = block.text;
    if (validateProgram(request.language, source) !== true) continue;
    const catalog = observeSourceCallables(source, request.module);
    if (catalog.moduleEffects !== 'none' || catalog.moduleSyntax !== 'supported'
      || catalog.imports.length || catalog.declarations.length !== 1) continue;
    const declaration = catalog.declarations[0];
    if (declaration.contract.status !== 'supported' || declaration.contract.callEffects !== 'none'
      || declaration.parameters.length !== request.parameters.length) continue;
    const bindings = new Map([[declaration.name, request.name],
      ...declaration.parameters.map((name, at) => [name, request.parameters[at]])]);
    if (bindings.size !== declaration.parameters.length + 1) continue;
    const bytes = new TextEncoder().encode(declaration.source);
    const edits = leaves(tokenizeWithSpans(declaration.source))
      .filter(token => token.kind === 'identifier' && bindings.has(token.text))
      .map(token => ({ start: token.span.start, end: token.span.end, replacement: bindings.get(token.text) }))
      .sort((left, right) => right.start - left.start);
    let rendered = bytes;
    for (const edit of edits) {
      const value = new TextEncoder().encode(edit.replacement);
      rendered = new Uint8Array([...rendered.slice(0, edit.start), ...value, ...rendered.slice(edit.end)]);
    }
    let program = new TextDecoder().decode(rendered);
    if (!catalog.exports.some(item => item.local === declaration.name)) program = 'export ' + program;
    if (validateProgram(request.language, program) !== true) continue;
    proposals.push({ source: program + (program.endsWith('\n') ? '' : '\n'),
      provenance: { url: page.url, sha256: page.sha256, query: page.query,
        declaration: declaration.identity, contract: declaration.contract,
        bindings: [...bindings].map(([from, to]) => ({ from, to })),
        status: 'proposal', verified: false } });
  }
  return proposals.length === 1 ? proposals[0] : null;
}

/** Query existing token trees for a finite independent assertion contract, not source readback. */
export function independentAcceptance(request, source) {
  if (validateProgram('javascript', source) !== true
    || request.command !== 'node --test ' + request.test) return false;
  let trees;
  try { trees = tokenizeWithSpans(source); } catch { return false; }
  const text = tree => tree?.$ === 'leaf' ? tree.text : '';
  const statements = [];
  let statement = [];
  for (const tree of trees) {
    if (text(tree) === ';') { if (statement.length) statements.push(statement); statement = []; }
    else statement.push(tree);
  }
  if (statement.length) statements.push(statement);
  const moduleURL = './' + request.module.split('/').at(-1);
  const targetDirectory = request.module.split('/').slice(0, -1).join('/');
  if (request.test.split('/').slice(0, -1).join('/') !== targetDirectory) return false;
  let callable = false, assertions = 0, assertionImport = false;
  const literal = tree => tree?.kind === 'numeric' || tree?.kind === 'string'
    || ['true', 'false', 'null'].includes(text(tree));
  const argumentTokens = request.parameters.length === 0 ? 0 : request.parameters.length * 2 - 1;
  const commaList = values => values.length === argumentTokens && values.every((value, at) =>
    at % 2 === 1 ? text(value) === ',' : literal(value));
  for (const entry of statements) {
    if (text(entry[0]) === 'import' && entry.length === 4 && text(entry[2]) === 'from') {
      const specifier = text(entry[3]);
      if (entry[1]?.$ === 'group' && entry[1].delim === 'brace'
        && entry[1].trees.length === 1 && text(entry[1].trees[0]) === request.name
        && [JSON.stringify(moduleURL), "'" + moduleURL + "'"].includes(specifier)) { callable = true; continue; }
      if (text(entry[1]) === 'assert' && ['"node:assert/strict"', "'node:assert/strict'"].includes(specifier)) {
        assertionImport = true; continue;
      }
      return false;
    }
    if (entry.length !== 4 || text(entry[0]) !== 'assert' || text(entry[1]) !== '.'
      || !['equal', 'strictEqual', 'deepEqual', 'deepStrictEqual'].includes(text(entry[2]))
      || entry[3]?.$ !== 'group' || entry[3].delim !== 'paren') return false;
    const callArguments = entry[3].trees;
    if (callArguments.length !== 4 || text(callArguments[0]) !== request.name || callArguments[1]?.$ !== 'group'
      || callArguments[1].delim !== 'paren' || !commaList(callArguments[1].trees)
      || text(callArguments[2]) !== ',' || !literal(callArguments[3])) return false;
    assertions += 1;
  }
  return callable && assertionImport && assertions > 0;
}

/** Repairs on this bounded route retain the same pure declaration/effect contract. */
function pureResearchProgram(request, source) {
  if (validateProgram(request.language, source) !== true) return false;
  const catalog = observeSourceCallables(source, request.module);
  const declaration = catalog.declarations[0];
  return catalog.moduleEffects === 'none' && catalog.moduleSyntax === 'supported'
    && catalog.declarations.length === 1 && catalog.imports.length === 0
    && declaration.name === request.name && declaration.contract.callEffects === 'none'
    && declaration.contract.status === 'supported'
    && JSON.stringify(declaration.parameters) === JSON.stringify(request.parameters)
    && catalog.exports.length === 1 && catalog.exports[0].local === request.name
    && catalog.exports[0].exposed === request.name;
}

/** Known Node test verification needs actual nonempty TAP evidence, never bare exit zero. */
export function verificationReceiptGap(output) {
  const complete = !incompleteReceipt(output) && stepOutcome(output) === StepOutcome.Succeeded;
  const payload = normalizedPayload(output) ?? '';
  const counts = {};
  let unique = true;
  for (const field of ['tests', 'pass', 'fail', 'skipped', 'cancelled']) {
    const matches = [...payload.matchAll(new RegExp('^# ' + field + ' ([0-9]+)\\r?$', 'gmu'))];
    if (matches.length > 1) unique = false;
    counts[field] = matches.length ? Number(matches[0][1]) : ['skipped', 'cancelled'].includes(field) ? 0 : -1;
  }
  return tapVerificationDecision(complete && unique, counts.tests, counts.pass,
    counts.fail, counts.skipped, counts.cancelled) || null;
}

/** A real coding miss supplies the typed need; searches and fetches stay advertised tools. */
export function planCodingResearchStep(task, messages, toolNames) {
  const request = moduleFunctionRequest(task, { requireTestContract: false });
  if (!request) return null;
  if (!['javascript', 'typescript'].includes(request.language)) return gap('UnsupportedGrammar');
  if (validateProgram(request.language, '') !== true) return gap('GrammarUnavailable');
  if (![request.name, ...request.parameters].every(safeName)
    || new Set(request.parameters).size !== request.parameters.length
    || request.parameters.includes(request.name)) return gap('UnsupportedBinding');
  if (!request.command || !request.test) return gap('IndependentVerificationRequired');
  const progress = Progress.scan(messages);
  const readTool = toolFor(toolNames, Capability.Read);
  const moduleRead = progress.sourceReadFor(request.module);
  const testRead = progress.sourceReadFor(request.test);
  for (const [path, read] of [[request.module, moduleRead], [request.test, testRead]]) {
    if (!read) return readTool ? planOne(readTool, readArguments(path)) : gap('ReadToolMissing');
    if (!read.complete && !(path === request.module && read.absent)) {
      return gap('SourceReadIncomplete', { path });
    }
  }
  if (testRead.error || !testRead.source?.trim()) return gap('IndependentVerificationMissing');
  if (!independentAcceptance(request, testRead.source)) return gap('IndependentVerificationUnproved');
  const query = codingResearchQuery(request);
  const searchTool = toolFor(toolNames, Capability.Search);
  const search = [...progress.attempts].reverse().find(attempt => {
    if (attempt.capability !== Capability.Search || !attempt.succeeded || !attempt.arguments) return false;
    try {
      const callArguments = JSON.parse(attempt.arguments);
      return callArguments.query === query && Object.keys(callArguments).length === 1;
    } catch { return false; }
  });
  if (!search) return searchTool ? planOne(searchTool, jsonText({ query })) : gap('SearchToolMissing');
  const searchText = normalizedPayload(search.detail) ?? '';
  if (new TextEncoder().encode(searchText).length > MAXIMUM_PAGE_BYTES) return gap('SearchResultTooLarge');
  const urls = [...new Set(urlsIn(searchText))]
    .filter(url => { try { return new URL(url).protocol === 'https:'; } catch { return false; } }).slice(0, MAXIMUM_PAGES);
  const formalize = realm()?.formalizedPageFromCapture;
  if (typeof formalize !== 'function') return gap('PageFormalizerMissing');
  const proposals = [];
  for (const [url, text] of progress.fetched_pages) {
    if (!urls.includes(url) || new TextEncoder().encode(text).length > MAXIMUM_PAGE_BYTES) continue;
    const capture = { url, text, sha256: sha256Hex(text), fetchedAt: '', cached: false };
    const page = formalize(capture, query, urls.indexOf(url) + 1, 0, null);
    const proposal = deriveCodingProcedure(request, page);
    if (proposal) proposals.push(proposal);
  }
  if (proposals.length > 1) return gap('AmbiguousSourceProcedure');
  if (!proposals.length) {
    const url = urls.find(candidate => !progress.attempted_fetches.includes(candidate));
    const fetchTool = toolFor(toolNames, Capability.Fetch);
    if (url && fetchTool) return planOne(fetchTool, fetchArguments(url));
    return gap('NoValidatedSourceProcedure', { query, attempted: progress.attempted_fetches.length });
  }
  const proposal = proposals[0];
  if (moduleRead.source?.trim()) {
    if (moduleRead.source !== proposal.source) return gap('ExistingModuleRequiresComposition');
    let lastRead = -1;
    for (let at = 0; at < progress.attempts.length; at++) {
      const attempt = progress.attempts[at];
      if (attempt.capability === Capability.Read) lastRead = at;
    }
    const observation = progress.attempts.slice(lastRead + 1).findLast(attempt =>
      attempt.capability === Capability.Run && attempt.arguments
      && commandArgument(attempt.arguments) === request.command);
    if (!observation) {
      const runTool = toolFor(toolNames, Capability.Run);
      return runTool ? planOne(runTool, jsonText({ command: request.command })) : gap('ExecutionToolsMissing');
    }
    const outcome = observation.succeeded ? stepOutcome(observation.detail) : StepOutcome.Failed;
    if (outcome === StepOutcome.Unreported) return gap('VerificationReceiptIncomplete');
    const succeeded = outcome === StepOutcome.Succeeded;
    const receiptGap = succeeded ? verificationReceiptGap(observation.detail) : null;
    if (receiptGap) return gap(receiptGap, proposal.provenance);
    const plan = resolvedFinalAnswer(succeeded
      ? agenticMessage('coding-research-verified-existing')
      : agenticMessage('coding-research-existing-failed', { detail: observation.detail }),
    succeeded ? FinalDisposition.Finding : FinalDisposition.Failure,
    succeeded ? 'coding-research-verified-existing' : 'coding-research-existing-verification-failed');
    plan.research = { ...proposal.provenance, status: succeeded ? 'verified' : 'proposal',
      verified: succeeded, sourceAuthored: false, acceptanceSha256: sha256Hex(testRead.source) };
    return plan;
  }
  const latestSource = progress.successfulWriteContentFor(request.module);
  if (latestSource !== null && !pureResearchProgram(request, latestSource)) return gap('UnsupportedRepairSource');
  const recipe = { language: request.language, path: request.module, source: proposal.source,
    supporting_files: [], commands: [request.command] };
  const plan = planSymbolicCommandReroute(messages, toolNames, { execution_recipe: recipe });
  if (!plan) return gap('ExecutionToolsMissing', proposal.provenance);
  for (const call of plan.calls ?? []) {
    if (call.tool !== toolFor(toolNames, Capability.Write)) continue;
    let callArguments;
    try { callArguments = JSON.parse(call.arguments); } catch { return gap('UnsupportedRepairSource'); }
    if ((callArguments.path ?? callArguments.filePath ?? callArguments.file_path) === request.module
      && !pureResearchProgram(request, callArguments.content)) return gap('UnsupportedRepairSource');
  }
  const writtenAt = progress.latestSuccessfulWriteIndex(request.module);
  const observation = writtenAt === null ? null : progress.attempts.slice(writtenAt + 1).findLast(attempt =>
    attempt.capability === Capability.Run && attempt.arguments
    && commandArgument(attempt.arguments) === request.command);
  if (plan.kind === 'final') {
    const outcome = observation === null ? StepOutcome.Unreported
      : observation.succeeded ? stepOutcome(observation.detail) : StepOutcome.Failed;
    if (outcome === StepOutcome.Unreported) return gap('VerificationReceiptIncomplete', proposal.provenance);
    const receiptGap = outcome === StepOutcome.Succeeded ? verificationReceiptGap(observation.detail) : null;
    if (receiptGap) return gap(receiptGap, proposal.provenance);
    const verified = outcome === StepOutcome.Succeeded;
    const final = resolvedFinalAnswer(plan.answer,
      verified ? FinalDisposition.Finding : FinalDisposition.Failure,
      verified ? 'coding-research-verified-created' : 'coding-research-verification-failed');
    final.research = { ...proposal.provenance, status: verified ? 'verified' : 'proposal',
      verified, acceptanceSha256: sha256Hex(testRead.source) };
    return final;
  }
  plan.research = { ...proposal.provenance, acceptanceSha256: sha256Hex(testRead.source) };
  // Command reroute owns retries and effect observations. Discovery never grants success.
  return plan;
}
