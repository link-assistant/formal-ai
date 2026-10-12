// Source-bound structural observations are not semantic usefulness or provider usage.
import { readFileSync, lstatSync, realpathSync } from 'node:fs';
import { resolve, dirname, isAbsolute } from 'node:path';
import { parse } from '@babel/parser';
import { declareTask, categoryRules } from './task-relations.mjs';
import { measureCodingRun, sourceDigest } from './coding-amplification.mjs';
import { executeCandidateBoundOracle } from './candidate-bound-oracle.mjs';
import { tokenize } from '../../scripts/lib/translation-blockers.mjs';

const observations = new WeakMap();
const codingKinds = new Set(['coding', 'self-coding']);
const dispositions = new Set(['final', 'failed', 'refused', 'unfinished']);
const roles = new Set(['production', 'test', 'generated', 'evidence', 'documentation']);

function physical(binding) {
  if (!binding || !isAbsolute(binding.path) || !/^[a-f0-9]{64}$/u.test(binding.sha256)) {
    throw new Error('canonical physical binding required');
  }
  if (!lstatSync(binding.path).isFile() || realpathSync(binding.path) !== binding.path) {
    throw new Error('canonical regular file required');
  }
  const bytes = readFileSync(binding.path);
  if (bytes.length > 1048576 || sourceDigest(bytes) !== binding.sha256) {
    throw new Error('physical source drift or bound');
  }
  return bytes;
}

function declared(binding) {
  const value = JSON.parse(physical(binding));
  const task = declareTask(value.task);
  if (value.version !== 1 || JSON.stringify(task) !== JSON.stringify(value.task)) {
    throw new Error('frozen task declaration required');
  }
  const ids = value.expectedAttempts;
  if (!Array.isArray(ids) || !ids.length || ids.length > 128
    || ids.some(id => typeof id !== 'string' || !id) || new Set(ids).size !== ids.length) {
    throw new Error('complete declared attempt identities required');
  }
  return value;
}

function oracleView(contract) {
  const source = physical(contract.oracle).toString('utf8');
  const tree = parse(source, { sourceType: 'module' });
  const imports = tree.program.body.filter(node => node.type === 'ImportDeclaration')
    .filter(node => {
      const specifier = node.source.value;
      return (specifier.startsWith('.') || isAbsolute(specifier))
        && resolve(dirname(contract.oracle.path), specifier) === contract.candidate.path;
    });
  if (imports.length !== 1) throw new Error('one candidate-owned import required');
  const slot = imports[0].source;
  return source.slice(0, slot.start) + "'SOURCE_CANDIDATE'" + source.slice(slot.end);
}

function oraclePair(value) {
  const before = value.baselineContract;
  const after = value.candidateContract;
  const beforeDeclaration = JSON.parse(physical(before.declaration));
  const afterDeclaration = JSON.parse(physical(after.declaration));
  if (JSON.stringify(beforeDeclaration.interface) !== JSON.stringify(afterDeclaration.interface)
    || JSON.stringify(before.dependencies) !== JSON.stringify(after.dependencies)
    || oracleView(before) !== oracleView(after)) {
    throw new Error('original assertions or dependency closure changed');
  }
  return { before, after };
}

function literalBytes(source) {
  return tokenize(source).filter(token =>
    ['string', 'template', 'regex', 'number'].includes(token.kind))
    .reduce((sum, token) => sum + Buffer.byteLength(token.value), 0);
}

function issued(data) {
  const record = Object.freeze({ protocol: 'category-evidence-report-v1', runId: data.runId });
  observations.set(record, data);
  return record;
}

