import fs from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { cargoTestTargets } from './native-test-module-graph.mjs';
import * as runtime from '../../js/vendor/tree-sitter/web-tree-sitter.mjs';

const identifier = value => value?.replace(/^r#/, '');
function imports(node) {
  if (!node) throw new Error('Missing Rust import operand');
  if (node.type === 'identifier' || node.type === 'crate' || node.type === 'self' || node.type === 'super') return [[identifier(node.text)]];
  if (node.type === 'scoped_identifier') return imports(node.childForFieldName('path')).map(parts => [...parts, identifier(node.childForFieldName('name').text)]);
  if (node.type === 'scoped_use_list') {
    const prefix = imports(node.childForFieldName('path'));
    return prefix.flatMap(parts => imports(node.childForFieldName('list')).map(rest => [...parts, ...rest]));
  }
  if (node.type === 'use_list') return node.namedChildren.flatMap(imports);
  if (node.type === 'use_as_clause') return imports(node.childForFieldName('path') ?? node.namedChildren[0]);
  if (node.type === 'use_wildcard') return node.namedChildren.length ? imports(node.namedChildren[0]).map(parts => [...parts, '*']) : [['*']];
  throw new Error('Missing supported Rust import projection: ' + node.type);
}

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
  function entries(node, relative, prefix, visit) {
    let attributes = [];
    for (const child of node.namedChildren) {
      if (child.type === 'attribute_item') { attributes.push(child); continue; }
      if (child.type === 'line_comment' || child.type === 'block_comment') continue;
      visit(child, attributes, relative, prefix);
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
    function library(relative, prefix) {
      if (relative === changedPath) { owned = prefix; ownedFile = relative; return; }
      entries(tree(relative).node, relative, prefix, (node, attributes) => {
        if (node.type !== 'mod_item' || owned) return;
        const name = identifier(node.childForFieldName('name').text);
        const body = node.childForFieldName('body');
        if (body) return;
        const child = moduleFile(relative, name, attributes);
        const folder = path.posix.basename(child) === 'mod.rs' ? path.posix.dirname(child) : child.slice(0, -3);
        if (child === changedPath || changedPath.startsWith(folder + '/')) library(child, [...prefix, name]);
      });
    }
    library(libraryRoot, []);
    if (!owned) throw new Error('Missing registered changed Rust module');
    const selected = new Set(), selectedByTarget = new Map(), visited = new Set(), active = new Set();
    let currentTarget;
    function unit(relative, prefix, inherited = false) {
      const key = currentTarget + '\0' + relative + '\0' + prefix.join('::');
      if (active.has(relative)) throw new Error('Cyclic registered test source');
      if (visited.has(key)) throw new Error('Cyclic registered test module'); visited.add(key); active.add(relative);
      try { scan(tree(relative).node, relative, prefix, inherited); } finally { active.delete(relative); }
    }
    function scan(body, relative, prefix, inherited) {
      let direct = inherited;
      for (const node of body.namedChildren.filter(node => node.type === 'use_declaration')) {
        for (const route of imports(node.childForFieldName('argument'))) {
          if (route[0] === crateName && owned.every((part, index) => route[index + 1] === part)) direct = true;
        }
      }
      entries(body, relative, prefix, (node, attributes) => {
        if (node.type === 'function_item' && direct && attributes.some(attribute => attribute.namedChildren[0]?.namedChildren[0]?.text === 'test')) {
          const name = [...prefix, identifier(node.childForFieldName('name').text)].join('::');
          selectedByTarget.get(currentTarget).add(name);
          if (currentTarget === 'unit') selected.add(name);
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
        if (node.type === 'function_item' && attributes.some(attribute => attribute.namedChildren[0]?.namedChildren[0]?.text === 'test'))
          librarySelected.add([...prefix, identifier(node.childForFieldName('name').text)].join('::'));
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
    return {changedPath, crateName, libraryRoot, sourceModule: owned, filters: [...selected].sort(), jobs, sourceGuards,
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
