import fs from 'node:fs';
import { exclusiveCreate } from '../../experiments/js_dogfood/exclusive-create.mjs';
import path from 'node:path';
import { boundedCommandSync } from './bounded-command.mjs';
import { createHash } from 'node:crypto';
import { shellQuote } from '../agentic/git_commit.mjs';
import { registeredPhysicalReadObservation, issueTrackedSourceContext,
  trackedSourceContextObservation } from './node-source-session-host.mjs';
import { prepareRustConstantRename } from './rust-source-transform.mjs';

const handles = new WeakMap();
const receipts = new WeakMap();
const trackedEffects = new WeakMap();
const ownedArtifacts = new WeakMap();
const artifactReceipts = new WeakMap();
const hash = value => createHash('sha256').update(value).digest('hex');
const identity = value => [value.dev, value.ino, value.birthtimeNs].join(':');
function regular(file) {
  if (fs.realpathSync(file) !== file) throw Error('TrackedPathAlias');
  const value = fs.lstatSync(file, { bigint: true });
  if (!value.isFile() || value.isSymbolicLink() || value.nlink !== 1n) throw Error('AliasedTrackedFile');
  return identity(value);
}
function git(root, context, ...argumentsValue) {
  const argumentsList = ['-c', 'core.fsmonitor=false', '-c', 'core.untrackedCache=false',
    '-c', 'core.hooksPath=/dev/null', '-c', 'core.quotepath=false', '-C', root, ...argumentsValue];
  const command = ['/usr/bin/git', ...argumentsList].map(shellQuote).join(' ');
  if (trackedSourceContextObservation(context, command).commandPolicy === false) {
    throw Error('TrackedGitObservationDenied');
  }
  const result = boundedCommandSync('/usr/bin/git', argumentsList, {
    encoding: 'utf8', timeout: 5000, maxBuffer: 1048576,
    env: { GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null',
      GIT_OPTIONAL_LOCKS: '0', LC_ALL: 'C' },
  });
  if (result.error) throw result.error;
  if (result.status !== 0 || result.signal !== null || result.cleanup_complete !== true) {
    const failure = new Error('TrackedGitObservationFailed');
    Object.assign(failure, { status: result.status, signal: result.signal,
      stdout: result.stdout, stderr: result.stderr });
    throw failure;
  }
  trackedSourceContextObservation(context, command);
  return result.stdout;
}
function baselineFile(file) {
  const fileIdentity = regular(file);
  const bytes = fs.readFileSync(file);
  if (regular(file) !== fileIdentity) throw Error('TrackedGitFileDrift');
  return {file, fileIdentity, bytes};
}
function statusRecords(status, operand) {
  if (status !== '' && !status.endsWith('\0')) throw Error('IncompleteTrackedStatus');
  const fields = status === '' ? [] : status.slice(0, -1).split('\0');
  const unrelated = [];
  let target = null;
  for (let index = 0; index < fields.length; index += 1) {
    const field = fields[index];
    if (field.length < 4 || field[2] !== ' ' || !/^[ MADRCU?!T]{2}$/u.test(field.slice(0, 2))) {
      throw Error('UnknownTrackedStatusShape');
    }
    const name = field.slice(3);
    if (!name) throw Error('UnknownTrackedStatusPath');
    const moved = /[RC]/u.test(field.slice(0, 2));
    let previous = null;
    if (moved) {
      previous = fields[++index];
      if (!previous) throw Error('IncompleteTrackedRenameStatus');
    }
    if (name === operand || previous === operand) {
      if (target !== null || moved || /[?!UD]/u.test(field.slice(0, 2))) {
        throw Error('AmbiguousTrackedTargetStatus');
      }
      target = field.slice(0, 2);
    } else unrelated.push(moved ? field + '\0' + previous : field);
  }
  unrelated.sort();
  return {unrelated, target};
}
function captureGitBaseline(root, context, gitDirectory, operand, status) {
  const head = baselineFile(path.join(gitDirectory, 'HEAD'));
  const index = baselineFile(path.join(gitDirectory, 'index'));
  const namespace = git(root, context, 'ls-files', '--stage', '-z');
  const references = git(root, context, 'for-each-ref', '--format=%(refname)%00%(objectname)');
  const text = head.bytes.toString('utf8');
  const symbolic = /^ref: (refs\/[A-Za-z0-9_./-]+)\n$/u.exec(text);
  const detached = /^[0-9a-f]{40,64}\n$/u.test(text);
  if (!symbolic && !detached) throw Error('UnknownTrackedHeadShape');
  const headState = detached || references.split('\n').some(line =>
    symbolic && line.startsWith(symbolic[1] + '\0')) ? 'ObservedReference' : 'Missing';
  const records = statusRecords(status, operand);
  return {head, index, namespace, references, headState, records};
}
function unchangedGitBaseline(draft) {
  const baseline = draft.gitBaseline;
  for (const source of [baseline.head, baseline.index]) {
    if (regular(source.file) !== source.fileIdentity
        || !fs.readFileSync(source.file).equals(source.bytes)) throw Error('TrackedGitBaselineDrift');
  }
  if (git(draft.workspace, draft.context, 'ls-files', '--stage', '-z') !== baseline.namespace
      || git(draft.workspace, draft.context, 'for-each-ref', '--format=%(refname)%00%(objectname)') !== baseline.references) {
    throw Error('TrackedGitNamespaceDrift');
  }
}
function sourceOnlyStatus(draft, status) {
  const after = statusRecords(status, draft.operand);
  for (const artifact of ownedArtifacts.get(draft) ?? []) {
    if (regular(artifact.file) !== artifact.identity
        || !fs.readFileSync(artifact.file).equals(artifact.bytes)) throw Error('TrackedArtifactDrift');
    const record = '?? ' + artifact.path;
    const index = after.unrelated.indexOf(record);
    if (index < 0) throw Error('MissingTrackedArtifactStatus');
    after.unrelated.splice(index, 1);
  }
  const before = draft.gitBaseline.records;
  const expected = (before.target === null ? ' ' : before.target[0]) + 'M';
  if (after.target !== expected || JSON.stringify(after.unrelated) !== JSON.stringify(before.unrelated)) {
    throw Error('UnexpectedTrackedEffectScope');
  }
}

