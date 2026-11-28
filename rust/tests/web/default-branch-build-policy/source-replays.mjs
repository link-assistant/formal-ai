import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';

const ladderSourceFilters = [
  'entity_resolution::tests::',
  'program_skill_gap::tests::',
  'protocol_memory::tests::',
  'seed::meanings::parse::tests::',
  'solver_dispatch::tests::',
  'solver_handler_how_synthesis::tests::',
  'solver_handlers::shell_command_compose::tests::',
];
export const ladderSourceCommands = [
  'for filter in \\',
  ...ladderSourceFilters.map((filter, index) => '  ' + filter + (index + 1 < ladderSourceFilters.length ? ' \\' : '')),
  'do',
  '  cargo test --manifest-path rust/Cargo.toml --locked --lib "$filter" -- --test-threads=1 || regression_status=$?',
  'done',
];

export function assertNativeSourceReplays(shell) {
  // Execute every regression after any sibling fails, retaining the final
  // nonzero status. Every case runs the actual workflow shell body.
  const directory = mkdtempSync(join(tmpdir(), 'native-replay-failures-'));
  try {
    const stub = join(directory, 'cargo');
    writeFileSync(stub, [
      '#!/bin/sh',
      'printf "%s\\n" "$*" >> "$REPLAY_CALLS"',
      'case "$*" in',
      '  *"solver_unknown_reasoning::tests::"*) exit "$REPLAY_UNKNOWN_STATUS";;',
      '  *"solver_helpers::code::javascript_program_tests::"*) exit "$REPLAY_JAVASCRIPT_STATUS";;',
      '  *"rule_interpreter::parser::reference_list_tests::"*) exit "$REPLAY_PARSER_STATUS";;',
      '  *"solver_synthesis::parent_tests::"*) exit "$REPLAY_SYNTHESIS_STATUS";;',
      '  *"client_integrations::tests::"*) exit "$REPLAY_BACKUP_STATUS";;',
      '  *"links_format::tests::"*) exit "$REPLAY_NOTATION_STATUS";;',
      ...ladderSourceFilters.map((filter, index) => '  *"' + filter + '"*) exit "$REPLAY_LADDER_' + index + '_STATUS";;'),
      '  *"--test any_language "*) exit "$REPLAY_SPANISH_STATUS";;',
      '  *) exit 99;;',
      'esac',
      '',
    ].join('\n'), {mode: 0o755});
    const statusNames = ['UNKNOWN', 'JAVASCRIPT', 'PARSER', 'SYNTHESIS', 'BACKUP', 'NOTATION',
      ...ladderSourceFilters.map((_, index) => 'LADDER_' + index), 'SPANISH'];
    const allPass = statusNames.map(() => 0);
    const failures = statusNames.map((_, failed) => allPass.map((status, index) => index === failed ? index + 1 : status));
    const allFail = statusNames.map((_, index) => index + 1);
    for (const statuses of [allPass, ...failures, allFail]) {
      const calls = join(directory, `calls-${statuses.join('-')}.txt`);
      const receipts = Object.fromEntries(statusNames.map((name, index) => [`REPLAY_${name}_STATUS`, String(statuses[index])]));
      const result = spawnSync('bash', ['-e', '-c', shell], {encoding:'utf8', timeout:5000,
        env:{...process.env, PATH:directory+':'+process.env.PATH, REPLAY_CALLS:calls, ...receipts}});
      assert.equal(result.status, statuses.findLast(Boolean) ?? 0, result.stderr);
      const commands = readFileSync(calls, 'utf8').trim().split('\n');
      assert.equal(commands.length, statusNames.length);
      assert.match(commands[0], /--lib solver_unknown_reasoning::tests::/u);
      assert.match(commands[1], /--lib solver_helpers::code::javascript_program_tests::/u);
      assert.match(commands[2], /--lib rule_interpreter::parser::reference_list_tests::/u);
      assert.match(commands[3], /--lib solver_synthesis::parent_tests::/u);
      assert.match(commands[4], /--lib client_integrations::tests::/u);
      assert.match(commands[5], /--lib links_format::tests::/u);
      for (const [index, filter] of ladderSourceFilters.entries()) {
        assert(commands[index + 6].includes('--lib ' + filter), commands[index + 6]);
      }
      assert.match(commands.at(-1), /--test any_language/u);
    }
  } finally { rmSync(directory, {recursive:true, force:true}); }
}
