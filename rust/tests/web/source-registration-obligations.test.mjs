import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import test from 'node:test';
import { spawnSync } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { rustSourceForTask } from '../../../js/agentic/code_task.mjs';
import { sourceRegistrationContract } from '../../../js/agentic/code_task/source_contract.mjs';
await installNodeHost(new WorkerHost());
const fixtures = [{
  "task": "Create rust/src/ladder_units.rs with a public function metres_to_kilometres dividing an f64 by 1000.0, and register the module in rust/src/lib.rs so it is part of the crate.",
  "sourcePath": "rust/src/ladder_units.rs",
  "sourceContent": "pub fn metres_to_kilometres(value: f64) -> f64 {\n    value / 1000.0\n}\n",
  "registrationPath": "rust/src/lib.rs",
  "module": "ladder_units"
}, {
  "task": "Create forecasts.rs containing a public Rust function named pressure_value returning -37, and register the module in library.rs.",
  "sourcePath": "forecasts.rs",
  "sourceContent": "pub fn pressure_value() -> i64 {\n    -37\n}\n",
  "registrationPath": "library.rs",
  "module": "forecasts"
}, {
  "task": "Create altitude.rs containing a public Rust function named height_value that takes an f64 and returns it divided by 37.0, and register the module in engine.rs.",
  "sourcePath": "altitude.rs",
  "sourceContent": "pub fn height_value(value: f64) -> f64 {\n    value / 37.0\n}\n",
  "registrationPath": "engine.rs",
  "module": "altitude"
}];
const tools = ['read_file', 'write_file', 'run_command'];
for (const fixture of fixtures) {
  for (const suffix of ['', ' Then deploy it.', ' Then do unknown future work.', ' Do not write any files.', ' Read README.md first.']) {
    test('source and registration preserve complete obligation ownership: ' + fixture.task + suffix, async () => {
      const task = fixture.task + suffix;
      const contract = sourceRegistrationContract(task, rustSourceForTask);
      assert(contract);
      assert.equal(contract.source, task);
      assert.equal(contract.wholeRequestConsumed, suffix === '');
      assert.equal(task.slice(...contract.registration.target.span), fixture.registrationPath);
      assert.equal(task.slice(...contract.registration.action.span), contract.registration.action.text);
      if (suffix) assert.equal(task.slice(...contract.remainingSpan).trim(), suffix.trim());
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'source-registration-obligations-'));
      const before = 'pub mod language;\npub mod seed;\n';
      const messages = [{
          role: 'user',
          content: task
        }],
        receipts = [];
      let terminal;
      try {
        fs.mkdirSync(path.dirname(path.join(directory, fixture.registrationPath)), {
          recursive: true
        });
        fs.writeFileSync(path.join(directory, fixture.registrationPath), before);
        for (let turn = 0; turn < 6; turn++) {
          const plan = await planChatStep(messages, tools);
          if (plan.kind !== 'tool_calls') {
            terminal = plan;
            break;
          }
          assert.equal(plan.calls.length, 1);
          const call = plan.calls[0],
            argumentsValue = JSON.parse(call.arguments),
            identifier = 'receipt-' + turn;
          let result;
          if (call.tool === 'write_file') {
            const full = path.resolve(directory, argumentsValue.path);
            assert(full.startsWith(directory + path.sep));
            fs.mkdirSync(path.dirname(full), {
              recursive: true
            });
            fs.writeFileSync(full, argumentsValue.content);
            result = turn === 0 ? 'created' : 'updated';
          } else if (call.tool === 'read_file') {
            assert.equal(argumentsValue.path, fixture.registrationPath);
            result = fs.readFileSync(path.join(directory, argumentsValue.path), 'utf8');
          } else if (call.tool === 'run_command') {
            const allowed = ['cat ' + fixture.sourcePath, 'cat ' + fixture.registrationPath];
            assert(allowed.includes(argumentsValue.command));
            const processResult = spawnSync('/bin/sh', ['-c', argumentsValue.command], {
              cwd: directory,
              encoding: 'utf8',
              timeout: 3000
            });
            assert.equal(processResult.status, 0);
            result = JSON.stringify({
              stdout: processResult.stdout,
              stderr: processResult.stderr,
              exit_code: processResult.status
            });
          } else assert.fail('Unprovided tool ' + call.tool);
          receipts.push({
            call,
            result
          });
          messages.push({
            role: 'assistant',
            tool_calls: [{
              id: identifier,
              type: 'function',
              function: {
                name: call.tool,
                arguments: call.arguments
              }
            }]
          }, {
            role: 'tool',
            name: call.tool,
            tool_call_id: identifier,
            content: result
          });
        }
        if (suffix) {
          assert.equal(receipts.length, 0);
          assert(!fs.existsSync(path.join(directory, fixture.sourcePath)));
          assert.equal(fs.readFileSync(path.join(directory, fixture.registrationPath), 'utf8'), before);
        } else {
          assert.equal(receipts.length, 5);
          assert.equal(terminal.kind, 'final');
          assert(terminal.answer.includes('observed'));
          assert.deepEqual(receipts.map(receipt => receipt.call.tool), ['write_file', 'run_command', 'read_file', 'write_file', 'run_command']);
          assert.equal(fs.readFileSync(path.join(directory, fixture.sourcePath), 'utf8'), fixture.sourceContent);
          assert.equal(fs.readFileSync(path.join(directory, fixture.registrationPath), 'utf8'), before + 'pub mod ' + fixture.module + ';\n');
        }
      } finally {
        fs.rmSync(directory, {
          recursive: true,
          force: true
        });
      }
    });
  }
}
