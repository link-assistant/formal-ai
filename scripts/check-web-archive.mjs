#!/usr/bin/env node

/**
 * Check broken links against the Wayback Machine (web.archive.org)
 *
 * This script reads the lychee link checker output (markdown format),
 * extracts broken URLs, and checks each one against the Wayback Machine API.
 * It then outputs a report with:
 * - Links that have a web archive version (with suggestion to replace)
 * - Links that have no web archive version (clearly marked as unrecoverable)
 *
 * Usage:
 *   node scripts/check-web-archive.mjs
 *
 * Environment variables:
 *   - LYCHEE_OUTPUT: Path to lychee markdown output file (default: lychee/out.md)
 *   - RECOVERED_URLS: Path to the list of URLs that answered on a direct
 *     re-check (written by scripts/recheck-broken-links.mjs). Those are not
 *     broken, so they get no archive.org suggestion.
 *
 * GitHub Actions outputs:
 *   - all_archived: 'true' if all broken links have a web archive version
 *
 * Exit codes:
 *   - 0: All broken links have web archive versions (or no broken links)
 *   - 1: Some broken links have no web archive version
 */

import { readFileSync, appendFileSync, existsSync } from 'fs';
import { pathToFileURL } from 'url';

const WAYBACK_API = 'https://archive.org/wayback/available?url=';

/**
 * Lychee section categories that describe a working link.
 *
 * Everything not named here is treated as a failure, so a category this list
 * has not heard of is reported rather than dropped. See `extractBrokenUrls`.
 */
const HEALTHY_SECTIONS = /^(?:redirects|excluded|successes|suggestions)$/i;

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
 * Split a lychee report into the sections that describe a failure.
 *
 * Lychee's Markdown report groups links into level-two sections, one per
 * outcome: `## Errors per input`, `## Timeouts per input`, `## Redirects per
 * input`, and so on. It writes only the sections it has links for, so which
 * headings appear varies run to run. The permissive bullet parser below has to
 * be restricted to the failing sections, or a healthy redirect gets sent to
 * Wayback and reported as an error.
 *
 * The selection is by exclusion, not inclusion: every section counts as a
 * failure unless it is one of the outcomes known to be healthy. Listing the
 * failures instead would mean a category this list has not heard of -- a new
 * lychee release, a renamed heading -- is silently dropped, turning a real
 * broken link into a green build. Getting it wrong in this direction reports a
 * healthy link, which is loud and gets fixed; the other direction is silent.
 * Naming only `## Errors per input` here is what let a report whose sole
 * failure was a timeout fall through to the whole-document fallback and report
 * sixteen of its healthy redirects as broken links (issue #1021).
 *
 * @param {string} content - The markdown content from lychee
 * @returns {string} The concatenated bodies of the failing sections
 */
export function failingSections(content) {
  // `sectionHeading` is built per call rather than hoisted: it carries the `g`
  // flag, and a shared global regex keeps its `lastIndex` between calls.
  const sectionHeading = /^##\s+(.+?)\s+per input\s*$/gm;

  const sections = [];
  let heading;
  while ((heading = sectionHeading.exec(content)) !== null) {
    // `headingStart` bounds the previous section, `bodyStart` opens this one,
    // so no section ever swallows the next section's heading.
    sections.push({
      category: heading[1],
      headingStart: heading.index,
      bodyStart: heading.index + heading[0].length,
    });
  }

  // Older/plain Lychee output has no per-input headings, so keep parsing the
  // complete report in that case.
  if (sections.length === 0) {
    return content;
  }

  return sections
    .map((section, index) => ({
      ...section,
      body: content.slice(
        section.bodyStart,
        index + 1 < sections.length
          ? sections[index + 1].headingStart
          : content.length,
      ),
    }))
    .filter((section) => !HEALTHY_SECTIONS.test(section.category))
    .map((section) => section.body)
    .join('\n');
}

/**
 * Describe one failing link: the marker lychee printed for it and the note it
 * wrote after the pipe.
 *
 * The marker is what separates a verdict from the host -- `[404]`, `[502]` --
 * from a failure that never got one: `[ERROR] <url> | Network error:
 * Connection reset by peer (os error 104)` is the transport giving up below
 * HTTP, and it carries no status code at all. Callers that need to know which
 * kind of failure they are holding read `marker`.
 *
 * @param {string} errorContent - The failing sections of the report
 * @param {string} url - The URL to describe
 * @returns {{marker: string|null, detail: string, line: string}}
 */
function describeFailure(errorContent, url) {
  const line = errorContent.split('\n').find((entry) => entry.includes(url));
  if (line === undefined) {
    return { marker: null, detail: '', line: '' };
  }

  const marker = /\[([^\]\s]+)\]/.exec(line);
  const pipe = line.indexOf('|');

  return {
    marker: marker ? marker[1].toUpperCase() : null,
    detail: pipe === -1 ? '' : line.slice(pipe + 1).trim(),
    line: line.trim(),
  };
}

