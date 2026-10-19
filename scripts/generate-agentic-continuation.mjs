#!/usr/bin/env node
// One narrow profile of the maintained checked portable-program generator.
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {checkCodingResearchProgram} from './generate-coding-research-program.mjs';
export const CONTINUATION_PROFILE = {
  sources: ['js/agentic/portable/continuation_state.mjs'],
  nativeDirectory: 'rust/src/agentic_continuation_generated',
  metaPath: 'data/meta/agentic-continuation-program.lino',
  namespace: 'agentic_continuation_generated',
  generatorPath: 'scripts/generate-agentic-continuation.mjs',
  metaRecord: 'agentic-continuation-program',
  description: 'Checked portable continuation evidence windows; host observations grant no execution authority.',
};
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const mode = process.argv[2] ?? '--check';
  assert(['--check', '--write'].includes(mode));
  const flag = process.argv.indexOf('--meta-language');
  const compiler = flag >= 0 ? process.argv[flag + 1] : process.env.FORMAL_AI_META_LANGUAGE;
  const result = await checkCodingResearchProgram(root, compiler, mode === '--write', CONTINUATION_PROFILE);
  assert(!result.drift.length || mode === '--write', 'CheckedContinuationProgramDrift:' + result.drift.join(','));
  console.log('Canonical continuation program matches pinned checked IR');
}
