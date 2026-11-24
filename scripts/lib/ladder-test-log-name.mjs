import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

/** Bind the complete Cargo selection without placing test identifiers in paths. */
export function ladderTestLogFilename(target, ...filters) {
  if (typeof target !== 'string' || !/^(?:bin:)?[A-Za-z_][A-Za-z_0-9-]*$/u.test(target)
    || !filters.length || filters.some(filter => typeof filter !== 'string' || !filter || /[\0\r\n]/u.test(filter))) {
    throw new Error('Expected a declared Cargo target and exact Rust test names');
  }
  const digest = createHash('sha256').update(JSON.stringify(['formal-ai-ladder-test-log-v1', target, ...filters])).digest('hex');
  return 'cargo-test-' + digest + '.log';
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  console.log(ladderTestLogFilename(...process.argv.slice(2)));
}
