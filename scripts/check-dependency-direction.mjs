#!/usr/bin/env node
// Dependencies point inward (R1188-U2: the Layers and the Ports and Adapters
// principles of link-foundation/code-architecture-principles).
//
// The surfaces (the CLI, the HTTP server, Telegram, the client integrations)
// sit over one solver, so solver code must not name a surface module. And the
// agentic planner reaches the file system, processes and the network only
// through its host port: a module under js/agentic/ that imports a `node:`
// built-in or the Node server directly bypasses the port, unless it is one of
// the adapters data/meta/dependency-direction.lino lists with a reason.
//
// Both counts are held to the ceilings in data/meta/dependency-direction.lino,
// which only fall.
//
// Usage: node scripts/check-dependency-direction.mjs [--list]

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const RULES = 'data/meta/dependency-direction.lino';

/**
 * The quoted values of every `key "<value>"` line of a links-notation text.
 * @param {string} text
 * @param {string} key
 * @returns {Array<string>}
 */
export function valuesOf(text, key) {
  return [...text.matchAll(new RegExp(`^\\s*${key}\\s+"([^"]+)"`, 'gmu'))].map((match) => match[1]);
}

/**
 * The number on the `key <n>` line of a links-notation text.
 * @param {string} text
 * @param {string} key
 * @returns {number}
 */
export function numberOf(text, key) {
  const match = new RegExp(`^\\s*${key}\\s+(\\d+)\\s*$`, 'mu').exec(text);
  if (!match) {
    throw new Error(`${RULES} names no ${key}`);
  }
  return Number(match[1]);
}

/**
 * The top-level Rust module a source path under rust/src/ belongs to.
 * @param {string} path
 * @returns {string}
 */
export function rustModuleOf(path) {
  return path.replace(/^rust\/src\//u, '').split('/')[0].replace(/\.rs$/u, '');
}

/**
 * Whether `module` is one of the surface modules: an exact name, or a name
 * that starts with a listed prefix ending in `_`.
 * @param {string} module
 * @param {Array<string>} surfaces
 * @returns {boolean}
 */
export function isSurface(module, surfaces) {
  return surfaces.some((surface) => (surface.endsWith('_') ? module.startsWith(surface) : module === surface));
}

/**
 * The references solver code makes to surface modules, comments excluded.
 * @param {Array<{path: string, text: string}>} sources
 * @param {Array<string>} surfaces
 * @returns {Array<{path: string, line: number, module: string}>}
 */
export function outwardReferences(sources, surfaces) {
  const found = [];
  for (const { path, text } of sources) {
    if (isSurface(rustModuleOf(path), surfaces)) {
      continue;
    }
    text.split('\n').forEach((line, index) => {
      if (/^\s*\/\//u.test(line)) {
        return;
      }
      for (const match of line.matchAll(/\b(?:crate|super)::([a-z_][a-z0-9_]*)::/gu)) {
        if (isSurface(match[1], surfaces)) {
          found.push({ path, line: index + 1, module: match[1] });
        }
      }
    });
  }
  return found;
}

/**
 * The imports by which a js/agentic/ module bypasses the host port.
 * @param {Array<{path: string, text: string}>} sources
 * @param {Array<string>} adapters
 * @returns {Array<{path: string, line: number, specifier: string}>}
 */
export function portBypasses(sources, adapters) {
  const found = [];
  for (const { path, text } of sources) {
    if (adapters.includes(path)) {
      continue;
    }
    text.split('\n').forEach((line, index) => {
      const match = /(?:\bfrom\s+|\bimport\s*\(\s*|\brequire\s*\(\s*)['"]((?:node:[^'"]+)|(?:(?:\.\.\/)+server\/[^'"]+))['"]/u.exec(line);
      if (match) {
        found.push({ path, line: index + 1, specifier: match[1] });
      }
    });
  }
  return found;
}

function main(argv) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const listed = (pathspec, extension) =>
    execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', pathspec], {
      cwd: root,
      encoding: 'utf8',
      maxBuffer: 1 << 28,
    })
      .split('\n')
      .filter((path) => extension.test(path) && existsSync(join(root, path)))
      .map((path) => ({ path, text: readFileSync(join(root, path), 'utf8') }));
  const rules = readFileSync(join(root, RULES), 'utf8');
  const outward = outwardReferences(listed('rust/src', /\.rs$/u), valuesOf(rules, 'surface-module'));
  const bypasses = portBypasses(listed('js/agentic', /\.m?js$/u), valuesOf(rules, 'host-adapter'));
  const checks = [
    ['outward-references-ceiling', 'solver references to surface modules', outward, (item) => `crate::${item.module}`],
    ['port-bypasses-ceiling', 'js/agentic imports that bypass the host port', bypasses, (item) => item.specifier],
  ];
  let failed = 0;
  for (const [key, label, items, describe] of checks) {
    const ceiling = numberOf(rules, key);
    console.log(`${label}: ${items.length} (ceiling ${ceiling})`);
    if (argv.includes('--list')) {
      for (const item of items) {
        console.log(`  ${item.path}:${item.line} ${describe(item)}`);
      }
    }
    if (items.length > ceiling) {
      console.error(
        `::error file=${RULES}::${items.length - ceiling} new ${label}: dependencies point inward ` +
          '(R1188-U2); reach the surface or the host through a port instead. Run with --list.',
      );
      failed += 1;
    } else if (items.length < ceiling) {
      console.error(`::error file=${RULES}::${label} fell to ${items.length}; lower ${key} in this commit.`);
      failed += 1;
    }
  }
  return failed === 0 ? 0 : 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
