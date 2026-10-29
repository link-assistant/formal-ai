import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';

export const documentationPaths = ['docs/**', '*.md', '**/README.md', '**/CHANGELOG.md', 'changelog.d/**'];
export const lightweightWorkflows = new Set(['links.yml', 'workflows.yml', 'web-ui-boundary.yml', 'documentation-policy.yml']);
/** Exact native twin: is_documentation_path in scripts/detect-code-changes.rs. */
export function isDocumentationPath(path) {
  return typeof path === 'string' && path.length > 0 && path.length <= 1024
    && !path.startsWith('/') && !path.split('/').some(part => part === '.' || part === '..')
    && (path.startsWith('docs/') || path.startsWith('changelog.d/')
    || (!path.includes('/') && path.endsWith('.md')) || path.endsWith('/README.md') || path.endsWith('/CHANGELOG.md'));
}
/** Missing pages, excessive changes and renames out of source fail closed to normal CI. */
export function documentationOnlyPullRequest(pages, expectedCount) {
  if (!Array.isArray(pages) || !pages.every(Array.isArray)) return false;
  const files = pages.flat();
  return Number.isSafeInteger(expectedCount) && expectedCount > 0 && expectedCount <= 3000
    && files.length === expectedCount && files.every(file => isDocumentationPath(file?.filename)
      && (file.previous_filename === undefined || isDocumentationPath(file.previous_filename)));
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  let result = false;
  try { result = documentationOnlyPullRequest(JSON.parse(readFileSync(0,'utf8')), Number(process.argv[2])); }
  catch { /* Unknown observations select the full default profile. */ }
  console.log(result ? 'true' : 'false');
}
