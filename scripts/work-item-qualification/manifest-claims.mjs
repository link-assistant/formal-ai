// Host-only independent literal checker. Declarations do not establish executable use.
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const sha = value => createHash('sha256').update(value).digest('hex');
function probe(files) {
  const unknown = reason => ({
    facts: [],
    observations: files.filter(file => /\.(?:json|toml)$/u.test(file.path)).map(file => ({
      path: file.path,
      sourceIdentity: sha(file.content),
      status: 'unknown',
      reason
    }))
  });
  const input = JSON.stringify(files);
  if (Buffer.byteLength(input) > 2 * 1024 * 1024) return unknown('independent-input-budget');
  const result = spawnSync('python3', ['-I', fileURLToPath(new URL('./manifest-observations.py', import.meta.url))], {
    input,
    encoding: 'utf8',
    timeout: 5000,
    maxBuffer: 1024 * 1024,
    killSignal: 'SIGKILL'
  });
  if (result.error || result.status !== 0 || result.signal) return unknown('independent-parser-unavailable-or-incomplete');
  try {
    const value = JSON.parse(result.stdout);
    if (!Array.isArray(value.facts) || !Array.isArray(value.observations)) return unknown('independent-invalid-result');
    return value;
  } catch {
    return unknown('independent-invalid-result');
  }
}
export function observedManifestEvidence(files) {
  return probe(files);
}
export function literalManifestFacts(files) {
  return observedManifestEvidence(files).facts;
}
function same(left, right) {
  if (left === right) return true;
  if (left === null || right === null || typeof left !== 'object' || typeof right !== 'object') return false;
  if (Array.isArray(left) || Array.isArray(right)) return Array.isArray(left) && Array.isArray(right) && left.length === right.length && left.every((value, index) => same(value, right[index]));
  const keys = Object.keys(left);
  return keys.length === Object.keys(right).length && keys.every(key => Object.hasOwn(right, key) && same(left[key], right[key]));
}
export function supportedManifestClaims(markdown, facts, observations = []) {
  const blocks = [...markdown.matchAll(/```formal-ai-source-claim\s*\n([\s\S]*?)\n```/gu)];
  if (blocks.length > 32) return [{
    status: 'unresolved',
    reason: 'claim-budget-exceeded'
  }];
  if (!blocks.length) return [];
  const parsed = probe(blocks.map((block, index) => ({
    path: 'claim-' + index + '.json',
    content: block[1],
    claim: true
  })));
  return blocks.map((_block, index) => {
    const observation = parsed.observations.find(row => row.path === 'claim-' + index + '.json');
    if (observation?.status !== 'observed') return {
      status: 'unresolved',
      reason: 'invalid-or-unavailable-claim-json'
    };
    const claim = observation.value;
    if (!claim || typeof claim !== 'object' || Array.isArray(claim)) return {
      status: 'unresolved',
      reason: 'typed-claim-object-required'
    };
    const fields = Object.keys(claim),
      feature = claim.kind === 'cargo-feature';
    const required = ['kind', 'manifest', 'sourceIdentity', 'section', 'name', feature ? 'enables' : 'version'];
    const allowed = feature ? required : [...required, 'target', 'options'];
    const schemaValid = required.every(key => fields.includes(key)) && fields.every(key => allowed.includes(key)) && ['manifest-declaration',
     'cargo-feature'].includes(claim.kind) && ['manifest', 'sourceIdentity', 'section', 'name'].every(key => typeof claim[key] === 'string') &&
     (feature ? claim.section === 'features' && Array.isArray(claim.enables) && claim.enables.every(value => typeof value === 'string') : (typeof claim.version === 'string' ||
     claim.version === null) && (!Object.hasOwn(claim, 'target') || typeof claim.target === 'string'));
    if (!schemaValid) return {
      claim,
      status: 'unresolved',
      reason: 'unsupported-typed-claim'
    };
    const source = observations.find(row => row.path === claim.manifest && row.sourceIdentity === claim.sourceIdentity && row.status === 'observed');
    if (!source) return {
      claim,
      status: 'unresolved',
      reason: 'complete-frozen-source-binding-required'
    };
    const fact = facts.find(row => row.kind === claim.kind && row.manifest === claim.manifest && row.sourceIdentity === claim.sourceIdentity && row.section === claim.section && row.name === claim.name && (row.target ?? null) === (claim.target ?? null));
    const valid = fact && fields.every(key => Object.hasOwn(fact, key) && same(fact[key], claim[key]));
    return {
      claim,
      status: valid ? 'verified-literal-declaration' : 'contradicted',
      reason: valid ? 'exact-observed-literal-fields' : fact ? 'claimed-fields-differ-from-frozen-source' : 'declaration-absent-from-completely-observed-manifest',
      semantics: 'literal declarations only; executable use and integration roles remain unverified'
    };
  });
}
