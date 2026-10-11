// Repository workspace process, location and edit stages through an injected host.
// Mirrors rust/src/repository_workspace/{locate,edit,verify}.rs.
import { agenticMessage } from '../messages.mjs';
import { readText } from '../host.mjs';
import { findChildValue, parseLinoRoot } from '../write_lino.mjs';
import { resolveIn } from '../requirement_resolution.mjs';
import { insertQuotedMembersIntoNamedList } from '../structured_edit.mjs';
import { meaningsWithRole, wordIn, mentionsInLanguagesRaw } from './seed_meanings.mjs';

function commandArguments(line) {
  const argumentsList = [];
  let current = '';
  let quote = null;
  let started = false;
  for (const character of line) {
    if (quote !== null) {
      if (character === quote) quote = null;
      else current += character;
    } else if (character === '"' || character === "'") {
      quote = character;
      started = true;
    } else if (/\s/u.test(character)) {
      if (started) argumentsList.push(current);
      current = '';
      started = false;
    } else {
      current += character;
      started = true;
    }
  }
  if (started) argumentsList.push(current);
  return argumentsList;
}

function commandRow(program, argumentsList, document) {
  let tail = argumentsList;
  if (program === 'git') {
    while (tail.length > 1 && ['-C', '-c'].includes(tail[0])) tail = tail.slice(2);
  }
  return parseLinoRoot(document).children.find((node) => {
    if (findChildValue(node, 'record_type') !== 'allowed_command'
      || findChildValue(node, 'program') !== program
      || findChildValue(node, 'subcommand') !== tail[0]) return false;
    const shape = commandArguments(findChildValue(node, 'arguments').replace(/^\(|\)$/gu, ''));
    const actual = tail.slice(1);
    for (let position = 0; position < shape.length; position += 1) {
      const token = shape[position];
      if (token === '{rest}') return position === shape.length - 1;
      if (position >= actual.length || (!token.startsWith('{') && token !== actual[position])) return false;
    }
    return shape.length === actual.length;
  });
}

/** Mirrors `fn allows`: program, subcommand and argument shape, default-deny. */
export function allowsRepositoryCommand(program, argumentsList, document = readText('data/seed/repository-command-allowlist.lino')) {
  return commandRow(program, argumentsList, document) !== undefined;
}

/** Mirrors `fn run_named_tests`: all commands use the caller's bounded runner. */
export async function runRepositoryCommand(workspace, program, argumentsList, { probe = false } = {}) {
  const document = workspace.allowlist ?? readText('data/seed/repository-command-allowlist.lino');
  if (probe) {
    const prerequisite = await workspace.io.run(workspace.root, program, ['--version'], { deadline_seconds: 30, network: 'denied' });
    if (prerequisite.missing || prerequisite.exit_code === 127) throw new Error(`missing prerequisite ${program}: ${prerequisite.stderr ?? ''}`);
  }
  const row = commandRow(program, argumentsList, document);
  if (!row) throw new Error(agenticMessage('repository-command-disallowed', { program }));
  const deadline = Number(findChildValue(row, 'deadline_seconds')) || 300;
  const result = await workspace.io.run(workspace.root, program, argumentsList, { deadline_seconds: deadline, network: 'denied' });
  if (result.missing) throw new Error(`missing prerequisite ${program}: ${result.stderr ?? ''}`);
  if (result.timed_out) throw new Error(agenticMessage('repository-command-timeout', { elapsed: result.elapsed_seconds, deadline }));
  return result;
}

function words(text) {
  return text.toLowerCase().split(/[^\p{L}\p{N}]+/u).filter((word) => word.length > 1)
    .map((word) => word.length > 3 && word.endsWith('s') && !word.endsWith('ss') ? word.slice(0, -1) : word);
}

