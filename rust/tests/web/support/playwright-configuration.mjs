// Evaluate checked-in configuration without starting browsers or build commands.
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { runInNewContext } from 'node:vm';
import { REPO_ROOT } from './browser-runtime.mjs';

export function loadPlaywrightConfig(relative, env = {}) {
  const filename = path.join(REPO_ROOT, relative);
  const module = { exports: {} };
  const require = createRequire(filename);
  runInNewContext(readFileSync(filename, 'utf8'), {
    module, __dirname: path.dirname(filename), process: { env },
    require: name => name === '@playwright/test'
      ? { defineConfig: config => config, devices: { 'Desktop Chrome': {} } }
      : require(name),
  }, { filename, timeout: 1000 });
  return module.exports;
}
