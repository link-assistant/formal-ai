// Candidate discovery observes resources; it does not infer their domain semantics.
import { Capability } from './capability.mjs';
import { qualifiedToolAttempt } from './progress.mjs';
import { toolFor } from './capability_router.mjs';
import { childrenNamed, parseLino, readText } from './host.mjs';
import { roleWordForms } from './write_lexicon.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { jsonText, planOne } from './plan.mjs';
import { observedPayload, reportedExitCode, harnessReportedFailure, observedBytesMatch, incompleteReceipt } from './tool_result.mjs';


/** Mirrors `contract_text` in rust/src/agentic_coding/workspace_discovery.rs. */
export function contractText(key, argumentsList = []) {
  const parsed = parseLino(readText('data/seed/workspace-discovery-contracts.lino'));
  const roots = parsed?.name === 'workspace-discovery-contracts' ? [parsed] : childrenNamed(parsed, 'workspace-discovery-contracts');
  if (roots.length !== 1) throw Error('MissingSourceTemplateRoot');
  const entries = childrenNamed(roots[0], 'template').filter(entry => entry.id === key);
  if (entries.length !== 1) throw Error('AmbiguousSourceTemplate');
  const texts = childrenNamed(entries[0], 'text');
  if (texts.length !== 1 || typeof texts[0].id !== 'string') throw Error('AmbiguousSourceTemplateText');
  const pieces = texts[0].id.split('{}');
  if (pieces.length !== argumentsList.length + 1 || argumentsList.some(value => typeof value !== 'string')) throw Error('DifferentSourceTemplateArity');
  let output = pieces[0];
  for (let index = 0; index < argumentsList.length; index++) output += argumentsList[index] + pieces[index + 1];
  return output;
}

const GRAMMAR = 'data/seed/workspace-discovery-grammar.lino';
const START = 'workspace-discovery-v1';
const END = 'workspace-discovery-end';
const MAXIMUM_FILES = 128;
const MAXIMUM_BYTES = 65536;
const escape = text => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

function incompleteObservation(raw) {
  if (incompleteReceipt(raw)) return true;
  try {
    const value=JSON.parse(raw);
    return value?.command_output_complete===false || value?.stdout_capture?.complete===false;
  } catch { return false; }
}

export function workspaceDiscoveryContract(task) {
  if (typeof task !== 'string') return null;
  // Reject malformed UTF-16 before source spans cross a native UTF-8 boundary.
  for (let index=0;index<task.length;index++) {
    const unit=task.charCodeAt(index);
    if (unit>=0xd800 && unit<=0xdbff) {
      const next=task.charCodeAt(++index);
      if (!(next>=0xdc00 && next<=0xdfff)) return null;
    } else if (unit>=0xdc00 && unit<=0xdfff) return null;
  }
  let seed;
  try { seed=readText(GRAMMAR); }
  catch { return null; } // An unavailable optional grammar never changes an existing route.
  const root = parseLino(seed ?? '');
  if (!root) return null;
  const grammar = root.name === 'workspace-discovery-grammar' ? root : childrenNamed(root, 'workspace-discovery-grammar')[0];
  if (!grammar) return null;
  const forms = roleWordForms('workspace_inspection_action').filter(form => form.slot === 'bare');
  const actions = forms.map(form => escape(form.text)).sort((a,b) => b.length-a.length).join('|');
  const template = childrenNamed(grammar, 'pattern')[0]?.id;
  const subject = childrenNamed(grammar, 'subject-pattern')[0]?.id;
  if (!actions || typeof template !== 'string' || typeof subject !== 'string') return null;
  if (template.split('{action}').length !== 2 || template.split('{subject}').length !== 2) return null;
  let match;
  try { match = new RegExp(template.replace('{action}', '('+actions+')').replace('{subject}', '('+subject+')'), 'diu').exec(task); }
  catch { return null; }
  if (!match || match.indices[0][0] !== 0) return null;
  const span = match.indices[2];
  if (!/^[\x00-\x7f]*$/u.test(match[2])) return null;
  const words = match[2].toLowerCase().split(/[ _-]+/u);
  if (!words.length || words.some(word => word.length < 2)) return null;
  return { source:task, subject:match[2], subjectSpan:span, remainingSpan:[span[1],task.length],
    words, model:'Unknown', schema:'Unknown', fulfilled:false };
}

/** Structural source binding only; this predicate grants no observation authority. */
export function workspaceDiscoveryContractBound(contract, task) {
  if (!contract || typeof task !== 'string' || contract.source !== task
      || typeof contract.subject !== 'string' || !/^[\x00-\x7f]*$/u.test(contract.subject)) return false;
  const span = contract.subjectSpan;
  const remaining = contract.remainingSpan;
  if (!Array.isArray(span) || span.length !== 2
      || !span.every(value => Number.isSafeInteger(value) && value >= 0)
      || span[0] > span[1] || span[1] > task.length
      || !Array.isArray(remaining) || remaining.length !== 2
      || remaining[0] !== span[1] || remaining[1] !== task.length
      || task.slice(span[0], span[1]) !== contract.subject) return false;
  const words = contract.subject.toLowerCase().split(/[ _-]+/u);
  return words.length > 0 && words.every(word => word.length >= 2)
    && Array.isArray(contract.words) && contract.words.length === words.length
    && words.every((word, index) => contract.words[index] === word);
}

