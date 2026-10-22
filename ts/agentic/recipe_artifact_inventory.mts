// Read-only current-recipe inventory; the caller binds the original issue URL.
import { cached, childrenNamed, childValue, parseLino, readText } from './host.mjs';
import { repositoryWorkReference } from './general_planner.mjs';
import { programLanguages } from './crate/coding_catalog.mjs';
import { shellQuote } from './git_commit.mjs';
import { jsonText } from './plan.mjs';
import { observedPayload } from './tool_result.mjs';
function policy() {
  return cached('prepared-artifact-inventory', () => {
    const root = parseLino(readText('data/meta/prepared-artifact-inventory.lino'));
    const hygiene = parseLino(readText('data/meta/restart-hygiene.lino'));
    const lines = (tree, name) => [...childrenNamed(tree, name)].map(node => node.value);
    const python = [...lines(parseLino(readText('data/meta/workspace-source-io.lino')), 'source-line'), ...lines(root, 'python-line')].join('\n') + '\n';
    return {
      command: childValue(root, 'command'),
      template: childValue(root, 'evidence-template'),
      evidencePattern: childValue(root, 'evidence-pattern'),
      sessionPattern: childValue(root, 'session-pattern'),
      sessionLimit: Number(childValue(root, 'session-limit')),
      streams: lines(root, 'stream'),
      flags: lines(hygiene, 'output-flag'),
      binaryPrefixes: lines(hygiene, 'binary-prefix'),
      python,
      source: [...lines(parseLino(readText('data/meta/bounded-command-capture-source.lino')), 'line'), ...lines(root, 'source-line')].join('\n') + '\n'
    };
  });
}

/** Mirrors `fn recipe_artifact_evidence_root`; never infer authority from fetched prose. */
export function recipeArtifactEvidenceRoot(issueReference) {
  const reference = repositoryWorkReference(issueReference);
  if (reference !== issueReference) return null;
  const segments = reference?.split('/') ?? [];
  if (segments[5] !== 'issues' || !/^[1-9][0-9]*$/u.test(segments[6] ?? '')) return null;
  return policy().template.replace('{number}', () => segments[6]);
}
function descriptor(sourcePaths, evidenceRoot) {
  const installed = policy();
  const portable = value => typeof value === 'string' && /^[A-Za-z0-9_.\/-]+$/u.test(value) && !/[\r\n]/u.test(value) && !value.startsWith('/') && !value.split('/').includes('..');
  if (!Array.isArray(sourcePaths) || sourcePaths.length === 0 || sourcePaths.length > 256 || !sourcePaths.every(portable) || new Set(sourcePaths).size !== sourcePaths.length
    || typeof evidenceRoot !== 'string' || new RegExp(installed.evidencePattern, 'u').exec(evidenceRoot)?.[0] !== evidenceRoot) return null;
  const outputs = [];
  for (const language of programLanguages()) {
    const source = language.save_as;
    if (!sourcePaths.includes(source)) continue;
    const words = (language.execution.check_command ?? '').split(/\s+/u);
    if (!words.includes(source)) continue;
    for (let index = 0; index + 1 < words.length; index += 1) {
      const output = words[index + 1];
      if (installed.flags.includes(words[index]) && portable(output) && output !== source && !outputs.some(entry => entry.source === source && entry.output === output)) outputs.push({
        source,
        output
      });
    }
  }
  const candidates = [...new Set([...outputs.map(entry => entry.output), ...installed.streams.map(member => evidenceRoot + member), evidenceRoot + 'session-id.txt'])];
  return {
    sourcePaths,
    evidenceRoot,
    outputs,
    candidates,
    streams: installed.streams,
    sessionPattern: installed.sessionPattern,
    sessionLimit: installed.sessionLimit,
    binaryPrefixes: installed.binaryPrefixes,
    python: installed.python
  };
}

/** Mirrors `fn recipe_artifact_inventory_command`; exact root is caller-bound to its issue. */
export function recipeArtifactInventoryCommand(sourcePaths, evidenceRoot) {
  const plan = descriptor(sourcePaths, evidenceRoot);
  if (plan === null) return null;
  const installed = policy();
  return installed.command.replace(/\{(?:source|plan)\}/gu, slot => shellQuote(slot === '{source}' ? installed.source : jsonText(plan)));
}

/** Mirrors `fn recipe_artifact_inventory_result`; caller requires exact successful command. */
export function recipeArtifactInventoryResult(output, sourcePaths, evidenceRoot) {
  const plan = descriptor(sourcePaths, evidenceRoot);
  if (plan === null) return null;
  let value;
  try {
    value = JSON.parse(observedPayload(output));
  } catch {
    return null;
  }
  if (value?.schema !== 'prepared-artifact-inventory/v1' || value.complete !== true || value.evidenceRoot !== evidenceRoot || !Array.isArray(value.sourcePaths) || JSON.stringify(value.sourcePaths) !== JSON.stringify(sourcePaths)
    || !Array.isArray(value.ownedPaths) || value.ownedPaths.length > 256 || new Set(value.ownedPaths).size !== value.ownedPaths.length || !value.ownedPaths.every(path => plan.candidates.includes(path))) return null;
  return {
    ownedPaths: value.ownedPaths,
    sourcePaths: [...sourcePaths],
    evidenceRoot,
    sessionPolicy: {
      evidenceRoot,
      filename: 'session-id.txt',
      pattern: plan.sessionPattern,
      maximumBytes: plan.sessionLimit,
      path: evidenceRoot + 'session-id.txt'
    }
  };
}
