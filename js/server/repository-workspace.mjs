// Node's repository boundary: bounded processes and confined file access.
// A caller supplies an existing isolated checkout at the task's exact base.
import { serverMessage } from './messages.mjs';
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { executeWorkspaceProtocol } from '../agentic/crate/repository_workspace_runner.mjs';
import { runRepositoryCommand } from '../agentic/crate/repository_workspace_stages.mjs';
import { moduleFromDocument } from '../agentic/crate/self_ast_census.mjs';

function confined(root, relative) {
  if (typeof relative !== 'string' || !relative || path.isAbsolute(relative)) throw new Error(serverMessage('repository-relative-path-required'));
  const base = fs.realpathSync(root);
  const target = path.resolve(base, relative);
  if (!target.startsWith(base + path.sep)) throw new Error(serverMessage('repository-path-outside-workspace') + relative);
  let existing = target;
  for (;;) {
    try {
      if (fs.lstatSync(existing).isSymbolicLink()) throw new Error(serverMessage('repository-symlink-path-refused') + relative);
      break;
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
      existing = path.dirname(existing);
    }
  }
  const physical = fs.realpathSync(existing);
  if (physical !== base && !physical.startsWith(base + path.sep)) throw new Error(serverMessage('repository-symlink-outside-workspace') + relative);
  return target;
}

function boundedProcess(root, program, argumentsList, policy) {
  const limit = policy.output_limit_bytes ?? 16 * 1024 * 1024;
  const deadline = policy.deadline_seconds ?? 300;
  if (!(deadline > 0) || !(limit > 0)) throw new Error(serverMessage('repository-process-limits-required'));
  return new Promise((resolve) => {
    const started = Date.now();
    const stdout = []; const stderr = [];
    const text = (chunks) => Buffer.concat(chunks).toString('utf8');
    let byteLength = 0; let timedOut = false; let overflow = false;
    const child = spawn(program, argumentsList, { cwd: root, detached: process.platform !== 'win32',
      env: { ...process.env, GIT_TERMINAL_PROMPT: '0', npm_config_offline: 'true', PIP_NO_INDEX: '1', ...policy.env },
      stdio: ['ignore', 'pipe', 'pipe'] });
    const kill = () => {
      try { if (process.platform === 'win32') child.kill('SIGKILL'); else process.kill(-child.pid, 'SIGKILL'); } catch {}
    };
    const timer = setTimeout(() => { timedOut = true; kill(); }, deadline * 1000);
    const collect = (name, bytes) => {
      byteLength += bytes.length;
      if (byteLength > limit) { overflow = true; kill(); return; }
      if (name === 'stdout') stdout.push(bytes); else stderr.push(bytes);
    };
    child.stdout.on('data', (bytes) => collect('stdout', bytes));
    child.stderr.on('data', (bytes) => collect('stderr', bytes));
    child.once('error', (error) => {
      clearTimeout(timer);
      resolve({ exit_code: error.code === 'ENOENT' ? 127 : null, stdout: text(stdout), stderr: text(stderr) + error.message,
        missing: error.code === 'ENOENT', elapsed_seconds: (Date.now() - started) / 1000 });
    });
    child.once('close', (code, signal) => {
      clearTimeout(timer);
      resolve({ exit_code: overflow ? null : code, stdout: text(stdout), stderr: text(stderr) + (overflow ? serverMessage('repository-process-output-limit') : ''),
        timed_out: timedOut, output_limit_exceeded: overflow, signal, elapsed_seconds: (Date.now() - started) / 1000 });
    });
  });
}

