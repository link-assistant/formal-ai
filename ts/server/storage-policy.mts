// Real storage measurement and persisted auto-free-space consent:
// rust/src/storage_policy.rs (`measure_storage`,
// `auto_free_space_preference_path`, `auto_free_space_choice`,
// `auto_free_space_enabled`, `persist_auto_free_space_choice`,
// `plan_for_real_storage`, `apply_auto_free_space_for_write`,
// `apply_auto_free_space_with_snapshot`, `existing_ancestor`).
//
// Missing or unreadable consent means "not enabled", so the default can never
// silently delete data.

import fs from 'node:fs';
import path from 'node:path';

import { applyDreamingPlan, dreamingConfig, planMemoryDreaming } from './dreaming-plan.mjs';
import { memoizedReplay } from './dreaming-replay.mjs';
import { trim } from './dreaming-support.mjs';

export const AUTO_FREE_SPACE_CHOICE = Object.freeze({
  NeverAsked: 'NeverAsked',
  Declined: 'Declined',
  Enabled: 'Enabled',
});

/** Mirrors rust/src/storage_policy.rs `existing_ancestor`. */
export function existingAncestor(file) {
  let candidate;
  let isDirectory = false;
  try {
    isDirectory = fs.statSync(file).isDirectory();
  } catch {
    isDirectory = false;
  }
  if (isDirectory) candidate = file;
  else {
    const parent = path.dirname(file);
    candidate = parent && parent !== file ? parent : '.';
  }
  while (!fs.existsSync(candidate)) {
    const parent = path.dirname(candidate);
    if (parent === candidate) return '.';
    candidate = parent;
  }
  return candidate;
}

/**
 * Mirrors rust/src/storage_policy.rs `measure_storage`: `fs2::total_space`
 * and `fs2::available_space` of the nearest existing ancestor (the same
 * `statfs` reading js/server/memory-store.mjs `currentContextCapacity` uses).
 */
export function measureStorage(memoryPath) {
  const stats = fs.statfsSync(existingAncestor(memoryPath), { bigint: true });
  return {
    capacity_bytes: Number(stats.blocks * stats.bsize),
    free_bytes: Number(stats.bavail * stats.bsize),
  };
}

/** Mirrors rust/src/storage_policy.rs `auto_free_space_preference_path`. */
export function autoFreeSpacePreferencePath(memoryPath) {
  const filename = path.basename(memoryPath) || 'formal-ai-memory.lino';
  return path.join(path.dirname(memoryPath), `${filename}.auto-free-space`);
}

/** Mirrors rust/src/storage_policy.rs `auto_free_space_choice`. */
export function autoFreeSpaceChoice(memoryPath) {
  let value;
  try {
    value = fs.readFileSync(autoFreeSpacePreferencePath(memoryPath), 'utf8');
  } catch {
    return AUTO_FREE_SPACE_CHOICE.NeverAsked;
  }
  if (trim(value) === 'enabled') return AUTO_FREE_SPACE_CHOICE.Enabled;
  if (trim(value) === 'disabled') return AUTO_FREE_SPACE_CHOICE.Declined;
  return AUTO_FREE_SPACE_CHOICE.NeverAsked;
}

/** Mirrors rust/src/storage_policy.rs `auto_free_space_enabled`. */
export function autoFreeSpaceEnabled(memoryPath) {
  return autoFreeSpaceChoice(memoryPath) === AUTO_FREE_SPACE_CHOICE.Enabled;
}

/** Mirrors rust/src/storage_policy.rs `persist_auto_free_space_choice`. */
export function persistAutoFreeSpaceChoice(memoryPath, enabled) {
  const preference = autoFreeSpacePreferencePath(memoryPath);
  const parent = path.dirname(preference);
  if (parent) fs.mkdirSync(parent, { recursive: true });
  fs.writeFileSync(preference, enabled ? 'enabled\n' : 'disabled\n');
}

/** Mirrors rust/src/storage_policy.rs `plan_for_real_storage`. */
export function planForRealStorage(events, memoryPath, incomingBytes, replay = memoizedReplay()) {
  const snapshot = measureStorage(memoryPath);
  return planMemoryDreaming(events, dreamingConfig({
    storage_capacity_bytes: snapshot.capacity_bytes,
    free_bytes: snapshot.free_bytes,
    incoming_bytes: incomingBytes,
  }), replay);
}

/**
 * Mirrors rust/src/storage_policy.rs `apply_auto_free_space_with_snapshot`.
 * @returns {null | {events: Array<object>, plan: object, outcome: object}}
 */
export function applyAutoFreeSpaceWithSnapshot(events, memoryPath, incomingBytes, snapshot, replay = memoizedReplay()) {
  if (!autoFreeSpaceEnabled(memoryPath)) return null;
  const plan = planMemoryDreaming(events, dreamingConfig({
    storage_capacity_bytes: snapshot.capacity_bytes,
    free_bytes: snapshot.free_bytes,
    incoming_bytes: incomingBytes,
  }), replay);
  const applied = applyDreamingPlan(events, plan, replay);
  return { events: applied.events, plan, outcome: applied.outcome };
}

/**
 * Mirrors rust/src/storage_policy.rs `apply_auto_free_space_for_write`: only
 * with persisted consent, sized by the caller's real next write. Throws on a
 * filesystem error, like the Rust `io::Result`.
 */
export function applyAutoFreeSpaceForWrite(events, memoryPath, incomingBytes, replay = memoizedReplay()) {
  if (!autoFreeSpaceEnabled(memoryPath)) return null;
  return applyAutoFreeSpaceWithSnapshot(events, memoryPath, incomingBytes, measureStorage(memoryPath), replay);
}
