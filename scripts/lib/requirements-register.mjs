// The assembled requirement register and the requirement ids it names, as the
// Rust originals `scripts/generate-requirement-status.rs` and
// `scripts/check-requirement-status.rs` read them (each mirrors its own
// `fn read_register` and `fn requirement_ids`; the two copies agree).
import { compareStrings, fileName, ioErrorDisplay, joinPath, readDirPaths, readToString } from './requirements-rust-compat.mjs';

export const REQUIREMENT_PARTS = 'docs/requirements/assembled';

const ID_RUN = /[A-Za-z0-9_-]*/y;

/**
 * Mirrors `fn requirement_ids`: every `R<digit>...` run of ASCII
 * alphanumerics, `-` and `_`, in first-seen order without repeats.
 */
export function requirementIds(text) {
  const ids = [];
  for (let index = text.indexOf('R'); index >= 0; index = text.indexOf('R', index + 1)) {
    ID_RUN.lastIndex = index;
    const id = ID_RUN.exec(text)[0];
    if (/^R[0-9]/.test(id) && !ids.includes(id)) ids.push(id);
  }
  return ids;
}

/** Mirrors `fn read_register`; throws the Rust error text. */
export function readRegister(root) {
  const directory = joinPath(root, REQUIREMENT_PARTS);
  let paths;
  try {
    paths = readDirPaths(directory);
  } catch (error) {
    throw new Error(`cannot read ${REQUIREMENT_PARTS}: ${ioErrorDisplay(error)}`);
  }
  const parts = paths.filter((path) => {
    const name = fileName(path);
    return name.startsWith('part-') && name.endsWith('.md');
  }).sort(compareStrings);
  if (!parts.length) throw new Error(`${REQUIREMENT_PARTS} holds no part-NN.md files`);
  let register = '';
  for (const part of parts) {
    try {
      register += readToString(part);
    } catch (error) {
      throw new Error(`cannot read ${part}: ${ioErrorDisplay(error)}`);
    }
    register += '\n';
  }
  return register;
}