/**
 * Extract the failing links from lychee markdown output.
 * Lychee markdown format includes lines like:
 *   * [404] https://example.com/broken-link
 *   * [ERROR] <https://another-broken.com> (at 1:1) | Network error: ...
 * @param {string} content - The markdown content from lychee
 * @returns {{url: string, marker: string|null, detail: string, line: string}[]}
 */
export function extractFailures(content) {
  const errorContent = failingSections(content);
  const urls = [];

  // Match lines with error status codes or ERROR markers followed by URLs
  // Lychee output format: [STATUS_CODE] URL or bullet points with links
  const urlPattern =
    /\[(?:4\d\d|5\d\d|ERROR|TIMEOUT|UNKNOWN)\]\s+(https?:\/\/[^\s)]+)/gi;
  let match;

  while ((match = urlPattern.exec(errorContent)) !== null) {
    const url = match[1].trim();
    if (url && !urls.includes(url)) {
      urls.push(url);
    }
  }

  // Also match plain URL lines in broken sections
  // Lychee sometimes outputs: `[ERROR] url | description`
  const linePattern = /^\s*(?:\*|-)\s+.*?(https?:\/\/[^\s|)>\]]+)/gm;
  let lineMatch;

  while ((lineMatch = linePattern.exec(errorContent)) !== null) {
    const url = lineMatch[1].trim().replace(/[.,;!?]+$/, '');
    if (url && !urls.includes(url) && url.startsWith('http')) {
      urls.push(url);
    }
  }

  return urls.map((url) => ({ url, ...describeFailure(errorContent, url) }));
}

/**
 * Extract broken URLs from lychee markdown output
 * @param {string} content - The markdown content from lychee
 * @returns {string[]} Array of broken URLs
 */
export function extractBrokenUrls(content) {
  return extractFailures(content).map((failure) => failure.url);
}

/**
 * Check if a URL has an archived version in the Wayback Machine
 * Uses the Wayback Machine Availability API:
 * https://archive.org/help/wayback_api.php
 *
 * The result distinguishes three outcomes, because only one of them is a
 * statement about the link itself:
 *   - 'archived'   the API answered and a snapshot exists
 *   - 'unarchived' the API answered and no snapshot exists
 *   - 'unknown'    the API could not answer (5xx, rate limit, timeout,
 *                  network failure, malformed body)
 * Treating 'unknown' as 'unarchived' would let an archive.org outage fail a
 * build over links that are perfectly fine, so callers must keep them apart.
 *
 * @param {string} url - The URL to check
 * @param {typeof fetch} [fetchImpl] - Injectable fetch, for tests
 * @returns {Promise<{status: 'archived'|'unarchived'|'unknown', archiveUrl: string|null, timestamp: string|null, reason: string|null}>}
 */
