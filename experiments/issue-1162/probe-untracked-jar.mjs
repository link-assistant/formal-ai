// Issue #1160: after a finished delivery, a stray `?? Main.jar` build output
// appears in the checkout; a regular Hive Mind continuation must not commit it
// and must still leave the pull request ready with a clean tree.
import fs from 'node:fs';
import path from 'node:path';
import { workspace, session, outcome, hivePrompt } from './offline-hive-e2e.mjs';
const language = process.argv[2] ?? 'Rust';
const env = workspace(language);
const first = await session(env, hivePrompt('hive'));
console.log('first stop', first.stop, JSON.stringify(outcome(env).pr));
fs.writeFileSync(path.join(env.dir, 'Main.jar'), 'PK\u0003\u0004 not a real jar');
const second = await session(env, hivePrompt('hive'));
for (const entry of second.transcript) {
  const args = JSON.parse(entry.arguments);
  console.log('step', entry.tool, String(args.command ?? args.path ?? '').slice(-200).replace(/\n/gu, ' '));
}
console.log('second stop', second.stop);
console.log('answer', String(second.answer ?? '').slice(-900));
console.log(JSON.stringify(outcome(env), null, 1));
if (!process.env.KEEP) fs.rmSync(env.root, { recursive: true, force: true });
