// Production server installs the real vendored Rust syntax grammar once.
// Local bounded JS drivers explicitly opt out; direct installNodeHost defaults off.
const installed = new WeakMap();

/** Install one immutable host capability policy for a worker. */
export async function ensureNodeHost(ctx, options = {}) {
  const existing = installed.get(ctx.worker);
  const descriptor = Object.getOwnPropertyDescriptor(options, 'sourceUsageRustGrammar');
  if (descriptor && !Object.hasOwn(descriptor, 'value')) throw Error('host_source_usage_grammar_must_be_boolean');
  const sourceUsageRustGrammar = descriptor ? descriptor.value : existing?.sourceUsageRustGrammar ?? true;
  if (typeof sourceUsageRustGrammar !== 'boolean') throw Error('host_source_usage_grammar_must_be_boolean');
  if (existing && existing.sourceUsageRustGrammar !== sourceUsageRustGrammar) {
    throw Error('host_source_usage_grammar_installation_conflict');
  }
  if (!existing) {
    const promise = import('../agentic/node-host.mjs').then(({
      installNodeHost
    }) => installNodeHost(ctx.worker, {
      prerequisiteGrant: ctx.prerequisiteGrant ?? null,
      sourceUsageRustGrammar
    }));
    installed.set(ctx.worker, {
      promise,
      sourceUsageRustGrammar
    });
  }
  await installed.get(ctx.worker).promise;
}
