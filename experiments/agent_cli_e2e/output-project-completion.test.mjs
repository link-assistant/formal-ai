import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { verifyOutputProject, completionResponse, readCompletionRecords } from './verify-output-project.mjs';
import { encodeDialogRecord } from '../../js/server/dialog-codec.mjs';
const here = path.dirname(fileURLToPath(import.meta.url));
function fixture(output = 'Juniper 29!', sourcePath = 'src/held-out.py') {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'output-project-effects-'));
  const task = 'Implement the declared program with output ' + output;
  const verifier = 'tests/verify-output.sh';
  const contents = {
    [sourcePath]: '# Independent declarative source fixture; no Python/native product execution.\n',
    [verifier]: "#!/bin/sh\nprintf '%s\\n' '" + output + "'\n",
    'ci/held-out.yml': 'name: declared workflow fixture\n'
  };
  const messages = [{
      role: 'user',
      content: task
    }],
    records = [];
  const calls = Object.entries(contents).map(([file, content], index) => ({
    id: 'write-' + index,
    type: 'function',
    function: {
      name: 'write',
      arguments: JSON.stringify({
        filePath: file,
        content
      })
    }
  }));
  calls.push({
    id: 'verification-unique',
    type: 'function',
    function: {
      name: 'bash',
      arguments: JSON.stringify({
        command: 'sh ' + verifier
      })
    }
  });
  for (const [index, call] of calls.entries()) {
    records.push({
      timestamp_unix_ms: index + 1,
      method: 'POST',
      path: '/v1/chat/completions',
      request_body: JSON.stringify({
        messages
      }),
      response_body: JSON.stringify({
        choices: [{
          message: {
            role: 'assistant',
            content: null,
            tool_calls: [call]
          },
          finish_reason: 'tool_calls'
        }]
      })
    });
    messages.push({
      role: 'assistant',
      content: null,
      tool_calls: [call]
    });
    messages.push({
      role: 'tool',
      tool_call_id: call.id,
      content: index === calls.length - 1 ? output + '\n' : 'actual fixture write acknowledged'
    });
  }
  records.push({
    timestamp_unix_ms: 5,
    method: 'POST',
    path: '/v1/chat/completions',
    request_body: JSON.stringify({
      messages
    }),
    response_body: JSON.stringify({
      choices: [{
        message: {
          role: 'assistant',
          content: 'Created and verified ' + output
        },
        finish_reason: 'stop'
      }]
    })
  });
  for (const [file, content] of Object.entries(contents)) {
    fs.mkdirSync(path.dirname(path.join(workspace, file)), {
      recursive: true
    });
    fs.writeFileSync(path.join(workspace, file), content);
  }
  return {
    workspace,
    task,
    expectedFiles: Object.keys(contents),
    verifier,
    expectedOutput: output,
    records,
    cleanup: () => fs.rmSync(workspace, {
      recursive: true,
      force: true
    })
  };
}
const request = (row, change) => {
  const decoded = JSON.parse(row.request_body);
  change(decoded);
  row.request_body = JSON.stringify(decoded);
};
const response = (row, change) => {
  const decoded = JSON.parse(row.response_body);
  change(decoded);
  row.response_body = JSON.stringify(decoded);
};
for (const [line, source] of [['Juniper 29!', 'src/held-out.py'], ['Кедр 83!', 'other/held-out.rs']]) test('five synthetic wire exchanges prove actual file effects and independent shell stdout ' + line, () => {
  const context = fixture(line, source);
  try {
    const report = verifyOutputProject(context);
    assert.equal(report.exchanges, 5);
    assert.equal(report.files.length, 3);
    assert.equal(report.verification.exitCode, 0);
    assert.equal(report.verification.callId, 'verification-unique');
    assert.equal(report.terminal.finish, 'stop');
  } finally {
    context.cleanup();
  }
});
for (const [name, change] of [['missing final', context => context.records.pop()], ['foreign task', context => request(context.records[0], body => {
  body.messages[0].content = 'unrelated prompt';
})], ['stale result', context => request(context.records[1], body => {
  body.messages.at(-1).tool_call_id = 'stale-id';
})], ['unbound assistant call', context => request(context.records[1], body => {
  body.messages[1].tool_calls[0].id = 'stale-id';
})], ['changed assistant arguments', context => request(context.records[1], body => {
  body.messages[1].tool_calls[0].function.arguments = '{}';
})], ['missing returned verification', context => request(context.records.at(-1), body => {
  body.messages.pop();
})], ['failed returned verification', context => request(context.records.at(-1), body => {
  body.messages.at(-1).is_error = true;
})], ['missing stdout receipt', context => request(context.records.at(-1), body => {
  body.messages.at(-1).content = 'no returned output';
})], ['changed repeated result', context => request(context.records.at(-1), body => {
  body.messages[2].content = 'changed old result';
})], ['duplicate current call id', context => response(context.records[1], body => {
  body.choices[0].message.tool_calls[0].id = 'write-0';
})], ['missing physical file', context => fs.rmSync(path.join(context.workspace, context.expectedFiles[0]))], ['foreign physical bytes', context => fs.appendFileSync(path.join(context.workspace, context.expectedFiles[0]), 'foreign bytes')], ['duplicate configured effect', context => context.expectedFiles.push(context.expectedFiles[0])], ['undeclared verifier', context => {
  context.verifier = 'elsewhere.sh';
}], ['terminal omitted output', context => response(context.records.at(-1), body => {
  body.choices[0].message.content = 'Done';
})], ['truncated terminal', context => response(context.records.at(-1), body => {
  body.choices[0].finish_reason = null;
})], ['extra exchange after final', context => context.records.push(structuredClone(context.records.at(-1)))]]) test(name + ' cannot receive completion proof', () => {
  const context = fixture();
  try {
    change(context);
    assert.throws(() => verifyOutputProject(context));
  } finally {
    context.cleanup();
  }
});
function replaceVerifier(context, script) {
  fs.writeFileSync(path.join(context.workspace, context.verifier), script);
  for (const row of context.records) {
    request(row, body => {
      for (const message of body.messages) for (const call of message.tool_calls ?? []) {
        if (call.id === 'write-1') call.function.arguments = JSON.stringify({
          filePath: context.verifier,
          content: script
        });
      }
    });
  }
  response(context.records[1], body => {
    body.choices[0].message.tool_calls[0].function.arguments = JSON.stringify({
      filePath: context.verifier,
      content: script
    });
  });
}
for (const [name, script] of [['wrong stdout', "printf 'Wrong\\n'\n"], ['extra stdout', "printf 'Juniper 29!\\nextra\\n'\n"], ['failed process', "printf 'Juniper 29!\\n'\nexit 7\n"]]) test('independent verifier refuses ' + name + ' despite matching file and claimed result/final', () => {
  const context = fixture();
  try {
    replaceVerifier(context, script);
    assert.throws(() => verifyOutputProject(context));
  } finally {
    context.cleanup();
  }
});
test('symlink effects are refused without executing their target', () => {
  const context = fixture();
  try {
    const file = path.join(context.workspace, context.expectedFiles[0]);
    fs.renameSync(file, file + '.real');
    fs.symlinkSync(file + '.real', file);
    assert.throws(() => verifyOutputProject(context), /symlink/);
  } finally {
    context.cleanup();
  }
});
test('wire SSE fragments preserve exact call identity and arguments; incomplete streams refuse', () => {
  const first = {
    choices: [{
      delta: {
        tool_calls: [{
          index: 0,
          id: 'held-out-call',
          function: {
            name: 'ba',
            arguments: '{"com'
          }
        }]
      },
      finish_reason: null
    }]
  };
  const second = {
    choices: [{
      delta: {
        tool_calls: [{
          index: 0,
          function: {
            name: 'sh',
            arguments: 'mand":"true"}'
          }
        }]
      },
      finish_reason: 'tool_calls'
    }]
  };
  const body = 'data: ' + JSON.stringify(first) + '\n\ndata: ' + JSON.stringify(second) + '\n\ndata: [DONE]\n\n';
  assert.deepEqual(completionResponse(body).calls, [{
    id: 'held-out-call',
    name: 'bash',
    arguments: {
      command: 'true'
    }
  }]);
  assert.throws(() => completionResponse(body.replace('data: [DONE]\n\n', '')));
  assert.throws(() => completionResponse('{"choices":[]}'));
});
test('actual typed reader requires complete UTF8 records and retains current wire bytes', () => {
  const context = fixture();
  try {
    const directory = path.join(context.workspace, 'dialog');
    fs.mkdirSync(directory);
    const file = path.join(directory, 'actual.lino');
    const bytes = context.records.map(encodeDialogRecord).join('\n') + '\n';
    fs.writeFileSync(file, bytes);
    assert.deepEqual(readCompletionRecords(directory), context.records);
    assert.equal(verifyOutputProject({
      ...context,
      records: readCompletionRecords(directory)
    }).exchanges, 5);
    fs.writeFileSync(file, bytes.slice(0, -1));
    assert.throws(() => readCompletionRecords(directory));
    fs.writeFileSync(file, Buffer.from([255, 10]));
    assert.throws(() => readCompletionRecords(directory));
  } finally {
    context.cleanup();
  }
});
test('self-mutating verifier cannot retain a pre-execution file proof', () => {
  const context = fixture();
  try {
    replaceVerifier(context, "printf 'foreign' >> src/held-out.py\nprintf 'Juniper 29!\\n'\n");
    assert.throws(() => verifyOutputProject(context), /effect changed during independent verification/);
  } finally {
    context.cleanup();
  }
});
test('option-shaped verifier cannot change independent shell interpretation', () => {
  const context = fixture();
  try {
    context.verifier = '-c';
    context.expectedFiles.push('-c');
    assert.throws(() => verifyOutputProject(context), /unsupported verification command path/);
  } finally {
    context.cleanup();
  }
});
for (const [mode, posts, expected] of [['output-project', 1, 0], ['minimum-posts', 3, 1], ['minimum-posts', 4, 0], ['unknown', 4, 1]]) test('shared harness completion contract ' + mode + ' retains explicit recipe minimums ' + posts, () => {
  const source = fs.readFileSync(path.join(here, 'run_agent_cli.sh'), 'utf8');
  const start = source.indexOf('case "$COMPLETION_EVIDENCE" in');
  const end = source.indexOf('\nesac', start);
  assert.ok(start > 0 && end > start);
  const script = 'fail(){ return 1; }; node(){ printf "%s\\n" "$@"; }; ROOT=declared-root; DIALOG_DIR=declared-dialogs; WORKDIR=declared-work; TASK=declared-task; EXPECT_FILE=main.py; EXPECT_FILES=declared-files; EFFECT_VERIFIER=tests/verify-output.sh; EXPECT_OUTPUT_LINE=declared-output; EVIDENCE_DIR=declared-evidence; MIN_POSTS=4; COMPLETION_EVIDENCE="$1"; posts="$2";\n' + source.slice(start, end + 5);
  const observed = spawnSync('bash', ['-c', script, 'contract-control', mode, String(posts)], {
    encoding: 'utf8',
    timeout: 10000
  });
  assert.equal(observed.status, expected, observed.stderr);
  if (mode === 'output-project') {
    assert.ok(observed.stdout.includes('verify-output-project.mjs'));
    assert.ok(observed.stdout.includes('declared-task'));
    assert.ok(observed.stdout.includes('declared-output'));
  }
});
test('equivalent existing write path aliases retain exact content and physical byte custody', () => {
  const context = fixture();
  try {
    for (const row of context.records) request(row, body => {
      for (const message of body.messages) for (const call of message.tool_calls ?? []) {
        if (call.function.name === 'write') {
          const arguments_ = JSON.parse(call.function.arguments);
          call.function.arguments = JSON.stringify({
            content: arguments_.content,
            path: arguments_.filePath
          });
        }
      }
    });
    assert.equal(verifyOutputProject(context).exchanges, 5);
  } finally {
    context.cleanup();
  }
});
test('conflicting write aliases cannot forge an equivalent client acknowledgement', () => {
  const context = fixture();
  try {
    request(context.records[1], body => {
      const call = body.messages[1].tool_calls[0];
      const arguments_ = JSON.parse(call.function.arguments);
      call.function.arguments = JSON.stringify({
        ...arguments_,
        path: 'foreign.py'
      });
    });
    assert.throws(() => verifyOutputProject(context), /ambiguous emitted effect path/);
  } finally {
    context.cleanup();
  }
});
