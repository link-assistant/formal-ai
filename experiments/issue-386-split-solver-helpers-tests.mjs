// Split the #[cfg(test)] mod tests of src/solver_helpers.rs into a sibling
// `src/solver_helpers_tests.rs` mounted via #[path], and drop the temporary
// `zzz_temp_probe_is_write_script_request` probe along the way. Byte-accurate,
// anchored on ASCII markers so mixed-script test prompts are never matched.
import { readFileSync, writeFileSync } from 'node:fs';

const path = 'src/solver_helpers.rs';
const lines = readFileSync(path, 'utf8').split('\n');

// 1. Locate `#[cfg(test)]` immediately followed by `mod tests {`.
let cfgIdx = -1;
for (let i = 0; i < lines.length - 1; i++) {
  if (lines[i].trim() === '#[cfg(test)]' && lines[i + 1].trim() === 'mod tests {') {
    cfgIdx = i;
    break;
  }
}
if (cfgIdx < 0) throw new Error('test module not found');
const modOpenIdx = cfgIdx + 1; // `mod tests {`

// 2. Module close = last `}` at column 0 (the module is the final top-level item).
let modCloseIdx = lines.length - 1;
while (modCloseIdx > modOpenIdx && lines[modCloseIdx].trim() === '') modCloseIdx--;
if (lines[modCloseIdx] !== '}') {
  throw new Error('module close brace mismatch: ' + JSON.stringify(lines[modCloseIdx]));
}

// 3. Body = strictly inside the module.
let body = lines.slice(modOpenIdx + 1, modCloseIdx);

// 4. Remove the temp probe fn (its `#[test]` attr through its closing brace).
const probeSigIdx = body.findIndex((l) => l.includes('fn zzz_temp_probe_is_write_script_request'));
if (probeSigIdx < 0) throw new Error('probe fn not found in body');
let probeStart = probeSigIdx;
while (probeStart > 0 && body[probeStart - 1].trim().startsWith('#[')) probeStart--;
const fnIndent = body[probeSigIdx].match(/^\s*/)[0];
let probeEnd = probeSigIdx;
while (probeEnd < body.length && body[probeEnd] !== fnIndent + '}') probeEnd++;
if (probeEnd >= body.length) throw new Error('probe fn close brace not found');
let removeEnd = probeEnd;
if (body[removeEnd + 1] !== undefined && body[removeEnd + 1].trim() === '') removeEnd++;
body = [...body.slice(0, probeStart), ...body.slice(removeEnd + 1)];

// 5. Write the sibling tests file (indentation is normalized later by cargo fmt).
const header = [
  '//! Unit tests for `src/solver_helpers.rs`. Extracted into a sibling file and',
  '//! mounted with `#[path]` so the implementation file stays under the 1000-line',
  '//! Rust file-size limit enforced by `scripts/check-file-size.rs`.',
  '',
];
writeFileSync(
  'src/solver_helpers_tests.rs',
  [...header, ...body].join('\n').replace(/\n*$/, '\n'),
);

// 6. Replace the inline module with a #[path] mount.
const newDecl = ['#[cfg(test)]', '#[path = "solver_helpers_tests.rs"]', 'mod tests;'];
const out = [...lines.slice(0, cfgIdx), ...newDecl, ...lines.slice(modCloseIdx + 1)];
writeFileSync(path, out.join('\n'));

console.log(
  `Extracted ${body.length} body lines to solver_helpers_tests.rs; ` +
    `solver_helpers.rs module replaced with #[path] mount.`,
);
