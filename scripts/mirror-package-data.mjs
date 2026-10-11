#!/usr/bin/env node
// JavaScript twin of mirror-package-data.rs. Package-demand mirroring only;
// this filesystem operation supplies no Read/Run/import/namespace authority.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isWhitespace, trim } from '../js/agentic/crate/rust_str.mjs';

export function parseIncludeSite(argument) {
  if (argument.includes('file!()')) return { literal: null, selfRead: true };
  const value = trim(argument);
  if (value.startsWith('"') && value.endsWith('"') && !value.slice(1, -1).includes('"')) {
    return { literal: value.slice(1, -1), selfRead: false };
  }
  throw new Error('an include_str! argument that is neither a plain literal nor the file!() self-read idiom: `'
    + value + '`; rewrite it to a plain relative literal pointing into rust/embedded/');
}

export function includeSites(text) {
  const needle = 'include_str!';
  const sites = [];
  let cursor = 0;
  while (true) {
    const site = text.indexOf(needle, cursor);
    if (site < 0) break;
    const line = text.slice(0, site).split('\n').length;
    const after = text.slice(site + needle.length);
    let open = 0;
    while (open < after.length && isWhitespace(String.fromCodePoint(after.codePointAt(open)))) {
      open += String.fromCodePoint(after.codePointAt(open)).length;
    }
    if (after[open] !== '(') { cursor = site + needle.length; continue; }
    let depth = 0;
    let end = null;
    for (let offset = open; offset < after.length; offset += 1) {
      if (after[offset] === '(') depth += 1;
      if (after[offset] === ')') {
        depth -= 1;
        if (depth === 0) { end = offset; break; }
      }
    }
    if (end === null) throw new Error(`line ${line}: unterminated include_str!`);
    let parsed;
    try { parsed = parseIncludeSite(after.slice(open + 1, end)); }
    catch (error) { throw new Error(`line ${line}: ${error.message}`); }
    sites.push([line, parsed]);
    cursor = site + needle.length + end + 1;
  }
  return sites;
}

function relativeInside(target, root) {
  const relative = path.relative(path.resolve(root), path.resolve(target));
  return relative === '' || (!path.isAbsolute(relative) && relative !== '..'
    && !relative.startsWith('..' + path.sep)) ? relative : null;
}

export function classify(target, packageRoot, repoRoot) {
  const normalized = path.resolve(target);
  const rest = relativeInside(normalized, packageRoot);
  if (rest !== null) {
    const parts = rest.split(path.sep);
    if (parts[0] === 'embedded' && parts.length > 1) {
      return { kind: 'mirror', repoRelative: parts.slice(1).join(path.sep) };
    }
    return { kind: 'in-package' };
  }
  const shown = relativeInside(normalized, repoRoot) ?? normalized;
  throw new Error('resolves outside the package root: `' + shown
    + '`; rust/src may only read files inside rust/, through the rust/embedded/ mirror');
}

function isDirectory(file) {
  try { return fs.statSync(file).isDirectory(); } catch { return false; }
}
const readUTF8 = (file) => new TextDecoder('utf-8', { fatal: true, ignoreBOM: true })
  .decode(fs.readFileSync(file));
const byteOrder = (left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right));

export function collectDemands(srcRoot, packageRoot, repoRoot) {
  const demands = new Map();
  const stack = [srcRoot];
  while (stack.length > 0) {
    const directory = stack.pop();
    let entries;
    try { entries = fs.readdirSync(directory); }
    catch (error) { throw new Error(`${directory} readable: ${error.message}`); }
    for (const name of entries) {
      const file = path.join(directory, name);
      if (isDirectory(file)) { stack.push(file); continue; }
      if (path.extname(file) !== '.rs') continue;
      let text;
      try { text = readUTF8(file); }
      catch (error) { throw new Error(`${file} readable: ${error.message}`); }
      const display = relativeInside(file, repoRoot) ?? file;
      let sites;
      try { sites = includeSites(text); }
      catch (error) { throw new Error(`${display}: ${error.message}`); }
      for (const [line, site] of sites) {
        if (site.selfRead || site.literal === null) continue;
        const demand = classify(path.resolve(path.dirname(file), site.literal), packageRoot, repoRoot);
        if (demand.kind !== 'mirror') continue;
        const previous = demands.get(demand.repoRelative) ?? [];
        previous.push(`${display}:${line}`);
        demands.set(demand.repoRelative, previous);
      }
    }
  }
  return new Map([...demands].sort(([left], [right]) => byteOrder(left, right)));
}

function walkFiles(root, files) {
  let entries;
  try { entries = fs.readdirSync(root); } catch { return; }
  for (const name of entries) {
    const file = path.join(root, name);
    if (isDirectory(file)) walkFiles(file, files);
    else files.push(file);
  }
}

export function run(args, repoRoot = process.cwd()) {
  if (args.length !== 1 || !['--write', '--check'].includes(args[0])) {
    throw new Error('usage: mirror-package-data.mjs --write|--check');
  }
  const write = args[0] === '--write';
  const packageRoot = path.join(repoRoot, 'rust');
  const mirrorRoot = path.join(packageRoot, 'embedded');
  const demands = collectDemands(path.join(packageRoot, 'src'), packageRoot, repoRoot);
  const problems = [];
  let copied = 0;
  let bytes = 0;
  for (const [relative, sites] of demands) {
    const source = path.join(repoRoot, relative);
    const mirror = path.join(mirrorRoot, relative);
    let content;
    try { content = fs.readFileSync(source); }
    catch (error) {
      problems.push(`${source} is mirrored by ${sites.join(', ')} but unreadable: ${error.message}`);
      continue;
    }
    let equal = false;
    try { equal = fs.readFileSync(mirror).equals(content); } catch {}
    if (equal) continue;
    if (write) {
      fs.mkdirSync(path.dirname(mirror), { recursive: true });
      fs.writeFileSync(mirror, content);
      copied += 1;
      bytes += content.length;
    } else {
      problems.push(`${path.relative(repoRoot, mirror)} is stale (demanded by ${sites.join(', ')}); run `
        + '`node scripts/mirror-package-data.mjs --write`');
    }
  }
  const files = [];
  walkFiles(mirrorRoot, files);
  for (const file of files) {
    if (demands.has(path.relative(mirrorRoot, file))) continue;
    if (write) fs.unlinkSync(file);
    else problems.push(`${path.relative(repoRoot, file)} mirrors a file no rust/src include_str! names anymore; run `
      + '`node scripts/mirror-package-data.mjs --write`');
  }
  if (problems.length > 0) throw new Error(problems.join('\n'));
  return `package-data mirror: ${demands.size} files demanded, ${copied} refreshed (${bytes} bytes), all byte-equal`;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { console.log(run(process.argv.slice(2))); }
  catch (error) { console.error(`package-data mirror failed:\n${error.message}`); process.exitCode = 1; }
}
