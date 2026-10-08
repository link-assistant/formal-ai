// `crate::orchestration::permission` (rust/src/orchestration/permission.rs): the
// unforgeable-by-default capability bound to one workspace. Running an external
// process requires an explicit grant for exactly the workspace it will run in.

import { realpathSync } from 'node:fs';
import path from 'node:path';

/** The components of a path the way `Path` equality reads them. */
const components = (value) => value.split(path.sep).filter((part) => part !== '' && part !== '.');

/** Mirrors `fn same_path` in rust/src/orchestration/permission.rs. */
function samePath(left, right) {
  try {
    return realpathSync(left) === realpathSync(right);
  } catch {
    const a = components(left);
    const b = components(right);
    return path.isAbsolute(left) === path.isAbsolute(right) && a.length === b.length && a.every((part, index) => part === b[index]);
  }
}

/**
 * Mirrors `AgentRunPermission::default` in rust/src/orchestration/permission.rs: no workspace is granted.
 * @returns {{workspace: string|null}}
 */
export function noPermission() {
  return { workspace: null };
}

/**
 * Mirrors `AgentRunPermission::grant_for` in rust/src/orchestration/permission.rs: explicitly grants external-process
 * access to `workspace`.
 * @param {string} workspace
 * @returns {{workspace: string|null}}
 */
export function grantFor(workspace) {
  return { workspace: String(workspace) };
}

/**
 * Mirrors `AgentRunPermission::permits` in rust/src/orchestration/permission.rs: whether the grant covers `workspace`.
 * @param {{workspace: string|null}} permission
 * @param {string} workspace
 */
export function permits(permission, workspace) {
  return permission.workspace !== null && samePath(permission.workspace, String(workspace));
}
