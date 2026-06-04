// Byte-accurate replacement of is_write_script_request in src/solver_helpers.rs.
// Splices by line number to avoid matching mixed-script (mangled) comment lines.
import { readFileSync, writeFileSync } from 'node:fs';

const path = 'src/solver_helpers.rs';
const src = readFileSync(path, 'utf8');
const lines = src.split('\n');

// Locate the doc-comment start and the function's closing brace.
const sigIdx = lines.findIndex((l) => l.includes('pub fn is_write_script_request(normalized: &str) -> bool {'));
if (sigIdx < 0) throw new Error('signature not found');

// Doc comment begins at the first contiguous /// block above the signature.
let startIdx = sigIdx;
while (startIdx > 0 && lines[startIdx - 1].startsWith('///')) startIdx--;

// Closing brace: first line that is exactly "}" at column 0 after the signature.
let endIdx = sigIdx;
while (endIdx < lines.length && lines[endIdx] !== '}') endIdx++;
if (endIdx >= lines.length) throw new Error('closing brace not found');

const replacement = [
  '/// Return true when the normalized prompt asks for a script or code to be',
  '/// *authored* — the author verb ([`ROLE_SCRIPT_AUTHORING_VERB`], carried by the',
  '/// `write` meaning) paired with a script-or-code artifact noun',
  '/// ([`ROLE_SCRIPT_OR_CODE_ARTIFACT`], carried by `script` and `code`) — in any',
  '/// supported language. No natural-language word is hardcoded here; the lexicon',
  '/// answers which surface forms evidence each role.',
  '///',
  '/// Defers to the parametric write-program route for prompts that name the broad',
  '/// program genus ([`ROLE_PROGRAM_GENUS`]) or the canonical hello-world archetype',
  '/// ([`ROLE_HELLO_WORLD_REFERENCE`]), so those keep their richer formalization',
  '/// instead of collapsing into a bare script.',
  '///',
  '/// [`ROLE_SCRIPT_AUTHORING_VERB`]: crate::seed::ROLE_SCRIPT_AUTHORING_VERB',
  '/// [`ROLE_SCRIPT_OR_CODE_ARTIFACT`]: crate::seed::ROLE_SCRIPT_OR_CODE_ARTIFACT',
  '/// [`ROLE_PROGRAM_GENUS`]: crate::seed::ROLE_PROGRAM_GENUS',
  '/// [`ROLE_HELLO_WORLD_REFERENCE`]: crate::seed::ROLE_HELLO_WORLD_REFERENCE',
  'pub fn is_write_script_request(normalized: &str) -> bool {',
  '    use crate::seed::{',
  '        ROLE_HELLO_WORLD_REFERENCE, ROLE_PROGRAM_GENUS, ROLE_SCRIPT_AUTHORING_VERB,',
  '        ROLE_SCRIPT_OR_CODE_ARTIFACT,',
  '    };',
  '    let lexicon = crate::seed::lexicon();',
  '    // The parametric write-program route owns the broad program genus and the',
  '    // canonical hello-world archetype; step aside for those.',
  '    if lexicon.mentions_role(ROLE_PROGRAM_GENUS, normalized)',
  '        || lexicon.mentions_role(ROLE_HELLO_WORLD_REFERENCE, normalized)',
  '    {',
  '        return false;',
  '    }',
  '    // Author a script: the write verb plus a script-or-code artifact noun.',
  '    lexicon.mentions_role(ROLE_SCRIPT_AUTHORING_VERB, normalized)',
  '        && lexicon.mentions_role(ROLE_SCRIPT_OR_CODE_ARTIFACT, normalized)',
  '}',
];

const out = [...lines.slice(0, startIdx), ...replacement, ...lines.slice(endIdx + 1)];
writeFileSync(path, out.join('\n'));
console.log(`Replaced lines ${startIdx + 1}..${endIdx + 1} (${endIdx - startIdx + 1} lines) with ${replacement.length} lines.`);
