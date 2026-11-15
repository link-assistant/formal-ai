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
import { refusal, workItemRecordFinal } from './work_item_report.mjs';
import { clientWorkingDirectory } from './content.mjs';
import { currentDirectory } from './host.mjs';
import {
  attemptFor, productName, rankedCandidates, readAttemptFor, readOutcome, samePath, searchActions, searchMatches,
  workItemEvidencePaths, workItemReferences, workspaceRoots, MAXIMUM_CANDIDATE_READS
} from './work_item_references.mjs';

export { workItemEvidencePaths };

/** The host root a relative read falls back to when the client names no workspace. */
function hostRoot() {
  try { return currentDirectory(); } catch { return null; }
}

export function planWorkItemEvidence(task, messages, tools, workItem = null) {
  if (!workItemEvidencePaths(task).length) return null;
  const references = workItemReferences(task, workItem);
  const progress = Progress.scan(messages),
    roots = workspaceRoots(clientWorkingDirectory(messages), hostRoot()),
    observations = [],
    unresolved = [],
    terms = [];
  // The work item is split into requirements before any evidence is read, so
  // a refusal names the requirement it concerns (issue #1560).
  const requirements = workItemRequirements(task).map(([text, span]) => ({
    text,
    span,
    status: 'unbound'
  }));
  const finish = (reason, refused = null) => workItemRecordFinal({
    schema: 'repository-work-item-evidence/v1',
    reason,
    workItem,
    taskIdentity: sha256Hex(task),
    requirements,
    ...(refused === null ? {} : { refused }),
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
    coverage: 'explicit-references; exact path, then file-name search; at most eight files',
    authored: false,
    verified: false
  }, task, FinalDisposition.Gap, 'repository_work_item_evidence_gap');
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
  if (unresolved.length) return finish('UnresolvedReferences');
  return finish(incomplete ? 'IncompleteSourceObservation' : 'EvidenceCollected');
}
