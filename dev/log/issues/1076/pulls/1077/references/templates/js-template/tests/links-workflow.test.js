import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'test-anywhere';

const workflow = readFileSync('.github/workflows/links.yml', 'utf8').replaceAll(
  '\r\n',
  '\n'
);

function getStepBlock(stepName) {
  const lines = workflow.split('\n');
  const header = `      - name: ${stepName}`;
  const start = lines.findIndex((line) => line === header);

  if (start === -1) {
    return '';
  }

  const end = lines.findIndex(
    (line, index) => index > start && line.startsWith('      - ')
  );

  return lines.slice(start, end === -1 ? lines.length : end).join('\n');
}

describe('broken link workflow', () => {
  it('fails for every nonzero Lychee exit even when an archive exists', () => {
    const failureStep = getStepBlock('Fail if broken links were found');

    expect(failureStep).toContain(
      'if: always() && steps.lychee.outputs.exit_code != 0'
    );
    expect(failureStep).not.toContain('steps.webarchive.outputs.all_archived');
    expect(failureStep).toContain(
      'An archive is a suggested replacement; it does not make the live link valid.'
    );
  });
});
