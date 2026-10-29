// Bounded repository observations before an unsupported work-item execution gap.
// Native twin: rust/src/agentic_coding/work_item_evidence.rs.
import {Progress} from './progress.mjs';
import {Capability} from './capability.mjs';
import {toolFor} from './capability_router.mjs';
import {FinalDisposition, resolvedFinalAnswer, planOne, jsonText} from './plan.mjs';
import {sha256Hex} from './crate/source_fetch.mjs';
import {readPolicyBlocksPlan} from './file_read/ownership.mjs';
import {clausesWithSpans} from './crate/obligation_ledger.mjs';
import {splitWhitespace} from './crate/rust_str.mjs';

/** File references are evidence candidates, never shell commands or write authority. */
export function workItemEvidencePaths(task) {
  if (new TextEncoder().encode(task).length > 32768) return [];
  const paths = [];
  for (const word of splitWhitespace(task).flatMap(part => part.split(/[`'"()[\],;]+/u))) {
    const path = word.replace(/^[`'"(\[]+|[`'"),.;:\]]+$/gu, '');
    if (!/^[A-Za-z0-9_-][A-Za-z0-9_./-]*\.(?:md|toml|json|rs|mjs|js|ts|yml|yaml|lino)$/u.test(path)
      || path.split('/').some(part => !part || part === '.' || part === '..') || paths.includes(path)) continue;
    paths.push(path);
    if (paths.length === 8) break;
  }
  return paths;
}

export function planWorkItemEvidence(task, messages, tools, workItem = null) {
  const paths = workItemEvidencePaths(task);
  if (!paths.length) return null;
  const progress = Progress.scan(messages), observations = [];
  const finish = reason => resolvedFinalAnswer(jsonText({
    schema: 'repository-work-item-evidence/v1', reason, workItem, taskIdentity: sha256Hex(task),
    requirements: clausesWithSpans(task).map(([text, span]) => ({text, span, status: 'unbound'})),
    decomposition: [
      {step: 'collect_repository_evidence', status: reason === 'EvidenceCollected' ? 'observed' : 'incomplete'},
      {step: 'bind_requirements_to_source', status: 'unverified'},
      {step: 'implement_requested_change', status: 'not_executed'},
      {step: 'verify_artifact_and_requirements', status: 'not_executed'},
    ], observations, coverage: 'explicit-paths-only; at most eight files',
    authored: false, verified: false,
  }), FinalDisposition.Gap, 'repository_work_item_evidence_gap');
  for (const path of paths) {
    const observed = progress.sourceReadFor(path);
    if (!observed) {
      const read = toolFor(tools, Capability.Read);
      if (read === null) return finish('MissingReadTool');
      const step = planOne(read, jsonText({path, filePath: path, file_path: path}));
      return readPolicyBlocksPlan(task, step) ? finish('ReadPolicyRefused') : step;
    }
    const complete = observed.error === null && observed.complete && typeof observed.source === 'string'
      && new TextEncoder().encode(observed.source).length <= 262144;
    observations.push({path, complete, sourceIdentity: complete ? sha256Hex(observed.source) : null,
      excerpt: complete ? Array.from(observed.source).slice(0, 1024).join('') : null});
    if (!complete) return finish('IncompleteSourceObservation');
  }
  return finish('EvidenceCollected');
}
