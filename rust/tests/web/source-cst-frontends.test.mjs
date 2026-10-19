import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { sourceCstFrontends } from '../../../scripts/lib/source-cst-frontends.mjs';
import { createSourceCstPacket, restoreSourceCstPacket } from '../../../scripts/lib/source-cst-packets.mjs';
import { sourceCstRuntimeFiles } from '../../../scripts/generate-source-cst-runtime.mjs';
import { provisionSourceCstRuntime } from '../../../scripts/provision-source-cst-runtime.mjs';
const root = fileURLToPath(new URL('../../../', import.meta.url));
test('actual grammar frontends retain shorthand aliases, punctuation, comments, Unicode and empty programs', async () => {
  const frontends = await sourceCstFrontends();
  try {
    for (const [name, language, source] of [['js/example.mjs', 'JavaScript', '\uFEFF// Москва 東京 हिन्दी 😀\r\nconst {name = "x"} = object; export const value = {name};\r\n'], ['ts/example.ts', 'TypeScript', '/* preserved */\ninterface Box<T> { value: T; }\nconst box: Box<string> = {value: "é"};\n'], ['rust/src/example.rs', 'Rust', '//! retained documentation\nmacro_rules! example { ($value:expr) => {{ $value }}; }\nfn main() { let value = r#"東京"#; println!("{}", example!(value)); }\n'], ['js/empty.mjs', 'JavaScript', '']]) {
      const frontend = frontends.forSource(name, language),
        packet = createSourceCstPacket({
          path: name,
          language,
          source
        }, frontend);
      assert.equal(restoreSourceCstPacket(packet.receipt, packet.compressed).source, source);
      assert(frontend.provenance.syntaxRulesSha256.match(/^[a-f0-9]{64}$/u));
      if (language === 'Rust') assert.match(frontend.provenance.grammarSha256, /^[a-f0-9]{64}$/u);else assert.match(frontend.provenance.version, /^\d+\.\d+\.\d+$/u);
    }
    assert.throws(() => frontends.forSource('example.py', 'Python'), /UnsupportedSourceCstLanguage/u);
    for (const [name, language, source] of [['bad.js', 'JavaScript', 'const = ;'], ['bad.ts', 'TypeScript', 'interface {'], ['bad.rs', 'Rust', 'fn main( {']]) {
      assert.throws(() => createSourceCstPacket({
        path: name,
        language,
        source
      }, frontends.forSource(name, language)));
    }
  } finally {
    frontends.close();
  }
});
test('the committed CST runtime is derived from canonical frozen dependencies and refuses missing integrity', () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'source-cst-lock-control-'));
  try {
    const expected = sourceCstRuntimeFiles(root);
    for (const [name, body] of expected) assert.equal(fs.readFileSync(path.join(root, 'packages/source-cst-runtime', name), 'utf8'), body);
    fs.writeFileSync(path.join(scratch, 'package.json'), fs.readFileSync(path.join(root, 'package.json')));
    const original = fs.readFileSync(path.join(root, 'bun.lock'), 'utf8');
    fs.writeFileSync(path.join(scratch, 'bun.lock'), original.replace(/^(    "@babel\/parser": \[.*), "sha512-[^"]+"\],?$/mu, '$1, "invalid-integrity"],'));
    assert.throws(() => sourceCstRuntimeFiles(scratch));
    fs.writeFileSync(path.join(scratch, 'bun.lock'), original.split('\n').filter(line => !line.startsWith('    "@babel/helper-string-parser": ')).join('\n'));
    assert.throws(() => sourceCstRuntimeFiles(scratch), /missing frozen CST dependency/u);
  } finally {
    fs.rmSync(scratch, {
      recursive: true,
      force: true
    });
  }
});
test('hosted provisioning uses frozen npm ci without lifecycle scripts and refuses wrong installed grammar', () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'source-cst-install-control-')),
    previous = {
      actions: process.env.GITHUB_ACTIONS,
      sink: process.env.GITHUB_ENV
    };
  try {
    delete process.env.GITHUB_ACTIONS;
    let calls = 0;
    assert.throws(() => provisionSourceCstRuntime(path.join(scratch, 'forbidden'), {
      root,
      run: () => {
        calls++;
        return {
          status: 0
        };
      }
    }), /hosted-only/u);
    assert.equal(calls, 0);
    assert.equal(fs.existsSync(path.join(scratch, 'forbidden')), false);
    process.env.GITHUB_ACTIONS = 'true';
    process.env.GITHUB_ENV = path.join(scratch, 'env');
    const wanted = JSON.parse(fs.readFileSync(path.join(root, 'packages/source-cst-runtime/package.json'), 'utf8')).dependencies['@babel/parser'];
    const fakeInstall = version => (command, args) => {
      calls++;
      assert.equal(command, 'npm');
      assert.deepEqual(args.filter(arg => arg !== '--prefix' && !arg.startsWith(scratch)), ['ci', '--omit=dev', '--ignore-scripts', '--no-audit', '--no-fund']);
      const directory = args[args.indexOf('--prefix') + 1],
        target = path.join(directory, 'node_modules/@babel/parser');
      fs.mkdirSync(target, {
        recursive: true
      });
      fs.writeFileSync(path.join(target, 'package.json'), JSON.stringify({
        version
      }));
      return {
        status: 0
      };
    };
    const good = path.join(scratch, 'good');
    assert.deepEqual(provisionSourceCstRuntime(good, {
      root,
      run: fakeInstall(wanted)
    }), {
      version: wanted
    });
    assert.equal(JSON.parse(fs.readFileSync(path.join(good, 'installation.json'))).lifecycleScripts, false);
    assert.equal(fs.readFileSync(process.env.GITHUB_ENV, 'utf8'), 'FORMAL_AI_CST_RUNTIME=' + good + '\n');
    assert.throws(() => provisionSourceCstRuntime(good, {
      root,
      run: fakeInstall(wanted)
    }), /fresh grammar/u);
    assert.throws(() => provisionSourceCstRuntime(path.join(scratch, 'wrong'), {
      root,
      run: fakeInstall('0.0.0')
    }), /installed grammar differs/u);
    assert.equal(fs.readFileSync(process.env.GITHUB_ENV, 'utf8'), 'FORMAL_AI_CST_RUNTIME=' + good + '\n');
    assert.equal(calls, 2);
  } finally {
    for (const [key, value] of [['GITHUB_ACTIONS', previous.actions], ['GITHUB_ENV', previous.sink]]) {
      if (value === undefined) delete process.env[key];else process.env[key] = value;
    }
    fs.rmSync(scratch, {
      recursive: true,
      force: true
    });
  }
});


