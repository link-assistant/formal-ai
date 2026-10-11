// Running a named test file (PR #1188 T91, gap G11): "Run the tests in
// m.test.mjs." ran the workspace's whole-suite command, chosen by whichever
// marker file the server's own directory held (`bun test`). A request that
// names a test file runs that file with the runtime its extension needs,
// read from the seeded `test_file_runners` group of
// data/seed/shell-intents.lino. rust/src/agentic_coding/test_file_runner.rs.

import { parseRoot } from './crate/seed_parser.mjs';
import { cached, childValue, childrenNamed, readText } from './host.mjs';

const SHELL_INTENTS_FILE = 'data/seed/shell-intents.lino';
const WORKSPACE_TEST_COMMAND = 'formal-ai:workspace-test';
const FILE_SLOT = '{file}';

/** The seeded `[extension, command]` runners, in declaration order. */
function runners() {
  return cached('test-file-runners', () => {
    const root = parseRoot(readText(SHELL_INTENTS_FILE)).children[0];
    const group = childrenNamed(root, 'test_file_runners')[0];
    return childrenNamed(group, 'runner').map((node) => [childValue(node, 'extension'), childValue(node, 'command')]);
  });
}

/**
 * Mirrors `fn test_file_command` in rust/src/agentic_coding/test_file_runner.rs:
 * the command that runs the named test file for the workspace-test intent,
 * or null when the intent is another one, no file is named, or no runner is
 * seeded for its extension.
 * @param {string} intentCommand the matched intent's command
 * @param {string|null} path the file the request names
 */
export function testFileCommand(intentCommand, path) {
  if (intentCommand !== WORKSPACE_TEST_COMMAND || path === null) return null;
  const dot = path.lastIndexOf('.');
  if (dot < 0) return null;
  const extension = path.slice(dot + 1).toLowerCase();
  const runner = runners().find(([candidate]) => candidate === extension);
  return runner === undefined ? null : runner[1].split(FILE_SLOT).join(path);
}
