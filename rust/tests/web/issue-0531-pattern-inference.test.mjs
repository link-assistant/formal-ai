// Issue #531 pattern inference, JavaScript first. The Rust twins:
//
// - rust/tests/unit/solver/issue_531_pattern_inference.rs: the browser worker's
//   tryPatternInference (js/worker/formal_ai_worker_pattern_inference.js),
//   entered through tryVerifiableTask exactly as the native verifiable-task
//   route enters try_pattern_inference, answers the same prompts with the same
//   reports in every seeded language (R403, R405).
// - rust/tests/unit/sequences_{patterns_1d,grid_2d,compression}.rs: the
//   worker's 1D classifier, grid transforms and lossless compression (R403).
// - rust/tests/unit/sequences_{store,symbols,converter,compression}.rs: the
//   link-native store js/agentic/crate/sequences.mjs (R402).
// - rust/tests/unit/issue_531_algorithm_discovery.rs: episodes inside one log
//   and held-out rejection in js/agentic/crate/algorithm_discovery.mjs
//   (R531-19, R531-21).
// - rust/tests/unit/docs_requirements/pattern_inference_research.rs: the research record
//   (R396-R401, R406, R531-25), checked here for the claims that test leaves out.

import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { before, describe, it } from 'node:test';

import { discoverAlgorithms, validatedCandidates } from '../../../js/agentic/crate/algorithm_discovery.mjs';
import { NULL_LINK, SequenceStore, SymbolTable, balancedConvert, compress } from '../../../js/agentic/crate/sequences.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

const ROOT = new URL('../../../', import.meta.url);
const read = (path) => readFileSync(new URL(path, ROOT), 'utf8');

let host;
let context;
before(async () => {
  host = new WorkerHost();
  context = await host.boot();
  await installNodeHost(new WorkerHost());
});

/** Evaluate `expression` in the worker realm and return its JSON value. */
const worker = (expression) => JSON.parse(evaluate(context, `JSON.stringify(${expression})`));

