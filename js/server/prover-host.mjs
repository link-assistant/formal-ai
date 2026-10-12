// The theorem-prover seam of the formalization task (issue #1186 R4), host
// side. Twin of `prover_command_in` and `run_prover_with` in
// rust/src/solver_handlers/formalization_task_prover.rs.
//
// The worker builds each prover's compile unit from the `prover` records of
// data/seed/formal-targets.lino and calls the `formalAiProverHost` global with
// `(binary, extension, unit)`. A browser installs no host, so every prover is
// reported absent there. The JavaScript server installs the host this module
// makes: the binary is looked up in the PATH it is given (never a shell), the
// unit is written under `<tmpdir>/formal-ai-prover/` with a content-derived
// name, and the binary runs once on it. The answer reports the exit status
// and the unit's path; `null` means the binary is absent from PATH.

import { spawnSync } from 'node:child_process';
import { mkdirSync, statSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

/** The directory under the temporary directory that holds compile units. */
export const PROVER_DIRECTORY = 'formal-ai-prover';

/**
 * `clause_<fnv1a-64 hex>`: one compile unit always lands on one file name,
 * and the name is a valid module identifier for every prover (Rocq rejects a
 * dash in a file name).
 * @param {string} unit
 * @returns {string}
 */
export function proverFileStem(unit) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(unit)) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return `clause_${hash.toString(16).padStart(16, '0')}`;
}

/**
 * The executable `binary` names in some directory of `pathValue`, else null.
 * @param {string|undefined} pathValue a PATH-shaped list
 * @param {string} binary
 * @returns {string|null}
 */
export function proverCommandIn(pathValue, binary) {
  if (!pathValue) return null;
  for (const directory of pathValue.split(path.delimiter)) {
    if (directory === '') continue;
    const candidate = path.join(directory, binary);
    try {
      if (statSync(candidate).isFile()) return candidate;
    } catch {
      // Not in this directory.
    }
  }
  return null;
}

/**
 * Write `unit` under `directory` and run `command` on it.
 * @param {string} command
 * @param {string} unit
 * @param {string} directory
 * @param {string} extension
 * @returns {{command: string, exit: number|null, sourcePath: string}}
 */
export function runProverWith(command, unit, directory, extension) {
  const sourcePath = path.join(directory, `${proverFileStem(unit)}.${extension}`);
  let exit = null;
  try {
    mkdirSync(directory, { recursive: true });
    writeFileSync(sourcePath, unit);
    const result = spawnSync(command, [sourcePath], { cwd: directory, stdio: 'ignore' });
    exit = typeof result.status === 'number' ? result.status : null;
  } catch {
    exit = null;
  }
  return { command, exit, sourcePath };
}

/**
 * The `formalAiProverHost` function the worker calls.
 * @param {{path?: string, tmpdir?: string}} [options] the PATH to search and
 *   the temporary directory (default: this process's)
 * @returns {(binary: string, extension: string, unit: string) => object|null}
 */
export function createProverHost(options = {}) {
  const pathValue = options.path === undefined ? process.env.PATH : options.path;
  const directory = path.join(options.tmpdir === undefined ? os.tmpdir() : options.tmpdir, PROVER_DIRECTORY);
  return (binary, extension, unit) => {
    const command = proverCommandIn(pathValue, binary);
    return command === null ? null : runProverWith(command, unit, directory, extension);
  };
}
