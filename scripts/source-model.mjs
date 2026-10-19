#!/usr/bin/env node
// Readable source-content operations with explicit pinned or complete grammar providers.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {sourceToLinks, sourceFromLinks, readSourceLinks, writeSourceLinks,
  verifySourceLinks, readSyntaxRules, ownedSourceLinks, sourceOwner} from '../js/meta/readable-source.js';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const LANGUAGES = new Map([['js', 'JavaScript'], ['javascript', 'JavaScript'],
  ['ts', 'TypeScript'], ['typescript', 'TypeScript'], ['rs', 'Rust'], ['rust', 'Rust']]);

/** Reject unsupported encoding instead of replacing source characters.
 * @param {string} file @returns {string} */
export function readSourceText(file) {
  const status = fs.lstatSync(file);
  if (!status.isFile() || status.isSymbolicLink()) throw Error('RegularSourceFileRequired');
  if (status.size > 4 * 1024 * 1024) throw Error('SourceModelInputBudgetExceeded');
  return new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(fs.readFileSync(file));
}

/** Load a declared pinned development SDK. This never installs or downloads it.
 * @param {string} directory @param {string} expectedCommit */
export async function sourceModelFrontend(directory, expectedCommit) {
  if (!/^[0-9a-f]{40}$/u.test(expectedCommit)) throw Error('PinnedMetaLanguageRequired');
  const marker = path.join(directory, '.formal-ai-meta-language-commit');
  const actual = fs.existsSync(path.join(directory, '.git'))
    ? execFileSync('git', ['-C', directory, 'rev-parse', 'HEAD'], {encoding: 'utf8'}).trim()
    : fs.existsSync(marker) ? fs.readFileSync(marker, 'utf8').trim() : null;
  assert.equal(actual, expectedCommit, 'MetaLanguageSourceModelPinMismatch');
  const {parseProgrammingLanguage} = await import(pathToFileURL(path.join(directory,
    'js/src/programming-language-parser.js')).href);
  return parseProgrammingLanguage;
}

/** @param {string[]} args */
export async function runSourceModel(args) {
  const values = new Map();
  let check = false;
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === '--check') { if (check) throw Error('DuplicateSourceModelOption'); check = true; continue; }
    if (!['--from', '--input', '--output', '--meta-language', '--commit', '--frontend'].includes(arg)
      || values.has(arg) || index + 1 >= args.length) throw Error('InvalidSourceModelOption:' + arg);
    values.set(arg, args[++index]);
  }
  const input = values.get('--input'), from = values.get('--from');
  const directory = values.get('--meta-language');
  const provider = values.get('--frontend') ?? 'pinned';
  if (!['pinned', 'complete'].includes(provider)) throw Error('UnsupportedSourceModelFrontend');
  if (!input || !from) throw Error('SourceModelRequiresFromInputAndPinnedMetaLanguage');
  const source = readSourceText(path.resolve(input));
  let model = from === 'meta' ? readSourceLinks(source) : null;
  const language = model
    ? (model[0] === 'owned-source' ? model[2] : model).find(node => Array.isArray(node) && node[0] === 'language')?.[1]?.quoted
    : LANGUAGES.get(from.toLowerCase());
  if (!language) throw Error('UnsupportedSourceModelLanguage:' + from);
  let runtime;
  try {
    let options;
    if (provider === 'complete') {
      if (directory || values.has('--commit')) throw Error('AmbiguousSourceModelFrontend');
      const {sourceCstFrontends} = await import('./lib/source-cst-frontends.mjs');
      runtime = await sourceCstFrontends();
      options = runtime.forSource(model ? sourceOwner(model) ?? input : input, language);
    } else {
      if (!directory) throw Error('SourceModelRequiresFromInputAndPinnedMetaLanguage');
      let commit = values.get('--commit');
      if (!commit) {
        const {workflowPin} = await import('./translate-js-rust.mjs');
        commit = workflowPin(fs.readFileSync(path.join(ROOT, '.github/workflows/layered-ci.yml'), 'utf8'));
      }
      const parse = await sourceModelFrontend(path.resolve(directory), commit);
      const rules = readSyntaxRules(readSourceText(path.join(ROOT, 'data/meta/source-model/syntax-rules.lino')));
      options = {parse, rules};
    }
    if (!model) {
      model = sourceToLinks(source, language, options);
      if (provider === 'complete') model = ownedSourceLinks(path.basename(input), model);
    }
    verifySourceLinks(readSourceLinks(writeSourceLinks(model)), options);
  } finally {
    runtime?.close();
  }
  if (check) {
    if (values.has('--output')) throw Error('SourceModelCheckDoesNotWrite');
    return JSON.stringify({valid: true, recursiveContent: true, exactSource: true,
      independentGrammarReparse: true, semanticEquivalenceClaim: false}) + '\n';
  }
  const output = from === 'meta' ? sourceFromLinks(model) : writeSourceLinks(model);
  const destination = values.get('--output');
  if (destination) { fs.writeFileSync(path.resolve(destination), output, {flag: 'wx'}); return ''; }
  return output;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { process.stdout.write(await runSourceModel(process.argv.slice(2))); }
  catch (error) { process.stderr.write(String(error.message ?? error) + '\n'); process.exitCode = 1; }
}
