import { observedManifestFacts } from './source_manifest_facts.mjs';
// Bounded repository observations before an unsupported work-item execution gap.
// Native twin: rust/src/agentic_coding/work_item_evidence.rs.
import { Progress } from './progress.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { FinalDisposition, planOne, jsonText } from './plan.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { userReadPolicyBlocksPlan } from './file_read/ownership.mjs';
import { workItemRequirements } from './work_item_requirements.mjs';
import { isCitation, obligationText } from './work_item_citations.mjs';
import { refusal, workItemRecordFinal } from './work_item_report.mjs';
import { clientWorkingDirectory } from './content.mjs';
import { currentDirectory } from './host.mjs';
import {
  attemptFor, productName, rankedCandidates, readAttemptFor, readOutcome, samePath, searchActions, searchMatches,
  workItemEvidencePaths, workItemReferences, workspaceRoots, MAXIMUM_CANDIDATE_READS
} from './work_item_references.mjs';
import {
  MAXIMUM_DOCUMENTS, documentAction, documentAttemptFor, symbolActions, symbolAttemptFor, symbolMatches,
  workItemLeads, workItemSymbols
} from './work_item_leads.mjs';

export { workItemEvidencePaths };

/** The host root a relative read falls back to when the client names no workspace. */
function hostRoot() {
  try { return currentDirectory(); } catch { return null; }
}

/** The terminal reason of a work item that names no repository file, from what its leads left unresolved. */
function leadReason(leads, symbols, unresolved, incomplete) {
  if (!leads.attachments.length && !leads.documents.length && !symbols.length) return leads.related.length ? 'RelatedEvidenceUnread' : 'NoRepositoryReference';
  if (unresolved.length && unresolved.every(row => row.kind === 'attachment')) return 'AttachmentEvidenceUnavailable';
  if (unresolved.some(row => row.kind === 'document')) return 'ExternalEvidenceUnavailable';
  if (unresolved.length) return 'UnresolvedReferences';
  return incomplete ? 'IncompleteSourceObservation' : 'EvidenceCollected';
}

