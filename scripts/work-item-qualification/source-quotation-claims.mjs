// Externally specified host-only grader. Shared Babel grammar, independent
// AST/scope verification logic; no production CST, renderer or extractor imports.
import { createHash } from 'node:crypto';
import { parse } from '@babel/parser';
import traversal from '@babel/traverse';
import { checkOwnedManifestTables } from './owned-manifest-claims.mjs';
const traverse = traversal.default ?? traversal;
const digest = value => createHash('sha256').update(value).digest('hex');
const maximumBytes = 262144;
const ownerValid = owner => typeof owner === 'string' && /^[A-Za-z0-9_][A-Za-z0-9_./-]*$/u.test(owner) && owner.split('/').every(part => part !== '' && part !== '.' && part !== '..');
const languageOf = owner => /\.(?:ts|mts|cts|tsx)$/u.test(owner) ? 'typescript' : /\.(?:js|mjs|cjs|jsx)$/u.test(owner) ? 'javascript' : owner.endsWith('.rs') ? 'rust' : null;
function packageRoot(module) {
  if (module.startsWith('.') || module.startsWith('/') || module.includes(':')) return null;
  const parts = module.split('/');
  return module.startsWith('@') && parts.length >= 2 ? parts.slice(0, 2).join('/') : parts[0] || null;
}
function dependencyBinding(owner, module, files, evidence) {
  const name = packageRoot(module);
  if (!name) return {
    status: 'unknown',
    reason: 'nonpackage-module-resolution-pending'
  };
  const directory = owner.split('/');
  directory.pop();
  for (let depth = directory.length; depth >= 0; depth--) {
    const manifest = [...directory.slice(0, depth), 'package.json'].join('/');
    const file = files.get(manifest);
    if (!file) continue;
    const sourceIdentity = digest(file.content);
    const observed = evidence.observations?.find(row => row.path === manifest && row.sourceIdentity === sourceIdentity && row.status === 'observed');
    if (!observed) return {
      status: 'unknown',
      reason: 'nearest-npm-manifest-not-independently-observed',
      manifest
    };
    const facts = (evidence.facts ?? []).filter(row => row.kind === 'manifest-declaration' && row.manifest === manifest && row.sourceIdentity === sourceIdentity && row.name === name);
    return facts.length ? {
      status: 'verified-literal-declaration',
      manifest,
      sourceIdentity,
      name,
      declarations: facts
    } : {
      status: 'unknown',
      reason: 'module-not-declared-in-nearest-npm-manifest',
      manifest,
      name
    };
  }
  return {
    status: 'unknown',
    reason: 'nearest-npm-manifest-unavailable',
    name
  };
}
function literalOptions(node, source) {
  const result = [];
  for (const [argumentIndex, argument] of (node.arguments ?? []).entries()) {
    if (argument.type !== 'ObjectExpression') continue;
    for (const property of argument.properties) {
      if (property.type !== 'ObjectProperty' || property.computed || property.method) {
        result.push({
          argumentIndex,
          kind: 'unresolved-option-syntax'
        });
        continue;
      }
      const key = property.key.type === 'Identifier' ? property.key.name : property.key.type === 'StringLiteral' ? property.key.value : null;
      result.push({
        argumentIndex,
        key,
        valueSource: source.slice(property.value.start, property.value.end)
      });
    }
  }
  return result;
}
function independentCalls(file) {
  const source = file.content,
    language = languageOf(file.path);
  const ast = parse(source, {
    sourceType: 'unambiguous',
    errorRecovery: false,
    plugins: [...(language === 'typescript' ? ['typescript'] : []), ...(/\.(?:jsx|tsx)$/u.test(file.path) || language === 'javascript' ? ['jsx'] : [])]
  });
  const result = [];
  let nodes = 0;
  traverse(ast, {
    enter(path) {
      if (++nodes > 50000) throw Error('IndependentAstNodeBudget');
      if (!['CallExpression', 'NewExpression', 'OptionalCallExpression'].includes(path.node.type)) return;
      const node = path.node,
        callee = node.callee;
      const observation = {
        source: source.slice(node.start, node.end),
        span: [Buffer.byteLength(source.slice(0, node.start)), Buffer.byteLength(source.slice(0, node.end))],
        sourceIdentity: digest(source),
        kind: node.type,
        options: literalOptions(node, source)
      };
      let local = null,
        member = null;
      if (callee.type === 'Identifier') local = callee.name;else if (callee.type === 'MemberExpression' && !callee.computed && callee.object.type === 'Identifier' && callee.property.type === 'Identifier') {
        local = callee.object.name;
        member = callee.property.name;
      } else {
        result.push({
          ...observation,
          resolution: 'unsupported-callee-syntax'
        });
        return;
      }
      const binding = path.scope.getBinding(local),
        specifier = binding?.path,
        importPath = specifier?.parentPath;
      let resolution = 'unresolved-static-import',
        module = null,
        imported = null;
      if (specifier && importPath?.isImportDeclaration()) {
        const declaration = importPath.node;
        module = declaration.source.value;
        const kind = specifier.node.type;
        if (kind === 'ImportSpecifier') imported = specifier.node.imported.name ?? specifier.node.imported.value;else if (kind === 'ImportDefaultSpecifier') imported = 'default';else if (kind === 'ImportNamespaceSpecifier') imported = '*';
        const callable = member === null ? kind !== 'ImportNamespaceSpecifier' : kind === 'ImportNamespaceSpecifier';
        const valueImport = declaration.importKind !== 'type' && specifier.node.importKind !== 'type';
        if (callable && valueImport && !node.optional && binding.constant && binding.constantViolations.length === 0) resolution = 'independent-static-import-binding';
      }
      result.push({
        ...observation,
        module,
        imported,
        local,
        member,
        resolution
      });
    }
  });
  return result;
}
function verifyQuote(owner, caption, quotation, files, evidence, cache) {
  const file = files.get(owner),
    sourceIdentity = file ? digest(file.content) : null;
  const base = {
    kind: 'source-quotation',
    owner,
    sourceIdentity
  };
  if (!file) return {
    ...base,
    status: 'unknown',
    reason: 'frozen-source-unavailable'
  };
  if ((file.sourceIdentity ?? file.sha256 ?? sourceIdentity) !== sourceIdentity) return {
    ...base,
    status: 'unknown',
    reason: 'frozen-source-identity-mismatch'
  };
  if (quotation.length === 0 || Buffer.byteLength(quotation) > 512 || quotation.includes('```')) return {
    ...base,
    status: 'unknown',
    reason: 'quotation-budget-or-fence'
  };
  if (caption.kind === 'call' && [caption.imported, caption.module].some(value => /[`\p{Cc}\p{Cf}]/u.test(value))) return {
    ...base,
    status: 'unknown',
    reason: 'unsafe-source-caption'
  };
  if (caption.kind === 'reference') return {
    ...base,
    status: 'unknown',
    reason: 'qualified-reference-binding-unverified',
    quotationVerified: file.content.includes(quotation),
    quotationScope: 'exact-bytes-only'
  };
  if (languageOf(owner) === 'rust') return {
    ...base,
    status: 'unknown',
    reason: 'independent-rust-binding-parser-unavailable',
    quotationVerified: file.content.includes(quotation),
    quotationScope: 'exact-bytes-only'
  };
  if (!['javascript', 'typescript'].includes(languageOf(owner))) return {
    ...base,
    status: 'unknown',
    reason: 'source-language-unsupported'
  };
  let calls;
  try {
    if (!cache.has(owner)) cache.set(owner, independentCalls(file));
    calls = cache.get(owner);
  } catch {
    return {
      ...base,
      status: 'unknown',
      reason: 'independent-source-parse-unavailable'
    };
  }
  const candidates = calls.filter(call => call.source === quotation);
  if (!candidates.length) return {
    ...base,
    status: 'contradicted',
    reason: 'quotation-is-not-an-exact-observed-call'
  };
  const matched = candidates.filter(call => call.resolution === 'independent-static-import-binding' && call.module === caption.module && call.imported === caption.imported);
  if (!matched.length) return candidates.some(call => call.resolution === 'independent-static-import-binding') ? {
    ...base,
    status: 'contradicted',
    reason: 'caption-does-not-match-lexical-import-binding'
  } : {
    ...base,
    status: 'unknown',
    reason: 'static-import-binding-not-established'
  };
  return {
    ...base,
    status: 'verified-static-binding',
    reason: 'exact-ast-call-and-independent-scope-binding',
    witnesses: matched,
    dependency: dependencyBinding(owner, caption.module, files, evidence),
    semantics: 'syntax and literal declaration only; no API role or runtime execution credit'
  };
}
function inputFiles(values) {
  if (!Array.isArray(values) || values.length > 16) throw Error('FrozenFileBudget');
  const files = new Map();
  let bytes = 0;
  for (const value of values) {
    const content = value.content ?? value.source;
    if (!ownerValid(value.path) || typeof content !== 'string' || Buffer.byteLength(content) > maximumBytes || files.has(value.path)) throw Error('FrozenFileContract');
    bytes += Buffer.byteLength(content);
    if (bytes > 2 * 1024 * 1024) throw Error('FrozenInputBudget');
    files.set(value.path, {
      ...value,
      content
    });
  }
  return files;
}
/** Complete changed-tail accounting, with deliberately partial semantic verdicts. */
export function checkGeneratedDocumentation(candidate, originalGuide, frozenFiles, evidence = {
  facts: [],
  observations: []
}) {
  const claims = [],
    segments = [],
    cache = new Map(),
    seen = new Set();
  let files,
    prefixPreserved = false;
  const unknown = (reason, extra = {}) => claims.push({
    status: 'unknown',
    reason,
    ...extra
  });
  const finish = () => ({
    schema: 'independent-generated-documentation/v1',
    prefixPreserved,
    segments,
    claims,
    obligations: [{
      id: 'original-guide-preserved',
      status: prefixPreserved ? 'passed' : 'failed'
    }, {
      id: 'all-added-segments-accounted-for',
      status: claims.length && claims.every(row => ['verified-static-binding', 'verified-literal-declaration'].includes(row.status)) ? 'passed-static-fields-only' : 'unresolved'
    }, {
      id: 'javascript-module-manifest-binding',
      status: claims.some(row => row.status === 'verified-static-binding') && claims.filter(row => row.status === 'verified-static-binding').every(row => row.dependency?.status === 'verified-literal-declaration') ? 'passed-literal-fields-only' : 'pending'
    }, {
      id: 'external-api-roles',
      status: 'pending'
    }, {
      id: 'consumer-discovery',
      status: 'pending'
    }, {
      id: 'rust-root-resolution',
      status: 'pending'
    }],
    complete: false,
    semanticAcceptance: 'pending',
    independence: 'Direct Babel AST/scope logic; shared Babel grammar dependency; no production CST/extractor/renderer',
    authorshipCredit: 0
  });
  if (typeof candidate !== 'string' || typeof originalGuide !== 'string' || Buffer.byteLength(candidate) > maximumBytes) {
    unknown('guide-input-budget');
    return finish();
  }
  try {
    files = inputFiles(frozenFiles);
  } catch {
    unknown('frozen-input-contract');
    return finish();
  }
  if (!Array.isArray(evidence.facts) || !Array.isArray(evidence.observations)) {
    unknown('independent-manifest-evidence-unavailable');
    return finish();
  }
  const observations = evidence.observations.filter(row => {
    const file = files.get(row.path);
    return file && row.sourceIdentity === digest(file.content) && (file.sourceIdentity ?? file.sha256 ?? row.sourceIdentity) === row.sourceIdentity;
  });
  evidence = {
    observations,
    facts: evidence.facts.filter(fact => observations.some(row => row.status === 'observed' && row.path === fact.manifest && row.sourceIdentity === fact.sourceIdentity))
  };
  if (!candidate.startsWith(originalGuide)) {
    claims.push({
      status: 'contradicted',
      reason: 'original-guide-content-changed'
    });
    return finish();
  }
  prefixPreserved = true;
  const lines = candidate.slice(originalGuide.length).split('\n');
  let cursor = 0;
  while (cursor < lines.length) {
    if (lines[cursor].trim() === '') {
      cursor++;
      continue;
    }
    if (segments.length >= 32 || claims.length >= 256) {
      unknown('generated-segment-budget');
      break;
    }
    const start = cursor,
      line = lines[cursor];
    const manifest = /^<!-- formal-ai-manifest-start: ([A-Za-z0-9_][A-Za-z0-9_./-]*) -->$/u.exec(line);
    if (manifest) {
      const owner = manifest[1],
        end = lines.indexOf('<!-- formal-ai-manifest-end: ' + owner + ' -->', cursor + 1);
      if (end < 0) {
        unknown('unclosed-owned-manifest', {
          owner
        });
        break;
      }
      const identity = 'manifest:' + owner;
      if (seen.has(identity)) {
        unknown('duplicate-owned-manifest', {
          owner
        });
        cursor = end + 1;
        continue;
      }
      seen.add(identity);
      claims.push(...checkOwnedManifestTables(lines.slice(cursor, end + 1).join('\n'), evidence.facts ?? [], evidence.observations ?? [], {
        expectedOwners: [owner]
      }));
      segments.push({
        kind: 'manifest-table',
        owner,
        lines: [cursor, end + 1]
      });
      cursor = end + 1;
      continue;
    }
    const sourceHeading = /^Source syntax observed in `([^`\r\n]+)`:$/u.exec(line);
    if (!sourceHeading) {
      unknown('unsupported-added-segment', {
        line: cursor + 1,
        textIdentity: digest(line)
      });
      cursor++;
      continue;
    }
    const owner = sourceHeading[1],
      identity = 'syntax:' + owner;
    if (!ownerValid(owner) || seen.has(identity)) {
      unknown('invalid-or-duplicate-source-section', {
        owner
      });
      cursor++;
      continue;
    }
    seen.add(identity);
    cursor++;
    let count = 0;
    while (cursor < lines.length) {
      while (cursor < lines.length && lines[cursor] === '') cursor++;
      if (count >= 128 || claims.length >= 256) {
        unknown('source-quotation-count-budget', {
          owner
        });
        break;
      }
      const captionLine = lines[cursor] ?? '';
      const call = /^Imported `([^`\r\n]+)` from `([^`\r\n]+)` is called in `([^`\r\n]+)`; the following excerpt records its syntax\.$/u.exec(captionLine);
      const reference = /^Qualified source reference in `([^`\r\n]+)`; module-root resolution is unverified\.$/u.exec(captionLine);
      if (!call && !reference) break;
      const caption = call ? {
        kind: 'call',
        imported: call[1],
        module: call[2],
        owner: call[3]
      } : {
        kind: 'reference',
        owner: reference[1]
      };
      const language = languageOf(owner);
      if (caption.owner !== owner || lines[cursor + 1] !== '' || lines[cursor + 2] !== '```' + language) {
        unknown('caption-owner-or-fence-mismatch', {
          owner
        });
        cursor++;
        break;
      }
      const first = cursor + 3,
        end = lines.indexOf('```', first);
      if (end < 0) {
        unknown('unclosed-source-fence', {
          owner
        });
        cursor = lines.length;
        break;
      }
      claims.push(verifyQuote(owner, caption, lines.slice(first, end).join('\n'), files, evidence, cache));
      count++;
      cursor = end + 1;
    }
    if (!count) unknown('empty-source-syntax-section', {
      owner
    });
    segments.push({
      kind: 'source-syntax',
      owner,
      lines: [start, cursor]
    });
  }
  return finish();
}
