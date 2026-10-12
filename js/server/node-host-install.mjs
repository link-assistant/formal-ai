// Install the agentic planner host over a server's worker once, for every
// server path that runs js/agentic code (the agentic planner, and the native
// conversation-summary envelope in js/server/solve.mjs).

const installed = new WeakMap();

/** Install the planner host over `ctx.worker` once. @param {{worker: object}} ctx */
export async function ensureNodeHost(ctx) {
  if (!installed.has(ctx.worker)) {
    installed.set(ctx.worker, import('../agentic/node-host.mjs').then(({ installNodeHost }) => installNodeHost(ctx.worker)));
  }
  await installed.get(ctx.worker);
}