test('complete source CLI restores every language from readable content after original deletion', async () => {
  const {runSourceModel} = await import('../../../scripts/source-model.mjs');
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(),'source-cst-cli-control-'));
  try {
    for (const [extension, language, source] of [['mjs','js','// retained 😀\r\nexport const {name} = object;\r\n'],['tsx','ts','/* comment */\nexport const view = <span>東京</span>;\n'],['rs','rust','// retained\nfn main() { println!("hello"); }\n']]) {
      const input = path.join(scratch, 'example.' + extension), model = path.join(scratch, 'example.' + extension + '.lino');
      fs.writeFileSync(input, source);
      const notation = await runSourceModel(['--frontend','complete','--from',language,'--input',input]);
      assert.match(notation,/owned-source/u);
      fs.writeFileSync(model,notation);fs.unlinkSync(input);
      assert.equal(await runSourceModel(['--frontend','complete','--from','meta','--input',model]),source);
      assert.equal(JSON.parse(await runSourceModel(['--frontend','complete','--from','meta','--input',model,'--check'])).semanticEquivalenceClaim,false);
      await assert.rejects(runSourceModel(['--frontend','complete','--from','meta','--input',model,'--meta-language',scratch]),/AmbiguousSourceModelFrontend/u);
    }
  } finally { fs.rmSync(scratch,{recursive:true,force:true}); }
});