describe('the browser worker answers pattern-inference requests (R403, R405)', () => {
  it('detects a repetition and predicts the next element', async () => {
    const answer = await host.solve('find the pattern in 1 2 1 2 1 2');
    assert.equal(answer.intent, 'pattern_inference');
    assert.equal(answer.content, [
      'Sequence: 1 2 1 2 1 2',
      'Sequence of 6 element(s), 2 distinct.',
      'It is a repetition: a block of 2 element(s) repeated 3 times.',
      'Associative deduplication replaced 1 repeated pair(s), compressing to 50% of the original length (lossless).',
      'Most likely next element: 1.',
    ].join('\n'));
    for (const link of [
      'pattern_inference:kind:sequence',
      'pattern_inference:length:6',
      'pattern_inference:distinct:2',
      'pattern_inference:compression_ratio:0.50',
      'response:pattern_inference',
    ]) {
      assert.ok(answer.evidence.includes(link), link);
    }
  });

  it('reports a palindrome over letters', async () => {
    const answer = await host.solve('is the sequence A B B A a palindrome?');
    assert.equal(answer.intent, 'pattern_inference');
    assert.equal(answer.content, [
      'Sequence: A B B A',
      'Sequence of 4 element(s), 2 distinct.',
      'It is periodic with period 3.',
      'It reads the same forwards and backwards (palindrome).',
      'No repeated adjacent pairs to deduplicate.',
      'Most likely next element: B.',
    ].join('\n'));
  });

  it('predicts the next element of a constant run', async () => {
    const answer = await host.solve('what comes next in 7 7 7 7');
    assert.equal(answer.intent, 'pattern_inference');
    assert.equal(answer.content.split('\n').at(-1), 'Most likely next element: 7.');
    assert.equal(answer.content.split('\n')[2], 'Every element is identical (constant sequence).');
  });

  it('infers the symmetry of a grid', async () => {
    const answer = await host.solve('what is the pattern in this grid?\n1 2 1\n3 4 3');
    assert.equal(answer.intent, 'pattern_inference');
    assert.equal(answer.content, [
      'Grid pattern inference.',
      'Grid 2x3.',
      'Symmetric under: left-right mirror.',
      'Sequence of 6 element(s), 4 distinct.',
      'It has no exact repeating period.',
      'No repeated adjacent pairs to deduplicate.',
    ].join('\n'));
    assert.ok(answer.evidence.includes('pattern_inference:dimensions:2x3'));
    assert.ok(answer.evidence.includes('pattern_inference:symmetries:1'));
  });

  it('a definition question about the word pattern reaches the concept lookup, as natively', async () => {
    // definitional_question_routes_to_concept_lookup: the pattern expectation is
    // read only by the data-gated arm, so with no run of atoms nothing claims a
    // verifiable task (formal_ai_worker_claim_evidence.js `verifiable_spec`).
    const answer = await host.solve('what is a pattern?');
    assert.equal(answer.intent, 'concept_lookup');
    assert.ok(answer.evidence.includes('concept_lookup:hit:concept_pattern'));
    assert.equal(worker('recogniseBrowserVerifiableTask("what is a pattern?")'), null);
  });

  it('leaves prompts without a run of atoms to other handlers', async () => {
    for (const prompt of ['what is a pattern?', 'what pattern does issue 531 describe?', '1 2 1 2 1 2']) {
      assert.notEqual((await host.solve(prompt)).intent, 'pattern_inference', prompt);
    }
  });

  it('a response-language follow-up replays the report in every seeded language', async () => {
    const prior = 'find the pattern in 1 2 1 2 1 2';
    const english = await host.solve(prior);
    const history = [{ role: 'user', content: prior }, { role: 'assistant', content: english.content }];
    for (const [followUp, slug, line] of [
      ['answer in English', 'en', 'Most likely next element: 1.'],
      ['ответь на русском', 'ru', 'Наиболее вероятный следующий элемент: 1.'],
      ['हिंदी में उत्तर दें', 'hi', 'सबसे संभावित अगला तत्व: 1।'],
      ['用中文回答', 'zh', '最可能的下一个元素：1。'],
      ['responde en español', 'es', 'Elemento siguiente más probable: 1.'],
    ]) {
      const answer = await host.solve(followUp, history);
      assert.equal(answer.intent, 'pattern_inference', followUp);
      assert.equal(answer.content.split('\n').at(-1), line, slug);
      assert.ok(answer.evidence.includes(`language_to:${slug}`), slug);
    }
  });

  it('the symmetry separator is the seeded template in every language (grid_symmetries_join_with_the_seeded_separator)', () => {
    const separators = worker('["en", "ru", "hi", "zh", "es"].map((language) => answerFor("pattern_grid_symmetry_separator", language))');
    assert.deepEqual(separators, [', ', ', ', ', ', '、', ', ']);
    const grid = '({ rows: 2, cols: 3, cells: [1, 2, 1, 1, 2, 1] })';
    const lines = (language) => worker(`patternGridSummary(${grid}, patternInferSequence(patternStoreNew(), [1, 2, 1, 1, 2, 1]), "${language}").split("\\n")`);
    assert.equal(lines('en')[1], 'Symmetric under: left-right mirror, top-bottom mirror, 180-degree rotation.');
    assert.equal(lines('zh')[1], '对称于：左右镜像、上下镜像、180度旋转。');
  });

  it('the Chinese grid report joins its symmetries with the Chinese list comma', async () => {
    const prior = 'what is the pattern in this grid?\n1 2 1\n1 2 1';
    const english = await host.solve(prior);
    assert.equal(english.content.split('\n')[2], 'Symmetric under: left-right mirror, top-bottom mirror, 180-degree rotation.');
    const chinese = await host.solve('用中文回答', [{ role: 'user', content: prior }, { role: 'assistant', content: english.content }]);
    assert.equal(chinese.content.split('\n')[2], '对称于：左右镜像、上下镜像、180度旋转。');
  });
});