function censusCandidates(census, files, terms) {
  const candidates = [];
  for (const module of census.modules) {
    for (const symbol of module.symbols) {
      if (!['const', 'static'].includes(symbol.kind)) continue;
      const parts = new Set(words(symbol.name));
      const matched = [...parts].filter((part) => terms.has(part)).length;
      if (parts.size < 2 || matched < 2) continue;
      const source = files.find(([path]) => path === module.path)?.[1] ?? '';
      const declaration = source.split('\n').slice(Math.max(0, symbol.start_line - 1), symbol.end_line).join(' ');
      const value = declaration.includes('=') ? declaration.slice(declaration.indexOf('=') + 1).trim().replace(/;$/u, '').trim() : '';
      candidates.push({ matched, parts: parts.size,
        path_matches: new Set(words(module.path).filter((part) => terms.has(part))).size,
        owns_value: /[\[{}(0-9]/u.test(value) || /^["'&]/u.test(value),
        location: { relative_path: module.path, symbol: symbol.name, how: 'Census' } });
    }
  }
  const compare = (left, right) => left.matched * right.parts - right.matched * left.parts
    || left.matched - right.matched || left.path_matches - right.path_matches
    || Number(left.owns_value) - Number(right.owns_value);
  if (!candidates.length) return [];
  const best = candidates.reduce((left, right) => compare(left, right) >= 0 ? left : right);
  return candidates.filter((candidate) => compare(candidate, best) === 0).map((candidate) => candidate.location);
}

function locationCandidates(files, requirement, census, language) {
  const named = files.filter(([path]) => /[/.]/u.test(path) && requirement.toLowerCase().includes(path.toLowerCase()));
  if (named.length) return { named: true,
    locations: named.map(([relative_path]) => ({ relative_path, symbol: null, how: 'NamedPath' })) };
  const terms = new Set(words(requirement));
  const languages = language === 'en' ? ['en'] : [language, 'en'];
  for (const meaning of meaningsWithRole('coding_search_subject_kind')) {
    if (!mentionsInLanguagesRaw(meaning, requirement.toLowerCase(), languages)) continue;
    for (const term of words(meaning.slug + ' ' + (wordIn(meaning, 'en') ?? ''))) terms.add(term);
  }
  const modules = census.modules.some((module) => module.path.split('/').includes('src'))
    ? census.modules.filter((module) => module.path.split('/').includes('src')) : census.modules;
  const selected = { ...census, modules };
  const resolved = resolveIn(selected, [...terms].join(' '));
  if (resolved) return { named: false,
    locations: [{ relative_path: resolved.module_path, symbol: resolved.symbol, how: 'Census' }] };
  const partial = censusCandidates(selected, files, terms);
  if (partial.length) return { named: false, locations: partial };
  const candidates = [];
  for (const [relative_path, contents] of files) {
    for (const line of contents.split('\n')) {
      const separator = line.indexOf('=');
      if (separator < 0) continue;
      for (const symbol of line.slice(0, separator).split(/[^A-Za-z0-9_]+/u)) {
        const parts = words(symbol);
        if (!symbol.includes('_') || !/^[A-Z0-9_]+$/u.test(symbol)
          || parts.length < 2 || !parts.every((part) => terms.has(part))) continue;
        if (!candidates.some((candidate) => candidate.relative_path === relative_path && candidate.symbol === symbol)) {
          candidates.push({ relative_path, symbol, how: 'LiteralOccurrence' });
        }
      }
    }
  }
  return { named: false, locations: candidates };
}

/** Mirrors `fn locate_targets`: exact names then native partial-census evidence; ties stay open. */
export function locateRepositoryTargets(files, requirement, census = { modules: [] }, language = '') {
  const found = locationCandidates(files, requirement, census, language);
  return found.named || found.locations.length === 1 ? found.locations : [];
}

/** Mirrors `fn locate_ambiguity`: every equally evidenced candidate, without a path-order tiebreaker. */
export function locateRepositoryAmbiguity(files, requirement, census = { modules: [] }, language = '') {
  const found = locationCandidates(files, requirement, census, language);
  return found.locations.length > 1 ? found.locations : [];
}

/** Mirrors `fn derive_change`: structural list edits require a located symbol. */
export function deriveRepositoryChange(location, source, requirement) {
  if (location.symbol === null) return null;
  const result = insertQuotedMembersIntoNamedList(source, location.symbol, requirement);
  return result ? { relative_path: location.relative_path, contents: result[0] } : null;
}

/** Mirrors `RunCommand::argv`, preserving quoted empty arguments. */
export const repositoryCommandArguments = commandArguments;
