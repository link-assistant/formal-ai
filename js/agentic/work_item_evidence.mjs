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
import { splitWhitespace } from './crate/rust_str.mjs';

/** File references are evidence candidates, never shell commands or write authority. */
export function workItemEvidencePaths(task) {
  if (new TextEncoder().encode(task).length > 32768) return [];
  const paths = [];
  for (const word of splitWhitespace(task).flatMap(part => part.split(/[`'"()[\],;]+/u))) {
    const path = word.replace(/^[`'"(\[]+|[`'"),.;:\]]+$/gu, '');
    if (!/^[A-Za-z0-9_-][A-Za-z0-9_./-]*\.(?:md|toml|json|rs|mjs|js|ts|yml|yaml|lino)$/u.test(path) || path.split('/').some(part => !part || part === '.' || part === '..') || paths.includes(path)) continue;
    paths.push(path);
    if (paths.length === 8) break;
  }
  return paths;
}
/** Mirrors `fn blob_paths`: the repository paths of the work item's own GitHub blob links. */
function blobPaths(task, workItem) {
  const repository = /^https:\/\/github\.com\/([^/\s]+\/[^/\s]+)\//u.exec(workItem ?? '')?.[1];
  if (repository === undefined) return [];
  const prefix = 'https://github.com/' + repository + '/blob/';
  const paths = [];
  for (const word of splitWhitespace(task).flatMap(part => part.split(/[`'"()[\],;<>]+/u))) {
    if (!word.startsWith(prefix)) continue;
    const rest = word.slice(prefix.length).split(/[#?]/u)[0];
    const slash = rest.indexOf('/');
    if (slash > 0 && slash < rest.length - 1) paths.push(rest.slice(slash + 1));
  }
  return paths;
}
/**
 * Mirrors `fn linked_evidence_paths`: a bare file name that the work item's own
 * GitHub link places in the repository is read at the linked path, so a link
 * label is not taken for a file at the root (issue #1560). A label whose
 * linked path is not an evidence path is dropped.
 */
export function linkedEvidencePaths(task, workItem = null) {
  const linked = blobPaths(task, workItem);
  const paths = [];
  for (const path of workItemEvidencePaths(task)) {
    const target = path.includes('/') ? path : linked.find(candidate => candidate.endsWith('/' + path)) ?? path;
    if (target !== path && workItemEvidencePaths(target)[0] !== target) continue;
    if (!paths.includes(target)) paths.push(target);
  }
  return paths;
}
export function planWorkItemEvidence(task, messages, tools, workItem = null) {
  const paths = linkedEvidencePaths(task, workItem);
  if (!paths.length) return null;
  const progress = Progress.scan(messages),
    observations = [];
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
    coverage: 'explicit-paths-only; at most eight files',
    authored: false,
    verified: false
  }, task, FinalDisposition.Gap, 'repository_work_item_evidence_gap');
  for (const path of paths) {
    const observed = progress.sourceReadFor(path);
    if (!observed) {
      const read = toolFor(tools, Capability.Read);
      if (read === null) return finish('MissingReadTool');
      const step = planOne(read, jsonText({
        path,
        filePath: path,
        file_path: path
      }));
      return userReadPolicyBlocksPlan(messages, task, step) ? finish('ReadPolicyRefused', refusal(requirements, path)) : step;
    }
    const complete = observed.error === null && observed.complete && typeof observed.source === 'string' && new TextEncoder().encode(observed.source).length <= 262144;
    observations.push({
      path,
      complete,
      sourceIdentity: complete ? sha256Hex(observed.source) : null,
      excerpt: complete ? Array.from(observed.source).slice(0, 1024).join('') : null
    });
    if (!complete) return finish('IncompleteSourceObservation');
    if (path.endsWith('.toml') || path.endsWith('.json')) {
      const observation = observations.at(-1);
      try {
        observation.manifestFacts = observedManifestFacts(path, observed.source);
        observation.factScope = 'literal-declarations-only; executable use remains unverified';
      } catch {
        observation.manifestFacts = [];
        observation.factGap = 'UnsupportedOrInvalidManifest';
      }
    }
  }
  return finish('EvidenceCollected');
}
