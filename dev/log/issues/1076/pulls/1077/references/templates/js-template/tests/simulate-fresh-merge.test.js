import { describe, it, expect } from 'test-anywhere';
import { readFileSync } from 'node:fs';
import { URL } from 'node:url';

const scriptPath = new URL(
  '../scripts/simulate-fresh-merge.sh',
  import.meta.url
);

describe('simulate-fresh-merge.sh', () => {
  it('quotes base ref arguments to prevent word splitting', () => {
    const script = readFileSync(scriptPath, 'utf8');

    expect(script).toContain('git rev-list --count "HEAD..origin/$BASE_REF"');
    expect(script).toContain('git merge "origin/$BASE_REF" --no-edit');
    expect(script).not.toContain('git rev-list --count HEAD..origin/$BASE_REF');
    expect(script).not.toContain('git merge origin/$BASE_REF --no-edit');
  });
});
