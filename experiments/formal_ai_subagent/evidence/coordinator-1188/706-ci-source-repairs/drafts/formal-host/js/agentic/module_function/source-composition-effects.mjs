import { sourceContractDiagnostic } from "./source-contract-diagnostics.mjs";
import assert from "./source-invariants.mjs";
import { tokenizeWithSpans } from "../crate/es_tokenizer.mjs";
const text = value => value?.$ === 'leaf' ? value.text : null;
const group = (value, delim) => value?.$ === 'group' && value.delim === delim;
const same = (node, value) => assert.equal(text(node), value, sourceContractDiagnostic("unexpected-source-operation"));
/** Close the emitted module's operation inventory over the already-qualified import graph; no purity claim for called source bodies. */
export function checkedCompositionSource(source, request, imports) {
  const trees = tokenizeWithSpans(source);
  assert.equal(trees.length, 15, sourceContractDiagnostic("extra-module-operation"));
  assert.equal(imports.length, 2);
  const locals = [];
  for (let index = 0; index < 2; index++) {
    const at = index * 5;
    same(trees[at], 'import');
    assert.ok(group(trees[at + 1], 'brace'), 'unknown import');
    const binding = trees[at + 1].trees;
    assert.equal(binding.length, 3);
    same(binding[0], imports[index].exported);
    same(binding[1], 'as');
    locals.push(text(binding[2]));
    assert.equal(binding[2].kind, 'identifier');
    same(trees[at + 2], 'from');
    assert.equal(trees[at + 3].kind, 'string');
    assert.equal(JSON.parse(text(trees[at + 3])), imports[index].moduleURL);
    same(trees[at + 4], ';');
  }
  same(trees[10], 'export');
  same(trees[11], 'function');
  same(trees[12], request.name);
  assert.ok(group(trees[13], 'paren'));
  assert.equal(trees[13].trees.length, 1);
  same(trees[13].trees[0], request.parameters[0]);
  assert.ok(group(trees[14], 'brace'));
  const body = trees[14].trees;
  assert.equal(body.length, 16, sourceContractDiagnostic("extra-function-operation"));
  same(body[0], 'const');
  assert.equal(body[1].kind, 'identifier');
  const selected = text(body[1]);
  assert.equal(new Set([...locals, selected, request.name, request.parameters[0]]).size, 5, 'binding collision');
  same(body[2], '=');
  same(body[3], locals[0]);
  assert.ok(group(body[4], 'paren'));
  assert.equal(body[4].trees.length, 1);
  same(body[4].trees[0], request.parameters[0]);
  same(body[5], ';');
  same(body[6], 'return');
  same(body[7], selected);
  same(body[8], '===');
  same(body[9], 'null');
  same(body[10], '?');
  same(body[11], 'null');
  same(body[12], ':');
  same(body[13], locals[1]);
  assert.ok(group(body[14], 'paren'));
  assert.equal(body[14].trees.length, 1);
  same(body[14].trees[0], selected);
  same(body[15], ';');
  return {
    kind: 'checked-source-import-composition',
    imports: imports.map(value => ({
      moduleURL: value.moduleURL,
      exported: value.exported,
      sourceIdentity: value.sourceIdentity
    })),
    calls: [{
      source: 0,
      argument: request.parameters[0],
      result: selected
    }, {
      source: 1,
      argument: selected,
      guard: 'nonnull'
    }],
    output: {
      absent: null,
      present: 'source-call-1'
    },
    localMutation: false,
    additionalModuleInitialization: false,
    calledSourceEffects: 'conditional; inherited, not pure',
    scope: sourceContractDiagnostic("different-acceptance-operation")
  };
}
/** JavaScript array indexing may yield undefined: an optional schema alone cannot certify a null guard. */
export function sourceNullAbsence(ir) {
  const proven = node => node?.op === 'literal' && node.value === null || node?.op === '??' && node.right?.op === 'literal' && node.right.value === null || node?.op === 'conditional' && proven(node.yes) && proven(node.no);
  return ir.status === 'parsed' && ir.ir.length === 1 && ir.ir[0].op === 'return' && proven(ir.ir[0].value);
}
