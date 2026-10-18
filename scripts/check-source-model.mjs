#!/usr/bin/env node
// Actual shared grammar acceptance: content is the oracle, never a node census.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {parseLinks} from '../js/logging/lino-codec.mjs';
import {sourceToLinks, sourceFromLinks, readSourceLinks, writeSourceLinks,
  readSyntaxRules, verifySourceLinks, replaceSourceText} from '../js/meta/readable-source.js';
import {sourceModelFrontend, readSourceText} from './source-model.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/** @param {string} directory @param {string} commit */
export async function checkSourceModel(directory, commit) {
  const parse = await sourceModelFrontend(directory, commit);
  const rules = readSyntaxRules(readSourceText(path.join(ROOT, 'data/meta/source-model/syntax-rules.lino')));
  const options = {parse, rules};
  const corpus = parseLinks(readSourceText(path.join(ROOT, 'rust/tests/fixtures/source-model/cases.lino')), {quotedText: 'json'});
  const observed = [];
  for (const row of corpus) {
    assert.equal(row[0], 'source-case');
    const value = name => {
      const fields = row.filter(item => Array.isArray(item) && item[0] === name);
      assert.equal(fields.length, 1);
      assert.equal(fields[0].length, 2);
      const text = fields[0][1];
      assert(text && typeof text === 'object' && !Array.isArray(text) && typeof text.quoted === 'string');
      return text.quoted;
    };
    const language = value('language'), source = value('source');
    const expected = readSourceText(path.join(ROOT, 'rust/tests/fixtures/source-model', value('document')));
    const model = sourceToLinks(source, language, options);
    assert.equal(writeSourceLinks(model), expected, 'Complete recursive content drift');
    const restored = readSourceLinks(expected);
    assert.equal(sourceFromLinks(restored), source);
    assert(verifySourceLinks(restored, options));
    // A new process receives the persisted content and parser; never the original source.
    const cli = spawnSync(process.execPath, [path.join(ROOT, 'scripts/source-model.mjs'),
      '--from', 'meta', '--input', path.join(ROOT, 'rust/tests/fixtures/source-model', value('document')),
      '--meta-language', directory, '--commit', commit], {
      cwd: ROOT, encoding: 'utf8', timeout: 30000, maxBuffer: 2 * 1024 * 1024,
    });
    assert.equal(cli.status, 0, cli.stderr);assert.equal(cli.stdout, source);
    const rename = {role: 'name', form: 'identifier', expectedText: 'keep', replacementText: 'preserved'};
    const updated = replaceSourceText(model, rename, options);
    assert.equal(sourceFromLinks(updated.document), source.replace('keep', 'preserved'));
    assert.deepEqual(replaceSourceText(updated.document, updated.inverse, options).document, model);
    assert.throws(() => replaceSourceText(model, {...rename, expectedText: 'missing'}, options));
    const tampered = structuredClone(model);
    const changeKind = node => {
      for (const item of node) if (Array.isArray(item)) {
        if (item[0] === 'form' && item[1]?.quoted?.includes('function')) { item[1].quoted = 'wrong_construct'; return true; }
        if (changeKind(item)) return true;
      }
      return false;
    };
    assert(changeKind(tampered));
    assert.throws(() => verifySourceLinks(tampered, options), /SourceStructureMismatch/u);
    observed.push({language, exactContent: true, sourceFreeCli: true, independentReparse: true, editAndInverse: true});
  }
  assert(observed.length > 0, 'Empty source-model acceptance corpus');
  // The same operations consume actual product code; no pinned expected node counts.
  for (const [file, language] of [
    ['js/meta/readable-source.js', 'JavaScript'], ['ts/meta/readable-source.ts', 'TypeScript'],
    ['rust/src/source_fetch.rs', 'Rust'],
  ]) {
    const source = readSourceText(path.join(ROOT, file));
    const loaded = readSourceLinks(writeSourceLinks(sourceToLinks(source, language, options)));
    assert.equal(sourceFromLinks(loaded), source, file);
    assert(verifySourceLinks(loaded, options));
  }
  for (const language of ['JavaScript', 'TypeScript', 'Rust'])
    assert.throws(() => sourceToLinks('function (', language, options));
  return {observed, actualProductSource: true, pinnedGrammar: commit,
    semanticEquivalenceClaim: false, commentUnderstandingClaim: false};
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    assert.equal(args[0], '--meta-language');assert.equal(args.length, 2);
    const {workflowPin} = await import('./translate-js-rust.mjs');
    const commit = workflowPin(fs.readFileSync(path.join(ROOT, '.github/workflows/layered-ci.yml'), 'utf8'));
    console.log(JSON.stringify(await checkSourceModel(path.resolve(args[1]), commit)));
  } catch (error) { console.error(error.stack);process.exitCode = 1; }
}