/** Execute frozen physical baseline/candidate oracles; caller flags cannot issue quality. */
export async function observeDeclaredCategoryEvidence(preregistration, attempts) {
  let task;
  const unknown = {
    disposition: 'refused', structuralOutcome: 'Unknown', semanticUsefulness: 'Unknown',
    usefulNetCodeBytes: null, autonomousUsefulNetCodeBytes: 0,
    completeModelInput: 'Unknown', providerAuthenticity: 'Unknown',
    modelUsage: 'Unknown', actualCost: null, monetarySavings: null,
    independentSemanticReview: 'Missing', paddingReview: 'Unknown',
    preExecutionTiming: 'Unknown', authorshipAuthenticity: 'Unknown',
  };
  try {
    const registration = Object.freeze({
      path: preregistration?.path, sha256: preregistration?.sha256,
    });
    const value = declared(registration);
    task = value.task;
    if (!Array.isArray(attempts) || attempts.length !== value.expectedAttempts.length) {
      throw new Error('retain every declared retry');
    }
    const byId = new Map();
    let observedRequestBytes = 0;
    let observedOutputBytes = 0;
    let suppliedInputBytes = 0;
    const inputs = [];
    const outputs = [];
    const supplied = [];
    for (const attempt of attempts) {
      if (!value.expectedAttempts.includes(attempt.attemptId) || byId.has(attempt.attemptId)
        || !['autonomous', 'reviewed-patch', 'generator'].includes(attempt.origin)
        || !Array.isArray(attempt.suppliedInputs) || attempt.suppliedInputs.length > 128) {
        throw new Error('unique declared retry and origin required');
      }
      const inputBytes = physical(attempt.input);
      const outputBytes = physical(attempt.output);
      const input = JSON.parse(inputBytes);
      const output = JSON.parse(outputBytes);
      if (input.runId !== task.runId || output.runId !== task.runId
        || input.attemptId !== attempt.attemptId || output.attemptId !== attempt.attemptId
        || typeof input.task !== 'string' || !Array.isArray(input.observedContext)
        || input.observedContext.some(text => typeof text !== 'string')
        || !dispositions.has(output.disposition)) {
        throw new Error('source-bound attempt identity or disposition drift');
      }
      if (attempt.attemptId === value.expectedAttempts[0] && input.task !== task.task) {
        throw new Error('retain original first task');
      }
      const suppliedBytes = attempt.suppliedInputs.map(binding => physical(binding));
      observedRequestBytes += inputBytes.length;
      observedOutputBytes += outputBytes.length;
      suppliedInputBytes += suppliedBytes.reduce((sum, bytes) => sum + bytes.length, 0);
      inputs.push(inputBytes.toString('utf8'));
      outputs.push(outputBytes.toString('utf8'));
      supplied.push(...suppliedBytes.map(bytes => bytes.toString('utf8')));
      byId.set(attempt.attemptId, {
        attemptId: attempt.attemptId, input, output, origin: attempt.origin,
        suppliedInputBytes: suppliedBytes.reduce((sum, bytes) => sum + bytes.length, 0),
      });
    }
    const pair = oraclePair(value);
    if (!Array.isArray(value.changes) || value.changes.length !== 1) {
      throw new Error('one physically bound candidate change required');
    }
    const change = value.changes[0];
    if (change.path !== pair.after.candidate.path || change.language !== 'javascript'
      || !roles.has(change.role)
      || JSON.stringify(change.before) !== JSON.stringify(pair.before.candidate)
      || JSON.stringify(change.after) !== JSON.stringify(pair.after.candidate)) {
      throw new Error('requested physical candidate effect differs');
    }
    const beforeBytes = physical(change.before);
    const afterBytes = physical(change.after);
    const beforeSource = beforeBytes.toString('utf8');
    const afterSource = afterBytes.toString('utf8');
    const baselineOracle = executeCandidateBoundOracle(pair.before);
    const candidateOracle = executeCandidateBoundOracle(pair.after);
    const ordered = value.expectedAttempts.map(id => byId.get(id));
    const final = ordered.at(-1);
    const coding = codingKinds.has(task.taskKind);
    const measurement = await measureCodingRun({
      runId: task.runId, taskKind: task.taskKind, origin: 'reviewed-patch',
      task: task.task, attemptInputs: ordered.map(row => row.input.task),
      repositoryContext: inputs, reviewedPatches: supplied, toolReceipts: outputs,
      changes: coding ? [{
        path: change.path, language: 'javascript', role: change.role,
        before: beforeSource, after: afterSource,
      }] : [],
    }, async changes => ({
      passed: candidateOracle.structuralComplete === true,
      bindings: changes.map(row => ({
        path: row.path, before: sourceDigest(row.before), after: sourceDigest(row.after),
      })),
      checks: [{ name: 'physical candidate-consuming original oracle',
        exitCode: candidateOracle.structuralComplete === true ? 0 : null }],
    }));
    physical(registration);
    for (const attempt of attempts) {
      physical(attempt.input); physical(attempt.output);
      for (const binding of attempt.suppliedInputs) physical(binding);
    }
    physical(change.before); physical(change.after);
    const passed = candidateOracle.structuralComplete === true;
    const response = typeof final.output.response === 'string' ? final.output.response : null;
    return issued({
      ...unknown, runId: task.runId, taskKind: task.taskKind, category: task.category,
      expectedRelation: task.expectedRelation, declarationDigest: task.digest,
      preregistration: registration, expectedAttempts: [...value.expectedAttempts],
      disposition: final.output.disposition,
      structuralOutcome: passed ? 'PassedOracle'
        : candidateOracle.process?.status !== undefined ? 'FailedOracle' : 'RefusedOracle',
      baselineOracle, candidateOracle, declarationTaskBytes: Buffer.byteLength(task.task),
      observedRequestBytes, observedOutputBytes, suppliedInputBytes,
      declaredRetryCoverage: 'CompleteDeclaredSubset',
      declaredAttempts: ordered.map(row => ({
        attemptId: row.attemptId, disposition: row.output.disposition,
        origin: row.origin, suppliedInputBytes: row.suppliedInputBytes,
      })),
      requestedPhysicalChangeObserved: beforeSource !== afterSource,
      rawPhysicalByteDelta: afterBytes.length - beforeBytes.length,
      lexicalNetCodeProxy: coding ? measurement.rawNetCodeBytes : null,
      literalTokenByteDelta: literalBytes(afterSource) - literalBytes(beforeSource),
      removedCodeProxy: coding ? measurement.removedCodeBytes : null,
      responseBytes: coding || response === null ? null : Buffer.byteLength(response),
      responseSemanticAcceptance: 'Unknown',
      outputMeasure: coding ? 'lexical-net-code-proxy' : 'observed-response-bytes',
      suppliedOrGenerated: ordered.some(row =>
        row.origin !== 'autonomous' || row.suppliedInputBytes > 0),
      autonomousCredit: 0,
    });
  } catch (error) {
    return issued({
      ...unknown, runId: task?.runId ?? null, taskKind: task?.taskKind ?? null,
      category: task?.category ?? null, declarationDigest: task?.digest ?? null,
      error: String(error?.message ?? error),
    });
  }
}