export async function checkWaybackMachine(url, fetchImpl = fetch) {
  const apiUrl = `${WAYBACK_API}${encodeURIComponent(url)}`;

  const controller = new AbortController();
  const timeoutId = globalThis.setTimeout(() => controller.abort(), 10000);

  const unknown = (reason) => ({
    status: 'unknown',
    archiveUrl: null,
    timestamp: null,
    reason,
  });

  try {
    const response = await fetchImpl(apiUrl, {
      headers: {
        'User-Agent': 'broken-link-checker/1.0 (GitHub Actions CI)',
      },
      signal: controller.signal,
    });

    if (!response.ok) {
      // A 4xx/5xx from archive.org describes archive.org, not the link.
      console.warn(`  Wayback API returned ${response.status} for ${url}`);
      return unknown(`Wayback API returned ${response.status}`);
    }

    let data;
    try {
      data = await response.json();
    } catch (error) {
      console.warn(`  Wayback API returned an unreadable body for ${url}`);
      return unknown(`unreadable Wayback response: ${error.message}`);
    }

    if (data?.archived_snapshots?.closest?.available === true) {
      const snapshot = data.archived_snapshots.closest;
      const archiveUrl = snapshot.url.replace(/^http:\/\//, 'https://');
      return {
        status: 'archived',
        archiveUrl,
        timestamp: snapshot.timestamp,
        reason: null,
      };
    }

    if (!data || typeof data.archived_snapshots !== 'object') {
      console.warn(`  Wayback API returned an unexpected shape for ${url}`);
      return unknown('unexpected Wayback response shape');
    }

    return {
      status: 'unarchived',
      archiveUrl: null,
      timestamp: null,
      reason: null,
    };
  } catch (error) {
    const reason =
      error.name === 'AbortError'
        ? 'Wayback API timed out'
        : `Wayback API unreachable: ${error.message}`;
    console.warn(`  Failed to check Wayback Machine for ${url}: ${reason}`);
    return unknown(reason);
  } finally {
    globalThis.clearTimeout(timeoutId);
  }
}

/**
 * Read the URLs a direct re-check reached, written one per line by
 * `scripts/recheck-broken-links.mjs`.
 *
 * lychee reports a link it could not *connect* to the same way it reports a
 * link the host says is gone, and only the second of those is a broken link.
 * When the re-check step has since fetched the page itself, suggesting an
 * archive.org copy of it is a suggestion to replace a working link.
 *
 * @returns {Set<string>} The recovered URLs, empty when there is no such file
 */
function readRecoveredUrls() {
  const path = process.env.RECOVERED_URLS;
  if (!path || !existsSync(path)) {
    return new Set();
  }

  return new Set(
    readFileSync(path, 'utf-8')
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line.length > 0)
  );
}

/**
 * Format a timestamp from Wayback Machine (YYYYMMDDHHmmss) to readable date
 * @param {string} timestamp - e.g. "20231015143022"
 * @returns {string} - e.g. "2023-10-15"
 */
function formatTimestamp(timestamp) {
  if (!timestamp || timestamp.length < 8) {
    return timestamp;
  }
  const year = timestamp.slice(0, 4);
  const month = timestamp.slice(4, 6);
  const day = timestamp.slice(6, 8);
  return `${year}-${month}-${day}`;
}

/**
 * Main function
 */
