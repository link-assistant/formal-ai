// Reflow the four new ROLE_* doc comments in src/seed/roles.rs so each opens
// with a short summary paragraph followed by a blank `///` line, satisfying
// clippy::too_long_first_doc_paragraph without dropping any multilingual detail.
// Operates by line-splice off the ASCII `pub const` anchors (never matching the
// mixed-script body text) and reuses the file's own bytes — no transcription.
import { readFileSync, writeFileSync } from 'node:fs';

const path = 'src/seed/roles.rs';
let lines = readFileSync(path, 'utf8').split('\n');

const wrap = (s, w = 74) => {
  const words = s.split(' ');
  const out = [];
  let cur = '';
  for (const word of words) {
    if (cur && (cur + ' ' + word).length > w) {
      out.push(cur);
      cur = word;
    } else {
      cur = cur ? cur + ' ' + word : word;
    }
  }
  if (cur) out.push(cur);
  return out;
};

function fixBlock(constName) {
  const ci = lines.findIndex((l) => l.startsWith(`pub const ${constName}:`));
  if (ci < 0) throw new Error('no const ' + constName);
  let start = ci;
  while (start > 0 && lines[start - 1].startsWith('///')) start--;
  const text = lines
    .slice(start, ci)
    .map((l) => l.replace(/^\/\/\/ ?/, ''))
    .join(' ')
    .replace(/\s+/g, ' ')
    .trim();
  const brk = text.indexOf('. ');
  if (brk < 0) throw new Error('no sentence break in ' + constName);
  const summary = text.slice(0, brk + 1);
  const rest = text.slice(brk + 2);
  const newDoc = [
    ...wrap(summary).map((l) => '/// ' + l),
    '///',
    ...wrap(rest).map((l) => '/// ' + l),
  ];
  lines = [...lines.slice(0, start), ...newDoc, ...lines.slice(ci)];
}

// Bottom-up so earlier indices stay valid after each splice.
[
  'ROLE_HELLO_WORLD_REFERENCE',
  'ROLE_SCRIPT_OR_CODE_ARTIFACT',
  'ROLE_SCRIPT_AUTHORING_VERB',
  'ROLE_PROGRAM_GENUS',
].forEach(fixBlock);

writeFileSync(path, lines.join('\n'));
console.log('reflowed 4 role doc comments');
