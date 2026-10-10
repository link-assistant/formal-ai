import { installNodeHost } from '../agentic/node-host.mjs';
import { host, installHost } from '../agentic/host.mjs';
import { createNodeSourceSessionHost } from './node-source-session-host.mjs';
/** Constructor-owned Node source session. Request data and receipts cannot install this boundary. */
export async function installDefaultNodeSourceHost(worker) {
  const realm = await installNodeHost(worker);
  const base = host();
  const session = createNodeSourceSessionHost(base.readText);
  installHost({ ...base, sourceSession: session, sourceOperation: session.sourceOperation });
  return realm;
}
