import { describe, it, expect } from 'test-anywhere';
import {
  parseChangedLines,
  filterWarningsToChangedLines,
} from '../scripts/lint-changed-lines.mjs';

const DIFF = [
  'diff --git a/scripts/example.mjs b/scripts/example.mjs',
  '--- a/scripts/example.mjs',
  '+++ b/scripts/example.mjs',
  '@@ -10,0 +11,2 @@',
  '+// this line is new',
  '+const value = 1;',
  '',
].join('\n');

function warning(line) {
  return {
    severity: 1,
    line,
    ruleId: 'local/no-changelog-comments',
    message: 'history',
  };
}

describe('parseChangedLines', () => {
  it('collects added line numbers per file', () => {
    const changed = parseChangedLines(DIFF);

    expect(Array.from(changed.get('scripts/example.mjs'))).toEqual([11, 12]);
  });
});

describe('filterWarningsToChangedLines', () => {
  const cwd = '/repo';
  const changed = parseChangedLines(DIFF);

  it('keeps warnings on changed lines', () => {
    const [result] = filterWarningsToChangedLines(
      [
        {
          filePath: '/repo/scripts/example.mjs',
          messages: [warning(11)],
          errorCount: 0,
          warningCount: 1,
        },
      ],
      changed,
      cwd
    );

    expect(result.messages.length).toBe(1);
    expect(result.warningCount).toBe(1);
  });

  it('drops warnings on unchanged lines of a changed file', () => {
    const [result] = filterWarningsToChangedLines(
      [
        {
          filePath: '/repo/scripts/example.mjs',
          messages: [warning(3)],
          errorCount: 0,
          warningCount: 1,
        },
      ],
      changed,
      cwd
    );

    expect(result.messages.length).toBe(0);
    expect(result.warningCount).toBe(0);
  });

  it('drops warnings from files that were not touched at all', () => {
    const [result] = filterWarningsToChangedLines(
      [
        {
          filePath: '/repo/scripts/other.mjs',
          messages: [warning(1)],
          errorCount: 0,
          warningCount: 1,
        },
      ],
      changed,
      cwd
    );

    expect(result.messages.length).toBe(0);
  });

  it('keeps every warning of a wholly new file', () => {
    const newFileChanges = new Map([['scripts/new.mjs', true]]);
    const [result] = filterWarningsToChangedLines(
      [
        {
          filePath: '/repo/scripts/new.mjs',
          messages: [warning(42)],
          errorCount: 0,
          warningCount: 1,
        },
      ],
      newFileChanges,
      cwd
    );

    expect(result.messages.length).toBe(1);
  });

  it('always keeps errors regardless of the diff', () => {
    const [result] = filterWarningsToChangedLines(
      [
        {
          filePath: '/repo/scripts/other.mjs',
          messages: [{ severity: 2, line: 5, ruleId: 'no-var' }],
          errorCount: 1,
          warningCount: 0,
        },
      ],
      changed,
      cwd
    );

    expect(result.messages.length).toBe(1);
    expect(result.errorCount).toBe(1);
  });
});
