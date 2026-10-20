import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { sourceCstFrontends } from '../../../scripts/lib/source-cst-frontends.mjs';
import { ownedSourceLinks, sourceToLinks, sourceFromLinks, writeSourceLinks, readSourceLinks } from '../../../js/meta/readable-source.js';
import { parseLinks, writeLink } from '../../../js/logging/lino-codec.mjs';
import { parameterDocumentationGrammar, formalizeParameterDocumentation, validateParameterDocumentation, naturalizeParameterDocumentation } from '../../../js/meta/comment-meaning.js';
import { createDocumentedParameterRule, applyDocumentedParameterTransform, sourceCstRuleForObservedParameter, readQualifiedSourceCstRule } from '../../../scripts/lib/source-cst-documentation-transform.mjs';
import { executeSourceCstTransform } from '../../../scripts/source-cst-transform-request.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { hasHost, host, installHost } from '../../../js/agentic/host.mjs';
import { withSourceCstCommentContext } from '../../../scripts/lib/source-cst-comment-host.mjs';
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-documentation-control-'));
const digest = value => createHash('sha256').update(value).digest('hex');
let runtime, grammar;
before(async () => {
  await installNodeHost(new WorkerHost());
  runtime = await sourceCstFrontends();
  grammar = parameterDocumentationGrammar();
});
after(() => {
  runtime?.close();
  fs.rmSync(scratch, {
    recursive: true,
    force: true
  });
});
function fixture(source, language = 'JavaScript') {
  const owner = language === 'TypeScript' ? 'subject.ts' : language === 'Rust' ? 'subject.rs' : 'subject.mjs',
    frontend = runtime.forSource(owner, language);
  const document = ownedSourceLinks(owner, sourceToLinks(source, language, frontend));
  const rule = createDocumentedParameterRule(document, {
    declaration: 'measure',
    binding: 'amount',
    target: 'total'
  });
  return {
    owner,
    frontend,
    document,
    rule
  };
}
for (const tag of ['param', 'arg', 'argument']) test('declared ' + tag + ' binding is formalized and naturally renamed with unrelated words exact', () => {
  const source = '/**\n * Count values.\n * @' + tag + ' {number} amount - input value\n * @returns {number} derived value\n */\nfunction measure(amount) { return amount * 2; }\n';
  const item = fixture(source),
    result = applyDocumentedParameterTransform(item.document, item.rule, item.frontend);
  assert.equal(sourceFromLinks(result.document), source.replace('@' + tag + ' {number} amount', '@' + tag + ' {number} total').replace('measure(amount)', 'measure(total)').replace('return amount', 'return total'));
  assert.equal(result.verification.parameterDocumentation, 'proven-syntax-bound-reference');
  assert.equal(result.verification.arbitraryProse, 'not-certified');
  assert.equal(result.verification.exactInverse, true);
  assert.equal(result.verification.independentReparse, true);
  assert.equal(result.verification.authorshipCredit, 0);
  assert.equal(sourceFromLinks(applyDocumentedParameterTransform(result.document, result.inverse, item.frontend).document), source);
});
test('recursive type syntax, Unicode prose, BOM and CRLF layout survive complete source roundtrip', () => {
  const source = '\uFEFF/**\r\n * @param {{ nested: { value: number }, label: "雪}" }} amount - 雪 input\r\n */\r\nexport function measure(amount: number): number { return amount; }\r\n';
  const item = fixture(source, 'TypeScript'),
    result = applyDocumentedParameterTransform(item.document, item.rule, item.frontend);
  assert.equal(sourceFromLinks(result.document), source.replace('}} amount', '}} total').replace('measure(amount:', 'measure(total:').replace('return amount', 'return total'));
  const persisted = writeSourceLinks(result.document);
  assert.deepEqual(readSourceLinks(persisted), result.document);
});
test('unannotated description references remain exact source-linked needs, never rewritten as meanings', () => {
  const source = '/** @param amount - amount must remain bounded */\nfunction measure(amount){ /* amount remains unmodeled */ return amount;}';
  const item = fixture(source),
    result = applyDocumentedParameterTransform(item.document, item.rule, item.frontend);
  assert.ok(sourceFromLinks(result.document).startsWith('/** @param total - amount must remain bounded */'));
  assert.equal(result.needs.length, 2);
  assert.equal(result.needs[1].text, '/* amount remains unmodeled */');
  assert.notDeepEqual(result.needs[0].cstPath, result.needs[1].cstPath);
  assert.equal(result.needs[0].kind, 'unresolved-prose-parameter-reference');
  assert.equal(result.needs[0].binding, 'amount');
  assert.equal(result.needs[0].owner, item.owner);
  assert.ok(Array.isArray(result.needs[0].cstPath));
});
for (const [name, comment] of [['duplicate', '/** @param amount\n * @param amount */'], ['property', '/** @param amount.value */'], ['optional', '/** @param [amount] */'], ['default', '/** @param [amount=1] */'], ['rest', '/** @param ...amount */'], ['owner override', '/** @alias anotherOwner\n * @param amount */'], ['unbound other parameter', '/** @param amount\n * @param nonexistent */'], ['type reference', '/** @param {typeof amount} amount */'], ['unbalanced type', '/** @param {{broken} amount */'], ['multiline type', '/** @param {\n * number} amount */']]) test(name + ' documentation is refused without producing a partial changed source', () => {
  const item = fixture(comment + '\nfunction measure(amount){return amount;}'),
    before = writeSourceLinks(item.document);
  assert.throws(() => applyDocumentedParameterTransform(item.document, item.rule, item.frontend));
  assert.equal(writeSourceLinks(item.document), before);
});
for (const prefix of ['/** @param amount */\n/** second comment */\n', '/** @param amount */\n\n', 'const previous=1; /** @param amount */\n']) test('ambiguous attachment refuses: ' + JSON.stringify(prefix), () => {
  const item = fixture(prefix + 'function measure(amount){return amount;}');
  assert.throws(() => applyDocumentedParameterTransform(item.document, item.rule, item.frontend), /Attachment/u);
});
test('Rust Markdown doc text is not invented as a standard parameter binding', () => {
  const item = fixture('/// @param amount\nfn measure(amount:i32)->i32{amount}', 'Rust');
  assert.throws(() => applyDocumentedParameterTransform(item.document, item.rule, item.frontend), /RustDocumentationBindingNotStandardized/u);
});
test('forged documentary name-span and weakened readable contract fail independent re-formalization', () => {
  const input = {
    text: '/** @param {number} amount - observed */',
    source: {
      path: 'subject.mjs',
      language: 'JavaScript'
    },
    owner: {
      kind: 'function',
      name: 'measure'
    },
    role: 'documentation'
  };
  const model = formalizeParameterDocumentation(input);
  assert.equal(validateParameterDocumentation(model), true);
  const forged = structuredClone(model);
  forged.bindings[0].start += 1;
  assert.equal(validateParameterDocumentation(forged), false);
  assert.throws(() => naturalizeParameterDocumentation(forged, 'amount', 'total'));
  const item = fixture(input.text + '\nfunction measure(amount){return amount;}'),
    changed = structuredClone(item.rule);
  changed[1].push(['bypass', 'unknown']);
  assert.throws(() => applyDocumentedParameterTransform(item.document, changed, item.frontend), /UnsupportedDocumentationRule/u);
  assert.throws(() => readQualifiedSourceCstRule(writeLink(item.rule, {
    quotedText: 'json'
  }) + '\n(unapproved extra)'), /QualifiedSourceTransformRuleRequired/u);
});
test('held-out actual repository function documentation and code change together with full module inverse', () => {
  const owner = 'js/agentic/portable/coding_research_kernel.mjs',
    source = fs.readFileSync(path.join(ROOT, owner), 'utf8'),
    frontend = runtime.forSource(owner, 'JavaScript');
  const document = ownedSourceLinks(owner, sourceToLinks(source, 'JavaScript', frontend));
  const rule = createDocumentedParameterRule(document, {
    declaration: 'safeTapCount',
    binding: 'value',
    target: 'candidateValue'
  });
  const result = applyDocumentedParameterTransform(document, rule, frontend);
  const after = sourceFromLinks(result.document);
  assert.ok(after.includes('/** @param {number} candidateValue @returns {boolean} */'));
  assert.ok(after.includes('function safeTapCount(candidateValue)'));
  assert.equal(sourceFromLinks(applyDocumentedParameterTransform(result.document, result.inverse, frontend).document), source);
  const directory = path.join(scratch, 'safeTapCount');
  fs.mkdirSync(directory, {
    recursive: true
  });
  const pretty = (node, indent = '') => node.every(item => !Array.isArray(item)) ? indent + writeLink(node, {
    quotedText: 'json'
  }) : indent + '(' + node[0] + '\n' + node.slice(1).map(item => pretty(item, indent + '  ')).join('\n') + ')';
  for (const [name, content] of [['before.lino', writeSourceLinks(document)], ['after.lino', writeSourceLinks(result.document)], ['rule.lino', pretty(rule) + '\n'], ['inverse.lino', pretty(result.inverse) + '\n']]) fs.writeFileSync(directory + '/' + name, content);
  const storedAfter = readSourceLinks(fs.readFileSync(directory + '/after.lino', 'utf8')),
    storedInverse = parseLinks(fs.readFileSync(directory + '/inverse.lino', 'utf8'), {
      quotedText: 'json'
    })[0];
  assert.equal(sourceFromLinks(applyDocumentedParameterTransform(storedAfter, storedInverse, frontend).document), source, 'persisted full source-free inverse');
  fs.writeFileSync(directory + '/receipt.json', JSON.stringify({
    owner,
    operation: 'qualified-parameter-and-proven-JSDoc-binding',
    actualOwnerReadOnly: true,
    fullModelInverseVerified: true,
    verification: result.verification,
    authorshipCredit: 0
  }, null, 2) + '\n');
  assert.equal(fs.readFileSync(path.join(ROOT, owner), 'utf8'), source, 'held-out source owner is read-only in this controlled transform');
});
test('ordinary summary preceding the actual leading JSDoc does not invent a competing documentation owner', () => {
  const source = '// Module summary\n/** @param amount */\nfunction measure(amount){return amount;}';
  const item = fixture(source),
    result = applyDocumentedParameterTransform(item.document, item.rule, item.frontend);
  assert.ok(sourceFromLinks(result.document).startsWith('// Module summary\n/** @param total */'));
});
test('installed natural-language planner updates proven documentation and code and restores through both persisted CLIs', async () => {
  const directory = fs.mkdtempSync(path.join(scratch, 'foreign-repository-'));
  const owner = 'human.ts',
    source = '/**\r\n * @param {number} amount - amount is supplied 雪\r\n */\r\nexport function measure(amount: number): number { return amount * 2; }\r\n';
  execFileSync('git', ['init', '-q'], {
    cwd: directory
  });
  fs.writeFileSync(path.join(directory, owner), source);
  execFileSync('git', ['add', owner], {
    cwd: directory
  });
  const task = 'Rename parameter amount of function measure to total in human.ts',
    tools = ['read_file', 'run_shell_command'],
    messages = [{
      role: 'user',
      content: task
    }];
  const first = await planChatStep(messages, tools);
  assert.equal(first.calls[0].tool, 'read_file');
  messages.push({
    role: 'assistant',
    tool_calls: [{
      id: 'documentation-read',
      function: {
        name: first.calls[0].tool,
        arguments: first.calls[0].arguments
      }
    }]
  }, {
    role: 'tool',
    tool_call_id: 'documentation-read',
    content: source
  });
  const next = await planChatStep(messages, tools),
    command = JSON.parse(next.calls[0].arguments).command;
  const output = execFileSync('/bin/sh', ['-c', command], {
    cwd: directory,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
    timeout: 180000
  });
  const receipt = JSON.parse(output),
    evidence = receipt.evidenceDirectory;
  try {
    assert.equal(receipt.transformKind, 'documented-source-transform');
    assert.equal(receipt.verification.commentMeaning, 'parameter-binding-only');
    assert.equal(receipt.verification.arbitraryProse, 'not-certified');
    assert.equal(receipt.verification.commentNeeds.length, 1);
    assert.equal(receipt.verification.commentNeeds[0].owner, owner);
    const expected = source.replace('{number} amount', '{number} total').replace('measure(amount:', 'measure(total:').replace('return amount', 'return total');
    assert.equal(fs.readFileSync(path.join(directory, owner), 'utf8'), expected);
    assert.ok(fs.existsSync(path.join(evidence, 'documentation.json')));
    const restoredModel = path.join(scratch, 'restored.lino'),
      forwardRule = path.join(scratch, 'forward.lino');
    execFileSync(process.execPath, [path.join(ROOT, 'scripts/source-model.mjs'), '--frontend', 'complete', '--from', 'meta', '--input', path.join(evidence, 'after.lino'), '--transform', path.join(evidence, 'inverse.lino'), '--output', restoredModel, '--inverse', forwardRule], {
      cwd: directory,
      encoding: 'utf8',
      timeout: 180000,
      maxBuffer: 1024 * 1024
    });
    assert.equal(sourceFromLinks(readSourceLinks(fs.readFileSync(restoredModel, 'utf8'))), source);
    const restored = JSON.parse(execFileSync(process.execPath, [path.join(ROOT, 'scripts/source-cst-transform-request.mjs'), '--inverse', evidence, '--expected', receipt.afterSha256], {
      cwd: directory,
      encoding: 'utf8',
      timeout: 180000,
      maxBuffer: 1024 * 1024
    }));
    assert.equal(fs.readFileSync(path.join(directory, owner), 'utf8'), source);
    fs.rmSync(restored.evidenceDirectory, {
      recursive: true,
      force: true
    });
    messages.push({
      role: 'assistant',
      tool_calls: [{
        id: 'documentation-write',
        function: {
          name: next.calls[0].tool,
          arguments: next.calls[0].arguments
        }
      }]
    }, {
      role: 'tool',
      tool_call_id: 'documentation-write',
      content: output
    });
    assert.equal((await planChatStep(messages, tools)).kind, 'final');
  } finally {
    fs.rmSync(evidence, {
      recursive: true,
      force: true
    });
  }
});
test('automatic selection retains plain alpha and unproven prose needs, and refuses ambiguous documented mutation', async () => {
  for (const source of ['// amount stays prose\nfunction measure(amount){return amount;}', '/** @param other - amount is mentioned */\nfunction measure(amount,other){return amount+other;}']) {
    const item = fixture(source),
      rule = sourceCstRuleForObservedParameter(item.document, {
        declaration: 'measure',
        binding: 'amount',
        target: 'total'
      });
    assert.equal(rule[0], 'source-transform');
  }
  const directory = fs.mkdtempSync(path.join(scratch, 'ambiguous-owner-')),
    owner = 'subject.mjs';
  const source = '/** @param amount */\n/** @param amount */\nfunction measure(amount){return amount;}';
  execFileSync('git', ['init', '-q'], {
    cwd: directory
  });
  fs.writeFileSync(path.join(directory, owner), source);
  execFileSync('git', ['add', owner], {
    cwd: directory
  });
  await assert.rejects(executeSourceCstTransform({
    root: directory,
    request: 'Rename parameter amount of function measure to total in subject.mjs',
    expected: digest(source)
  }), /Attachment/u);
  assert.equal(fs.readFileSync(path.join(directory, owner), 'utf8'), source);
});
test('standalone installed documentary context restores caller host capabilities on success and refusal', () => {
  const previous = host();
  assert.equal(withSourceCstCommentContext(() => host()), previous);
  assert.throws(() => withSourceCstCommentContext(() => {
    throw Error('controlled');
  }), /controlled/u);
  assert.equal(host(), previous);
  installHost(null);
  try {
    assert.equal(withSourceCstCommentContext(() => parameterDocumentationGrammar().tags.includes('param')), true);
    assert.equal(hasHost(), false);
    assert.throws(() => withSourceCstCommentContext(() => {
      throw Error('controlled');
    }), /controlled/u);
    assert.equal(hasHost(), false);
  } finally {
    installHost(previous);
  }
});
