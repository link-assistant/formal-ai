// Intended owner: js/agentic/observed_source_usage.mjs.
// Syntax observations over an authenticated complete readable-source CST.
// No source evaluation, token-pattern call search or external API semantics.
import { sourceToLinks, sourceFromLinks, ownedSourceLinks, verifySourceLinks } from '../meta/readable-source.js';
import { textOf } from '../logging/lino-codec.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
const metadata = new Set(['form', 'role', 'text', 'language', 'path', 'meaning', 'semantic-gap']);
const children = node => node.filter(item => Array.isArray(item) && !metadata.has(item[0]));
function value(node, name) {
  const found = node.filter(item => Array.isArray(item) && item[0] === name);
  if (found.length > 1) throw Error('SourceUsageAmbiguousField');
  return found[0] ? textOf(found[0][1]) : null;
}
const form = node => value(node, 'form');
const roles = node => (value(node, 'role') ?? '').split('|');
const hasRole = (node, name) => roles(node).includes(name);
const concrete = node => value(node, 'text') ?? children(node).map(concrete).join('');
const named = (node, names) => children(node).filter(child => names.some(name => hasRole(child, name)));
function one(node, names) {
  const found = named(node, names);
  return found.length === 1 ? found[0] : null;
}
const identifier = node => ['identifier', 'type_identifier', 'field_identifier', 'property_identifier', 'crate', 'self', 'super'].includes(form(node)) && value(node, 'text') !== null ? concrete(node) : null;
const trivia = node => ['whitespace', 'comment', 'line_comment', 'block_comment'].includes(form(node));
function literal(node) {
  if (!node || !['string_literal', 'string'].includes(form(node))) return null;
  const raw = concrete(node),
    quote = raw[0];
  if (!['"', "'"].includes(quote) || raw.at(-1) !== quote) return null;
  const body = raw.slice(1, -1);
  return body.includes('\\') || /[\r\n\u0000-\u001f]/u.test(body) ? null : body;
}
function pathSegments(node) {
  const name = identifier(node);
  if (name !== null) return [name];
  if (['scoped_identifier', 'scoped_type_identifier'].includes(form(node))) {
    const prefix = one(node, ['path']),
      tail = one(node, ['name']);
    if (!tail) return null;
    const first = prefix ? pathSegments(prefix) : [],
      last = pathSegments(tail);
    return first && last ? [...first, ...last] : null;
  }
  return null;
}

