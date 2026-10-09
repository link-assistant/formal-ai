// Execute procedure fixture effects with real filesystem/process or library observations.
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { COMPILED_PROCEDURE_PATH } from '../../../../js/agentic/procedure.mjs';
import { PROCEDURE_CONFORMANCE_TRIGGER, conformanceLinksNotation } from '../../../../js/agentic/crate/skill_procedure.mjs';
import { extractCompiledProcedureArtifact } from '../../../../js/agentic/crate/skill_procedure_artifact.mjs';

export function procedureWorkspace() {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-procedure-fixture-'));
  const artifact = join(directory, COMPILED_PROCEDURE_PATH);
  return {
    close() { rmSync(directory, { recursive: true, force: true }); },
    corrupt(bytes) { writeFileSync(artifact, bytes); },
    execute(call) {
      const argumentsValue = JSON.parse(call.arguments);
      if (call.tool === 'write_file') {
        if (argumentsValue.path !== COMPILED_PROCEDURE_PATH) throw Error('unexpected procedure target');
        writeFileSync(artifact, argumentsValue.content);
        return JSON.stringify({ success: true, path: argumentsValue.path });
      }
      if (call.tool !== 'run_command') throw Error('unsupported procedure fixture tool');
      const command = argumentsValue.command;
      if (command === ['cat', COMPILED_PROCEDURE_PATH].join(' ')) {
        const observed = spawnSync('sh', ['-c', command], { cwd: directory, encoding: 'utf8', timeout: 2000 });
        return JSON.stringify({ schema: 'command-execution-receipt/v1', command,
          stdout: observed.stdout ?? '', stderr: observed.stderr ?? '', exit_code: observed.status,
          complete: !observed.error, timed_out: observed.error?.code === 'ETIMEDOUT',
          truncated: false, ...(observed.error ? { is_error: true, error: observed.error.message } : {}) });
      }
      const expected = ['formal-ai', 'procedure', 'conformance', '--artifact', COMPILED_PROCEDURE_PATH,
        '--trigger', PROCEDURE_CONFORMANCE_TRIGGER].join(' ');
      if (command !== expected) throw Error('unbound procedure operation');
      let stdout = '', error = null;
      try {
        const compiled = extractCompiledProcedureArtifact(readFileSync(artifact, 'utf8'));
        if (compiled === null) throw Error('invalid actual procedure artifact');
        stdout = conformanceLinksNotation(compiled, PROCEDURE_CONFORMANCE_TRIGGER);
      } catch (failure) { error = failure.message; }
      return JSON.stringify({ schema: 'procedure-command-receipt/v1', command, operation_success: error === null,
        exit_code: null, stdout, stderr: '', error, complete: true, truncated: false, timed_out: false });
    },
  };
}
