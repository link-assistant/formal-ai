// `write_session` and `read_session` of `crate::orchestration::replay`
// (rust/src/orchestration/replay.rs): the canonical session bytes on disk.
// The pure replay verifier is orchestration_replay.mjs; this module adds the
// file edge Rust keeps beside it.

import fs from 'node:fs';

import { ReplayError, canonicalSessionText, replaySession } from './orchestration_replay.mjs';
import { ioErrorFrom } from './orchestration_workspace.mjs';

/** Mirrors `fn write_session` in rust/src/orchestration/replay.rs. */
export function writeSession(file, session) {
  const rendered = canonicalSessionText(session);
  try {
    fs.writeFileSync(file, rendered);
  } catch (error) {
    throw new ReplayError('io', ioErrorFrom(error).message);
  }
}

/** Mirrors `fn read_session` in rust/src/orchestration/replay.rs: the replayed session at `file`. */
export function readSession(file) {
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (error) {
    throw new ReplayError('io', ioErrorFrom(error).message);
  }
  return replaySession(text);
}