/** The frontend must be installed by the host, never decoded from issue data. */
export function observedSourceUsage({
  owner,
  source,
  language,
  frontend
}) {
  if (typeof owner !== 'string' || owner.startsWith('/') || owner.split('/').some(part => !part || part === '.' || part === '..') || typeof source !== 'string' || new TextEncoder().encode(source).length > 262144 || !['JavaScript', 'TypeScript', 'Rust'].includes(language) || typeof frontend?.parse !== 'function') throw Error('SourceUsageObservedInputRequired');
  const document = ownedSourceLinks(owner, sourceToLinks(source, language, frontend));
  verifySourceLinks(document, frontend);
  if (sourceFromLinks(document) !== source) throw Error('SourceUsageReconstructionMismatch');
  const bytes = new TextEncoder().encode(source);
  const {
    records,
    byNode
  } = observedSourceView(source, document);
  const imports = [],
    calls = [],
    references = [],
    gaps = [],
    bindingMap = new Map(),
    unsafe = new Set(),
    localModules = new Set();
  let truncated = false;
  const observation = record => {
    let end = Math.min(record.end, record.start + 512);
    while (end > record.start && end < bytes.length && (bytes[end] & 0xc0) === 0x80) end--;
    return {
      span: [record.start, record.end],
      source: new TextDecoder('utf-8', {
        fatal: true
      }).decode(bytes.subarray(record.start, end)),
      sourceTruncated: end < record.end
    };
  };
  function gap(kind, record, binding = null) {
    if (gaps.length >= 64) {
      truncated = true;
      return;
    }
    const range = record ? observation(record) : {};
    if (!gaps.some(item => item.kind === kind && item.binding === binding && JSON.stringify(item.span) === JSON.stringify(range.span))) gaps.push({
      kind,
      ...(binding ? {
        binding
      } : {}),
      ...range
    });
  }
  function append(array, item) {
    if (array.length >= 128) {
      truncated = true;
      return false;
    }
    array.push(item);
    return true;
  }
  function register(module, bindings, record, kind) {
    if (!append(imports, {
      module,
      bindings,
      kind,
      ...observation(record)
    })) return;
    for (const binding of bindings) {
      if (binding.local === '_') {
        gap('anonymous-import-binding-unresolved', record, binding.local);
        continue;
      }
      if (bindingMap.has(binding.local)) {
        unsafe.add(binding.local);
        gap('duplicate-import-binding', record, binding.local);
      } else bindingMap.set(binding.local, {
        module,
        ...binding
      });
    }
  }
  function javascriptImport(record) {
    const node = record.node,
      moduleNode = one(node, ['source']),
      module = literal(moduleNode);
    if (module === null) {
      gap('module-literal-escape-or-grammar-unsupported', record);
      return;
    }
    const typeOnly = children(node).some(child => value(child, 'text') === 'type');
    const bindings = [];
    function specifier(current) {
      const kind = form(current);
      if (kind === 'import_specifier') {
        const imported = one(current, ['imported', 'name']),
          local = one(current, ['local', 'alias']) ?? imported;
        const importedName = imported ? identifier(imported) ?? literal(imported) : null,
          localName = local ? identifier(local) : null;
        if (importedName === null || localName === null) {
          gap('import-binding-grammar-unsupported', byNode.get(current));
          return;
        }
        bindings.push({
          imported: importedName,
          local: localName,
          kind: typeOnly || children(current).some(child => value(child, 'text') === 'type') ? 'type' : 'named'
        });
        return;
      }
      if (['import_default_specifier', 'import_namespace_specifier', 'namespace_import'].includes(kind)) {
        const local = one(current, ['local', 'alias']) ?? children(current).find(child => identifier(child) !== null);
        if (!local || identifier(local) === null) {
          gap('import-binding-grammar-unsupported', byNode.get(current));
          return;
        }
        bindings.push({
          imported: kind === 'import_default_specifier' ? 'default' : '*',
          local: identifier(local),
          kind: typeOnly ? 'type' : kind === 'import_default_specifier' ? 'default' : 'namespace'
        });
        return;
      }
      if (kind === 'identifier' && form(record.node) === 'import_statement') {
        bindings.push({
          imported: 'default',
          local: identifier(current),
          kind: typeOnly ? 'type' : 'default'
        });
        return;
      }
      if (['import_clause', 'named_imports'].includes(kind)) for (const child of children(current)) specifier(child);
    }
    for (const child of children(node)) if (hasRole(child, 'specifiers') || form(child) === 'import_clause') specifier(child);
    register(module, bindings, record, 'static-import');
  }
  function rustUse(record) {
    const argument = one(record.node, ['argument']);
    if (!argument) {
      gap('rust-use-grammar-unsupported', record);
      return;
    }
    const grouped = new Map();
    function binding(segments, alias, kind = 'named') {
      if (!segments?.length) {
        gap('rust-use-path-unresolved', record);
        return;
      }
      const module = segments[0],
        imported = segments.slice(1).join('::') || 'self',
        local = alias ?? segments.at(-1);
      if (!grouped.has(module)) grouped.set(module, []);
      grouped.get(module).push({
        imported,
        local,
        kind: segments.length === 1 ? 'namespace' : kind
      });
    }
    function tree(node, prefix = []) {
      const kind = form(node);
      if (kind === 'use_as_clause') {
        const path = one(node, ['path']),
          alias = one(node, ['alias']);
        const segments = path ? pathSegments(path) : null;
        if (!segments || !alias || identifier(alias) === null) {
          gap('rust-use-alias-grammar-unsupported', byNode.get(node));
          return;
        }
        binding(segments.length === 1 && segments[0] === 'self' && prefix.length ? prefix : [...prefix, ...segments], identifier(alias), segments.length === 1 && segments[0] === 'self' ? 'namespace' : 'named');
        return;
      }
      if (kind === 'scoped_use_list') {
        const path = one(node, ['path']),
          list = one(node, ['list']);
        const segments = path ? pathSegments(path) : [];
        if (!segments || !list) {
          gap('rust-use-list-grammar-unsupported', byNode.get(node));
          return;
        }
        tree(list, [...prefix, ...segments]);
        return;
      }
      if (kind === 'use_list') {
        for (const child of children(node)) if (!trivia(child) && !['{', '}', ','].includes(concrete(child))) tree(child, prefix);
        return;
      }
      if (kind === 'use_wildcard') {
        gap('glob-import-resolution-unavailable', byNode.get(node));
        return;
      }
      const segments = pathSegments(node);
      if (segments) binding(segments.length === 1 && segments[0] === 'self' && prefix.length ? prefix : [...prefix, ...segments], null, segments.length === 1 && segments[0] === 'self' ? 'namespace' : 'named');else gap('rust-use-path-grammar-unsupported', byNode.get(node));
    }
    tree(argument);
    for (const [module, bindings] of grouped) register(module, bindings, record, 'rust-use');
  }
  for (const record of records) {
    if (['import_declaration', 'import_statement'].includes(form(record.node))) javascriptImport(record);
    if (form(record.node) === 'use_declaration') rustUse(record);
    if (form(record.node) === 'import_expression') gap('dynamic-module-loading-not-followed', record);
    if (form(record.node) === 'ts_import_equals_declaration') gap('module-binding-grammar-not-supported', record);
    if (form(record.node) === 'extern_crate_declaration') gap('extern-crate-binding-grammar-not-supported', record);
    if (['mod_item', 'module'].includes(form(record.node))) {
      const name = one(record.node, ['name']);
      if (name && identifier(name)) localModules.add(identifier(name));
    }
  }
  const inImport = record => record.ancestors.some(node => ['import_declaration', 'import_statement', 'use_declaration'].includes(form(node)));
  const descendants = node => {
    const record = byNode.get(node);
    return record ? records.slice(record.index, record.subtreeEnd) : [];
  };
  for (const record of records) {
    if (inImport(record)) continue;
    const kind = form(record.node);
    let regions = [];
    if (['function_declaration', 'function_expression', 'arrow_function_expression', 'function_item', 'closure_expression', 'class_declaration', 'struct_item', 'enum_item', 'trait_item', 'mod_item', 'type_item', 'const_item', 'static_item'].includes(kind)) regions = named(record.node, ['name', 'params', 'param', 'parameters']);
    if (['variable_declarator', 'let_declaration', 'catch_clause', 'parameter', 'required_parameter', 'optional_parameter'].includes(kind)) regions = named(record.node, ['name', 'pattern', 'param', 'parameter']);
    if (['assignment_expression', 'augmented_assignment_expression', 'update_expression'].includes(kind)) regions = named(record.node, ['left', 'argument']);
    for (const region of regions) for (const nested of descendants(region)) {
      const name = identifier(nested.node);
      if (name !== null && bindingMap.has(name)) {
        unsafe.add(name);
        gap('possible-shadow-or-reassignment', nested, name);
      }
    }
    if (['variable_declarator', 'let_declaration'].includes(kind)) {
      const initializer = one(record.node, ['init', 'value']);
      const name = initializer ? identifier(initializer) ?? (['member_expression', 'field_expression'].includes(form(initializer)) ? identifier(one(initializer, ['object', 'value']) ?? initializer) : null) : null;
      if (name && bindingMap.has(name)) gap('alias-flow-not-followed', record, name);
    }
  }
  function options(node) {
    if (!node || !['object_expression', 'object', 'field_initializer_list'].includes(form(node))) return [];
    const fields = [];
    for (const child of children(node)) {
      if (trivia(child) || ['{', '}', ','].includes(concrete(child))) continue;
      if (!['object_property', 'pair', 'field_initializer', 'shorthand_field_initializer', 'shorthand_property_identifier'].includes(form(child))) {
        gap('option-spread-or-field-grammar-unresolved', byNode.get(child));
        continue;
      }
      const key = one(child, ['key', 'field', 'name']),
        expression = one(child, ['value']);
      const name = key ? identifier(key) ?? literal(key) : identifier(child);
      if (name === null) {
        gap('option-key-resolution-unavailable', byNode.get(child));
        continue;
      }
      if (fields.length >= 16) {
        truncated = true;
        break;
      }
      fields.push({
        name,
        ...observation(byNode.get(child)),
        valueKind: expression ? form(expression) : 'shorthand-reference'
      });
    }
    return fields;
  }
  for (const record of records) {
    const kind = form(record.node);
    if (!['call_expression', 'optional_call_expression', 'new_expression', 'struct_expression'].includes(kind) || inImport(record)) continue;
    const callee = one(record.node, kind === 'struct_expression' ? ['name'] : ['callee', 'function']);
    if (!callee) {
      gap('call-grammar-unavailable', record);
      continue;
    }
    if (form(callee) === 'import') {
      gap('dynamic-module-loading-not-followed', record);
      continue;
    }
    let segments = pathSegments(callee),
      member = null,
      root = segments?.[0] ?? null,
      dynamic = false;
    if (['member_expression', 'optional_member_expression', 'field_expression'].includes(form(callee))) {
      const object = one(callee, ['object', 'value']),
        property = one(callee, ['property', 'field']);
      const objectName = object ? identifier(object) : null;
      const computed = children(callee).some(child => value(child, 'text') === '[');
      if (objectName && bindingMap.has(objectName)) {
        root = objectName;
        member = property ? identifier(property) : null;
        dynamic = computed || member === null || bindingMap.get(root).kind !== 'namespace';
      } else if (language === 'Rust' && object && ['call_expression', 'struct_expression'].includes(form(object))) {
        const inner = one(object, ['function', 'name']),
          innerSegments = inner ? pathSegments(inner) : null;
        if (innerSegments && innerSegments.length > 1) {
          append(references, {
            root: innerSegments[0],
            segments: innerSegments,
            kind: 'receiver-method-call',
            resolution: 'receiver-dispatch-unresolved',
            ...observation(record),
            options: []
          });
          gap('receiver-dispatch-unresolved', record, innerSegments[0]);
        }
        continue;
      } else continue;
    }
    if (!root) continue;
    const bound = bindingMap.get(root);
    const argumentFields = named(record.node, ['arguments']);
    const argumentNodes = argumentFields.length === 1 && form(argumentFields[0]) === 'arguments' ? children(argumentFields[0]).filter(node => !trivia(node) && !['(', ')', ','].includes(concrete(node))) : argumentFields;
    const observedOptions = kind === 'struct_expression' ? options(one(record.node, ['body'])) : argumentNodes.flatMap(options);
    if (observedOptions.length > 16) {
      observedOptions.length = 16;
      truncated = true;
    }
    if (!bound) {
      if (language === 'Rust' && segments && segments.length > 1) {
        append(references, {
          root,
          segments,
          kind: kind === 'struct_expression' ? 'struct-initializer' : 'qualified-call',
          resolution: 'unresolved-module-root',
          ...observation(record),
          options: observedOptions
        });
        gap(localModules.has(root) ? 'qualified-root-has-local-declaration' : 'qualified-root-resolution-required', record, root);
      }
      continue;
    }
    if (unsafe.has(root) || bound.kind === 'type') {
      gap(bound.kind === 'type' ? 'type-only-binding-not-runtime-call' : 'import-binding-scope-unresolved', record, root);
      continue;
    }
    if (dynamic || kind === 'optional_call_expression') {
      gap('dynamic-dispatch-not-followed', record, root);
      continue;
    }
    if (language === 'Rust' && segments?.length > 1) member = segments.slice(1).join('::');
    append(calls, {
      module: bound.module,
      binding: root,
      imported: bound.imported,
      ...(member ? {
        member
      } : {}),
      kind: kind === 'struct_expression' ? 'struct-initializer' : kind === 'new_expression' ? 'constructor' : 'call',
      resolution: 'static-import-binding-syntax',
      ...observation(record),
      options: observedOptions
    });
  }
  if (truncated) gap('observation-budget-exceeded', null);
  return {
    schema: 'observed-source-usage/v1',
    owner,
    sourceIdentity: sha256Hex(source),
    language,
    imports,
    calls,
    references,
    gaps,
    complete: gaps.length === 0 && !truncated,
    truncated,
    scope: 'static-source-observations-only; no execution or external API semantics',
    limits: {
      sourceBytes: 262144,
      cstNodes: 50000,
      depth: 256,
      observationsPerKind: 128,
      snippetBytes: 512
    }
  };
}
export const sourceCstViews = {
  children,
  value,
  form,
  hasRole,
  concrete,
  one,
  identifier,
  trivia
};
/** Mirrors `View::new` in rust/src/agentic_coding/observed_source_usage/view.rs. */
export function observedSourceView(source, document) {
  const bytes = new TextEncoder().encode(source),
    records = [],
    byNode = new Map();
  function visit(node, start, ancestors) {
    if (ancestors.length > 256 || records.length >= 50000) throw Error('SourceUsageCstBudgetExceeded');
    const record = {
      node,
      start,
      end: start,
      ancestors,
      index: records.length,
      subtreeEnd: 0
    };
    records.push(record);
    byNode.set(node, record);
    if (value(node, 'text') !== null) record.end = start + new TextEncoder().encode(concrete(node)).length;else {
      let cursor = start;
      for (const child of children(node)) cursor = visit(child, cursor, [...ancestors, node]);
      record.end = cursor;
    }
    record.subtreeEnd = records.length;
    return record.end;
  }
  if (visit(document, 0, []) !== bytes.length) throw Error('SourceUsageSpanReconstructionMismatch');
  return {
    records,
    byNode
  };
}
