// Bounded coding research: shared canonical decisions, client-owned execution effects.
import { tapVerificationDecision } from './portable/coding_research_kernel.mjs';
import { codingResearchQueryKernel } from './portable/coding_research_query.mjs';
import { codingResearchRebindingKernel, canonicalUniqueSourceIndex } from './portable/coding_research_rebinding.mjs';
import { independentAcceptanceKernel } from './portable/coding_research_acceptance.mjs';
import * as research from './portable/coding_research_state.mjs';
import { observeCodingResearchRebindingCandidate, observeCodingResearchAcceptanceTokens } from './coding_research_observation.mjs';
import { Capability } from './capability.mjs';
import { agenticMessage } from './messages.mjs';
import { toolFor } from './capability_router.mjs';
import { Progress } from './progress.mjs';
import { realm, validateProgram } from './host.mjs';
import { moduleFunctionRequest } from './module_function.mjs';
import { observeSourceCallables } from './module_function/callable_catalog.mjs';
import { isKeyword } from './crate/es_tokenizer.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { urlsIn } from './web_research.mjs';
import { planSymbolicCommandReroute } from './command_reroute.mjs';
import { FinalDisposition, resolvedFinalAnswer, planOne, jsonText, fetchArguments } from './plan.mjs';
import { readArguments } from './workspace_change.mjs';
import { commandArgument, incompleteReceipt, normalizedPayload, stepOutcome, StepOutcome } from './tool_result.mjs';
const gap = (reason, evidence = {}) => {
  const plan = resolvedFinalAnswer(agenticMessage('coding-research-gap', { reason }),
    FinalDisposition.Gap, 'coding-research-gap');
  plan.result.discovery = { reason, authored: false, verified: false, ...evidence };
  return plan;
};
/** Typed query decision delegates to the same generated kernel as the native route. */
export function codingResearchQuery(request) {
  let clause = request.clause;
  for (const quoted of quotedSegmentSpans(clause).reverse()) {
    clause = clause.slice(0, quoted.start) + ' ' + clause.slice(quoted.end);
  }
  return codingResearchQueryKernel(request.language, Array.from(clause), request.name, request.parameters);
}
/** Catalog/token observations carry no acceptance; canonical kernels bind and select. */
export function deriveCodingProcedure(request, page) {
  const proposals = [];
  for (const block of page.codeBlocks) {
    if (!research.codingResearchLanguageEligible(request.language, block.language)) continue;
    const observation = observeCodingResearchRebindingCandidate(block.text, block.language, request.module);
    const rendered = codingResearchRebindingKernel(observation, request.language, request.name, request.parameters);
    if (rendered.$ !== 'rendered' || validateProgram(request.language, rendered.source) !== true) continue;
    const declaration = observeSourceCallables(block.text, request.module).declarations[0];
    proposals.push({ source: rendered.source, provenance: { url: page.url, sha256: page.sha256, query: page.query,
      declaration: declaration.identity, contract: declaration.contract,
      bindings: rendered.bindingsFrom.map((from, index) => ({ from, to: rendered.bindingsTo[index] })),
      status: 'proposal', verified: false } });
  }
  const selected = canonicalUniqueSourceIndex(proposals.map(() => true));
  return selected >= 0 ? proposals[selected] : null;
}
/** Independent grammar is shared; this adapter observes parser and tokenizer outputs. */
export function independentAcceptance(request, source) {
  const observed = observeCodingResearchAcceptanceTokens(source);
  return independentAcceptanceKernel(observed.texts, observed.kinds, request.name,
    './' + request.module.split('/').at(-1), request.module.split('/').slice(0, -1).join('/'),
    request.test?.split('/').slice(0, -1).join('/') ?? '', request.test ?? '', request.command ?? '',
    request.parameters.length, observed.complete && validateProgram('javascript', source) === true);
}
function pureResearchProgram(request, source) {
  const observed = observeCodingResearchRebindingCandidate(source, request.language, request.module);
  const catalog = observeSourceCallables(source, request.module);
  return research.codingResearchPureRepairKernel(observed.syntaxValid, observed.moduleEffectsNone,
    observed.moduleSyntaxSupported, observed.contractSupported, observed.callEffectsNone,
    observed.declarationsCount, observed.importsCount, observed.name, observed.parameters,
    catalog.exports.map(item => item.local), catalog.exports.map(item => item.exposed), request.name, request.parameters);
}
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
function observedAttempts(progress) {
  const records = progress.attempts.map(attempt => {
    let value;
    try { value = JSON.parse(attempt.arguments); } catch { value = null; }
    const valid = value !== null && typeof value === 'object' && !Array.isArray(value);
    return { capability: attempt.capability, succeeded: attempt.succeeded, valid,
      query: typeof value?.query === 'string' ? value.query : '', keys: valid ? Object.keys(value).length : -1,
      command: commandArgument(attempt.arguments ?? '') ?? '' };
  });
  return { capabilities: records.map(row => row.capability), succeeded: records.map(row => row.succeeded),
    valid: records.map(row => row.valid), queries: records.map(row => row.query), keys: records.map(row => row.keys),
    commands: records.map(row => row.command) };
}
function outcome(observation) {
  return !observation ? 'missing' : !observation.succeeded ? 'failed' : stepOutcome(observation.detail);
}
function decidedGap(decision, request, evidence = {}) {
  const [reason, path] = decision.slice(4).split(':');
  return gap(reason, path ? { ...evidence, path: request[path] } : evidence);
}
/** Native and JavaScript routes share the actual preflight/discovery/effect state decisions. */
export function planCodingResearchStep(task, messages, toolNames) {
  const request = moduleFunctionRequest(task, { requireTestContract: false });
  if (!request) return null;
  const progress = Progress.scan(messages);
  const readTool = toolFor(toolNames, Capability.Read), searchTool = toolFor(toolNames, Capability.Search);
  const fetchTool = toolFor(toolNames, Capability.Fetch), runTool = toolFor(toolNames, Capability.Run);
  const moduleRead = progress.sourceReadFor(request.module), testRead = progress.sourceReadFor(request.test);
  const names = [request.name, ...request.parameters];
  const preflight = research.codingResearchPreflightKernel(request.language,
    validateProgram(request.language, '') === true,
    research.codingResearchBindingsKernel(names, names.map(name => Array.from(name)), names.map(isKeyword)),
    request.name, request.parameters, Boolean(request.command), Boolean(request.test), Boolean(moduleRead),
    Boolean(moduleRead?.complete), Boolean(moduleRead?.absent), Boolean(testRead), Boolean(testRead?.complete),
    Boolean(testRead?.error), Boolean(testRead?.source?.trim()),
    Boolean(testRead?.source && independentAcceptance(request, testRead.source)), Boolean(readTool));
  if (preflight.startsWith('gap:')) return decidedGap(preflight, request);
  if (preflight.startsWith('read:')) return planOne(readTool, readArguments(request[preflight.slice(5)]));
  const query = codingResearchQuery(request), attempts = observedAttempts(progress);
  const searchIndex = research.codingResearchSearchIndex(attempts.capabilities, attempts.succeeded,
    attempts.valid, attempts.queries, attempts.keys, query);
  const searchDecision = research.codingResearchSearchDecision(searchIndex >= 0, Boolean(searchTool));
  if (searchDecision.startsWith('gap:')) return decidedGap(searchDecision, request);
  if (searchDecision === 'search') return planOne(searchTool, jsonText({ query }));
  const searchText = normalizedPayload(progress.attempts[searchIndex].detail) ?? '';
  const formalize = realm()?.formalizedPageFromCapture;
  const captureDecision = research.codingResearchCaptureDecision(new TextEncoder().encode(searchText).length,
    typeof formalize === 'function');
  if (captureDecision.startsWith('gap:')) return decidedGap(captureDecision, request);
  const foundURLs = urlsIn(searchText);
  const secure = foundURLs.map(url => { try { return new URL(url).protocol === 'https:'; } catch { return false; } });
  const urls = research.codingResearchSourceURLs(foundURLs, secure);
  const proposals = [];
  for (const [url, text] of progress.fetched_pages) {
    const index = research.codingResearchSourceIndex(urls, url);
    if (!research.codingResearchPageEligible(index, new TextEncoder().encode(text).length)) continue;
    const capture = { url, text, sha256: sha256Hex(text), fetchedAt: '', cached: false };
    const proposal = deriveCodingProcedure(request, formalize(capture, query, index + 1, 0, null));
    if (proposal) proposals.push(proposal);
  }
  const nextURL = research.codingResearchNextURL(urls, progress.attempted_fetches);
  const discovery = research.codingResearchDiscoveryKernel(proposals.length, Boolean(fetchTool), nextURL);
  if (discovery.startsWith('gap:')) return decidedGap(discovery, request, { query, attempted: progress.attempted_fetches.length });
  if (discovery === 'fetch') return planOne(fetchTool, fetchArguments(nextURL));
  const proposal = proposals[0], moduleSource = moduleRead?.source ?? '';
  const latestSource = progress.successfulWriteContentFor(request.module);
  const recipe = { language: request.language, path: request.module, source: proposal.source,
    supporting_files: [], commands: [request.command] };
  const plan = moduleSource.trim() ? null : planSymbolicCommandReroute(messages, toolNames, { execution_recipe: recipe });
  const plannedWrites = (plan?.calls ?? []).filter(call => call.tool === toolFor(toolNames, Capability.Write));
  let plannedWritePure = true;
  for (const call of plannedWrites) {
    try {
      const callArguments = JSON.parse(call.arguments);
      if ((callArguments.path ?? callArguments.filePath ?? callArguments.file_path) === request.module
        && !pureResearchProgram(request, callArguments.content)) plannedWritePure = false;
    } catch { plannedWritePure = false; }
  }
  const boundary = moduleSource.trim() ? research.codingResearchLatestReadIndex(attempts.capabilities)
    : progress.latestSuccessfulWriteIndex(request.module) ?? progress.attempts.length;
  const runIndex = research.codingResearchRunIndex(attempts.capabilities, attempts.valid, attempts.commands,
    boundary, request.command);
  const observation = runIndex >= 0 ? progress.attempts[runIndex] : null;
  const effect = research.codingResearchEffectKernel(moduleSource, proposal.source, latestSource !== null,
    latestSource === null || pureResearchProgram(request, latestSource), plannedWritePure, Boolean(plan),
    plan?.kind ?? '', outcome(observation), observation ? verificationReceiptGap(observation.detail) ?? '' : '', Boolean(runTool));
  if (effect.startsWith('gap:')) return decidedGap(effect, request, proposal.provenance);
  if (effect === 'run') return planOne(runTool, jsonText({ command: request.command }));
  if (effect === 'reroute') {
    plan.research = { ...proposal.provenance, acceptanceSha256: sha256Hex(testRead.source) };
    return plan;
  }
  const verified = effect.endsWith(':verified'), existing = effect.startsWith('existing:');
  const answer = existing ? verified ? agenticMessage('coding-research-verified-existing')
    : agenticMessage('coding-research-existing-failed', { detail: observation.detail }) : plan.answer;
  const origin = existing ? verified ? 'coding-research-verified-existing' : 'coding-research-existing-verification-failed'
    : verified ? 'coding-research-verified-created' : 'coding-research-verification-failed';
  const final = resolvedFinalAnswer(answer, verified ? FinalDisposition.Finding : FinalDisposition.Failure, origin);
  final.research = { ...proposal.provenance, status: verified ? 'verified' : 'proposal', verified,
    ...(existing ? { sourceAuthored: false } : {}), acceptanceSha256: sha256Hex(testRead.source) };
  return final;
}