/** Read-only clone; changing a returned report cannot alter the issued observation. */
export function categoryEvidenceObservation(record) {
  const data = observations.get(record);
  return data ? JSON.parse(JSON.stringify(data)) : null;
}

/** Keep every declared row; unissued/missing observations are explicit refusals. */
export function summarizeDeclaredCategoryEvidence(preregistrations, records) {
  if (!Array.isArray(preregistrations) || !Array.isArray(records)) {
    throw new Error('complete declared inventory required');
  }
  const bindings = preregistrations.map(binding => Object.freeze({
    path: binding?.path, sha256: binding?.sha256,
  }));
  const declarations = bindings.map(binding => declared(binding));
  const ids = declarations.map(value => value.task.runId);
  if (new Set(ids).size !== ids.length || records.length !== ids.length) {
    throw new Error('unique complete declared inventory required');
  }
  const cases = declarations.map((value, index) => {
    const data = observations.get(records[index]);
    if (!data || data.runId !== value.task.runId
      || data.declarationDigest !== value.task.digest
      || data.preregistration?.path !== bindings[index].path
      || data.preregistration?.sha256 !== bindings[index].sha256
      || JSON.stringify(data.expectedAttempts) !== JSON.stringify(value.expectedAttempts)) {
      return { runId: value.task.runId, taskKind: value.task.taskKind,
        category: value.task.category, expectedRelation: value.task.expectedRelation,
        disposition: 'refused', structuralOutcome: 'Unknown', reason: 'UnissuedOrForeignObservation' };
    }
    return categoryEvidenceObservation(records[index]);
  });
  const summarize = rows => {
    const successes = rows.filter(row =>
      codingKinds.has(row.taskKind) && row.disposition === 'final'
      && row.structuralOutcome === 'PassedOracle' && row.suppliedOrGenerated !== true);
    return {
      declared: rows.length, observedStructuralSuccessDenominator: successes.length,
      observedPositiveLexicalGrowth: successes.filter(row => row.lexicalNetCodeProxy > 0).length,
      final: rows.filter(row => row.disposition === 'final').length,
      failed: rows.filter(row => row.disposition === 'failed').length,
      refused: rows.filter(row => row.disposition === 'refused').length,
      failedOracle: rows.filter(row => row.structuralOutcome === 'FailedOracle').length,
      refusedOracle: rows.filter(row => row.structuralOutcome === 'RefusedOracle').length,
      suppliedOrGenerated: rows.filter(row => row.suppliedOrGenerated === true).length,
      unfinished: rows.filter(row => row.disposition === 'unfinished').length,
      unknown: rows.filter(row => row.structuralOutcome === 'Unknown').length,
      usefulExpansionDenominator: 0, usefulExpansionShare: null,
    };
  };
  const coding = cases.filter(row => codingKinds.has(row.taskKind));
  const expansion = coding.filter(row => categoryRules[row.category]?.expectedRelation === 'output-larger');
  return {
    declared: cases.length, cases,
    expansion: { combined: summarize(expansion),
      ordinaryCoding: summarize(expansion.filter(row => row.taskKind === 'coding')),
      selfCoding: summarize(expansion.filter(row => row.taskKind === 'self-coding')) },
    contraction: summarize(cases.filter(row => categoryRules[row.category]?.expectedRelation === 'output-smaller')),
    unrestricted: summarize(cases.filter(row => categoryRules[row.category]?.expectedRelation === 'unrestricted')),
    dispositions: summarize(cases),
    semanticUsefulness: 'Unknown', autonomousUsefulNetCodeBytes: 0,
    completeModelInput: 'Unknown', providerAuthenticity: 'Unknown',
    modelUsage: 'Unknown', actualCost: null, monetarySavings: null,
    paddingReview: 'Unknown', noGrowthFloor: true, representativeBaseline: 'Unknown',
    categoryQualifiedStructuralObservationsOnly: true,
  };
}
