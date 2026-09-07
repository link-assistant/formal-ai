import { describe, expect, test } from 'bun:test';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const WORKFLOW_DIR = '.github/workflows';
const RELEASE_WORKFLOW = join(WORKFLOW_DIR, 'release.yml');
const WORKFLOWS_WORKFLOW = join(WORKFLOW_DIR, 'workflows.yml');

// Contexts an attacker can set on a pull request from a fork. Interpolating any
// of them into a `run:` body is a script-injection sink, because the runner
// substitutes the text before the shell parses it. Reading the same value
// through `env:` is inert: the shell expands it after parsing. See issue #49
// and actionlint's "untrusted inputs" check.
const UNTRUSTED_CONTEXTS = [
  /github\.head_ref/,
  /github\.event\.pull_request\.(head\.(ref|label)|title|body)/,
  /github\.event\.(issue|comment|discussion|review)\b[^}]*\.(title|body)/,
  /github\.event\.head_commit\.(message|author)/,
  /github\.event\.commits\[[^\]]*\]\.(message|author)/,
  /github\.event\.workflow_run\.head_branch/,
];

function readWorkflow(filePath) {
  return readFileSync(filePath, 'utf-8').replaceAll('\r\n', '\n');
}

function listWorkflows() {
  return readdirSync(WORKFLOW_DIR)
    .filter((name) => name.endsWith('.yml') || name.endsWith('.yaml'))
    .map((name) => join(WORKFLOW_DIR, name));
}

// Yields every line that belongs to a block-scalar `run:` script, with its
// 1-based line number, so findings can be reported at a reviewable location.
function getRunScriptLines(workflow) {
  const lines = workflow.split('\n');
  const scriptLines = [];
  let runIndent = null;

  lines.forEach((line, index) => {
    if (runIndent !== null) {
      const isBlank = line.trim() === '';
      const indent = line.length - line.trimStart().length;
      if (!isBlank && indent <= runIndent) {
        runIndent = null;
      } else {
        if (!isBlank) {
          scriptLines.push({ number: index + 1, text: line });
        }
        return;
      }
    }

    const runMatch = /^(\s*)-?\s*run:\s*[|>][-+]?\s*$/.exec(line);
    if (runMatch) {
      runIndent = runMatch[1].length;
      return;
    }

    // Single-line `run: ...` form: the script is the value itself.
    const inlineMatch = /^\s*-?\s*run:\s+(?![|>]\s*$)(.+)$/.exec(line);
    if (inlineMatch) {
      scriptLines.push({ number: index + 1, text: inlineMatch[1] });
    }
  });

  return scriptLines;
}

describe('workflow script-injection policy', () => {
  test('no workflow interpolates attacker-controlled context into a run: script', () => {
    const findings = [];

    for (const filePath of listWorkflows()) {
      for (const { number, text } of getRunScriptLines(readWorkflow(filePath))) {
        for (const expression of text.match(/\$\{\{[^}]*\}\}/g) ?? []) {
          if (UNTRUSTED_CONTEXTS.some((pattern) => pattern.test(expression))) {
            findings.push(`${filePath}:${number}: ${expression.trim()}`);
          }
        }
      }
    }

    expect(
      findings,
      `Pass these values through env: instead of interpolating them:\n${findings.join('\n')}`
    ).toEqual([]);
  });

  test('changeset validation reads head_ref and the C# root from env', () => {
    const workflow = readWorkflow(RELEASE_WORKFLOW);

    expect(workflow).toContain('GITHUB_HEAD_REF: ${{ github.head_ref }}');
    expect(workflow).toContain(
      'if [[ "$GITHUB_HEAD_REF" == "changeset-release/"* ]] || [[ "$GITHUB_HEAD_REF" == "changeset-manual-release-"* ]]; then'
    );
    expect(workflow).toContain('bun run "$CSHARP_ROOT/scripts/validate-changeset.mjs"');
  });

  test('quotes $GITHUB_OUTPUT redirections', () => {
    for (const filePath of listWorkflows()) {
      const workflow = readWorkflow(filePath);

      expect(workflow, `${filePath} redirects to an unquoted $GITHUB_OUTPUT`).not.toMatch(
        />>\s*\$GITHUB_OUTPUT/
      );
    }
  });
});

describe('workflow linting policy', () => {
  test('lints every workflow with actionlint plus its bundled shellcheck', () => {
    const workflow = readWorkflow(WORKFLOWS_WORKFLOW);

    // The Docker image bundles shellcheck; a native binary without shellcheck on
    // PATH silently skips the shell checks that catch injection sinks.
    expect(workflow).toContain('uses: docker://rhysd/actionlint:');
    expect(workflow).toContain('permissions:\n  contents: read');
    expect(workflow).toContain('timeout-minutes: 10');
    expect(workflow).toContain("      - '.github/workflows/**'");
  });
});
