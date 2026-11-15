import test from 'node:test';
import assert from 'node:assert/strict';
import {
  mkdtempSync, readFileSync, writeFileSync, rmSync, unlinkSync
} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, posix} from 'node:path';
import {pathToFileURL} from 'node:url';
import {isBuiltin} from 'node:module';
import {execFileSync} from 'node:child_process';
import * as parser from '@babel/parser';
import YAML from 'yaml';
import {
  PROTOCOL_HELPERS, validateProtocolEvidence, sha256
} from '../../../scripts/native-release-evidence.mjs';

const scripts = new URL('../../../scripts/', import.meta.url);
const workflow = YAML.parse(readFileSync(
  new URL('../../../.github/workflows/desktop-release.yml', import.meta.url),
  'utf8'
));
const resolver = 'resolve-package-release.mjs';

function moduleImports(node, imports = []) {
  if (node === null || typeof node !== 'object') return imports;
  if (['ImportDeclaration', 'ExportNamedDeclaration', 'ExportAllDeclaration']
    .includes(node.type) && node.source) {
    imports.push(node.source.value);
  }
  if (node.type === 'ImportExpression' ||
      node.type === 'CallExpression' && node.callee.type === 'Import') {
    const source = node.source ?? node.arguments[0];
    assert.equal(source.type, 'StringLiteral', 'dynamic helper imports need explicit custody');
    imports.push(source.value);
  }
  for (const value of Object.values(node)) {
    if (Array.isArray(value)) {
      for (const child of value) moduleImports(child, imports);
    } else if (value && typeof value === 'object') {
      moduleImports(value, imports);
    }
  }
  return imports;
}

test('every detached native protocol captures and hashes its full helper import closure', () => {
  assert.ok(PROTOCOL_HELPERS.includes(resolver));
  const actualBytes = Object.fromEntries(PROTOCOL_HELPERS.map(name => [
    name, readFileSync(new URL(name, scripts))
  ]));
  for (const [name, bytes] of Object.entries(actualBytes)) {
    const syntax = parser.parse(bytes.toString('utf8'), {sourceType: 'module'});
    for (const dependency of moduleImports(syntax)) {
      if (isBuiltin(dependency)) continue;
      assert.ok(dependency.startsWith('./'), 'unexpected external helper dependency');
      const required = posix.normalize(posix.join(posix.dirname(name), dependency));
      assert.ok(PROTOCOL_HELPERS.includes(required), name + ' requires ' + required);
    }
  }

  const consumers = Object.entries(workflow.jobs).filter(([, job]) =>
    job.steps?.some(step => step.run?.includes('FORMAL_AI_NATIVE_PROTOCOL_DIR'))
  );
  assert.ok(consumers.length > 0);
  for (const [name, job] of consumers) {
    const captures = job.steps.filter(step =>
      step.name?.startsWith('Capture release protocol')
    );
    assert.equal(captures.length, 1, name + ' must select one pinned protocol');
    const capture = captures[0].run;
    const copied = [...capture.matchAll(
      /(?:^|\s)scripts\/([A-Za-z0-9-]+\.mjs)(?=\s|$)/gu
    )].map(match => match[1]);
    for (const helper of PROTOCOL_HELPERS) {
      assert.equal(
        copied.filter(value => value === helper).length,
        1,
        name + ' must copy ' + helper
      );
    }
    const declaration = /const files=(\[[^\]]+\])\.map/u.exec(capture);
    assert.ok(declaration, name + ' must hash captured helper bytes');
    const names = parser.parseExpression(declaration[1]).elements.map(element => {
      assert.equal(element.type, 'StringLiteral');
      return element.value;
    });
    assert.deepEqual(names.sort(), [...PROTOCOL_HELPERS].sort(), name);
  }

  const directory = mkdtempSync(join(tmpdir(), 'detached-native-protocol-'));
  try {
    for (const [name, bytes] of Object.entries(actualBytes)) {
      writeFileSync(join(directory, name), bytes);
    }
    const protocol = {
      version: 1,
      commit: 'a'.repeat(40),
      tree: 'b'.repeat(40),
      files: PROTOCOL_HELPERS.map(name => ({
        name, bytes: actualBytes[name].length, sha256: sha256(actualBytes[name])
      }))
    };
    const record = Buffer.from(JSON.stringify(protocol));
    const bindings = {
      protocolSha256: sha256(record),
      protocolCommit: protocol.commit,
      protocolTree: protocol.tree
    };
    validateProtocolEvidence(record, bindings, actualBytes);

    const detached = pathToFileURL(join(directory, 'native-release-trust.mjs')).href;
    const program = 'const trust = await import(' + JSON.stringify(detached) + ');' +
      'if (typeof trust.validateSelectedSourceAuthority !== "function") throw Error("missing trust export");';
    const execute = () => execFileSync(process.execPath, [
      '--input-type=module', '-e', program
    ], {cwd: directory, encoding: 'utf8', timeout: 8000, stdio: ['ignore', 'pipe', 'pipe']});
    assert.equal(execute(), '');

    const omittedBytes = {...actualBytes};
    delete omittedBytes[resolver];
    assert.throws(() => validateProtocolEvidence(record, bindings, omittedBytes));
    assert.throws(() => validateProtocolEvidence(record, bindings, {
      ...actualBytes, [resolver]: Buffer.from('tampered resolver')
    }));
    const omittedRecord = Buffer.from(JSON.stringify({
      ...protocol, files: protocol.files.filter(file => file.name !== resolver)
    }));
    assert.throws(() => validateProtocolEvidence(
      omittedRecord, {...bindings, protocolSha256: sha256(omittedRecord)}, actualBytes
    ));
    unlinkSync(join(directory, resolver));
    assert.throws(execute, /ERR_MODULE_NOT_FOUND/);
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});