/** Node implementation of the runner io; process sandbox/network enforcement is an injected host responsibility. */
export function nodeRepositoryIo({ run = boundedProcess } = {}) {
  const list = (root, directory = '') => fs.readdirSync(path.join(root, directory), { withFileTypes: true })
    .filter((entry) => !entry.isSymbolicLink() && !['.git', 'node_modules', 'target', '.cache'].includes(entry.name))
    .flatMap((entry) => {
      const relative = path.posix.join(directory, entry.name);
      return entry.isDirectory() ? list(root, relative) : [relative];
    });
  const read = (root, relative) => fs.readFileSync(confined(root, relative), 'utf8');
  const sourceFiles = (root) => list(root).filter((relative) => /\.(?:rs|py|js|mjs|jsx|ts|tsx|go|c|h|cpp|java|sh)$/u.test(relative))
    .map((relative) => [relative, read(root, relative)]);
  return {
    prepare: async (root) => fs.mkdirSync(root, { recursive: true }),
    read: async (root, relative) => read(root, relative),
    readBytes: async (root, relative) => fs.readFileSync(confined(root, relative)),
    readOptionalBytes: async (root, relative) => {
      const file = confined(root, relative); return fs.existsSync(file) ? fs.readFileSync(file) : null;
    },
    readOptional: async (root, relative) => {
      const file = confined(root, relative); return fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : null;
    },
    write: async (root, relative, contents) => {
      const file = confined(root, relative); fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, contents);
    },
    sourceFiles: async (root) => sourceFiles(root),
    census: async (root) => {
      const sources = sourceFiles(root);
      const modules = list(root).filter((relative) => relative.startsWith('data/meta/self-ast/') && relative.endsWith('.lino'))
        .map((relative) => moduleFromDocument(read(root, relative))).filter(Boolean)
        .map((module) => {
          const matching = sources.filter(([relative]) => relative === module.path || relative.endsWith('/' + module.path));
          return matching.length === 1 ? { ...module, path: matching[0][0] } : null;
        }).filter(Boolean);
      return { modules };
    },
    run: async (root, program, argumentsList, policy) => run(fs.realpathSync(root), program, argumentsList, policy),
  };
}

/** Materialize exactly the named base commit through the same allowlisted injected process port. */
export async function cloneRepositoryWorkspace(spec, root, { io = nodeRepositoryIo(), allowRemoteClone = false } = {}) {
  if (!/^[a-fA-F0-9]{40}$/u.test(spec?.base_commit ?? '')) throw new Error(serverMessage('repository-full-base-required'));
  if (!spec.origin) throw new Error(serverMessage('repository-origin-required'));
  if (fs.existsSync(root)) throw new Error(serverMessage('repository-destination-exists'));
  let origin = spec.origin;
  const local = fs.existsSync(origin);
  if (local) origin = fs.realpathSync(origin);
  else if (!allowRemoteClone) throw new Error(serverMessage('repository-network-grant-required'));
  else if (!origin.includes('://')) origin = 'https://github.com/' + origin + '.git';
  const destination = path.resolve(root);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  const cloned = await runRepositoryCommand({ root: path.dirname(destination), io }, 'git',
    ['clone', '--no-checkout', origin, destination]);
  if (cloned.exit_code !== 0) throw new Error(cloned.stderr ?? serverMessage('repository-clone-failed'));
  const workspace = { root: destination, io };
  const checked = await runRepositoryCommand(workspace, 'git', ['checkout', '--detach', spec.base_commit]);
  if (checked.exit_code !== 0) throw new Error(checked.stderr ?? serverMessage('repository-checkout-failed'));
  const observed = await runRepositoryCommand(workspace, 'git', ['rev-parse', 'HEAD']);
  if (observed.exit_code !== 0 || observed.stdout.trim() !== spec.base_commit) throw new Error(serverMessage('repository-base-unobserved'));
  return destination;
}

/** Live SWE-bench and solve/ladder surface over an explicitly owned isolated checkout. */
export async function runRepositoryCase(root, task, { caller = 'solve', io = nodeRepositoryIo(), ...options } = {}) {
  if (!['solve', 'swe_bench', 'coding_ladder'].includes(caller)) throw new Error(serverMessage('repository-caller-unknown') + caller);
  const workspaceRoot = fs.existsSync(root) ? fs.realpathSync(root)
    : await cloneRepositoryWorkspace(task.clone, root, { io, allowRemoteClone: options.allowRemoteClone === true });
  return executeWorkspaceProtocol({ root: workspaceRoot, io }, task, { caller, ...options });
}
