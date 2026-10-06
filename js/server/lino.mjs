// Links Notation reading for the JavaScript server.
//
// The server reads the same parser the browser worker uses
// (`js/seed_loader.js`, `FormalAiSeed.parse`), loaded once into a private
// `node:vm` realm, so the route manifest, the server wording and every seed
// file are read by one implementation everywhere.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

export const REPO_ROOT = path.resolve(import.meta.dirname, '../..');

let parser = null;

function seedParser() {
  if (parser) return parser;
  const sandbox = { console };
  sandbox.self = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.window = sandbox;
  const context = vm.createContext(sandbox);
  const file = path.join(REPO_ROOT, 'js/seed_loader.js');
  new vm.Script(readFileSync(file, 'utf8'), { filename: file }).runInContext(context);
  parser = sandbox.FormalAiSeed.parse;
  return parser;
}

/**
 * Parse Links Notation text into `{ name, id, value, children }` nodes.
 * @param {string} text
 * @returns {{name: string, id: string, value: string, children: Array<object>}}
 */
export function parseLino(text) {
  return JSON.parse(JSON.stringify(seedParser()(text)));
}

/** @param {string} relative @returns {string} */
export function readRepoFile(relative) {
  return readFileSync(path.join(REPO_ROOT, relative), 'utf8');
}

/** @param {object} node @param {string} name @returns {Array<object>} */
export function childrenNamed(node, name) {
  return (node?.children || []).filter((child) => child.name === name);
}

/** @param {object} node @param {string} name @returns {string} */
export function childValue(node, name) {
  return childrenNamed(node, name)[0]?.value ?? '';
}