describe('the worker substrate (sequences/patterns_1d.rs, grid_2d.rs, compression.rs)', () => {
  it('classifies constant, repetition, periodic and aperiodic sequences', () => {
    assert.deepEqual(worker('patternClassify([])'), { kind: 'empty', period: 0, repetitions: 0 });
    assert.deepEqual(worker('patternClassify([5, 5, 5])'), { kind: 'constant', period: 0, repetitions: 0 });
    assert.deepEqual(worker('patternClassify([1, 2, 3, 1, 2, 3])'), { kind: 'repetition', period: 3, repetitions: 2 });
    assert.deepEqual(worker('patternClassify([1, 2, 1, 2, 1])'), { kind: 'periodic', period: 2, repetitions: 0 });
    assert.deepEqual(worker('patternClassify([1, 2, 3, 4])'), { kind: 'aperiodic', period: 0, repetitions: 0 });
    assert.equal(worker('patternPalindrome([1, 2, 3, 2, 1])'), true);
    assert.equal(worker('patternPalindrome([1, 2, 3])'), false);
  });

  it('detects reversal and cyclic translation (patterns_1d.rs reversal_and_translation)', () => {
    assert.equal(worker('patternIsReverse([1, 2, 3, 4], [4, 3, 2, 1])'), true);
    assert.equal(worker('patternIsReverse([1, 2, 3, 4], [1, 2, 3, 4])'), false);
    assert.equal(worker('patternTranslation([1, 2, 3, 4], [2, 3, 4, 1])'), 1);
    assert.equal(worker('patternTranslation([1, 2, 3, 4], [1, 2, 3, 4])'), 0);
    assert.equal(worker('patternTranslation([1, 2, 3, 4], [4, 3, 2, 1])'), null);
  });

  it('finds the transform that maps one grid onto another (grid_2d.rs transform_onto_detects_analogy)', () => {
    const grid = '({ rows: 2, cols: 3, cells: [1, 2, 3, 4, 5, 6] })';
    assert.equal(worker(`patternGridTransformOnto(${grid}, patternGridApply(${grid}, "rotate_cw"))`), 'rotate_cw');
    assert.equal(worker(`patternGridTransformOnto(${grid}, ${grid})`), 'identity');
    assert.equal(worker(`patternGridTransformOnto(${grid}, { rows: 2, cols: 3, cells: [9, 9, 9, 9, 9, 9] })`), null);
  });

  it('rotations round-trip and reflections and transposes move cells as the native grid does', () => {
    const grid = '({ rows: 2, cols: 3, cells: [1, 2, 3, 4, 5, 6] })';
    assert.deepEqual(worker(`patternGridApply(${grid}, "rotate_cw")`), { rows: 3, cols: 2, cells: [4, 1, 5, 2, 6, 3] });
    assert.deepEqual(worker(`patternGridApply(patternGridApply(${grid}, "rotate_cw"), "rotate_ccw")`), { rows: 2, cols: 3, cells: [1, 2, 3, 4, 5, 6] });
    assert.deepEqual(worker(`patternGridApply(${grid}, "rotate_180")`).cells, [6, 5, 4, 3, 2, 1]);
    assert.deepEqual(worker(`patternGridApply(${grid}, "reflect_horizontal")`).cells, [3, 2, 1, 6, 5, 4]);
    assert.deepEqual(worker(`patternGridApply(${grid}, "reflect_vertical")`).cells, [4, 5, 6, 1, 2, 3]);
    assert.deepEqual(worker(`patternGridApply(${grid}, "transpose")`), { rows: 3, cols: 2, cells: [1, 4, 2, 5, 3, 6] });
    assert.deepEqual(worker(`patternGridApply(${grid}, "anti_transpose")`), { rows: 3, cols: 2, cells: [6, 3, 5, 2, 4, 1] });
  });

  it('a square grid symmetric on both diagonals is invariant under the whole dihedral group', () => {
    const grid = '({ rows: 2, cols: 2, cells: [1, 1, 1, 1] })';
    assert.deepEqual(worker(`patternGridSymmetries(${grid})`), [
      'pattern_grid_horizontal', 'pattern_grid_vertical', 'pattern_grid_rotation', 'pattern_grid_diagonal', 'pattern_grid_anti_diagonal',
    ]);
    assert.equal(worker(`patternGridInvariantTransforms(${grid})`).length, 7);
    assert.deepEqual(worker('patternGridInvariantTransforms({ rows: 2, cols: 3, cells: [1, 2, 1, 3, 4, 3] })'), ['reflect_horizontal']);
  });

  it('compression replaces the most frequent pair, ties to the smallest, and expands losslessly', () => {
    const run = worker(`(() => {
      const store = patternStoreNew();
      const a = patternStorePoint(store), b = patternStorePoint(store), c = patternStorePoint(store);
      const sequence = [a, b, a, b, c, a, b];
      const result = patternCompress(store, sequence);
      return { steps: result.steps, sequence: result.sequence, lossless: patternCompressionIsLossless(store, result) };
    })()`);
    assert.deepEqual(run.steps[0], { source: 1, target: 2, replacement: 4, occurrences: 3 });
    assert.deepEqual(run.sequence, [4, 4, 3, 4]);
    assert.equal(run.lossless, true);
    assert.deepEqual(worker('patternCompress(patternStoreNew(), [1, 2, 3, 4]).steps'), []);
  });

  it('ratios print as Rust prints an f64: ties to even', () => {
    assert.equal(worker('patternFormatFixed(0.125, 2)'), '0.12');
    assert.equal(worker('patternFormatFixed(2.5, 0)'), '2');
    assert.equal(worker('patternFormatFixed(1.5, 0)'), '2');
    assert.equal(worker('patternFormatFixed(100 / 3, 0)'), '33');
    assert.equal(worker('patternFormatFixed(1, 2)'), '1.00');
  });
});

