// `crate::dialog_log::current_dialog_id` (rust/src/dialog_log.rs): the session
// id the request being served declared. The Rust server keeps it in a
// thread-local set from the `x-formal-ai-dialog-id` header; here the installed
// host may answer it through an optional `currentDialogId()`.

import { hasHost, host } from '../host.mjs';

/** Mirrors `fn current_dialog_id` in rust/src/dialog_log.rs: the id, or null. */
export function currentDialogId() {
  if (!hasHost()) return null;
  const id = host().currentDialogId?.();
  return typeof id === 'string' ? id : null;
}
