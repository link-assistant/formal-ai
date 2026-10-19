import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import { test } from 'node:test';
const require = createRequire(import.meta.url),
  {
    parse
  } = require('@babel/parser');
const read = file => fs.readFileSync(new URL('../../../' + file, import.meta.url), 'utf8');
function nodes(source, predicate) {
  const ast = parse(source, {
      sourceType: 'module',
      plugins: ['jsx']
    }),
    out = [];
  function visit(node) {
    if (!node || typeof node !== 'object') return;
    if (predicate(node)) out.push(node);
    for (const [key, value] of Object.entries(node)) if (!['loc', 'start', 'end', 'extra'].includes(key)) {
      if (Array.isArray(value)) value.forEach(visit);else if (value && typeof value === 'object') visit(value);
    }
  }
  visit(ast);
  return out;
}
test('actual worker publishes one shared seed before held initialization without another fetch', {
  timeout: 1000
}, async () => {
  const source = read('js/worker/formal_ai_worker_solve.js');
  const found = nodes(source, node => node.type === 'FunctionDeclaration' && node.id?.name === 'loadSeed');
  assert.equal(found.length, 1);
  let calls = 0,
    resolveSeed;
  const earlySeed = new Promise(resolve => {
    resolveSeed = resolve;
  });
  const sent = [],
    seed = {
      raw: {
        'seed/handler-precedence.lino': 'held-out registry'
      },
      tools: [{
        id: 'held-out tool'
      }],
      agentInfo: {
        name: 'held-out metadata'
      }
    };
  const context = vm.createContext({
    self: {
      FormalAiSeed: {
        loadAll: async () => {
          calls++;
          return seed;
        }
      }
    },
    postMessage: message => {
      sent.push(message);
      resolveSeed();
    },
    installBrowserHandlerPrecedence: () => {},
    installWorkerHandlerRegistry: () => {},
    hydrateLinoSeedAndSourceCaches: () => new Promise(() => {})
  });
  vm.runInContext('let seedLoaded=false,seedLoadPromise=null,SEED_RAW={},seedSourceTexts={};\n' + source.slice(found[0].start, found[0].end) + '\nloadSeed();loadSeed();', context);
  await earlySeed;
  assert.equal(calls, 1);
  assert.equal(sent.length, 1);
  assert.equal(sent[0].kind, 'seed_loaded');
  assert.equal(sent[0].seed, seed);
  assert.equal(sent.some(message => message.kind === 'ready'), false, 'seed availability does not declare engine readiness');
});
test('actual UI consumes shared metadata while only ready enables the composer and cleanup terminates its owner', () => {
  const source = read('js/app/application-worker-hooks.jsx'),
    effects = nodes(source, node => node.type === 'CallExpression' && node.callee?.name === 'useEffect' && source.slice(node.start, node.end).includes('worker.onmessage ='));
  assert.equal(effects.length, 1);
  let worker,
    terminated = 0;
  const metadata = [],
    ready = [],
    status = [];
  const context = {
    Worker: class {
      constructor() {
        worker = this;
      }
      terminate() {
        terminated++;
      }
    },
    withAssetVersion: value => value,
    diagnosticJsFallback: false,
    workerRef: {
      current: null
    },
    setSeed: value => metadata.push(value),
    setWorkerState: value => status.push(value),
    setWorkerReady: value => ready.push(value),
    setEngineUnavailable: () => {},
    dispatchEvent: () => {},
    Event: class {},
    pendingResponses: {
      current: new Map()
    }
  };
  const cleanup = vm.runInNewContext('(' + source.slice(effects[0].arguments[0].start, effects[0].arguments[0].end) + ')', context)();
  const seed = {
    tools: [{
      id: 'independent source'
    }]
  };
  worker.onmessage({
    data: {
      kind: 'seed_loaded',
      seed
    }
  });
  assert.equal(metadata[0], seed);
  assert.deepEqual(ready, []);
  assert.deepEqual(status, []);
  worker.onmessage({
    data: {
      kind: 'ready',
      mode: 'wasm worker'
    }
  });
  assert.deepEqual(ready, [true]);
  assert.deepEqual(status, ['wasm worker']);
  cleanup();
  assert.equal(terminated, 1);
  const app = read('js/app/application.jsx');
  assert.equal(app.includes('window.FormalAiSeed.loadAll()'), false);
  assert.match(app, /disabled=\{demoMode \|\| !workerReady\}/u);
});
