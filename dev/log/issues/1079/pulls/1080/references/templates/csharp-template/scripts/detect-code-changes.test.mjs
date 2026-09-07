import { describe, expect, test } from 'bun:test';
import { detectChangeOutputs } from './detect-code-changes.mjs';

const EXCLUDED_EVENT_MATRIX = [
  ['pull_request', 'experiments/repro.mjs'],
  ['push', 'experiments/repro.md'],
  ['pull_request', 'dev/log/repro.mjs'],
  ['push', 'dev/log/repro.md'],
  ['pull_request', 'docs/case-studies/repro.mjs'],
  ['push', 'docs/case-studies/repro.md'],
];

describe('code change detection', () => {
  test.each(EXCLUDED_EVENT_MATRIX)(
    '%s ignores excluded-only change %s',
    (_eventName, changedFile) => {
      expect(detectChangeOutputs([changedFile])).toEqual({
        'any-code-changed': 'false',
      });
    }
  );

  test.each([
    'src/MyPackage/Calculator.cs',
    'scripts/check-file-size.mjs',
    '.github/workflows/release.yml',
  ])('detects code change %s', (changedFile) => {
    expect(detectChangeOutputs([changedFile])).toEqual({
      'any-code-changed': 'true',
    });
  });

  test('detects code when excluded and included files are mixed', () => {
    expect(
      detectChangeOutputs([
        'experiments/repro.mjs',
        'docs/case-studies/notes.md',
        'src/MyPackage/Calculator.cs',
      ])
    ).toEqual({ 'any-code-changed': 'true' });
  });
});
