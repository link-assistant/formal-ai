const fs = require('node:fs');
const Module = require('node:module');
const {project} = require('./typescript-projection.cjs');
const read = fs.readFileSync;
fs.readFileSync = function(file, options) {
  const bytes = project(file, 'actual-readFileSync');
  if (bytes === null) return read.apply(this, arguments);
  const encoding = typeof options === 'string' ? options : options?.encoding;
  return encoding ? bytes.toString(encoding) : bytes;
};
const extension = Module._extensions['.js'];
Module._extensions['.js'] = function(module, file) {
  const bytes = project(file, 'actual-commonjs-compile');
  if (bytes === null) return extension(module, file);
  module._compile(bytes.toString('utf8'), file);
};
Module.syncBuiltinESMExports();
