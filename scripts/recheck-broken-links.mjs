#!/usr/bin/env node

/**
 * Re-check transient HTTP and transport failures within a fixed budget.
 *
 * Issue #1081: run 34134986294 reddened this branch on one link,
 *
 *   [ERROR] https://allenai.org/data/arc (at 226:27)
 *           | Network error: Connection reset by peer (os error 104)
 *
 * a page that answers 200 on every attempt from anywhere else. Issue #1045 met
 * the same symptom on four links and answered it with `--max-retries 6
 * --retry-wait-time 2`, on the reasoning that a reset carries no status code,
 * so retrying is the only lever left. The retries never ran: lychee reported
 * this one 1.5 seconds into the step, and six retries with a growing wait
 * cannot fit in 1.5 seconds.
 *
 * lychee's own source says why. `lychee-lib/src/retry.rs` decides retryability
 * before it looks at the error kind:
 *
 *   } else if self.is_connect() {
 *       false
 *
 *   fn should_retry_io(error: &io::Error) -> bool {
 *       matches!(error.kind(),
 *           io::ErrorKind::ConnectionReset | ConnectionAborted | TimedOut)
 *   }
 *
 * A reset during the connect or TLS handshake takes the first branch and is
 * never retried, while the very same `ConnectionReset` reaching the body path
 * is listed as retryable. `--max-retries` is inert for this entire class of
 * failure, which is the class #1045 raised it for.
 *
 * So the retry has to live outside lychee. This script reads lychee's report
 * and retries transport failures and transient HTTP 408, 429 and 5xx responses,
 * spread over time within a fixed budget. Only a later successful response
 * recovers the URL. Permanent HTTP failures such as 404 remain broken without
 * retries; an exhausted transient failure also remains broken.
 *
 * Usage:
 *   node scripts/recheck-broken-links.mjs
 *
 * Environment variables:
 *   - LYCHEE_OUTPUT: lychee markdown report (default: lychee/out.md)
 *   - RECOVERED_OUTPUT: where to write the recovered URLs, one per line
 *     (default: lychee/recovered.txt)
 *   - RECHECK_ATTEMPTS: attempts per URL (default: 3)
 *   - RECHECK_WAIT_SECONDS: wait between rounds (default: 5, doubling)
 *   - RECHECK_TIMEOUT_SECONDS: per-request timeout (default: 20)
 *   - RECHECK_BUDGET_SECONDS: total wall clock for all re-checks (default: 180)
 *   - RECHECK_VERBOSE: '1' to log every attempt (default: off)
 *
 * GitHub Actions outputs:
 *   - all_recovered: 'true' only when every link lychee reported answered here
 *   - recovered / remaining: counts, for the log
 *
 * Exit code is 0 whether or not links remain broken. This script downgrades
 * failures, it does not raise them; the workflow decides what to do with
 * `all_recovered`, and an unset output reads as 'not recovered'.
 */

import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'fs';
import { dirname } from 'path';
import { pathToFileURL } from 'url';

import { extractFailures, extractNonHttpFailures, validateLycheeReport } from './check-web-archive.mjs';

/**
 * The status codes this re-check treats as "the link resolves".
 *
 * Identical to `--accept` in `.github/workflows/links.yml`, and pinned to it by
 * a test: two components deciding what "broken" means, by different rules, is
 * how a link starts passing one gate and failing the next. Transient errors
 * qualify for bounded retries, but cannot establish that the page resolves.
 */
export const ACCEPTED_STATUS = '200..=204';

function isTransientStatus(status) {
  return status === 408 || status === 429 || (status >= 500 && status <= 599);
}

/**
 * Sent by the workflow's lychee invocation, and sent again here so a host that
 * rate-limits by user agent treats both requests the same way.
 */
export const USER_AGENT =
  'formal-ai-link-checker/1.0 (+https://github.com/link-assistant/formal-ai)';