export function workspaceDiscoveryCommand(originalNeed) {
  // The identity binds the observed call, not approval or source closure.
  const identity = stableId('workspace_discovery_need', originalNeed);
  return contractText("source-template-1", [identity]);
}

function observation(kind, contract, detail) {
  return { kind:'observation', observationKind:kind, answer:detail, disposition:'gap',
    contract, schema:'Unknown', model:'Unknown', fulfilled:false,
    sourceEffects:'Unknown', approval:'Unknown', dependencySourceClosure:'Unknown' };
}

/** Mirrors `candidate_read_command` in rust/src/agentic_coding/workspace_discovery.rs. */
export function workspaceCandidateReadCommand(originalNeed,path) {
  if (!/^[A-Za-z0-9_.\/-]+$/u.test(path) || path.split('/').some(part => part === '.' || part === '..' || part === '')) return null;
  return contractText("source-template-2", [stableId('workspace_candidate_need',originalNeed),path]);
}

/** Existing transcript/provider contracts bind observations; supplied custom metadata is ignored. */
export function workspaceDiscoveryStep(task, progress, tools, originalNeed = task) {
  const contract = workspaceDiscoveryContract(task);
  if (!workspaceDiscoveryContractBound(contract, task)) return null;
  const command = workspaceDiscoveryCommand(originalNeed);
  const run = qualifiedToolAttempt(progress,Capability.Run,{command});
  if (!run) {
    const list = toolFor(tools, Capability.ListDir);
    const listAttempt = qualifiedToolAttempt(progress,Capability.ListDir,{path:'.'});
    if (list !== null && !listAttempt) return planOne(list, jsonText({path:'.'}));
    // An unframed native listing or unsupported advertised provider cannot prove scope/completeness.
    const tool = toolFor(tools, Capability.Run);
    return tool === null ? observation('unavailable',contract,contractText("source-template-8"))
      : planOne(tool,jsonText({command}));
  }
  if (!run.succeeded || incompleteObservation(run.detail) || harnessReportedFailure(run.detail) || reportedExitCode(run.detail) !== 0) {
    return observation('failed',contract,contractText("source-template-9"));
  }
  const payload = observedPayload(run.detail);
  if (typeof payload !== 'string' || !observedBytesMatch(run.detail,payload) || new TextEncoder().encode(payload).length > MAXIMUM_BYTES) {
    return observation('unqualified',contract,contractText("source-template-10"));
  }
  const lines = payload.split('\n');
  if (lines.at(-1) === '') lines.pop();
  if (lines.shift() !== START || lines.pop() !== END || lines.length > MAXIMUM_FILES
    || lines.some(path => !/^\.\/[A-Za-z0-9_.\/-]+$/u.test(path)
      || path.slice(2).split('/').some(part => part === '.' || part === '..' || part === ''))) {
    return observation('unqualified',contract,contractText("source-template-11"));
  }
  const unique = [...new Set(lines)];
  const candidates = unique.filter(path => {
    const terms = path.split(/[\/_.-]+/u).map(word => word.toLowerCase());
    return contract.words.every(word => terms.includes(word));
  });
  if (candidates.length === 0) return observation('no_candidate',contract,
    contractText("source-template-3",[String(unique.length),JSON.stringify(contract.words)]));
  if (candidates.length !== 1) return observation('ambiguous',contract,
    contractText("source-template-4",[String(candidates.length),candidates.join(', ')]));
  const path = candidates[0].slice(2);
  const readCommand=workspaceCandidateReadCommand(originalNeed,path);
  const read=qualifiedToolAttempt(progress,Capability.Run,{command:readCommand});
  if (!read) {
    const tool=toolFor(tools,Capability.Run);
    return tool===null?observation('unavailable',contract,contractText("source-template-5",[path]))
      :planOne(tool,jsonText({command:readCommand}));
  }
  const sourcePayload=observedPayload(read.detail);
  if (!read.succeeded || incompleteObservation(read.detail) || harnessReportedFailure(read.detail) || reportedExitCode(read.detail)!==0
    || typeof sourcePayload!=='string' || !observedBytesMatch(read.detail,sourcePayload) || !sourcePayload.startsWith('workspace-source-v1\n')
    || !sourcePayload.endsWith('\nworkspace-source-end\n')) {
    return observation('unqualified_read',contract,contractText("source-template-6",[path]));
  }
  const source=sourcePayload.slice('workspace-source-v1\n'.length,-'\nworkspace-source-end\n'.length);
  const bytes = new TextEncoder().encode(source).length;
  if (bytes > MAXIMUM_BYTES) return observation('unqualified_read',contract,contractText("source-template-13"));
  return observation('candidate_read',contract,contractText("source-template-7",[path,String(bytes),stableId('observed_candidate',source)]));
}
