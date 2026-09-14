import assert from 'node:assert/strict';
import test from 'node:test';

import {
  ACCEPTED_STATUS,
  classifyFailures,
  isAccepted,
  parseAcceptList,
  probe,
  recheckAll,
} from './recheck-broken-links.mjs';
import { extractFailures } from './check-web-archive.mjs';

/** The line that reddened run 34134986294, copied out of its log verbatim. */
const RESET_REPORT = `## Errors per input

### Errors in docs/benchmarks.md

* [ERROR] <https://allenai.org/data/arc> (at 226:27) | Network error: Connection reset by peer (os error 104)
`;

const MISSING_PAGE_REPORT = `## Errors per input

### Errors in docs/a.md

* [404] <https://gone.example/page> (at 1:1) | Rejected status code: 404
`;

const ok = (status = 200) => ({ status, body: null });

test('a connection reset is a failure with no answer in it', () => {
  const { answered, unanswered } = classifyFailures(
    extractFailures(RESET_REPORT)
  );

  assert.deepEqual(answered, []);
  assert.equal(unanswered.length, 1);
  assert.equal(unanswered[0].url, 'https://allenai.org/data/arc');
  assert.match(unanswered[0].detail, /Connection reset by peer/);
});

test('a status code from the host is an answer and is never re-checked', () => {
  const { answered, unanswered } = classifyFailures(
    extractFailures(MISSING_PAGE_REPORT)
  );

  assert.deepEqual(unanswered, []);
  assert.equal(answered.length, 1);
  assert.equal(answered[0].url, 'https://gone.example/page');
});

test('a timeout and an unknown outcome are also unanswered', () => {
  const report = `## Timeouts per input

### Timeouts in docs/a.md

* [TIMEOUT] <https://slow.example/a> (at 1:1) | Request timed out

## Unknown per input

### Unknown in docs/b.md

* [UNKNOWN] <https://odd.example/b> (at 2:1) | Something happened
`;

  const { answered, unanswered } = classifyFailures(extractFailures(report));

  assert.deepEqual(answered, []);
  assert.deepEqual(
    unanswered.map((failure) => failure.url),
    ['https://slow.example/a', 'https://odd.example/b']
  );
});

test('the accept list is read exactly as lychee reads it', () => {
  const ranges = parseAcceptList(ACCEPTED_STATUS);

  for (const status of [200, 201, 204, 429, 500, 503, 599]) {
    assert.ok(isAccepted(status, ranges), `${status} should be accepted`);
  }
  // The whole point of the gate: a page that is not there is not a link.
  for (const status of [301, 400, 403, 404, 410, 418]) {
    assert.ok(!isAccepted(status, ranges), `${status} must not be accepted`);
  }
  assert.deepEqual(parseAcceptList('200..204'), [{ from: 200, to: 203 }]);
});

test('a URL that answers on a later attempt is recovered', async () => {
  const seen = [];
  let attempt = 0;
  const fetchImpl = async (url) => {
    seen.push(url);
    attempt += 1;
    if (attempt < 3) {
      throw new Error('Connection reset by peer (os error 104)');
    }
    return ok();
  };

  const waits = [];
  const results = await recheckAll(['https://allenai.org/data/arc'], {
    fetchImpl,
    sleep: async (ms) => waits.push(ms),
  });

  const result = results.get('https://allenai.org/data/arc');
  assert.equal(result.outcome, 'alive');
  assert.equal(result.detail, 'HTTP 200');
  assert.equal(result.attempts, 3);
  assert.equal(seen.length, 3);
  // Doubling: asking again immediately asks inside the same bad moment.
  assert.deepEqual(waits, [5000, 10000]);
});

test('a URL nothing ever answers for stays broken', async () => {
  const fetchImpl = async () => {
    throw new Error('Connection reset by peer (os error 104)');
  };

  const results = await recheckAll(['https://dead.example/page'], {
    fetchImpl,
    sleep: async () => {},
  });

  const result = results.get('https://dead.example/page');
  assert.equal(result.outcome, 'unreachable');
  assert.equal(result.attempts, 3);
  assert.match(result.detail, /Connection reset/);
});

test('a host that answers 404 on the re-check is not retried and not recovered', async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls += 1;
    return ok(404);
  };

  const results = await recheckAll(['https://gone.example/page'], {
    fetchImpl,
    sleep: async () => {},
  });

  assert.equal(calls, 1, 'a verdict from the host is final');
  assert.equal(results.get('https://gone.example/page').outcome, 'rejected');
});

test('every URL is asked once before any URL is asked twice', async () => {
  const order = [];
  const fetchImpl = async (url) => {
    order.push(url);
    throw new Error('Connection reset by peer (os error 104)');
  };

  await recheckAll(['https://a.example/', 'https://b.example/'], {
    fetchImpl,
    attempts: 2,
    sleep: async () => {},
  });

  assert.deepEqual(order, [
    'https://a.example/',
    'https://b.example/',
    'https://a.example/',
    'https://b.example/',
  ]);
});