/**
 * Parse a lychee `--accept` list into inclusive ranges.
 * Accepts `200..=204` (inclusive), `200..204` (exclusive) and bare `429`.
 * @param {string} spec - e.g. '200..=204,429,500..=599'
 * @returns {{from: number, to: number}[]}
 */
export function parseAcceptList(spec) {
  return spec
    .split(',')
    .map((part) => part.trim())
    .filter((part) => part.length > 0)
    .map((part) => {
      const inclusive = /^(\d+)\.\.=(\d+)$/.exec(part);
      if (inclusive) {
        return { from: Number(inclusive[1]), to: Number(inclusive[2]) };
      }
      const exclusive = /^(\d+)\.\.(\d+)$/.exec(part);
      if (exclusive) {
        return { from: Number(exclusive[1]), to: Number(exclusive[2]) - 1 };
      }
      if (!/^\d+$/.test(part)) {
        throw new Error(`unrecognised status range in accept list: ${part}`);
      }
      return { from: Number(part), to: Number(part) };
    });
}

/**
 * @param {number} status - An HTTP status code
 * @param {{from: number, to: number}[]} ranges - From `parseAcceptList`
 * @returns {boolean}
 */
export function isAccepted(status, ranges) {
  return ranges.some((range) => status >= range.from && status <= range.to);
}

/**
 * Separate final HTTP verdicts from failures eligible for bounded retries.
 *
 * Keep the existing `answered` / `unanswered` result keys for callers. Permanent
 * HTTP failures such as 404 go into `answered`; transient 408, 429 and 5xx join
 * transport errors in `unanswered`. A numeric marker takes precedence over a
 * contradictory detail. Retrying never converts a failure into success unless
 * a later probe actually receives a status in ACCEPTED_STATUS.
 *
 * @param {{url: string, marker: string|null, detail: string}[]} failures
 * @returns {{answered: object[], unanswered: object[]}}
 */
export function classifyFailures(failures) {
  const answered = [];
  const unanswered = [];

  for (const failure of failures) {
    const numericMarker = /^\d{3}$/.test(failure.marker ?? '');
    const statusInDetail = /rejected status code/i.test(failure.detail ?? '');
    // Retrying a transient response requires a later successful observation.
    const status = numericMarker ? Number(failure.marker) :
      Number(/rejected status code:\s*(\d{3})/iu.exec(failure.detail ?? '')?.[1]);
    if ((numericMarker || statusInDetail) && !isTransientStatus(status)) {
      answered.push(failure);
    } else {
      unanswered.push(failure);
    }
  }

  return { answered, unanswered };
}

/**
 * Ask one URL once.
 *
 * Only 'alive' clears a link. 'rejected' is a permanent HTTP failure and ends
 * retries. 'retryable' is a transient HTTP failure; 'unreachable' is a transport
 * failure. Both may be retried within the unchanged attempt and time budgets.
 *
 * @param {string} url
 * @param {{fetchImpl?: typeof fetch, timeoutMs?: number, accept?: {from: number, to: number}[]}} [options]
 * @returns {Promise<{outcome: 'alive'|'rejected'|'unreachable'|'retryable', detail: string}>}
 */
export async function probe(url, options = {}) {
  const {
    fetchImpl = fetch,
    timeoutMs = 20000,
    accept = parseAcceptList(ACCEPTED_STATUS),
  } = options;

  const controller = new AbortController();
  const timer = globalThis.setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetchImpl(url, {
      redirect: 'follow',
      headers: { 'User-Agent': USER_AGENT },
      signal: controller.signal,
    });

    // Nothing here reads the body; releasing it keeps the connection from
    // being held open for the rest of the run.
    await response.body?.cancel?.().catch(() => {});

    if (isTransientStatus(response.status)) {
      return { outcome: 'retryable', detail: `HTTP ${response.status}` };
    }
    return isAccepted(response.status, accept)
      ? { outcome: 'alive', detail: `HTTP ${response.status}` }
      : { outcome: 'rejected', detail: `HTTP ${response.status}` };
  } catch (error) {
    return {
      outcome: 'unreachable',
      detail:
        error.name === 'AbortError'
          ? `no response within ${Math.round(timeoutMs / 1000)}s`
          : error.message,
    };
  } finally {
    globalThis.clearTimeout(timer);
  }
}

