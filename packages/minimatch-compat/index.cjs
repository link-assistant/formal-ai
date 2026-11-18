// Older build tools call require('minimatch') directly. Keep their API while
// delegating all pattern parsing/matching to the patched upstream implementation.
const upstream = require('minimatch-modern');
function callable(api) {
  const match = typeof api === 'function' ? api : api.minimatch;
  function legacy(...arguments_) { return match(...arguments_); }
  Object.assign(legacy, api);
  legacy.defaults = options => callable(api.defaults(options));
  return legacy;
}
module.exports = callable(upstream);
