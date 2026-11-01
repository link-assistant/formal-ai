// Evidence-to-documentation composition. Declaration facts and source syntax
// observations stay separate; partial useful edits never certify the whole task.
import { planObservedDocumentation } from './observed_documentation.mjs';
import { sourceDocumentationPolicy, deriveManifestDocumentation, documentationAuthority } from './source_documentation.mjs';
import { workItemEvidencePaths } from './work_item_evidence.mjs';
import { host } from './host.mjs';
import { replaceAllLiteral } from './crate/rust_str.mjs';
export function repositoryDocumentationContract(task, policy) {
  const paths = workItemEvidencePaths(task),
    guides = paths.filter(path => path.endsWith('.md')),
    manifests = paths.filter(path => /\.(?:json|toml)$/u.test(path)),
    sourcePaths = paths.filter(path => /\.(?:mjs|js|ts|rs)$/u.test(path));
  if (paths.length < 3 || guides.length !== 1 || !manifests.length || paths.length !== guides.length + manifests.length + sourcePaths.length) return null;
  const ownership = documentationAuthority(task, guides[0], policy, {
    requirePathInClause: false,
    requirePreservation: false
  });
  if (ownership === null) return null;
  return {
    sources: [...manifests, ...sourcePaths],
    target: guides[0],
    authority: ownership,
    schema: 'repository-work-item-controller/v1',
    coverage: 'explicit-source-paths-only; consumer-discovery-pending'
  };
}
function fill(template, values) {
  let text = template;
  for (const [key, value] of Object.entries(values)) text = replaceAllLiteral(text, '{' + key + '}', value);
  return text;
}
function syntaxParagraph(owner, usage, policy) {
  const forms = policy.sourceUsage;
  if (!forms || typeof forms.heading !== 'string' || typeof forms.call !== 'string' || typeof forms.reference !== 'string' || typeof forms.codeOpen !== 'string' || typeof forms.codeClose !== 'string') return null;
  const rows = [];
  for (const call of usage.calls ?? []) {
    if (typeof call.source !== 'string' || new TextEncoder().encode(call.source).length > 512) return null;
    // Literal snippets remain code quotations, not paraphrased API contracts.
    // A source fence inside a snippet is refused rather than escaped lossily.
    if (call.source.includes('```')) return null;
    if ([call.imported, call.module].some(value => typeof value !== 'string' || /[`\r\n\p{Cc}\p{Cf}]/u.test(value))) return null;
    const caption = fill(forms.call, {
      owner,
      imported: call.imported,
      module: call.module
    });
    rows.push(caption + '\n\n' + fill(forms.codeOpen, {
      language: usage.language.toLowerCase()
    }) + '\n' + call.source + '\n' + forms.codeClose);
  }
  for (const reference of usage.references ?? []) {
    if (typeof reference.source !== 'string' || new TextEncoder().encode(reference.source).length > 512 || reference.source.includes('```')) return null;
    rows.push(fill(forms.reference, {
      owner
    }) + '\n\n' + fill(forms.codeOpen, {
      language: usage.language.toLowerCase()
    }) + '\n' + reference.source + '\n' + forms.codeClose);
  }
  if (!rows.length) return null;
  return fill(forms.heading, {
    owner
  }) + '\n\n' + rows.join('\n\n');
}
export function deriveRepositoryDocumentation(task, sources, guide, policy, usageOperation = host().observedSourceUsage) {
  let content = guide;
  const facts = [],
    remainingWork = ['independent-integration-semantics', 'consumer-discovery'];
  for (const source of sources.filter(row => /\.(?:json|toml)$/u.test(row.path))) {
    const result = deriveManifestDocumentation(task, source.path, source.source, content, policy);
    if (result.gap) return result;
    content = result.content;
    facts.push(...(result.facts ?? result.value ?? []));
  }
  for (const source of sources.filter(row => /\.(?:mjs|js|ts|rs)$/u.test(row.path))) {
    if (typeof usageOperation !== 'function') {
      remainingWork.push('source-syntax-unavailable:' + source.path);
      continue;
    }
    const language = source.path.endsWith('.rs') ? 'Rust' : source.path.endsWith('.ts') ? 'TypeScript' : 'JavaScript';
    let usage;
    try {
      usage = usageOperation({
        owner: source.path,
        source: source.source,
        language
      });
    } catch {
      remainingWork.push('source-syntax-unavailable:' + source.path);
      continue;
    }
    if (usage?.schema !== 'observed-source-usage/v1' || usage.owner !== source.path || usage.sourceIdentity !== source.sourceIdentity || usage.language !== language) {
      remainingWork.push('source-syntax-unbound:' + source.path);
      continue;
    }
    const paragraph = syntaxParagraph(source.path, usage, policy);
    if (paragraph === null) {
      remainingWork.push('source-syntax-rendering-unavailable:' + source.path);
      continue;
    }
    const heading = fill(policy.sourceUsage.heading, {
      owner: source.path
    });
    if (!content.includes(paragraph) && content.includes(heading)) return {
      gap: 'StaleSourceUsageDocumentation'
    };
    if (!content.includes(paragraph)) content = content.replace(/\n?$/u, '') + '\n\n' + paragraph + '\n';
    facts.push({
      kind: 'observed-source-usage',
      owner: usage.owner,
      sourceIdentity: usage.sourceIdentity,
      language: usage.language,
      imports: usage.imports,
      calls: usage.calls,
      references: usage.references,
      gaps: usage.gaps
    });
    if (usage.gaps?.length) remainingWork.push('source-resolution-unverified:' + source.path);
  }
  return {
    content,
    facts,
    remainingWork,
    binding: {
      kind: 'composed-source-observations',
      sources: sources.map(row => ({
        path: row.path,
        sourceIdentity: row.sourceIdentity
      })),
      coverage: 'explicit-source-paths-only; no executable-role or whole-task acceptance credit'
    }
  };
}
export function planRepositoryDocumentation(task, messages, tools, workItem = null) {
  const policy = sourceDocumentationPolicy();
  if (!policy) return null;
  const contract = repositoryDocumentationContract(task, policy);
  if (!contract) return null;
  return planObservedDocumentation(task, messages, tools, {
    ...contract,
    workItem
  }, (sources, guide) => deriveRepositoryDocumentation(task, sources, guide, policy));
}
