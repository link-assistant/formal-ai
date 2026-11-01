import { planObservedDocumentation } from './observed_documentation.mjs';
import { observedSourceJson } from './source_manifest_facts.mjs';
import { sourceDocumentationPolicy } from './source_documentation_policy.mjs';
import { deriveManifestDocumentation } from './manifest_documentation.mjs';
export { sourceDocumentationPolicy, deriveManifestDocumentation };
import { workItemEvidencePaths } from './work_item_evidence.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { sentences } from './shell_command_policy.mjs';
import { quotedSegmentSpans, quoteFault } from './crate/normal_markov.mjs';
import { replaceAllLiteral } from './crate/rust_str.mjs';
export function conceptWords(text) {
  return text.replace(/([a-z0-9])([A-Z])/gu, '$1 $2').toLowerCase().split(/[^\p{L}\p{N}]+/u).filter(Boolean);
}
export function documentationAuthority(task, guidePath, policy, {
  requirePathInClause = true,
  requirePreservation = true
} = {}) {
  if (quoteFault(task) !== null) return 'DocumentationActionUnowned';
  let masked = task;
  for (const span of [...quotedSegmentSpans(task)].reverse()) masked = masked.slice(0, span.start) + masked.slice(span.start, span.end).replace(/[^\r\n]/g, ' ') + masked.slice(span.end);
  if (requirePreservation && !policy.preserve.some(cue => conceptWords(masked).includes(cue))) return null;
  let mentioned = false;
  for (const clause of sentences(masked)) {
    const original = task.slice(clause.span.start, clause.span.end);
    if (requirePathInClause && !original.includes(guidePath)) continue;
    const words = conceptWords(clause.text);
    for (const action of policy.actions) {
      const index = words.indexOf(action);
      if (index < 0) continue;
      mentioned = true;
      const before = words.slice(0, index).join(' ');
      if (policy.negations.some(negation => (' ' + before + ' ').includes(' ' + conceptWords(negation).join(' ') + ' '))) return 'NegatedDocumentationAction';
      if (index === 0 || policy.prefixes.includes(before)) return 'owned';
    }
  }
  const raw = conceptWords(task);
  return mentioned || policy.actions.some(action => raw.includes(action)) ? 'DocumentationActionUnowned' : null;
}
function manifestDeclarationRequest(task, policy) {
  if (!policy || quoteFault(task) !== null) return false;
  let masked = task;
  for (const span of [...quotedSegmentSpans(task)].reverse()) masked = masked.slice(0, span.start) + masked.slice(span.start, span.end).replace(/[^\r\n]/g, ' ') + masked.slice(span.end);
  return sentences(masked).some(clause => {
    const words = conceptWords(clause.text);
    if (policy.negations.some(cue => (' ' + words.join(' ') + ' ').includes(' ' + conceptWords(cue).join(' ') + ' '))) return false;
    return [policy.dependencyCues, policy.versionCues, policy.sectionCues].every(cues => cues?.some(cue => words.includes(cue)));
  });
}
export function deriveSourceDocumentation(task, sourcePath, source, guide, template, forms = null) {
  let object;
  try {
    object = observedSourceJson(source);
  } catch {
    return {
      gap: 'InvalidSourceJson'
    };
  }
  if (!object || typeof object !== 'object' || Array.isArray(object)) return {
    gap: 'SourceObjectRequired'
  };
  const requested = new Set(conceptWords(task));
  const bindings = Object.entries(object).filter(([key, value]) => {
    const words = conceptWords(key);
    return words.length > 0 && words.every(word => requested.has(word)) && ['string', 'boolean'].includes(typeof value);
  });
  const policy = forms ?? sourceDocumentationPolicy();
  if (manifestDeclarationRequest(task, policy)) {
    if (bindings.length) return {
      gap: 'AmbiguousDocumentationBinding'
    };
    return deriveManifestDocumentation(task, sourcePath, source, guide, policy);
  }
  if (bindings.length !== 1) return {
    gap: bindings.length ? 'AmbiguousPropertyBinding' : 'MissingPropertyBinding'
  };
  const [key, value] = bindings[0];
  const text = String(value);
  if (/[\r\n`]/u.test(text) || /[\r\n`]/u.test(sourcePath) || typeof template !== 'string') return {
    gap: 'UnsupportedDocumentationValue'
  };
  const sourceBound = replaceAllLiteral(template, '{source}', sourcePath);
  const conceptBound = replaceAllLiteral(sourceBound, '{concept}', conceptWords(key).join(' '));
  const paragraph = replaceAllLiteral(conceptBound, '{value}', text);
  const suffix = '\n\n' + paragraph + '\n';
  return {
    key,
    value,
    paragraph,
    sourceIdentity: sha256Hex(source),
    content: guide.includes(paragraph) ? guide : guide.replace(/\n?$/u, '') + suffix
  };
}
export function planSourceDocumentationStep(task, messages, tools, workItem = null) {
  const policy = sourceDocumentationPolicy();
  if (!policy) return null;
  const paths = workItemEvidencePaths(task),
    sources = paths.filter(path => path.endsWith('.json')),
    guides = paths.filter(path => path.endsWith('.md'));
  if (paths.length !== 2 || sources.length !== 1 || guides.length !== 1) return null;
  const [sourcePath] = sources,
    [guidePath] = guides,
    authority = documentationAuthority(task, guidePath, policy);
  if (authority === null) return null;
  return planObservedDocumentation(task, messages, tools, {
    sources,
    target: guidePath,
    authority,
    workItem
  }, (observed, guide) => deriveSourceDocumentation(task, sourcePath, observed[0].source, guide, policy.template, policy));
}
