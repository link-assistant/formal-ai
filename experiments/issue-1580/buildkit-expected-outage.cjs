// Fixture-only preload for the actual setup-buildx Node action. A command
// guard in another workflow step cannot contain this action's annotations.
// Preserve every exit status and every unrelated error; downgrade only the
// exact injected outage, whose real mirror recovery is independently checked.
const expected = '::error::Error response from daemon: canonical registry unavailable in deliberate fallback control';
const write = process.stdout.write;
process.stdout.write = function (chunk, ...args) {
  const text = Buffer.isBuffer(chunk) ? chunk.toString('utf8') : chunk;
  if (text === expected + '\n' || text === expected + '\r\n') {
    chunk = text.replace('::error::', '::notice::Expected fallback control: ');
  }
  return Reflect.apply(write, this, [chunk, ...args]);
};
