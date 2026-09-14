import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';

const LINKS_WORKFLOW = '.github/workflows/links.yml';
const LYCHEE_IGNORE = '.lycheeignore';

function readWorkflow() {
  return readFileSync(LINKS_WORKFLOW, 'utf-8').replaceAll('\r\n', '\n');
}

describe('broken-link workflow policy', () => {
  test('checks Markdown and HTML changes with least privilege', () => {
    const workflow = readWorkflow();

    expect(workflow).toContain("name: Broken Link Checker");
    expect(workflow).toContain("      - '**.md'");
    expect(workflow).toContain("      - '**.html'");
    expect(workflow).toContain('permissions:\n  contents: read');
    expect(workflow).toContain('uses: lycheeverse/lychee-action@v2');
  });

  test('excludes copied case studies and parser fixtures but not C# template HTML', () => {
    const workflow = readWorkflow();

    expect(workflow).toContain('--exclude-path docs/case-studies');
    expect(workflow).toContain('--exclude-path scripts/fixtures');
    expect(workflow).not.toContain('examples/universal-app/index.html');
  });

  test('ignores npm bot-protection responses', () => {
    const ignoredUrls = readFileSync(LYCHEE_IGNORE, 'utf-8');

    expect(ignoredUrls).toContain('https://www\\.npmjs\\.com');
  });

  test('uses a bounded, cancellation-safe Wayback fallback', () => {
    const workflow = readWorkflow();

    expect(workflow).toContain('timeout-minutes: 10');
    expect(workflow).toContain(
      'group: check-${{ github.workflow }}-${{ github.ref }}-link-checker'
    );
    expect(workflow).toContain('cancel-in-progress: true');
    expect(workflow).toContain('fail: false');
    expect(workflow).toContain('run: node scripts/check-web-archive.mjs');
    expect(workflow).toContain(
      "if: steps.lychee.outputs.exit_code != 0 && steps.webarchive.outputs.all_archived != 'true'"
    );
  });
});