describe('the link-native sequence store in js/agentic/crate/sequences.mjs (R402)', () => {
  it('deduplicates pairs and keeps a self-pairing distinct from its point', () => {
    const store = new SequenceStore();
    const a = store.createPoint();
    const b = store.createPoint();
    assert.equal(store.getOrCreate(a, b), store.getOrCreate(a, b));
    assert.notEqual(store.getOrCreate(a, a), a);
    assert.deepEqual(store.expand(store.getOrCreate(a, a)), [a, a]);
    assert.deepEqual(store.expand(NULL_LINK), []);
  });

  it('markers are stable by name', () => {
    const store = new SequenceStore();
    const symbols = new SymbolTable();
    assert.equal(symbols.marker(store, 'boundary'), symbols.marker(store, 'boundary'));
    assert.notEqual(symbols.marker(store, 'boundary'), symbols.marker(store, 'other'));
  });

  it('the balanced converter round-trips and reuses repeated halves', () => {
    const store = new SequenceStore();
    const points = Array.from({ length: 4 }, () => store.createPoint());
    const sequence = [points[0], points[1], points[2], points[3], points[0], points[1], points[2], points[3]];
    const root = balancedConvert(store, sequence);
    assert.deepEqual(store.expand(root), sequence);
    const doublet = store.get(root);
    assert.equal(doublet[0], doublet[1], 'both halves are the same deduplicated link');
    assert.equal(balancedConvert(store, []), NULL_LINK);
    assert.equal(balancedConvert(store, [points[2]]), points[2]);
  });

  it('compression is lossless and breaks ties toward the smallest pair', () => {
    const store = new SequenceStore();
    const p = Array.from({ length: 4 }, () => store.createPoint());
    const result = compress(store, [p[0], p[1], p[2], p[3], p[0], p[1], p[2], p[3]]);
    assert.deepEqual([result.steps[0].source, result.steps[0].target], [p[0], p[1]]);
    assert.ok(result.isLossless(store));
    assert.equal(compress(store, [p[0], p[0], p[0], p[0]]).steps[0].occurrences, 2);
  });
});

