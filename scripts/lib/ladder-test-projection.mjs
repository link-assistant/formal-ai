import fs from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { cargoTestTargets } from './native-test-module-graph.mjs';
import * as runtime from '../../js/vendor/tree-sitter/web-tree-sitter.mjs';

const identifier = value => value?.replace(/^r#/, '');
function testDisposition(attributes) {
  const names = attributes.map(item => item.namedChildren[0]?.namedChildren[0]?.text);
  return !names.includes('test') ? null : names.includes('ignore') ? 'ignored' : 'runnable';
}
function importBindings(node) {
  if (!node) throw new Error('Missing Rust import operand');
  if (['identifier', 'crate', 'self', 'super'].includes(node.type)) {
    const name = identifier(node.text); return [{route: [name], local: name}];
  }
  if (node.type === 'scoped_identifier') {
    const name = identifier(node.childForFieldName('name').text);
    const parent = node.childForFieldName('path');
    return parent ? importBindings(parent).map(item => ({route: [...item.route, name], local: name}))
      : [{route: [name], local: name}];
  }
  if (node.type === 'scoped_use_list') return importBindings(node.childForFieldName('path')).flatMap(prefix =>
    importBindings(node.childForFieldName('list')).map(item => item.route.length === 1 && item.route[0] === 'self'
      ? {route: prefix.route, local: item.local === 'self' ? prefix.local : item.local}
      : {route: [...prefix.route, ...item.route], local: item.local}));
  if (node.type === 'use_list') return node.namedChildren.flatMap(importBindings);
  if (node.type === 'use_as_clause') {
    const alias = node.childForFieldName('alias');
    if (!alias) throw new Error('Missing Rust import alias');
    return importBindings(node.childForFieldName('path')).map(item => ({...item, local: identifier(alias.text)}));
  }
  if (node.type === 'use_wildcard') return node.namedChildren.length
    ? importBindings(node.namedChildren[0]).map(item => ({route: [...item.route, '*'], local: '*'}))
    : [{route: ['*'], local: '*'}];
  throw new Error('Missing supported Rust import projection: ' + node.type);
}
const imports = node => importBindings(node).map(item => item.route);

/** Source-derived candidate filters; actual harness execution remains mandatory. */
export async function registeredRustTestFilters(root, changedPath) {
  root = path.resolve(root);
  const rootStatus = fs.lstatSync(root);
  if (!rootStatus.isDirectory() || rootStatus.isSymbolicLink()) throw new Error('Regular workspace root required');
  if (typeof changedPath !== 'string' || !changedPath.startsWith('rust/src/') || !changedPath.endsWith('.rs')
    || changedPath.split('/').some(part => !part || part === '.' || part === '..')) throw new Error('Missing source-owned Rust module projection');
  await runtime.Parser.init();
  const language = await runtime.Language.load(new URL('../../js/vendor/tree-sitter/tree-sitter-rust.wasm', import.meta.url));
  const parser = new runtime.Parser(); parser.setLanguage(language);
  const trees = [], sources = new Map();
  let targetRoots;
  function regular(relative) {
    const full = path.resolve(root, relative);
    if (!full.startsWith(root + path.sep)) throw new Error('Rust source escapes root');
    for (let current = full; current !== root; current = path.dirname(current)) {
      const status = fs.lstatSync(current);
      if (status.isSymbolicLink()) throw new Error('Rust source symlink refused');
      if (current === full && !status.isFile()) throw new Error('Regular Rust source required');
    }
    return full;
  }
  function tree(relative) {
    if (sources.has(relative)) return sources.get(relative);
    const source = fs.readFileSync(regular(relative), 'utf8');
    const parsed = parser.parse(source); trees.push(parsed);
    if (parsed.rootNode.hasError) throw new Error('Rust source CST error: ' + relative);
    const item = {source, node: parsed.rootNode}; sources.set(relative, item); return item;
  }
  function moduleFile(relative, name, attributes) {
    for (const item of attributes) {
      const attribute = item.namedChildren[0];
      if (attribute?.namedChildren[0]?.text !== 'cfg_attr') continue;
      const pending = [attribute];
      while (pending.length) {
        const node = pending.pop();
        if (node.type === 'identifier' && node.text === 'path')
          throw new Error('Missing conditional Rust module path projection');
        pending.push(...node.namedChildren);
      }
    }
    const overrides = attributes.filter(attribute => attribute.namedChildren[0]?.namedChildren[0]?.text === 'path');
    if (overrides.length > 1) throw new Error('Duplicate Rust module path');
    if (overrides.length) {
      const value = overrides[0].namedChildren[0].childForFieldName('value');
      if (value?.type !== 'string_literal') throw new Error('Missing literal Rust module path');
      const selected = path.posix.normalize(path.posix.join(path.posix.dirname(relative), JSON.parse(value.text)));
      regular(selected); return selected;
    }
    const parent = (targetRoots.has(relative) || path.posix.basename(relative) === 'mod.rs')
      ? path.posix.dirname(relative) : relative.slice(0, -3);
    const candidates = [parent + '/' + name + '.rs', parent + '/' + name + '/mod.rs'].filter(candidate => fs.existsSync(path.join(root, candidate)));
    if (candidates.length !== 1) throw new Error('Missing or ambiguous registered Rust module: ' + name);
    regular(candidates[0]); return candidates[0];
  }
  function entries(node, relative, prefix, visit, included = new Set()) {
    let attributes = [];
    const shadowsInclude = node.namedChildren.some(child => child.type === 'macro_definition'
      && child.childForFieldName('name')?.text === 'include');
    for (const child of node.namedChildren) {
      if (child.type === 'attribute_item') { attributes.push(child); continue; }
      if (child.type === 'line_comment' || child.type === 'block_comment') continue;
      const expression = child.type === 'expression_statement' && child.namedChildren.length === 1
        ? child.namedChildren[0] : child;
      if (expression.type === 'macro_invocation' && expression.childForFieldName('macro')?.text === 'include') {
        if (shadowsInclude || attributes.some(item => ['cfg', 'cfg_attr'].includes(item.namedChildren[0]?.namedChildren[0]?.text)))
          throw new Error('Missing unambiguous unconditional include projection: ' + relative);
        const argumentsNode = expression.namedChildren.find(item => item.type === 'token_tree');
        const operands = argumentsNode?.namedChildren.filter(item => !['line_comment', 'block_comment'].includes(item.type));
        if (operands?.length !== 1 || operands[0].type !== 'string_literal')
          throw new Error('Missing literal Rust include projection: ' + relative);
        const name = JSON.parse(operands[0].text);
        if (path.posix.isAbsolute(name) || name.includes('\\')) throw new Error('Invalid Rust include path');
        const selected = path.posix.normalize(path.posix.join(path.posix.dirname(relative), name));
        regular(selected);
        if (included.has(selected) || selected === relative) throw new Error('Cyclic registered Rust include');
        targetRoots.add(selected);
        entries(tree(selected).node, selected, prefix, visit, new Set([...included, relative]));
      } else visit(child, attributes, relative, prefix);
      attributes = [];
    }
  }
  try {
    const cargoBefore = fs.readFileSync(regular('rust/Cargo.toml'), 'utf8');
    const digest = bytes => createHash('sha256').update(bytes).digest('hex');
    function section(name) {
      const expression = new RegExp('^\\[' + name + '\\][ \t]*(?:#[^\r\n]*)?$', 'mu');
      const parts = cargoBefore.split(expression);
      if (parts.length > 2) throw new Error('Duplicate Cargo target section');
      return parts.length === 2 ? parts[1].split(/^\[/mu)[0] : null;
    }
    function literal(body, field) {
      if (body === null) return null;
      const lines = body.split('\n').filter(line => new RegExp('^' + field + '[ \t]*=').test(line));
      if (lines.length > 1) throw new Error('Duplicate Cargo target field');
      if (!lines.length) return null;
      const match = new RegExp('^' + field + '[ \t]*=[ \t]*"([^"\\\r\n]+)"[ \t]*(?:#[^\r\n]*)?$').exec(lines[0]);
      if (!match) throw new Error('Missing literal Cargo target field');
      return match[1];
    }
    const librarySection = section('lib');
    const crateName = literal(librarySection, 'name') ?? literal(section('package'), 'name')?.replaceAll('-', '_');
    if (!crateName || !/^[A-Za-z_][A-Za-z_0-9]*$/.test(crateName)) throw new Error('Missing source-owned Cargo crate name');
    const libraryRoot = path.posix.normalize(path.posix.join('rust', literal(librarySection, 'path') ?? 'src/lib.rs'));
    const targets = cargoTestTargets(root);
    if (targets.some(target => target.cargoSHA !== digest(cargoBefore))) throw new Error('Cargo source changed during target projection');
    targetRoots = new Set([libraryRoot, ...targets.map(target => path.relative(root, target.path).split(path.sep).join('/'))]);
    regular(changedPath);
    let owned = null, ownedFile = null;
    const reexports = [];
    function library(relative, prefix) {
      if (relative === changedPath) { owned = prefix; ownedFile = relative; return; }
      entries(tree(relative).node, relative, prefix, (node, attributes, declarationFile) => {
        if (node.type === 'use_declaration' && node.namedChildren.some(item => item.type === 'visibility_modifier' && item.text === 'pub')) {
          for (const binding of importBindings(node.childForFieldName('argument'))) {
            let route = [...binding.route], owner = [...prefix];
            if (route[0] === 'crate') { owner = []; route.shift(); }
            else if (route[0] === 'self') route.shift();
            else while (route[0] === 'super') { if (!owner.length) throw new Error('Rust reexport escapes crate'); owner.pop(); route.shift(); }
            if (binding.local !== '*') reexports.push({from: [...owner, ...route], to: [...prefix, binding.local]});
          }
        }
        if (node.type !== 'mod_item' || owned) return;
        const name = identifier(node.childForFieldName('name').text);
        const body = node.childForFieldName('body');
        if (body) return;
        const child = moduleFile(declarationFile, name, attributes);
        const folder = path.posix.basename(child) === 'mod.rs' ? path.posix.dirname(child) : child.slice(0, -3);
        if (child === changedPath || changedPath.startsWith(folder + '/')) library(child, [...prefix, name]);
      });
    }
    library(libraryRoot, []);
    if (!owned) throw new Error('Missing registered changed Rust module: ' + changedPath + ' in ' + libraryRoot);
    const ownershipRoutes = [owned], routeKeys = new Set([JSON.stringify(owned)]);
    for (let changed = true; changed;) {
      changed = false;
      for (const edge of reexports) {
        if (!ownershipRoutes.some(route => route.length <= edge.from.length && route.every((part, index) => edge.from[index] === part))) continue;
        const key = JSON.stringify(edge.to);
        if (!routeKeys.has(key)) { routeKeys.add(key); ownershipRoutes.push(edge.to); changed = true; }
      }
    }
    const selected = new Set(), selectedByTarget = new Map(), visited = new Set(), active = new Set(), ignoredJobs = [];
    let currentTarget;
    const unresolvedTestScopes = [];
    function unit(relative, prefix, inherited = false) {
      const key = currentTarget + '\0' + relative + '\0' + prefix.join('::');
      if (active.has(relative)) throw new Error('Cyclic registered test source');
      if (visited.has(key)) throw new Error('Cyclic registered test module'); visited.add(key); active.add(relative);
      try { scan(tree(relative).node, relative, prefix, inherited); } finally { active.delete(relative); }
    }
    function scan(body, relative, prefix, inherited) {
      // A generated include cannot supply a source-observed witness. Refuse this
      // complete scope as a candidate while retaining other registered scopes.
      const opaqueIncludes = body.namedChildren.map(child => child.type === 'expression_statement'
        && child.namedChildren.length === 1 ? child.namedChildren[0] : child).filter(child => {
        if (child.type !== 'macro_invocation' || child.childForFieldName('macro')?.text !== 'include') return false;
        const operands = child.namedChildren.find(item => item.type === 'token_tree')?.namedChildren
          .filter(item => !['line_comment', 'block_comment'].includes(item.type));
        return operands?.length !== 1 || operands[0].type !== 'string_literal';
      });
      if (opaqueIncludes.length) {
        unresolvedTestScopes.push({target: currentTarget, source: relative, prefix,
          reason: 'GeneratedOrComputedIncludeCannotSupplyObservedTestWitness',
          expressions: opaqueIncludes.map(item => item.text)});
        return;
      }
      let direct = inherited;
      for (const node of body.namedChildren.filter(node => node.type === 'use_declaration')) {
        for (const route of imports(node.childForFieldName('argument'))) {
          if (route[0] === crateName && ownershipRoutes.some(owner => owner.every((part, index) => route[index + 1] === part))) direct = true;
        }
      }
      entries(body, relative, prefix, (node, attributes) => {
        if (node.type === 'function_item' && direct && testDisposition(attributes)) {
          const name = [...prefix, identifier(node.childForFieldName('name').text)].join('::');
          if (testDisposition(attributes) === 'ignored') ignoredJobs.push({target: currentTarget, filter: name, source: relative});
          else {
            selectedByTarget.get(currentTarget).add(name);
            if (currentTarget === 'unit') selected.add(name);
          }
        }
        if (node.type === 'mod_item') {
          const name = identifier(node.childForFieldName('name').text), childPrefix = [...prefix, name];
          const nested = node.childForFieldName('body');
          if (nested) scan(nested, relative, childPrefix, direct);
          else unit(moduleFile(relative, name, attributes), childPrefix, direct);
        }
      });
    }
    for (const target of targets) {
      if (target.name === 'lib') throw new Error('Ambiguous library test target');
      currentTarget = target.name; selectedByTarget.set(currentTarget, new Set());
      unit(path.relative(root, target.path).split(path.sep).join('/'), []);
    }
    const librarySelected = new Set(), libraryVisited = new Set(), libraryActive = new Set([ownedFile]);
    function libraryTests(body, relative, prefix) {
      const key = relative + '\0' + prefix.join('::');
      if (libraryVisited.has(key)) throw new Error('Cyclic library test module');
      libraryVisited.add(key);
      entries(body, relative, prefix, (node, attributes) => {
        if (node.type === 'function_item' && testDisposition(attributes)) {
          const filter = [...prefix, identifier(node.childForFieldName('name').text)].join('::');
          if (testDisposition(attributes) === 'ignored') ignoredJobs.push({target: 'lib', filter, source: relative});
          else librarySelected.add(filter);
        }
        if (node.type === 'mod_item') {
          const name = identifier(node.childForFieldName('name').text), childPrefix = [...prefix, name];
          const nested = node.childForFieldName('body');
          if (nested) libraryTests(nested, relative, childPrefix);
          else {
            const child = moduleFile(relative, name, attributes);
            if (libraryActive.has(child)) throw new Error('Cyclic registered library source');
            libraryActive.add(child);
            try { libraryTests(tree(child).node, child, childPrefix); } finally { libraryActive.delete(child); }
          }
        }
      });
    }
    libraryTests(tree(ownedFile).node, ownedFile, owned);
    const jobs = [
      ...[...selectedByTarget].flatMap(([target, cases]) => [...cases].sort().map(filter => ({target, filter}))),
      ...[...librarySelected].sort().map(filter => ({target: 'lib', filter})),
    ];
    if (!jobs.length) throw new Error('Missing registered test projection for source module');
    const sourceGuards = [...sources].map(([relative, observed]) => {
      if (fs.readFileSync(regular(relative), 'utf8') !== observed.source) throw new Error('Rust source changed during projection');
      return {path: relative, sha256: digest(observed.source)};
    });
    if (fs.readFileSync(regular('rust/Cargo.toml'), 'utf8') !== cargoBefore) throw new Error('Cargo source changed during projection');
    return {changedPath, crateName, libraryRoot, sourceModule: owned, ownershipRoutes, filters: [...selected].sort(), jobs, ignoredJobs, unresolvedTestScopes, sourceGuards,
      declaredTargets: targets.map(target => ({name: target.name, path: path.relative(root, target.path), cargoSHA: target.cargoSHA})),
      namespaceRace: 'Unknown',
      registeredSources: [...sources.keys()].sort(), runtimeSelection: 'Pending'};
  } finally { for (const parsed of trees) parsed.delete(); parser.delete(); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (process.argv.length !== 4) throw new Error('Expected workspace and changed Rust source');
    const projection = await registeredRustTestFilters(process.argv[2], process.argv[3]);
    for (const job of projection.jobs) console.log(job.target + "\t" + job.filter);
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