function scope(draft) {
  if (!draft || !draft.active) throw Error('MissingActiveTrackedTransaction');
  const observed = trackedSourceContextObservation(draft.context);
  if (observed.request !== draft.request || observed.workspace !== draft.workspace
      || observed.path !== draft.target || observed.contract !== draft.contract) throw Error('TrackedContextDrift');
  if (fs.realpathSync(draft.workspace) !== draft.workspace
      || identity(fs.lstatSync(draft.workspace, { bigint: true })) !== draft.rootIdentity
      || fs.realpathSync(draft.gitDirectory) !== draft.gitDirectory
      || identity(fs.lstatSync(draft.gitDirectory, { bigint: true })) !== draft.gitIdentity) {
    throw Error('TrackedRepositoryIdentityDrift');
  }
  if (git(draft.workspace, draft.context, 'rev-parse', '--show-toplevel').trim() !== draft.workspace
      || git(draft.workspace, draft.context, 'rev-parse', '--absolute-git-dir').trim() !== draft.gitDirectory) {
    throw Error('DifferentTrackedRepository');
  }
  if (regular(draft.target) !== draft.fileIdentity) throw Error('TrackedLeafIdentityDrift');
  unchangedGitBaseline(draft);
  return observed;
}
function lookup(handle) {
  const draft = handles.get(handle);
  scope(draft);
  return draft;
}
function receipt(handle, draft, state, observedBytes = null) {
  draft.state = state;
  const result = Object.freeze({});
  receipts.set(result, { handle, state, before: draft.beforeSha256, after: draft.afterSha256,
    observedSha256: observedBytes === null ? null : hash(observedBytes) });
  return result;
}
function currentBytes(draft) {
  scope(draft);
  const descriptor = fs.openSync(draft.target, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
  let actual;
  try {
    if (identity(fs.fstatSync(descriptor, { bigint: true })) !== draft.fileIdentity) throw Error('TrackedOpenedLeafDrift');
    actual = fs.readFileSync(descriptor);
  } finally { fs.closeSync(descriptor); }
  scope(draft);
  return actual;
}
function exactBytes(draft, expected) {
  if (!currentBytes(draft).equals(expected)) throw Error('TrackedSourceBytesDrift');
}
function replaceBytes(draft, before, after) {
  exactBytes(draft, before);
  draft.ownedBytes = Buffer.from(before);
  const descriptor = fs.openSync(draft.target, fs.constants.O_RDWR | fs.constants.O_NOFOLLOW);
  try {
    if (identity(fs.fstatSync(descriptor, { bigint: true })) !== draft.fileIdentity
        || !fs.readFileSync(descriptor).equals(before)) throw Error('TrackedWritePreimageDrift');
    let offset = 0;
    while (offset < after.length) {
      const written = fs.writeSync(descriptor, after, offset, after.length - offset, offset);
      if (written === 0) throw Error('TrackedShortWrite');
      const next = Buffer.alloc(Math.max(draft.ownedBytes.length, offset + written));
      draft.ownedBytes.copy(next);
      after.copy(next, offset, offset, offset + written);
      draft.ownedBytes = next;
      offset += written;
    }
    fs.ftruncateSync(descriptor, after.length);
    draft.ownedBytes = Buffer.from(after);
    fs.fsyncSync(descriptor);
  } finally { fs.closeSync(descriptor); }
  exactBytes(draft, after);
}

/** Factory-bound tracked source effect; namespace races and native validity remain Unknown. */
export async function prepareTrackedSourceTransaction(token, request, workspace, declaration, proposed) {
  const source = registeredPhysicalReadObservation(token, request, workspace, declaration);
  const contract = source.referenceRequest;
  if (!contract || !contract.wholeRequestConsumed || contract.originalRequest !== request) throw Error('MissingCompleteTrackedRequest');
  const clause = contract.clauses.find(value => value.kind === 'private-constant-rename');
  if (!clause || clause.captures.sourcePath.text !== source.operand) throw Error('DifferentTrackedOperand');
  const candidate = await prepareRustConstantRename(source.content,
    clause.captures.previous.text, clause.captures.replacement.text);
  const latest = registeredPhysicalReadObservation(token, request, workspace, declaration);
  if (latest.sha256 !== source.sha256 || !candidate || !proposed
      || candidate.beforeSha256 !== source.sha256 || proposed.content !== candidate.content
      || proposed.beforeSha256 !== candidate.beforeSha256 || proposed.afterSha256 !== candidate.afterSha256
      || hash(candidate.content) !== candidate.afterSha256) throw Error('DifferentTrackedCandidate');
  const context = issueTrackedSourceContext(token, request, workspace, declaration);
  const target = latest.path;
  const root = git(workspace, context, 'rev-parse', '--show-toplevel').trim();
  const gitDirectory = git(workspace, context, 'rev-parse', '--absolute-git-dir').trim();
  if (root !== workspace || gitDirectory !== path.join(workspace, '.git')
      || fs.realpathSync(root) !== root || fs.realpathSync(gitDirectory) !== gitDirectory
      || !fs.lstatSync(gitDirectory).isDirectory()) {
    throw Error('UnownedTrackedRoot');
  }
  const tracked = git(root, context, 'ls-files', '--stage', '--', latest.operand);
  if (!/^(100644|100755) [0-9a-f]+ 0\t/u.test(tracked)
      || tracked.slice(tracked.indexOf('\t') + 1).trimEnd() !== latest.operand) throw Error('UnownedTrackedEntry');
  const beforeStatus = git(root, context, 'status', '--porcelain=v1', '-z', '--untracked-files=all');
  const gitBaseline = captureGitBaseline(root, context, gitDirectory, latest.operand, beforeStatus);
  const draft = { context, request, workspace, target, contract,
    declaration: source.declaration,
    operand: latest.operand, beforeStatus, gitDirectory, gitBaseline,
    rootIdentity: identity(fs.lstatSync(root, { bigint: true })),
    gitIdentity: identity(fs.lstatSync(gitDirectory, { bigint: true })), fileIdentity: regular(target),
    before: Buffer.from(source.content), after: Buffer.from(candidate.content),
    beforeSha256: source.sha256, afterSha256: candidate.afterSha256,
    active: true, state: 'prepared', journal: null };
  exactBytes(draft, draft.before);
  const handle = Object.freeze({});
  handles.set(handle, draft);
  return handle;
}

export function persistTrackedSourceInverse(handle) {
  const draft = lookup(handle);
  if (draft.state !== 'prepared') throw Error('InvalidInversePhase');
  exactBytes(draft, draft.before);
  if (git(draft.workspace, draft.context, 'status', '--porcelain=v1', '-z', '--untracked-files=all') !== draft.beforeStatus) {
    throw Error('TrackedWorktreeDrift');
  }
  const directory = fs.mkdtempSync(path.join(draft.gitDirectory, 'formal-ai-inverse-'));
  const bytes = Buffer.from(JSON.stringify({ request: draft.request, operand: draft.operand,
    before: draft.before.toString('base64'), after: draft.after.toString('base64'),
    beforeSha256: draft.beforeSha256, afterSha256: draft.afterSha256,
    pendingNeeds: draft.contract.unresolvedNeeds,
    gitBaseline: {head: draft.gitBaseline.head.bytes.toString('base64'),
      index: draft.gitBaseline.index.bytes.toString('base64'),
      headState: draft.gitBaseline.headState, namespace: draft.gitBaseline.namespace,
      references: draft.gitBaseline.references, status: draft.beforeStatus} }));
  const file = path.join(directory, 'inverse.json');
  const descriptor = fs.openSync(file, fs.constants.O_WRONLY | fs.constants.O_CREAT | fs.constants.O_EXCL, 0o600);
  try { fs.writeFileSync(descriptor, bytes); fs.fsyncSync(descriptor); } finally { fs.closeSync(descriptor); }
  const folder = fs.openSync(directory, fs.constants.O_RDONLY);
  try { fs.fsyncSync(folder); } finally { fs.closeSync(folder); }
  const parent = fs.openSync(draft.gitDirectory, fs.constants.O_RDONLY);
  try { fs.fsyncSync(parent); } finally { fs.closeSync(parent); }
  draft.journal = { file, bytes, identity: regular(file) };
  scope(draft);
  if (!fs.readFileSync(file).equals(bytes)) throw Error('DurableInverseReadbackMismatch');
  return receipt(handle, draft, 'inverse-durable');
}
function inverse(draft) {
  if (!draft.journal || regular(draft.journal.file) !== draft.journal.identity
      || !fs.readFileSync(draft.journal.file).equals(draft.journal.bytes)) throw Error('DurableInverseDrift');
}
export function writeTrackedSourceCandidate(handle) {
  const draft = lookup(handle);
  if (draft.state !== 'inverse-durable') throw Error('MissingDurableInverse');
  inverse(draft);
  if (git(draft.workspace, draft.context, 'status', '--porcelain=v1', '-z', '--untracked-files=all') !== draft.beforeStatus) {
    throw Error('TrackedWorktreeDrift');
  }
  draft.state = 'write-in-progress';
  replaceBytes(draft, draft.before, draft.after);
  return receipt(handle, draft, 'written-unverified');
}
export function readTrackedSourceCandidate(handle) {
  const draft = lookup(handle);
  if (draft.state !== 'written-unverified') throw Error('MissingOwnedTrackedWrite');
  inverse(draft);
  const observedBytes = currentBytes(draft);
  if (!observedBytes.equals(draft.after)) throw Error('TrackedSourceBytesDrift');
  const status = git(draft.workspace, draft.context, 'status', '--porcelain=v1', '-z', '--untracked-files=all');
  sourceOnlyStatus(draft, status);
  return receipt(handle, draft, 'source-change-observed', observedBytes);
}
function boundTrackedEffect(handle, providerReceipt) {
  const draft = lookup(handle);
  const proof = receipts.get(providerReceipt);
  if (!proof || proof.handle !== handle || proof.state !== 'source-change-observed'
      || draft.state !== proof.state || proof.before !== draft.beforeSha256
      || proof.after !== draft.afterSha256 || proof.observedSha256 !== draft.afterSha256) {
    throw Error('UnownedTrackedEffectReadback');
  }
  if (draft.contract.originalRequest !== draft.request || draft.contract.wholeRequestConsumed !== true) {
    throw Error('DifferentTrackedEffectRequest');
  }
  inverse(draft);
  exactBytes(draft, draft.after);
  const status = git(draft.workspace, draft.context, 'status', '--porcelain=v1', '-z', '--untracked-files=all');
  sourceOnlyStatus(draft, status);
  return {draft, status};
}
/** Only a real provider readback can issue a live candidate-bound observational token. */
export function issueTrackedSourceEffectReceipt(handle, providerReceipt) {
  const {draft} = boundTrackedEffect(handle, providerReceipt);
  const token = Object.freeze({});
  trackedEffects.set(token, {handle, providerReceipt, draft});
  return token;
}
/** Observation carries no permission to write evidence or claim native completion. */
export function trackedSourceEffectObservation(token, request, workspace) {
  const binding = trackedEffects.get(token);
  if (!binding) throw Error('UnissuedTrackedSourceEffectReceipt');
  const {draft, status} = boundTrackedEffect(binding.handle, binding.providerReceipt);
  if (draft !== binding.draft || request !== draft.request || workspace !== draft.workspace) {
    throw Error('DifferentTrackedEffectReceiptContext');
  }
  return Object.freeze({kind: 'OnlyTrackedSourceEffect', originalRequest: draft.request,
    requestSha256: hash(draft.request), workspace: draft.workspace, operand: draft.operand,
    declaration: draft.declaration, referenceRequest: draft.contract,
    beforeSha256: draft.beforeSha256, afterSha256: draft.afterSha256,
    readbackSha256: draft.afterSha256, inverseSha256: hash(draft.journal.bytes),
    indexSha256: hash(draft.gitBaseline.index.bytes), headSha256: hash(draft.gitBaseline.head.bytes),
    headState: draft.gitBaseline.headState, status, baselineStatus: draft.beforeStatus,
    trackedNamespaceSha256: hash(draft.gitBaseline.namespace),
    pendingNeeds: draft.contract.unresolvedNeeds, artifactWriteAuthority: 'Missing',
    nativeValidity: 'Unknown', namespaceAuthority: 'Unknown', raceProof: false,
    proofAuthority: 'Missing', restartRecovery: 'Missing', completed: false});
}

export function trackedSourceTransactionDisposition(handle, observation) {
  const draft = lookup(handle), proof = receipts.get(observation);
  if (!proof || proof.handle !== handle || proof.state !== draft.state) throw Error('UnownedTrackedReceipt');
  return Object.freeze({ state: proof.state, beforeSha256: proof.before, afterSha256: proof.after,
    observedSha256: proof.observedSha256,
    pendingNeeds: draft.contract.unresolvedNeeds, namespaceAuthority: 'Unknown', raceProof: false,
    nativeValidity: 'Unknown', receiptAuthority: 'Missing', proofAuthority: 'Missing',
    restartRecovery: 'Missing', recoveryScope: 'CurrentLiveFactoryContextOnly', completed: false });
}
export function abortTrackedSourceTransaction(handle) {
  const draft = lookup(handle);
  if (['write-in-progress', 'rollback-in-progress', 'written-unverified',
    'source-change-observed', 'refused-drift'].includes(draft.state)) {
    inverse(draft);
    const owned = draft.ownedBytes;
    if (!Buffer.isBuffer(owned)) throw Error('TrackedEffectOutcomeUnknown');
    const actual = currentBytes(draft);
    if (!actual.equals(owned)) return receipt(handle, draft, 'refused-drift', actual);
    draft.state = 'rollback-in-progress';
    replaceBytes(draft, owned, draft.before);
    return receipt(handle, draft, 'rolled-back');
  }
  exactBytes(draft, draft.before);
  return receipt(handle, draft, 'released-without-effect');
}
export function recoverTrackedSourceTransaction(handle) {
  const draft = lookup(handle);
  inverse(draft);
  return abortTrackedSourceTransaction(handle);
}

/** Exclusive artifact effects require a live private tracked token and compiled proposal. */
export async function createTrackedSourceEvidenceArtifact(token, request, workspace, operand, content) {
  const binding = trackedEffects.get(token);
  if (!binding) throw Error('UnissuedTrackedSourceEffectReceipt');
  const {draft} = boundTrackedEffect(binding.handle, binding.providerReceipt);
  if (request !== draft.request || workspace !== draft.workspace) throw Error('DifferentTrackedArtifactContext');
  const proposalModule = await import('./tracked-source-evidence-proposal.mjs');
  const proposal = await proposalModule.proposeTrackedSourceEvidence(token, request, workspace);
  boundTrackedEffect(binding.handle, binding.providerReceipt);
  if (!proposal || proposal.writeAuthority !== false
      || proposal.artifacts.filter(value => value.path === operand && value.content === content).length !== 1) {
    throw Error('UnownedTrackedArtifactBytes');
  }
  if (typeof operand !== 'string' || operand.split('/').some(part => !part || part === '.' || part === '..')
      || operand.includes('\0') || operand === draft.operand || operand.startsWith('.git/')) {
    throw Error('UnknownTrackedArtifactPath');
  }
  const root = fs.lstatSync(workspace, {bigint: true});
  const result = exclusiveCreate(workspace, operand, content, {dev: String(root.dev), ino: String(root.ino)});
  if (result.is_error || result.source_creation?.success !== true) throw Error('TrackedArtifactCreationFailed');
  const file = path.join(workspace, operand), bytes = Buffer.from(content, 'utf8');
  const fileIdentity = regular(file);
  const observed = fs.readFileSync(file);
  if (regular(file) !== fileIdentity || !observed.equals(bytes)) throw Error('TrackedArtifactReadbackFailed');
  const records = ownedArtifacts.get(draft) ?? [];
  records.push({path: operand, file, bytes, identity: fileIdentity});
  ownedArtifacts.set(draft, records);
  boundTrackedEffect(binding.handle, binding.providerReceipt);
  const receipt = Object.freeze({});
  artifactReceipts.set(receipt, {binding, path: operand, sha256: hash(bytes)});
  return receipt;
}
export function trackedSourceEvidenceArtifactObservation(receipt, request, workspace) {
  const proof = artifactReceipts.get(receipt);
  if (!proof) throw Error('UnissuedTrackedArtifactReceipt');
  const {draft} = boundTrackedEffect(proof.binding.handle, proof.binding.providerReceipt);
  if (request !== draft.request || workspace !== draft.workspace) throw Error('DifferentTrackedArtifactContext');
  return Object.freeze({kind: 'OnlyObservedTrackedArtifact', path: proof.path, sha256: proof.sha256,
    pendingNeeds: draft.contract.unresolvedNeeds, nativeValidity: 'Unknown', namespaceAuthority: 'Unknown',
    raceProof: false, proofAuthority: 'Missing', completed: false});
}