describe('algorithm discovery in js/agentic/crate/algorithm_discovery.mjs (R531-19, R531-21)', () => {
  const step = (operation, args) => ({ operation, arguments: [...args].sort(([a], [b]) => (a < b ? -1 : 1)) });
  const trace = (id, subject) => ({
    id,
    steps: [step('fetch', [['subject', subject]]), step('normalize', [['subject', subject], ['format', 'json']]), step('persist', [['subject', subject]])],
  });

  it('a repeated episode inside one log is discovered without trace boundaries', () => {
    const single = { id: 'single-log', steps: ['read', 'verify', 'read', 'verify', 'read', 'verify'].map((name) => step(name, [])) };
    const candidate = validatedCandidates(discoverAlgorithms([single]))
      .find((entry) => entry.steps.map((entry) => entry.operation).join(' ') === 'read verify');
    assert.ok(candidate, 'the repeated read/verify episode should be inferred');
    assert.equal(candidate.support_trace_ids.length, 2);
    assert.equal(candidate.held_out.length, 1);
  });

  it('held-out missing or changed steps reject an incomplete algorithm', () => {
    const truncated = { id: 'run-gamma', steps: [step('fetch', [['subject', 'gamma']]), step('persist', [['subject', 'gamma']])] };
    const run = discoverAlgorithms([trace('run-alpha', 'alpha'), trace('run-beta', 'beta'), truncated]);
    const candidate = run.candidates.find((entry) => entry.steps.length === 3);
    assert.ok(candidate, 'two support traces should retain an inspectable proposal');
    assert.equal(candidate.held_out[0].passed, false);
    assert.ok(candidate.held_out[0].failures.some((failure) => failure.startsWith('operation_mismatch')));
    assert.ok(candidate.held_out[0].failures.some((failure) => failure.startsWith('missing_step')));
  });
});

describe('the issue #531 research record (R396-R401, R406, R531-25)', () => {
  it('the inventory names every integration point it surveys (R398)', () => {
    const readme = read('docs/case-studies/issue-531/README.md');
    for (const module of ['src/link_store.rs', 'src/substitution.rs', 'src/solver.rs', 'src/meta_core.rs', 'src/solver_handlers/text_manipulation.rs']) {
      assert.ok(readme.includes(`\`${module}\``), module);
    }
    const inventory = read('docs/case-studies/issue-531/architecture-inventory.md');
    for (const converter of ['LinkFrequenciesCache', 'SequenceIndex', 'StringToUnicodeSequenceConverter']) {
      assert.ok(inventory.includes(converter), converter);
    }
  });

  it('the converter research and the C#/C++ max-frequency discrepancy are recorded with their sources (R399, R400)', () => {
    const readme = read('docs/case-studies/issue-531/README.md');
    for (const converter of ['BalancedVariantConverter', 'OptimalVariantConverter', 'CompressingConverter']) {
      assert.ok(readme.includes(`\`${converter}\``), converter);
    }
    assert.ok(readme.includes('differ in the max-frequency'));
    assert.ok(read('docs/case-studies/issue-531/architecture-inventory.md').includes('maximum-frequency discrepancy'));
    assert.ok(read('docs/case-studies/issue-531/solution-plan.md').includes('max-frequency discrepancy'));
    for (const excerpt of ['csharp-compressing-converter.cs.txt', 'cpp-compressing-converter.h.txt']) {
      assert.ok(existsSync(new URL(`docs/case-studies/issue-531/raw-data/${excerpt}`, ROOT)), excerpt);
    }
  });

  it('Phase 7 defines the five benchmark directions and their fixtures exist (R406)', () => {
    const plan = read('docs/case-studies/issue-531/solution-plan.md');
    const phase = plan.slice(plan.indexOf('## Phase 7: Benchmarks'), plan.indexOf('## Phase 8'));
    for (const direction of [
      'text repeated phrase examples',
      'symbolic sequences with nested repetition',
      'event streams from portable memory',
      'ARC-AGI inspired grid examples',
      'requirements-to-solution fact-checking examples',
    ]) {
      assert.ok(phase.includes(direction), direction);
    }
    assert.ok(existsSync(new URL('data/benchmarks/issue-531-algorithm-traces.lino', ROOT)));
  });

  it('the README states what the learner does not synthesize (R531-25)', () => {
    const readme = read('docs/case-studies/issue-531/README.md');
    const boundary = readme.slice(readme.indexOf('## Generalization Boundary'));
    assert.ok(boundary.includes('It does not invent branch\npredicates, concurrent partial orders, recursion, or new host operations.'));
  });
});
