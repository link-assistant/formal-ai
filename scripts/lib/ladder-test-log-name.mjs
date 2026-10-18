import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

/** Bind the complete Cargo selection without placing test identifiers in paths. */
export function ladderTestLogFilename(target, filter) {
  if (typeof target !== 'string' || !/^[A-Za-z_][A-Za-z_0-9-]*$/u.test(target)
    || typeof filter !== 'string' || !filter || /[\0\r\n]/u.test(filter)) {
    throw new Error('Expected a declared Cargo target and one exact Rust test name');
  }
  const digest = createHash('sha256').update(JSON.stringify(['formal-ai-ladder-test-log-v1', target, filter])).digest('hex');
  return 'cargo-test-' + digest + '.log';
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.argv.length !== 4) throw new Error('Expected Cargo target and exact test name');
  console.log(ladderTestLogFilename(process.argv[2], process.argv[3]));
}