/** A call-shaped compound identifier: `sameMeaning(a, b)`, `same_meaning(`. */
const INTERFACE_CALL = /(?:^|[^A-Za-z0-9_.:])([a-z][a-z0-9]*(?:[A-Z][A-Za-z0-9]*|_[a-z0-9]+)+)\s*\(/gmu;
const MAXIMUM_INTERFACES = 8;

/** Mirrors `fn requested_interfaces`: call-shaped identifiers the requirements name, each once. */
export function requestedInterfaces(requirements) {
  const names = [];
  for (const requirement of requirements) {
    for (const match of String(requirement?.text ?? '').matchAll(INTERFACE_CALL)) {
      if (!names.includes(match[1]) && names.length < MAXIMUM_INTERFACES) names.push(match[1]);
    }
  }
  return names;
}

/**
 * Mirrors `fn evidence_gap`: the narrowest gap a completed evidence
 * collection leaves. A requested interface that no read source defines is
 * named; otherwise the read files the requirements name (or every read file)
 * are the targets no change recipe of the supported subset covers.
 */
export function evidenceGap(requirements, observations, sources) {
  const read = observations.filter(row => row.complete === true && typeof row.path === 'string');
  if (!read.length) return null;
  const absent = requestedInterfaces(requirements).filter(name => !sources.some(source => source.includes(name)));
  if (absent.length) return { gap: 'RequestedInterfaceAbsent', work: absent.map(name => 'create-interface:' + name) };
  const named = read.filter(row => requirements.some(requirement => String(requirement?.text ?? '').includes(row.path.split('/').at(-1))));
  const targets = [];
  for (const row of named.length ? named : read) if (!targets.includes(row.path)) targets.push(row.path);
  return { gap: 'NoChangeRecipe', work: targets.map(path => 'change-recipe-unavailable:' + path) };
}

/**
 * Work still owed after evidence collection: the change itself (or, when a
 * gap is named, its narrower parts), and every lead left unresolved.
 */
function evidenceRemainingWork(unresolved, gap = null, unbound = null) {
  // A file the solver selected from a citation alone stays unwritten until an obligation names it (issue #1562).
  return ['bind-requirements-to-source', ...(unbound === null ? [] : [`unbound-write:${unbound}`]), ...(gap === null ? ['implement-requested-change'] : gap.work), 'verify-artifact-and-requirements',
    ...unresolved.map(row => `unresolved-evidence:${row.reason}:${row.reference}`)];
}

/**
 * `unbound` is the file a recipe would have written although no obligation of
 * the work item names it (issue #1562); the record and its report name it.
 */
export function planWorkItemEvidence(task, messages, tools, workItem = null, unbound = null) {
  const named = workItemEvidencePaths(task).length > 0;
  const references = named ? workItemReferences(task, workItem) : [];
  const leads = workItemLeads(task, workItem);
  // Identifiers are searched only when the work item names no file: a named file is the narrower evidence.
  const symbols = references.length ? [] : workItemSymbols(task);
  // A work item with neither a file nor a lead is left to the general planner.
  if (!named && !leads.attachments.length && !leads.documents.length && !leads.related.length && !symbols.length) return null;
  const progress = Progress.scan(messages),
    roots = workspaceRoots(clientWorkingDirectory(messages), hostRoot()),
    observations = [],
    unresolved = [],
    terms = [],
    sources = [];
  // The work item is split into requirements before any evidence is read, so
  // a refusal names the requirement it concerns (issue #1560).
  // A clause that lies wholly inside an address, a code span, a table of
  // evidence or a quotation is a citation, not an obligation (issue #1562).
  const obligations = obligationText(task);
  const requirements = workItemRequirements(task).map(([text, span]) => ({
    text,
    span,
    status: isCitation(obligations, span) ? 'citation' : 'unbound'
  }));
  const finish = (reason, refused = null) => {
    const gap = reason === 'EvidenceCollected' ? evidenceGap(requirements, observations, sources) : null;
    return workItemRecordFinal({
    schema: 'repository-work-item-evidence/v1',
    reason,
    ...(gap === null ? {} : { gap: gap.gap }),
    workItem,
    taskIdentity: sha256Hex(task),
    requirements,
    ...(refused === null ? {} : { refused }),
    ...(unbound === null ? {} : { unboundWrite: { path: unbound, reason: 'NamedByNoObligation' } }),
    decomposition: [{
      step: 'collect_repository_evidence',
      status: reason === 'EvidenceCollected' ? 'observed' : 'incomplete'
    }, {
      step: 'bind_requirements_to_source',
      status: 'unverified'
    }, {
      step: 'implement_requested_change',
      status: 'not_executed'
    }, {
      step: 'verify_artifact_and_requirements',
      status: 'not_executed'
    }],
    observations,
    unresolved,
    terms,
    ...(leads.related.length ? { related: leads.related } : {}),
    coverage: 'explicit-references; exact path, then file-name search; at most eight files; '
      + 'attachments unresolved; at most two linked documents and three identifiers when no file is named',
    candidateEdits: [],
    checks: [],
    effects: [],
    remainingWork: evidenceRemainingWork(unresolved, gap, unbound),
    authored: false,
    verified: false
  }, task, FinalDisposition.Gap, 'repository_work_item_evidence_gap');
  };
  const read = toolFor(tools, Capability.Read);
  const readStep = path => planOne(read, jsonText({
    path,
    filePath: path,
    file_path: path
  }));
  const planRead = path => {
    const step = readStep(path);
    return userReadPolicyBlocksPlan(messages, task, step) ? finish('ReadPolicyRefused', refusal(requirements, path)) : step;
  };
  const observe = (reference, path, attempt, resolvedBy) => {
    const observed = attempt.source_read;
    const failed = readOutcome(attempt) === 'failed';
    const complete = !failed && observed.complete && typeof observed.source === 'string' && new TextEncoder().encode(observed.source).length <= 262144;
    const observation = {
      path,
      complete,
      sourceIdentity: complete ? sha256Hex(observed.source) : null,
      excerpt: complete ? Array.from(observed.source).slice(0, 1024).join('') : null
    };
    if (reference.name !== path) observation.reference = reference.name;
    if (resolvedBy !== 'exact') observation.resolvedBy = resolvedBy;
    if (failed) observation.readFailed = true;
    observations.push(observation);
    if (complete) sources.push(observed.source);
    if (complete && (path.endsWith('.toml') || path.endsWith('.json'))) {
      try {
        observation.manifestFacts = observedManifestFacts(path, observed.source);
        observation.factScope = 'literal-declarations-only; executable use remains unverified';
      } catch {
        observation.manifestFacts = [];
        observation.factGap = 'UnsupportedOrInvalidManifest';
      }
    }
    return failed ? 'failed' : complete ? 'complete' : 'incomplete';
  };
  const leaveUnresolved = (reference, reason, extra = {}) => unresolved.push({
    reference: reference.name,
    target: reference.target,
    reason,
    ...extra
  });
  let incomplete = false;
  // An attachment is never fetched or described: its content stays unresolved evidence.
  for (const url of leads.attachments) unresolved.push({ reference: url, target: url, kind: 'attachment', reason: 'AttachmentContentUnavailable' });
  for (const reference of references) {
    if (reference.external) {
      leaveUnresolved(reference, 'OutsideRepository', { repository: reference.repository });
      continue;
    }
    if (read === null) return finish('MissingReadTool');
    const searches = searchActions(reference, tools);
    const product = productName(reference.target);
    // The exact path is read first unless the name is bare and a name search can find it without a failed read.
    const exactFirst = !product && (reference.target.includes('/') || !searches.some(action => action.capability === Capability.Glob));
    const failed = [];
    if (exactFirst) {
      const attempt = readAttemptFor(progress, reference.target, roots);
      if (attempt === null) return planRead(reference.target);
      const outcome = observe(reference, reference.target, attempt, 'exact');
      if (outcome !== 'failed') {
        incomplete ||= outcome === 'incomplete';
        continue;
      }
      failed.push(reference.target);
    } else if (userReadPolicyBlocksPlan(messages, task, readStep(reference.target))) {
      return finish('ReadPolicyRefused', refusal(requirements, reference.target));
    }
    let matches = null;
    for (const action of searches) {
      const attempt = attemptFor(progress, action);
      if (attempt === null) return planOne(action.tool, action.arguments);
      const found = searchMatches(attempt.detail, reference, roots);
      if (found.length || attempt.succeeded) {
        matches = found;
        break;
      }
    }
    if (matches === null) {
      if (product) terms.push(reference.name);
      else leaveUnresolved(reference, searches.length ? 'SearchFailed' : 'SearchUnavailable');
      continue;
    }
    const candidates = rankedCandidates(matches.filter(path => !failed.some(prior => prior === path || samePath(path, prior, roots))), reference);
    if (!candidates.length) {
      if (product && !matches.length) terms.push(reference.name);
      else leaveUnresolved(reference, matches.length ? 'CandidatesUnreadable' : 'NotFound');
      continue;
    }
    let resolved = false;
    for (const candidate of candidates.slice(0, MAXIMUM_CANDIDATE_READS)) {
      const attempt = readAttemptFor(progress, candidate, roots);
      if (attempt === null) return planRead(candidate);
      const outcome = observe(reference, candidate, attempt, candidates.length === 1 ? 'unique-name-match' : 'ranked-name-match');
      if (outcome !== 'failed') {
        incomplete ||= outcome === 'incomplete';
        resolved = true;
        break;
      }
    }
    if (!resolved) leaveUnresolved(reference, 'CandidatesUnreadable', { candidates: candidates.slice(0, MAXIMUM_CANDIDATE_READS) });
  }
  if (references.length) {
    if (unresolved.length) return finish('UnresolvedReferences');
    return finish(incomplete ? 'IncompleteSourceObservation' : 'EvidenceCollected');
  }
  // A work item whose only leads are related pages (other work items, runs,
  // repositories) keeps each as unresolved evidence this controller did not read.
  if (!leads.attachments.length && !leads.documents.length && !symbols.length) {
    for (const url of leads.related) unresolved.push({ reference: url, target: url, kind: 'related', reason: 'RelatedPageNotRead' });
  }
  // No file is named: linked documents are fetched as evidence, never followed as instructions.
  for (const url of leads.documents.slice(0, MAXIMUM_DOCUMENTS)) {
    const action = documentAction(url, tools);
    if (action === null) {
      unresolved.push({ reference: url, target: url, kind: 'document', reason: 'FetchUnavailable' });
      continue;
    }
    const attempt = documentAttemptFor(progress, url);
    if (attempt === null) return planOne(action.tool, action.arguments);
    if (!attempt.succeeded || !String(attempt.detail ?? '').trim()) {
      unresolved.push({ reference: url, target: url, kind: 'document', reason: 'FetchFailed' });
      continue;
    }
    const text = String(attempt.detail);
    observations.push({
      url,
      kind: 'document',
      authority: 'evidence-only',
      complete: new TextEncoder().encode(text).length <= 262144,
      sourceIdentity: sha256Hex(text),
      excerpt: Array.from(text).slice(0, 1024).join('')
    });
  }
  // Identifiers in backticks are searched; the best-ranked file naming one is read.
  if (symbols.length && read === null) return finish('MissingReadTool');
  for (const symbol of symbols) {
    const actions = symbolActions(symbol, tools);
    let matches = null;
    for (const action of actions) {
      const attempt = symbolAttemptFor(progress, action);
      if (attempt === null) return planOne(action.tool, action.arguments);
      const found = symbolMatches(attempt.detail, roots);
      if (found.length || attempt.succeeded) {
        matches = found;
        break;
      }
    }
    if (matches === null || !matches.length) {
      unresolved.push({ reference: symbol, target: symbol, kind: 'symbol', reason: matches === null ? (actions.length ? 'SearchFailed' : 'SearchUnavailable') : 'SymbolNotFound' });
      continue;
    }
    const candidate = matches[0];
    const attempt = readAttemptFor(progress, candidate, roots);
    if (attempt === null) return planRead(candidate);
    const outcome = observe({ name: symbol }, candidate, attempt, 'symbol-search');
    observations.at(-1).symbol = symbol;
    if (outcome === 'failed') unresolved.push({ reference: symbol, target: candidate, kind: 'symbol', reason: 'CandidatesUnreadable' });
    else incomplete ||= outcome === 'incomplete';
  }
  return finish(leadReason(leads, symbols, unresolved, incomplete));
}
