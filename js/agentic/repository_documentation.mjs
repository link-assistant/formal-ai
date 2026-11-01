import { sourceLexicalRule } from './source_lexical_policy.mjs';
import { WorkItemFileProgress, fileIoReadStep, fileIoSearchStep, fileIoPendingStep } from './work_item_file_io.mjs';
import { Capability } from './capability.mjs';
import { shellCommandTool } from './capability_router.mjs';
import { observedCargoSourceDirectories } from './source_manifest_facts.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
// Evidence-to-documentation composition. Declaration facts and source syntax
// observations stay separate; partial useful edits never certify the whole task.
import { planObservedDocumentation, ownerOf as discoveryOwner, complete as discoverySource } from './observed_documentation.mjs';
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
    if (sourceLexicalRule(call.source, 'source-fence-forbidden') !== false) return null;
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
    if (typeof reference.source !== 'string' || new TextEncoder().encode(reference.source).length > 512 || sourceLexicalRule(reference.source, 'source-fence-forbidden') !== false) return null;
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
// Externally specified bootstrap input, not autonomous Formal AI synthesis.
function discoveryReads(progress, owner) {
  return progress.attempts.filter(attempt => attempt.capability === Capability.Read && discoveryOwner(attempt) === owner);
}
function repositoryDiscoveryPreflight(task, messages, tools, contract, policy) {
  const discovery = {
    schema: 'observed-consumer-discovery/v1',
    state: 'explicit-only',
    queries: [],
    candidates: [],
    confirmedOwners: [],
    gaps: [],
    scope: 'bounded-observed-cargo-source-scopes; syntax witnesses only; no whole-repository absence or runtime semantics'
  };
  const result = {
    contract,
    discovery
  };
  const owned = path => typeof path === 'string' && /^[A-Za-z0-9_][A-Za-z0-9_./-]*$/u.test(path) && path.split('/').every(part => part !== '' && part !== '.' && part !== '..');
  if (contract.authority !== 'owned' || !Array.isArray(contract.sources) || !contract.sources.length || contract.sources.length > 16 || !contract.sources.every(owned) || !owned(contract.target) || new Set([...contract.sources, contract.target]).size !== contract.sources.length + 1 || !shellCommandTool(tools)) return result;
  const progress = WorkItemFileProgress.scan(messages);
  if (progress.gap) return {
    ...result,
    gap: progress.gap
  };
  const effects = progress.attempts.filter(attempt => [Capability.Edit, Capability.Write].includes(attempt.capability));
  if (effects.length > 1 || effects.some(attempt => attempt.capability !== Capability.Edit || discoveryOwner(attempt) !== contract.target) || progress.attempts.some(attempt => attempt.capability === Capability.Read && discoveryOwner(attempt) === null)) return result;
  const pending = fileIoPendingStep(progress, tools);
  if (pending) return pending.step ? {
    ...result,
    step: pending.step
  } : {
    ...result,
    gap: pending.gap
  };
  const effect = effects[0] ?? null;
  let cutoff = messages.length;
  if (effect) {
    const position = effect.transport?.completedAt,
      callIdentifier = effect.transport?.callId;
    if (!Number.isInteger(position) || position < 0 || position >= messages.length || typeof callIdentifier !== 'string' || messages[position].role.toLowerCase() !== 'tool' || messages[position].tool_call_id !== callIdentifier) {
      return {
        ...result,
        gap: 'ConsumerEffectProvenanceUnavailable'
      };
    }
    const declarations = messages.map((message, index) => ({
      message,
      index
    })).filter(({
      message,
      index
    }) => index < position && message.role.toLowerCase() === 'assistant' && (message.tool_calls ?? []).some(call => call.id === callIdentifier && call.function.name === effect.transport.actualTool));
    if (declarations.length !== 1) return {
      ...result,
      gap: 'ConsumerEffectProvenanceUnavailable'
    };
    cutoff = declarations[0].index;
  }
  const frozen = effect ? WorkItemFileProgress.scan(messages.slice(0, cutoff)) : progress;
  const queries = new Map();
  const note = reason => {
    if (!discovery.gaps.includes(reason)) discovery.gaps.push(reason);
  };
  const read = owner => {
    if (effect) return {
      ...result,
      gap: 'ConsumerPreEffectObservationMissing'
    };
    const planned = fileIoReadStep(messages, tools, owner, task);
    return planned.step ? {
      ...result,
      step: planned.step
    } : {
      ...result,
      gap: planned.gap
    };
  };
  for (const owner of contract.sources.filter(path => path.endsWith('.toml'))) {
    const observations = discoveryReads(frozen, owner);
    if (!observations.length) return read(owner);
    const first = observations.find(attempt => discoverySource(attempt) !== null);
    const source = discoverySource(first),
      latest = discoverySource(observations.at(-1));
    if (source === null || latest === null) return {
      ...result,
      gap: 'ConsumerManifestObservationIncomplete'
    };
    if (source !== latest) return {
      ...result,
      gap: 'ConsumerManifestSnapshotChanged'
    };
    let layout, selected;
    try {
      layout = observedCargoSourceDirectories(owner, source);
      selected = deriveManifestDocumentation(task, owner, source, '', policy);
    } catch {
      note('cargo-source-layout-unavailable:' + owner);
      continue;
    }
    const identity = sha256Hex(source);
    if (layout?.schema !== 'observed-cargo-source-directories/v1' || layout.manifest !== owner || layout.sourceIdentity !== identity || layout.complete !== true || !Array.isArray(layout.directories) || layout.directories.length > 4 || selected?.gap) {
      note('cargo-source-layout-unverified:' + owner);
      continue;
    }
    const aliases = [...new Set((selected.facts ?? selected.value ?? []).filter(fact => fact.kind === 'manifest-declaration').map(fact => fact.name.replace(/-/gu, '_')))].sort();
    if (!aliases.length || aliases.some(alias => !/^[A-Za-z_][A-Za-z0-9_]{0,127}$/u.test(alias))) {
      note('cargo-dependency-alias-unavailable:' + owner);
      continue;
    }
    for (const directory of layout.directories) {
      if (!queries.has(directory)) queries.set(directory, new Set());
      for (const alias of aliases) queries.get(directory).add(alias);
    }
  }
  const plannedQueries = [];
  for (const [directory, aliases] of [...queries].sort(([left], [right]) => left < right ? -1 : left > right ? 1 : 0)) {
    const identifiers = [...aliases].sort();
    for (let index = 0; index < identifiers.length; index += 8) plannedQueries.push({
      directory,
      identifiers: identifiers.slice(index, index + 8)
    });
  }
  if (queries.size > 4 || plannedQueries.length > 4) {
    note('consumer-query-budget-exceeded');
    discovery.state = 'incomplete';
    return result;
  }
  if (!plannedQueries.length) return result;
  const candidates = new Map();
  let incompleteSearch = false;
  for (const query of plannedQueries) {
    const observed = frozen.searchFor(query.directory, query.identifiers);
    if (!observed) {
      if (effect) return {
        ...result,
        gap: 'ConsumerPreEffectSearchMissing'
      };
      const planned = fileIoSearchStep(messages, tools, query.directory, query.identifiers, task);
      return planned.step ? {
        ...result,
        step: planned.step
      } : {
        ...result,
        gap: planned.gap
      };
    }
    discovery.queries.push({
      directory: query.directory,
      identifiers: query.identifiers,
      requestIdentity: observed.requestIdentity,
      status: observed.status,
      complete: observed.complete,
      root: observed.root,
      transport: {
        actualTool: observed.transport?.actualTool,
        callId: observed.transport?.callId,
        actualExitCode: observed.transport?.actualExitCode,
        completedAt: observed.transport?.completedAt
      }
    });
    if (observed.status !== 'reported-success' || observed.complete !== true || observed.root !== frozen.root || !Number.isInteger(observed.transport?.completedAt)) {
      incompleteSearch = true;
      note('consumer-search-incomplete:' + query.directory);
      continue;
    }
    for (const file of observed.files) {
      const prior = candidates.get(file.path);
      if (prior && prior.sha256 !== file.sha256) return {
        ...result,
        gap: 'ConsumerSearchSnapshotContradiction'
      };
      candidates.set(file.path, {
        path: file.path,
        sha256: file.sha256,
        after: Math.max(prior?.after ?? -1, observed.transport.completedAt),
        aliases: [...new Set([...(prior?.aliases ?? []), ...query.identifiers])].sort()
      });
    }
  }
  if (incompleteSearch) {
    discovery.state = 'incomplete';
    return result;
  }
  if (candidates.size > 8) {
    note('consumer-source-budget-exceeded');
    discovery.state = 'incomplete';
    return result;
  }
  const confirmed = [];
  for (const candidate of [...candidates.values()].sort((left, right) => left.path < right.path ? -1 : left.path > right.path ? 1 : 0)) {
    const reads = discoveryReads(frozen, candidate.path).filter(attempt => Number.isInteger(attempt.transport?.completedAt) && attempt.transport.completedAt > candidate.after);
    if (!reads.length) return read(candidate.path);
    const source = discoverySource(reads.at(-1));
    if (source === null || sha256Hex(source) !== candidate.sha256) return {
      ...result,
      gap: 'ConsumerSearchReadCustodyMismatch'
    };
    let usage;
    try {
      usage = host().observedSourceUsage({
        owner: candidate.path,
        source,
        language: 'Rust'
      });
    } catch {
      note('consumer-syntax-unavailable:' + candidate.path);
      continue;
    }
    if (usage?.schema !== 'observed-source-usage/v1' || usage.owner !== candidate.path || usage.sourceIdentity !== candidate.sha256 || usage.language !== 'Rust') {
      note('consumer-syntax-unbound:' + candidate.path);
      continue;
    }
    const witnessed = [...(usage.imports ?? []), ...(usage.calls ?? [])].some(row => typeof row.module === 'string' && candidate.aliases.includes(row.module.split('::')[0])) || (usage.references ?? []).some(row => typeof row.root === 'string' && candidate.aliases.includes(row.root.split('::')[0]));
    discovery.candidates.push({
      path: candidate.path,
      sourceIdentity: candidate.sha256,
      syntaxConfirmed: witnessed,
      gaps: usage.gaps ?? []
    });
    if (witnessed) confirmed.push(candidate.path);
    if (usage.gaps?.length) note('consumer-syntax-resolution-unverified:' + candidate.path);
  }
  const sources = [...new Set([...contract.sources, ...confirmed])];
  if (sources.length > 16) {
    note('documentation-source-budget-exceeded');
    discovery.state = 'incomplete';
    return result;
  }
  discovery.confirmedOwners = confirmed;
  discovery.state = discovery.gaps.length ? 'partial' : 'observed-scopes';
  result.contract = {
    ...contract,
    sources,
    coverage: discovery.scope
  };
  return result;
}
export function planRepositoryDocumentation(task, messages, tools, workItem = null) {
  const policy = sourceDocumentationPolicy();
  if (!policy) return null;
  const contract = repositoryDocumentationContract(task, policy);
  if (!contract) return null;
  if (contract.authority !== 'owned' || !shellCommandTool(tools)) {
    return planObservedDocumentation(task, messages, tools, {
      ...contract,
      workItem
    }, (sources, guide) => deriveRepositoryDocumentation(task, sources, guide, policy));
  }
  const preflight = repositoryDiscoveryPreflight(task, messages, tools, contract, policy);
  if (preflight.step) return preflight.step;
  return planObservedDocumentation(task, messages, tools, {
    ...preflight.contract,
    workItem
  }, (sources, guide) => {
    if (preflight.gap) return {
      gap: preflight.gap
    };
    const derived = deriveRepositoryDocumentation(task, sources, guide, policy);
    if (derived.gap) return derived;
    return {
      ...derived,
      facts: [...(derived.facts ?? []), {
        kind: 'observed-consumer-discovery',
        ...preflight.discovery
      }],
      binding: {
        ...derived.binding,
        discovery: preflight.discovery
      },
      remainingWork: [...new Set([...(derived.remainingWork ?? []), ...preflight.discovery.gaps.map(reason => 'consumer-discovery-unverified:' + reason)])]
    };
  });
}
