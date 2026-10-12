// Issue #1 preamble rows with no behaviour test of their own: R1/R5 (a
// symbolic engine with no neural-network inference or GPU requirement), R19
// (the issue research kept under docs/case-studies/issue-1) and R24 (the
// dataset script writes its records as Links Notation under data/). The
// behaviour rows of the same preamble are pinned elsewhere (see
// docs/requirements-traceability.md).

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const read = (relative) => readFileSync(join(ROOT, relative), 'utf8');

// Inference runtimes and tensor libraries, by their published package names.
const NEURAL_CRATES = ['candle-core', 'candle-nn', 'tch', 'ort', 'burn', 'tract-onnx', 'llama-cpp-2', 'cudarc', 'tensorflow', 'onnxruntime'];
const NEURAL_NPM = ['onnxruntime-node', 'onnxruntime-web', '@tensorflow/tfjs', '@tensorflow/tfjs-node', '@xenova/transformers', '@huggingface/transformers', 'node-llama-cpp'];

/** The dependency names of every `[*dependencies]` table in a Cargo manifest. */
function cargoDependencies(manifest) {
  const names = [];
  let inDependencies = false;
  for (const line of manifest.split('\n')) {
    const header = /^\s*\[([^\]]+)\]\s*$/.exec(line);
    if (header) {
      inDependencies = /(^|\.)(dev-|build-)?dependencies$/.test(header[1]);
      continue;
    }
    const entry = /^\s*([A-Za-z0-9_-]+)\s*=/.exec(line);
    if (inDependencies && entry) names.push(entry[1]);
  }
  return names;
}

describe('R1/R5: the engine is symbolic and needs no neural inference or GPU', () => {
  test('no Cargo manifest depends on an inference runtime or tensor library', () => {
    for (const manifest of ['rust/Cargo.toml', 'js/wasm-worker/Cargo.toml']) {
      if (!existsSync(join(ROOT, manifest))) continue;
      const dependencies = cargoDependencies(read(manifest));
      assert.ok(dependencies.length > 0, manifest);
      for (const name of NEURAL_CRATES) assert.ok(!dependencies.includes(name), `${manifest} depends on ${name}`);
    }
  });

  test('no npm manifest depends on an inference runtime', () => {
    for (const manifest of ['package.json', 'desktop/package.json', 'vscode/package.json', 'rust/tests/e2e/package.json']) {
      if (!existsSync(join(ROOT, manifest))) continue;
      const json = JSON.parse(read(manifest));
      const dependencies = { ...json.dependencies, ...json.devDependencies, ...json.optionalDependencies };
      for (const name of NEURAL_NPM) assert.ok(!(name in dependencies), `${manifest} depends on ${name}`);
    }
  });
});

describe('R19: the issue #1 research is kept in the repository', () => {
  test('the case study has a README and its raw GitHub data', () => {
    assert.ok(read('docs/case-studies/issue-1/README.md').length > 0);
    assert.ok(readdirSync(join(ROOT, 'docs/case-studies/issue-1/raw-data')).length > 0);
  });
});

describe('R24: the dataset script writes Links Notation under data/', () => {
  const script = read('scripts/download-datasets.rs');

  test('every file the script writes is a committed .lino file', () => {
    const written = [...script.matchAll(/data_dir\.join\("([^"]+\.lino)"\)/g)].map(([, path]) => `data/${path}`);
    assert.deepEqual(written, ['data/source-index.lino', 'data/seed/greetings.lino', 'data/seed/hello-world-programs.lino', 'data/seed/demo-dialogs.lino']);
    for (const path of written) assert.ok(read(path).trim().length > 0, path);
  });
});
