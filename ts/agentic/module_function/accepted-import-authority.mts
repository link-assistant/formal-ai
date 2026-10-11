import { contractText } from '../workspace_discovery.mjs';
import { sourceContractDiagnostic } from "./source-contract-diagnostics.mjs";
import assert from "./source-invariants.mjs";
import { sourceOperationModuleURL as moduleURL } from "./source-operation-port.mjs";
import { observeSourceCallables } from "./callable_catalog.mjs";
import { sha256Hex } from "../crate/source_fetch.mjs";

/** Static source evidence binds reuse within one already-authorized Node operation; it does not prove initializer purity. */
export function bindAcceptedImport(acceptance, source, binding, operation) {
  assert.equal(operation.command, contractText('accepted-node-test-command', [operation.acceptanceOperand]), 'different operation');
  assert.equal(operation.acceptancePath, acceptance.path, 'foreign acceptance');
  assert.equal(operation.acceptanceIdentity, sha256Hex(acceptance.content), 'acceptance drift');
  assert.equal(binding.contentId, sha256Hex(source.content), 'source drift');
  const catalog = observeSourceCallables(acceptance.content, acceptance.path);
  assert.ok(!catalog.gaps.some(gap => ['LexicalFailure', 'DuplicateBinding', 'DuplicateExport'].includes(gap.reason)), 'ambiguous acceptance');
  const sourceCatalog = observeSourceCallables(source.content, source.path);
  const exported = sourceCatalog.exports.filter(value => value.exposed === binding.exported);
  assert.equal(exported.length, 1, 'unproved export');
  const declaration = sourceCatalog.declarations.find(value => value.name === exported[0].local);
  assert.ok(declaration, 'unobserved declaration');
  const expected = moduleURL(binding.specifier, acceptance.path);
  assert.equal(expected, moduleURL(source.path, acceptance.path), sourceContractDiagnostic("source-module-url-mismatch"));
  const matches = catalog.imports.filter(value => moduleURLOrNull(value.specifier, acceptance.path) === expected && value.bindings.some(name => name.imported === binding.exported));
  assert.equal(matches.length, 1, sourceContractDiagnostic("unaccepted-module-import"));
  return Object.freeze({
    kind: 'accepted-operation-import-reuse',
    moduleURL: expected,
    exported: binding.exported,
    sourceIdentity: binding.contentId,
    acceptanceIdentity: catalog.contentId,
    operation: operation.command,
    additionalModuleInitialization: false,
    moduleEffects: sourceCatalog.moduleEffects,
    callEffects: 'conditional',
    scope: 'same-operation-same-module-URL',
    rustEquivalence: 'unknown'
  });
}
function moduleURLOrNull(specifier, parent) {
  try {
    return moduleURL(specifier, parent);
  } catch {
    return null;
  }
}