/**
 * Re-check every URL, one full round at a time.
 *
 * Round-robin rather than URL-by-URL: under a total budget, a first answer for
 * every link is worth more than a third attempt at the first link. It also
 * spaces each URL's own attempts apart by a whole round, which is the point --
 * a burst of resets is outlasted by asking later, not by asking again
 * immediately.
 *
 * @param {string[]} urls
 * @param {object} [options] - Injectable clock, sleep and fetch, for tests
 * @returns {Promise<Map<string, {outcome: string, detail: string, attempts: number}>>}
 */
export async function recheckAll(urls, options = {}) {
  const {
    fetchImpl = fetch,
    attempts = 3,
    waitMs = 5000,
    timeoutMs = 20000,
    budgetMs = 180000,
    now = () => Date.now(),
    sleep = (ms) => new Promise((resolve) => globalThis.setTimeout(resolve, ms)),
    accept = parseAcceptList(ACCEPTED_STATUS),
    log = () => {},
  } = options;

  const started = now();
  const results = new Map(
    urls.map((url) => [
      url,
      {
        outcome: 'unreachable',
        detail: 'the re-check budget ran out before this URL was asked',
        attempts: 0,
      },
    ])
  );

  let pending = [...urls];

  for (let round = 1; round <= attempts && pending.length > 0; round += 1) {
    const remaining = budgetMs - (now() - started);
    if (remaining <= 0) break;
    if (round > 1) {
      // A retry wait spends the same budget as a request. Stop when the next
      // wait leaves no time to ask, rather than sleeping past the deadline.
      const delay = waitMs * 2 ** (round - 2);
      if (delay >= remaining) break;
      await sleep(delay);
    }

    const stillPending = [];
    for (const url of pending) {
      const requestBudget = budgetMs - (now() - started);
      if (requestBudget <= 0) {
        stillPending.push(url);
        continue;
      }

      const result = await probe(url, {
        fetchImpl,
        timeoutMs: Math.min(timeoutMs, requestBudget),
        accept,
      });
      const record = results.get(url);
      record.outcome = result.outcome;
      record.detail = result.detail;
      record.attempts = round;
      log(`  attempt ${round}: ${url} -> ${result.outcome} (${result.detail})`);

      if (result.outcome === 'unreachable' || result.outcome === 'retryable') {
        stillPending.push(url);
      }
    }
    pending = stillPending;
  }

  return results;
}

/**
 * Write output to GitHub Actions output file
 * @param {string} name - Output name
 * @param {string} value - Output value
 */
function setOutput(name, value) {
  const outputFile = process.env.GITHUB_OUTPUT;
  if (outputFile) {
    appendFileSync(outputFile, `${name}=${value}\n`);
  }
  console.log(`${name}=${value}`);
}

/**
 * @param {string} name - Environment variable name
 * @param {number} fallback - Value to use when unset or unparseable
 * @returns {number}
 */
