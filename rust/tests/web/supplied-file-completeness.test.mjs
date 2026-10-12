import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { suppliedFileAnswer } from '../../../js/agentic/file_read/supplied.mjs';
await installNodeHost(new WorkerHost());

for (const path of ['alpha.txt', 'nested/renamed.txt', '資料.txt']) {
  test(`supplied current empty and incomplete listings conserve source identity: ${path}`, () => {
    const old = { role: 'user', content: `${path}\n\`\`\`\nOLDER_SOURCE\n\`\`\`\n` };
    const ask = { role: 'user', content: `read the file ${path} and print its contents` };
    const answer = listing => suppliedFileAnswer([old, { role: 'user', content: listing }, ask]);

    const empty = answer(`${path}\n\`\`\`\n\`\`\`\n`);
    assert.equal(typeof empty, 'string');
    assert.ok(!empty.includes('OLDER_SOURCE'));
    assert.ok(!empty.includes('   1 |'));
    assert.equal(answer(`${path}\n\`\`\`\nINCOMPLETE_SOURCE`), null);

    const replacement = answer(`${path}\n\`\`\`\nCURRENT_SOURCE\n\`\`\`\n`);
    assert.ok(replacement.includes('CURRENT_SOURCE'));
    assert.ok(!replacement.includes('OLDER_SOURCE'));

    const recovered = suppliedFileAnswer([
      old,
      { role: 'user', content: `${path}\n\`\`\`\nINCOMPLETE_SOURCE` },
      { role: 'user', content: `${path}\n\`\`\`\nRECOVERED_SOURCE\n\`\`\`\n` },
      ask,
    ]);
    assert.ok(recovered.includes('RECOVERED_SOURCE'));
    assert.ok(!recovered.includes('OLDER_SOURCE'));
    assert.equal(suppliedFileAnswer([
      { role: 'user', content: 'foreign.txt\n\`\`\`\nFOREIGN_SOURCE\n\`\`\`\n' }, ask,
    ]), null);

    const literal = answer(`${path}\n\`\`\`\nLITERAL\\nBYTES\n\`\`\`\n`);
    assert.ok(literal.includes('LITERAL\\nBYTES'));
    assert.ok(!literal.includes('OLDER_SOURCE'));
  });
}
