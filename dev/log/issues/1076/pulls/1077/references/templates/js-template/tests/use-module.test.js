/* eslint local/no-changelog-comments: "off" */

import { describe, it, expect } from 'test-anywhere';

import {
  describeModule,
  loadCommandStream,
  loadLinoArguments,
  loadUse,
  resolveNamedExport,
  USE_M_URL,
} from '../scripts/use-module.mjs';

const dollar = () => {};

function commandStreamExports() {
  return Object.assign(() => {}, { $: dollar, sh: () => {}, run: () => {} });
}

/**
 * Namespace `import()` produces for a CommonJS file on Node < 22.12. use-m
 * unwraps it because `default` is the only key, so `const { $ }` works.
 */
function nodeTwentyNamespace() {
  return { default: commandStreamExports() };
}

/**
 * Same file on Node >= 22.12: the synthetic `module.exports` key makes use-m
 * skip the unwrap and hand the raw namespace to the caller.
 */
function nodeTwentyFourNamespace() {
  return {
    default: commandStreamExports(),
    'module.exports': commandStreamExports(),
  };
}

function textResponse(
  body,
  { ok = true, status = 200, statusText = 'OK' } = {}
) {
  return {
    ok,
    status,
    statusText,
    async text() {
      return body;
    },
  };
}

describe('use-module interop shim', () => {
  it('reproduces the Node 24 namespace that breaks destructuring', () => {
    const { $ } = nodeTwentyFourNamespace();
    expect($).toBe(undefined);
  });

  it('resolves $ from the Node >= 22.12 CommonJS namespace', () => {
    const resolved = resolveNamedExport(
      nodeTwentyFourNamespace(),
      '$',
      'command-stream'
    );
    expect(typeof resolved.$).toBe('function');
  });

  it('resolves $ from the Node < 22.12 CommonJS namespace', () => {
    const resolved = resolveNamedExport(
      nodeTwentyNamespace(),
      '$',
      'command-stream'
    );
    expect(typeof resolved.$).toBe('function');
  });

  it('resolves $ when use-m already unwrapped the module', () => {
    const resolved = resolveNamedExport(
      commandStreamExports(),
      '$',
      'command-stream'
    );
    expect(typeof resolved.$).toBe('function');
  });

  it('resolves $ from a real ES module namespace with named exports', () => {
    const namespace = { $: dollar, sh: () => {}, default: dollar };
    const resolved = resolveNamedExport(namespace, '$', 'command-stream');
    expect(resolved).toBe(namespace);
  });

  it('resolves a double-wrapped default', () => {
    const resolved = resolveNamedExport(
      { default: { default: commandStreamExports() } },
      '$',
      'command-stream'
    );
    expect(typeof resolved.$).toBe('function');
  });

  it('names the observed keys when no candidate exposes the export', () => {
    let message = '';
    try {
      resolveNamedExport({ nope: 1 }, '$', 'command-stream');
    } catch (error) {
      message = error.message;
    }
    expect(message.includes("use('command-stream')")).toBe(true);
    expect(message.includes('keys [nope]')).toBe(true);
  });

  it('reports the received value when use-m resolved nothing', () => {
    let message = '';
    try {
      resolveNamedExport(undefined, '$', 'command-stream');
    } catch (error) {
      message = error.message;
    }
    expect(message.includes('Received undefined')).toBe(true);
  });

  it('describes null, primitives and objects', () => {
    expect(describeModule(null)).toBe('null');
    expect(describeModule(7)).toBe('number');
    expect(describeModule({ a: 1, b: 2 })).toBe('object with keys [a, b]');
  });

  it('loads command-stream through an injected use() implementation', async () => {
    const calls = [];
    const use = async (name) => {
      calls.push(name);
      return nodeTwentyFourNamespace();
    };
    const module = await loadCommandStream(use);
    expect(calls).toEqual(['command-stream']);
    expect(typeof module.$).toBe('function');
  });

  it('loads lino-arguments through an injected use() implementation', async () => {
    const calls = [];
    const use = async (name) => {
      calls.push(name);
      return { default: { makeConfig: () => ({}) } };
    };
    const module = await loadLinoArguments(use);
    expect(calls).toEqual(['lino-arguments']);
    expect(typeof module.makeConfig).toBe('function');
  });

  it('points at the unpinned use-m entry point', () => {
    expect(USE_M_URL).toBe('https://unpkg.com/use-m/use.js');
  });

  it('reports the HTTP status when use.js cannot be fetched', async () => {
    let message = '';
    try {
      await loadUse({
        fetchImpl: async () =>
          textResponse('<html>Not Found</html>', {
            ok: false,
            status: 404,
            statusText: 'Not Found',
          }),
      });
    } catch (error) {
      message = error.message;
    }
    expect(message.includes('404')).toBe(true);
    expect(message.includes('Not Found')).toBe(true);
  });

  it('reports a use.js payload that exposes no callable use()', async () => {
    let message = '';
    try {
      await loadUse({ fetchImpl: async () => textResponse('({ nope: 1 })') });
    } catch (error) {
      message = error.message;
    }
    expect(message.includes('did not export a callable "use"')).toBe(true);
  });

  it('evaluates a well-formed use.js payload', async () => {
    const use = await loadUse({
      fetchImpl: async () => textResponse('({ use: async () => ({}) })'),
    });
    expect(typeof use).toBe('function');
  });
});