function numericEnv(name, fallback) {
  const raw = process.env[name];
  if (!raw) {
    return fallback;
  }
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

/**
 * Read the report, re-check what deserves it, and set the outputs.
 *
 * Exported so a test can drive the whole path -- parse, classify, probe,
 * annotate, write -- rather than only its parts.
 */
export async function main() {
  const lycheeOutput = process.env.LYCHEE_OUTPUT || 'lychee/out.md';
  const recoveredOutput =
    process.env.RECOVERED_OUTPUT || 'lychee/recovered.txt';
  const verbose = process.env.RECHECK_VERBOSE === '1';

  mkdirSync(dirname(recoveredOutput), { recursive: true });
  writeFileSync(recoveredOutput, '');

  console.log('=== Re-check of transient link failures ===\n');
  console.log(`Reading lychee output from: ${lycheeOutput}\n`);

  // Nothing is recovered until it is proven recovered, so every early return
  // reports 'false': the absence of a report is not evidence of a working link.
  if (!existsSync(lycheeOutput)) {
    console.log('No lychee output file found; nothing to re-check.');
    setOutput('all_recovered', 'false');
    setOutput('recovered', '0');
    setOutput('remaining', '0');
    return;
  }

  let content;
  try {
    content = readFileSync(lycheeOutput, 'utf-8');
    validateLycheeReport(content);
  } catch (error) {
    console.error(`Lychee report is unverified: ${error.message}`);
    setOutput('all_recovered', 'false');
    setOutput('recovered', '0');
    setOutput('remaining', '0');
    return;
  }
  const unsupported = extractNonHttpFailures(content);
  const failures = extractFailures(content);
  const { answered, unanswered } = classifyFailures(failures);

  console.log(
    `lychee reported ${failures.length + unsupported.length} failing link(s): ` +
      `${answered.length} permanent HTTP failures, ` +
      `${unanswered.length} transient or transport failures, ${unsupported.length} non-HTTP failures.\n`
  );

  for (const failure of answered) {
    console.log(
      `  [${failure.marker ?? '?'}] ${failure.url} -- the host answered; not re-checked`
    );
  }

  if (unanswered.length === 0) {
    setOutput('all_recovered', 'false');
    setOutput('recovered', '0');
    setOutput('remaining', String(answered.length + unsupported.length));
    return;
  }

  console.log(`\nRe-checking ${unanswered.length} link(s) directly...\n`);

  const results = await recheckAll(
    unanswered.map((failure) => failure.url),
    {
      attempts: numericEnv('RECHECK_ATTEMPTS', 3),
      waitMs: numericEnv('RECHECK_WAIT_SECONDS', 5) * 1000,
      timeoutMs: numericEnv('RECHECK_TIMEOUT_SECONDS', 20) * 1000,
      budgetMs: numericEnv('RECHECK_BUDGET_SECONDS', 180) * 1000,
      log: verbose ? (line) => console.log(line) : () => {},
    }
  );

  const recovered = [];
  const stillFailing = [];
  for (const failure of unanswered) {
    const result = results.get(failure.url);
    if (result.outcome === 'alive') {
      recovered.push({ ...failure, result });
    } else {
      stillFailing.push({ ...failure, result });
    }
  }

  console.log('\n=== Re-check summary ===\n');

  for (const { url, detail, result } of recovered) {
    console.log(`✓ ${url} answered ${result.detail} on attempt ${result.attempts}`);
    console.log(
      `::notice title=Link answered on re-check::` +
        `${url} answered ${result.detail} when asked directly, ` +
        `after lychee reported it as "${detail || 'failed with no status code'}".\n` +
        `The later successful response recovers this link within the retry budget.`
    );
  }

  for (const { url, detail, result } of stillFailing) {
    console.log(
      `✗ ${url} still failing: ${result.detail} ` +
        `after ${result.attempts} direct attempt(s) (lychee: ${detail || 'no detail'})`
    );
  }

  mkdirSync(dirname(recoveredOutput), { recursive: true });
  writeFileSync(
    recoveredOutput,
    recovered.map(({ url }) => url).join('\n') + (recovered.length ? '\n' : '')
  );
  console.log(`\nRecovered URLs written to ${recoveredOutput}`);

  const remaining = answered.length + stillFailing.length + unsupported.length;
  setOutput('all_recovered', remaining === 0 ? 'true' : 'false');
  setOutput('recovered', String(recovered.length));
  setOutput('remaining', String(remaining));

  if (remaining === 0) {
    console.log(
      '\nEvery reported transient or transport failure recovered with a successful response.'
    );
  }
}

const isDirectRun =
  process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;

if (isDirectRun) {
  main().catch((error) => {
    console.error('Unexpected error:', error);
    process.exit(1);
  });
}
