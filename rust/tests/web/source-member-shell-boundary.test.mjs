import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {semanticShellCommandForTask} from '../../../js/agentic/shell_command.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
before(async () => installNodeHost(new WorkerHost()));
const requests = [
  "Move the function parse from source.rs to destination.rs",
  "перемести функцию parse из source.rs в destination.rs",
  "स्थानांतरित करो फ़ंक्शन parse source.rs में destination.rs",
  "移动函数 parse source.rs 到 destination.rs",
  "mueve la función parse de source.rs a destination.rs",
  "Move tests from source.rs to destination.rs",
  "перемести тест из source.rs в destination.rs",
  "स्थानांतरित करो परीक्षण source.rs में destination.rs",
  "移动测试 source.rs 到 destination.rs",
  "mueve la prueba de source.rs a destination.rs",
  "Copy methods from source.rs to destination.rs",
  "Copy regression from source.rs to destination.rs",
  "Move the two existing seed-byte roundtrip tests from rust/src/memory/bundle.rs into rust/tests/fixtures/renamed.rs"
];
for (const prompt of requests) {
  test('source member does not authorize a whole-file mutation: '+prompt, async () => {
    assert.equal(semanticShellCommandForTask(prompt), null);
    const plan = await planChatStep([{role:'user',content:prompt}], ['bash']);
    for (const call of plan?.calls ?? []) {
      if (call.tool !== 'bash') continue;
      const command = JSON.parse(call.arguments).command;
      assert.ok(!/^(?:mv|cp|rm)(?:\s|$)/u.test(command), command);
    }
  });
}
for (const word of ['test', 'tests', 'function', 'methods', 'regression']) {
  for (const verb of ['Copy', 'Move']) {
    test('path words remain whole-file operands: '+verb+' '+word, () => {
      const source = word+'-source.rs';
      const target = word+'-destination.rs';
      const expected = (verb === 'Copy' ? 'cp ' : 'mv ')+source+' '+target;
      for (const [left,right] of [['',''], ['«','»'], ['“','”']]) {
        const prompt = verb+' '+left+source+right+' to '+left+target+right;
        assert.equal(semanticShellCommandForTask(prompt), expected, prompt);
      }
    });
  }
}
