import {createRequire} from 'node:module';
import {fileURLToPath} from 'node:url';
const {project} = createRequire(import.meta.url)('./typescript-projection.cjs');

export async function load(url, context, next) {
  if (url.startsWith('file:')) {
    const source = project(fileURLToPath(url), 'actual-esm-load');
    if (source !== null) return {format: 'module', source, shortCircuit: true};
  }
  return next(url, context);
}
