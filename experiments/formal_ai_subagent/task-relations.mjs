// R1188-U35: declare the category before execution; size never proves quality.
import { measureCodingRun, sourceDigest } from './coding-amplification.mjs';

const codingKinds = ['coding', 'self-coding'];
const rules = {
  'feature-implementation': ['output-larger', codingKinds],
  'test-implementation': ['output-larger', codingKinds],
  explanation: ['output-larger', ['other', 'mathematics']],
  proof: ['output-larger', ['mathematics']],
  decision: ['output-smaller', ['other']],
  calculation: ['output-smaller', ['mathematics']],
  extraction: ['output-smaller', ['other']],
  summarization: ['output-smaller', ['other']],
  repair: ['unrestricted', codingKinds],
  refactoring: ['unrestricted', codingKinds],
  deletion: ['unrestricted', codingKinds],
  translation: ['unrestricted', [...codingKinds, 'other']],
};
export const categoryRules = Object.freeze(Object.fromEntries(Object.entries(rules).map(([category, rule]) =>
  [category, Object.freeze({ expectedRelation: rule[0], taskKinds: Object.freeze([...rule[1]]) })])));
export const nearUniversalObjective = Object.freeze({ minimumTasks: 20, acceptedMatchingShare: 0.95 });

/** Persist this declaration before scheduling the attempt, alongside its task receipt. */
export function declareTask({ runId, taskKind, category, task }) {
  if (typeof runId !== 'string' || !runId || typeof task !== 'string' || !task) throw new TypeError('original task identity required');
  const rule = categoryRules[category];
  if (!rule || !rule.taskKinds.includes(taskKind)) throw new TypeError('declared category must fit the task kind');
  const declaration = { runId, taskKind, category, task, expectedRelation: rule.expectedRelation };
  return Object.freeze({ ...declaration, digest: sourceDigest(JSON.stringify(declaration)) });
}

function validateDeclaration(declaration) {
  const expected = declareTask(declaration);
  if (expected.digest !== declaration.digest || expected.expectedRelation !== declaration.expectedRelation) {
    throw new TypeError('task declaration changed');
  }
  return expected;
}

/** Exact response binding supplements the same fresh verifier used for code. */
export async function measureTaskRelation(declaration, run, verify) {
  validateDeclaration(declaration);
  if (run.runId !== declaration.runId || run.taskKind !== declaration.taskKind || run.task !== declaration.task) {
    throw new TypeError('run differs from its original task declaration');
  }
  const coding = codingKinds.includes(run.taskKind);
  if (!coding && (typeof run.response !== 'string' || run.changes.length !== 0)) {
    throw new TypeError('noncoding output requires an exact response and no code changes');
  }
  let responseBound = false;
  const measurement = await measureCodingRun(run, async changes => {
    const proof = await verify(changes, run.response);
    responseBound = !coding && proof?.responseSHA256 === sourceDigest(run.response);
    return proof;
  });
  const accepted = measurement.accepted && (coding || responseBound);
  const outputBytes = accepted ? coding ? measurement.autonomousNetCodeBytes : Buffer.byteLength(run.response, 'utf8') : null;
  const observedRelation = outputBytes === null ? 'unknown' : outputBytes > measurement.taskInputBytes ? 'output-larger'
    : outputBytes < measurement.taskInputBytes ? 'output-smaller' : 'equal';
  const expectedRelation = declaration.expectedRelation;
  return {
    ...measurement, accepted, category: declaration.category, declarationDigest: declaration.digest,
    expectedRelation, observedRelation, responseBound: coding ? null : responseBound,
    outputBytes, outputMeasure: coding ? 'autonomous-validated-net-code-proxy' : 'verified-response-utf8-bytes',
    outputPerOriginalTask: outputBytes === null ? null : outputBytes / measurement.taskInputBytes,
    outputPerObservedInput: outputBytes === null ? null : outputBytes / measurement.totalObservedInputBytes,
    matchesExpectation: expectedRelation === 'unrestricted' ? null : accepted && observedRelation === expectedRelation,
  };
}

/** The saved declarations retain failures and disallow retrospective category selection. */
export function summarizeTaskRelations(measurements, declarations) {
  const declared = new Map();
  for (const declaration of declarations) {
    validateDeclaration(declaration);
    if (declared.has(declaration.runId)) throw new TypeError('duplicate declared task');
    declared.set(declaration.runId, declaration);
  }
  const observed = new Set();
  for (const measurement of measurements) {
    const declaration = declared.get(measurement.runId);
    if (observed.has(measurement.runId)) throw new TypeError('duplicate measured task');
    observed.add(measurement.runId);
    if (!declaration || measurement.declarationDigest !== declaration.digest || measurement.taskKind !== declaration.taskKind
      || measurement.category !== declaration.category
      || measurement.expectedRelation !== declaration.expectedRelation) throw new TypeError('measurement differs from declared category');
  }
  if (observed.size !== declared.size) throw new TypeError('missing declared task');
  return Object.fromEntries(Object.entries(categoryRules).map(([category, rule]) => {
    const cohort = measurements.filter(measurement => measurement.category === category);
    const matching = cohort.filter(measurement => measurement.accepted && measurement.matchesExpectation === true).length;
    const share = rule.expectedRelation === 'unrestricted' || cohort.length === 0 ? null : matching / cohort.length;
    return [category, { expectedRelation: rule.expectedRelation, attempted: cohort.length,
      accepted: cohort.filter(measurement => measurement.accepted).length, acceptedMatching: matching,
      acceptedMatchingShare: share, sufficientCohort: cohort.length >= nearUniversalObjective.minimumTasks,
      meetsNearUniversalObjective: share === null || cohort.length < nearUniversalObjective.minimumTasks ? null
        : share >= nearUniversalObjective.acceptedMatchingShare }];
  }));
}
