import { installNodeHost } from '../agentic/node-host.mjs';
/** The ordinary Node installer owns source-session registration; no second issuer is created. */
export async function installDefaultNodeSourceHost(worker) {
  return installNodeHost(worker);
}