async function main() {
  const lycheeOutput = process.env.LYCHEE_OUTPUT || 'lychee/out.md';

  console.log('=== Web Archive Fallback Check ===\n');
  console.log(`Reading lychee output from: ${lycheeOutput}\n`);

  if (!existsSync(lycheeOutput)) {
    console.log('No lychee output file found. Skipping web archive check.');
    setOutput('all_archived', 'true');
    process.exit(0);
  }

  const content = readFileSync(lycheeOutput, 'utf-8');
  const recovered = readRecoveredUrls();
  const reported = extractBrokenUrls(content);
  const brokenUrls = reported.filter((url) => !recovered.has(url));

  if (recovered.size > 0) {
    console.log(
      `${reported.length - brokenUrls.length} of ${reported.length} reported link(s) ` +
        'answered a direct re-check and are not treated as broken here.\n'
    );
  }

  if (brokenUrls.length === 0) {
    console.log('No broken URLs found in lychee output.');
    setOutput('all_archived', 'true');
    process.exit(0);
  }

  console.log(
    `Found ${brokenUrls.length} broken URL(s). Checking Web Archive...\n`
  );

  const withArchive = [];
  const withoutArchive = [];
  const undetermined = [];

  for (const url of brokenUrls) {
    console.log(`Checking: ${url}`);
    const result = await checkWaybackMachine(url);

    if (result.status === 'archived') {
      const date = formatTimestamp(result.timestamp);
      console.log(`  ✓ Archived on ${date}: ${result.archiveUrl}`);
      withArchive.push({ url, archiveUrl: result.archiveUrl, date });
    } else if (result.status === 'unarchived') {
      console.log('  ✗ Not found in Web Archive');
      withoutArchive.push(url);
    } else {
      console.log(`  ? Undetermined (${result.reason})`);
      undetermined.push({ url, reason: result.reason });
    }

    // Small delay to avoid rate-limiting the Wayback API
    await new Promise((resolve) => globalThis.setTimeout(resolve, 500));
  }

  console.log('\n=== Web Archive Check Summary ===\n');

  if (withArchive.length > 0) {
    console.log(
      `✓ ${withArchive.length} broken link(s) have Web Archive versions - consider replacing:`
    );
    for (const { url, archiveUrl, date } of withArchive) {
      console.log(`  Original: ${url}`);
      console.log(`  Archive (${date}): ${archiveUrl}`);
      console.log('');
    }

    // Print GitHub Actions annotations as suggestions (one per link)
    for (const { url, archiveUrl, date } of withArchive) {
      console.log(
        `::notice title=Broken link - Web Archive available (${date})::` +
          `Broken link detected: ${url}\n` +
          `A Web Archive snapshot from ${date} is available.\n` +
          `Suggested fix: replace the broken link with the archived version:\n` +
          `  ${archiveUrl}`
      );
    }
  }

  if (withoutArchive.length > 0) {
    console.log(
      `✗ ${withoutArchive.length} broken link(s) have NO Web Archive version:`
    );
    for (const url of withoutArchive) {
      console.log(`  ${url}`);
    }
    console.log('');

    // Print GitHub Actions annotations as errors (one per link)
    for (const url of withoutArchive) {
      console.log(
        `::error title=Broken link - No Web Archive fallback::` +
          `Broken link detected: ${url}\n` +
          `No archived version was found in the Wayback Machine.\n` +
          `How to fix:\n` +
          `  1. Find an updated URL for the same or equivalent content and replace the link.\n` +
          `  2. Remove the link if the content is no longer relevant.\n` +
          `  3. Add the URL to .lycheeignore if it is a known false positive (e.g. localhost, example.com).`
      );
    }
  }

  if (undetermined.length > 0) {
    console.log(
      `? ${undetermined.length} broken link(s) could not be checked against the Web Archive:`
    );
    for (const { url, reason } of undetermined) {
      console.log(`  ${url} (${reason})`);
    }
    console.log('');

    // A warning, never an error: archive.org being unreachable says nothing
    // about these links, so it must not gate the build.
    for (const { url, reason } of undetermined) {
      console.log(
        `::warning title=Web Archive unreachable::` +
          `Could not determine a Web Archive fallback for ${url} (${reason}).\n` +
          `This does not fail the check: the Wayback Machine was unavailable, ` +
          `which is a statement about archive.org and not about the link.`
      );
    }
  }

  const allArchived = withoutArchive.length === 0;
  setOutput('all_archived', allArchived ? 'true' : 'false');
  setOutput('undetermined', String(undetermined.length));

  if (!allArchived) {
    console.log(
      '\nAction required: Fix or remove the broken links listed above.'
    );
    console.log(
      'For links with Web Archive versions, you can replace them with the suggested archive.org URLs.'
    );
    process.exit(1);
  } else if (undetermined.length > 0) {
    console.log(
      `\nNo link was proven unarchived, but ${undetermined.length} could not be checked ` +
        'because the Wayback Machine was unavailable. Passing rather than ' +
        'failing the build on an archive.org outage.'
    );
    process.exit(0);
  } else {
    console.log(
      '\nAll broken links have Web Archive versions. Consider replacing them with the suggested archive.org URLs.'
    );
    process.exit(0);
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
