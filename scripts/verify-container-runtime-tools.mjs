#!/usr/bin/env node
// Inspect installed package metadata without importing or executing dependency code.
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';

function metadata(filename) {
  const status = fs.statSync(filename);
  if (!status.isFile() || status.size > 1048576) throw Error('Runtime package metadata bounds');
  const bytes = fs.readFileSync(filename);
  if (bytes.length > 1048576) throw Error('Runtime package metadata changed beyond bounds');
  return JSON.parse(bytes.toString('utf8'));
}

/** Verify the selected tools and the exact SDK edges that previously resolved unpublished releases. */
export function verifyContainerRuntimeTools(directory) {
  const home = fs.realpathSync(directory);
  const manifest = metadata(path.join(home, 'package.json'));
  const observed = {};
  const packages = new Map();
  for (const name of ['start-command', '@link-assistant/agent', 'agent-commander']) {
    const expected = manifest.dependencies?.[name];
    if (!/^\d+\.\d+\.\d+$/u.test(expected ?? '')) throw Error('Exact runtime tool pin required: ' + name);
    const filename = path.join(home, 'node_modules', name, 'package.json');
    const installed = metadata(filename);
    if (installed.name !== name || installed.version !== expected) throw Error('Runtime tool version mismatch: ' + name);
    packages.set(name, filename);
    observed[name] = installed.version;
  }
  const resolveMetadata = (parent, name) => {
    const filename = createRequire(parent).resolve(name + '/package.json');
    const installed = metadata(filename);
    if (installed.name !== name) throw Error('Resolved SDK package identity mismatch: ' + name);
    return {filename, installed};
  };
  for (const name of ['@ai-sdk/azure', '@ai-sdk/amazon-bedrock']) {
    const expected = manifest.overrides?.[name];
    if (!/^\d+\.\d+\.\d+$/u.test(expected ?? '')) throw Error('Exact compatible SDK pin required: ' + name);
    const provider = resolveMetadata(packages.get('@link-assistant/agent'), name);
    if (provider.installed.version !== expected) throw Error('Compatible SDK version mismatch: ' + name);
    observed[name] = provider.installed.version;
    for (const dependency of ['@ai-sdk/openai', '@ai-sdk/deepseek']) {
      const declared = provider.installed.dependencies?.[dependency];
      if (declared === undefined) continue;
      if (!/^\d+\.\d+\.\d+$/u.test(declared)) throw Error('Exact SDK compatibility edge required: ' + dependency);
      const actual = resolveMetadata(provider.filename, dependency).installed.version;
      if (actual !== declared) throw Error('Installed SDK compatibility edge mismatch: ' + name + ' -> ' + dependency);
      observed[name + ' -> ' + dependency] = actual;
    }
  }
  return {verified: true, versions: observed, fullTransitiveLock: false};
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  try {
    console.log(JSON.stringify(verifyContainerRuntimeTools(process.argv[2])));
  } catch (error) {
    console.error(String(error.stack ?? error));
    process.exitCode = 1;
  }
}