test('the budget stops the re-check without claiming a recovery', async () => {
  let clock = 0;
  const fetchImpl = async () => {
    clock += 40000;
    throw new Error('Connection reset by peer (os error 104)');
  };

  const results = await recheckAll(['https://a.example/', 'https://b.example/'], {
    fetchImpl,
    budgetMs: 30000,
    now: () => clock,
    sleep: async () => {},
  });

  // The first URL consumed the budget; the second was never asked, and an
  // unasked URL is reported as still failing rather than as recovered.
  assert.equal(results.get('https://a.example/').attempts, 1);
  assert.equal(results.get('https://b.example/').attempts, 0);
  assert.equal(results.get('https://b.example/').outcome, 'unreachable');
  assert.match(results.get('https://b.example/').detail, /budget ran out/);
});

test('a request that never returns is abandoned rather than hanging the job', async () => {
  const fetchImpl = (url, options) =>
    new Promise((_resolve, reject) => {
      options.signal.addEventListener('abort', () => {
        const error = new Error('aborted');
        error.name = 'AbortError';
        reject(error);
      });
    });

  const result = await probe('https://silent.example/', {
    fetchImpl,
    timeoutMs: 10,
  });

  assert.equal(result.outcome, 'unreachable');
  assert.equal(result.detail, 'no response within 0s');
});

test('the re-check sends the same user agent lychee sent', async () => {
  let headers = null;
  const fetchImpl = async (url, options) => {
    headers = options.headers;
    return ok();
  };

  await probe('https://example.com/', { fetchImpl });

  assert.match(headers['User-Agent'], /^formal-ai-link-checker\//);
});

test('end to end: the run that reddened this branch stops failing', async (t) => {
  const { mkdtempSync, writeFileSync, readFileSync, existsSync } = await import(
    'node:fs'
  );
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { main } = await import('./recheck-broken-links.mjs');

  const dir = mkdtempSync(join(tmpdir(), 'recheck-'));
  const report = join(dir, 'out.md');
  const recovered = join(dir, 'recovered.txt');
  const outputs = join(dir, 'github-output');
  writeFileSync(report, RESET_REPORT);
  writeFileSync(outputs, '');

  const savedFetch = globalThis.fetch;
  const savedEnv = { ...process.env };
  const lines = [];
  t.mock.method(console, 'log', (line) => lines.push(String(line)));

  globalThis.fetch = async () => ok();
  process.env.LYCHEE_OUTPUT = report;
  process.env.RECOVERED_OUTPUT = recovered;
  process.env.GITHUB_OUTPUT = outputs;

  try {
    await main();
  } finally {
    globalThis.fetch = savedFetch;
    process.env = savedEnv;
  }

  const written = readFileSync(outputs, 'utf-8');
  assert.match(written, /all_recovered=true/);
  assert.match(written, /recovered=1/);
  assert.match(written, /remaining=0/);
  assert.ok(existsSync(recovered));
  assert.equal(
    readFileSync(recovered, 'utf-8'),
    'https://allenai.org/data/arc\n'
  );
  assert.ok(
    lines.some((line) => line.includes('::notice title=Link answered on re-check::')),
    'the recovery has to be visible in the run, not silent'
  );
});

test('end to end: a link that is really gone still fails the build', async (t) => {
  const { mkdtempSync, writeFileSync, readFileSync } = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { main } = await import('./recheck-broken-links.mjs');

  const dir = mkdtempSync(join(tmpdir(), 'recheck-'));
  const report = join(dir, 'out.md');
  const outputs = join(dir, 'github-output');
  writeFileSync(report, MISSING_PAGE_REPORT);
  writeFileSync(outputs, '');

  const savedFetch = globalThis.fetch;
  const savedEnv = { ...process.env };
  t.mock.method(console, 'log', () => {});

  globalThis.fetch = async () => {
    throw new Error('the host that answered 404 must not be asked again');
  };
  process.env.LYCHEE_OUTPUT = report;
  process.env.RECOVERED_OUTPUT = join(dir, 'recovered.txt');
  process.env.GITHUB_OUTPUT = outputs;

  try {
    await main();
  } finally {
    globalThis.fetch = savedFetch;
    process.env = savedEnv;
  }

  const written = readFileSync(outputs, 'utf-8');
  assert.match(written, /all_recovered=false/);
  assert.match(written, /remaining=1/);
});

test('end to end: a missing report claims nothing', async (t) => {
  const { mkdtempSync, writeFileSync, readFileSync } = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { main } = await import('./recheck-broken-links.mjs');

  const dir = mkdtempSync(join(tmpdir(), 'recheck-'));
  const outputs = join(dir, 'github-output');
  writeFileSync(outputs, '');

  const savedEnv = { ...process.env };
  t.mock.method(console, 'log', () => {});
  process.env.LYCHEE_OUTPUT = join(dir, 'absent.md');
  process.env.GITHUB_OUTPUT = outputs;

  try {
    await main();
  } finally {
    process.env = savedEnv;
  }

  assert.match(readFileSync(outputs, 'utf-8'), /all_recovered=false/);
});
