// Expose an existing result of the exact pinned upstream implementation.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

export const SINGLE_PASS_CONTRACT_FILE = 'data/meta/translation-single-pass-adapter.lino';

export function parseSinglePassContract(text) {
  const lines = text.split('\n').map(line => line.trimEnd())
    .filter(line => line.trim() && !line.trimStart().startsWith('#'));
  assert.equal(lines.shift(), 'self-translation-combined-adapter');
  const fields = new Map();
  for (const line of lines) {
    const match = /^  ([a-z][a-z0-9-]*) (.+)$/u.exec(line);
    assert(match, 'Malformed single-pass adapter field');
    assert(!fields.has(match[1]), 'Duplicate single-pass adapter field');
    fields.set(match[1], match[2].startsWith('"') ? JSON.parse(match[2]) : match[2]);
  }
  assert.deepEqual([...fields.keys()].sort(), ['commit', 'sha256', 'source', 'upstream']);
  assert.equal(fields.get('upstream'), 'link-foundation/meta-language');
  assert.equal(fields.get('source'), 'js/src/self-translation.js');
  assert.match(fields.get('commit'), /^[a-f0-9]{40}$/u);
  assert.match(fields.get('sha256'), /^[a-f0-9]{64}$/u);
  return { commit: fields.get('commit'), source: fields.get('source'), sha256: fields.get('sha256') };
}

export function combinedTranslationModuleURL(source, originalURL, expectedSha256) {
  assert.equal(typeof source, 'string');
  assert(Buffer.byteLength(source) <= 262144, 'Upstream adapter source bound');
  assert.match(expectedSha256, /^[a-f0-9]{64}$/u);
  assert.equal(createHash('sha256').update(source).digest('hex'), expectedSha256,
    'Pinned upstream self-translation source differs');
  const location = new URL(originalURL);
  assert.equal(location.protocol, 'file:');
  assert(source.includes('function translateModule('), 'Pinned combined core is absent');
  assert(!/\bimport\s*\(|\bimport\.meta\b/u.test(source), 'Relative dynamic module context needs review');
  // A data URL has no relative base. Change only static import specifiers;
  // every function, check, source comment and returned value stays upstream's.
  const resolved = source.replace(/^(import [^\n]+ from )(['"])(\.{1,2}\/[^'"\n]+)\2;$/gmu,
    (_, prefix, quote, specifier) => prefix + JSON.stringify(new URL(specifier, location).href) + ';');
  const exposed = resolved + '\nexport { translateModule as formalAICombinedSelfTranslation };\n'
    + '//# sourceURL=formal-ai-pinned-self-translation-combined.mjs\n';
  return 'data:text/javascript;base64,' + Buffer.from(exposed).toString('base64');
}
