// Resolve a requirement's named declaration to the module that declares it
// (rust/src/agentic_coding/requirement_resolution.rs). The census comes from
// `crate/self_ast_census.mjs` `workspace()`.

import { moduleSymbol, modulesDeclaring, workspace } from './crate/self_ast_census.mjs';
import { isAlphanumeric, utf8Len } from './write_str.mjs';
import { hasHost, host, readText } from './host.mjs';
import { composeEditClauses } from './write_request.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { meaningFiles, parseLexiconText, words as meaningWords } from './crate/seed_meanings.mjs';

/** Mirrors `fn resolve_requirement_target`: `{module_path, symbol, kind}` or null. */
export function resolveRequirementTarget(requirement) {
  const census = workspace();
  const resolved = resolveIn(census, requirement);
  return resolved !== null ? resolved : resolveGeneratedSeedTarget(census, requirement);
}

/** Mirrors `fn resolve_in`. */
export function resolveIn(census, requirement) {
  const scoped = explicitModuleScope(census, requirement);
  if (scoped === null) return null;
  const resolved = resolveScoped(scoped, requirement);
  return resolved !== null || scoped === census ? resolved : resolveScopedLiteral(scoped, requirement);
}

/** Mirrors `fn resolve_scoped_literal`: unique scalar initializer from identity-checked source. */
function resolveScopedLiteral(census, requirement) {
  if (!hasHost() || typeof host().censusDocuments !== 'function') return null;
  const edit = composeEditClauses(requirement);
  const module = census.modules[0];
  if (!edit) return null;
  const document = host().censusDocuments().find((item) => item.sourceIdentity?.path === module.path);
  if (!document) return null;
  const source = readText('rust/' + module.path);
  if (stableId('source_module', source) !== document.sourceIdentity.content_id
    || utf8Len(source) !== document.sourceIdentity.byte_len) return null;
  const candidates = module.symbols.filter((symbol) => {
    if (symbol.kind !== 'const' && symbol.kind !== 'static') return false;
    const declaration = source.split('\n').slice(symbol.start_line - 1, symbol.end_line).join('\n');
    const separator = declaration.indexOf('=');
    if (separator < 0) return false;
    const initializer = declaration.slice(separator + 1).trim();
    const segments = quotedSegmentSpans(initializer);
    return initializer.startsWith('"') && segments.length === 1 && segments[0].start === 0
      && initializer.slice(segments[0].end).trim() === ';' && segments[0].text === edit.edit[1];
  });
  return candidates.length === 1 ? target(module, candidates[0]) : null;
}

/** Mirrors `fn explicit_module_scope`: bind exact observed paths before declaration ranking. */
function explicitModuleScope(census, requirement) {
  const references = requirement.split(/[^\p{Alphabetic}\p{N}_./-]+/u)
    .map((token) => token.replace(/\.+$/u, ''))
    .filter((token) => token.includes('/') && token.endsWith('.rs'));
  if (!references.length) return census;
  const selected = new Map();
  for (const reference of references) {
    const relative = reference.startsWith('./') ? reference.slice(2) : reference;
    const path = relative.startsWith('rust/') ? relative.slice(5) : relative;
    if (path.split('/').some((part) => part === '.' || part === '..')) return null;
    const module = census.modules.find((candidate) => candidate.path === path);
    if (!module) return null;
    selected.set(path, module);
  }
  return selected.size === 1 ? { modules: [...selected.values()] } : null;
}

/** Mirrors `fn resolve_scoped`: rank only declarations within the observed scope. */
function resolveScoped(census, requirement) {
  const tokens = tokensOf(requirement);
  for (const token of tokens) {
    if (!looksLikeDeclaredName(token)) continue;
    const named = modulesDeclaring(census, token)
      .map((module) => [module, moduleSymbol(module, token)])
      .filter(([, symbol]) => symbol !== null);
    if (named.length) return unique(named, tokens);
  }
  const words = tokens.map((token) => singular(token.toLowerCase()));
  let best = [];
  let bestLength = 0;
  for (const module of census.modules) {
    for (const symbol of module.symbols) {
      if (symbol.kind !== 'const' && symbol.kind !== 'static') continue;
      const parts = symbol.name.split('_').filter((part) => part !== '').map((part) => singular(part.toLowerCase()));
      if (parts.length < 2 || !parts.every((part) => words.includes(part))) continue;
      if (parts.length > bestLength) {
        bestLength = parts.length;
        best = [[module, symbol]];
      } else if (parts.length === bestLength) best.push([module, symbol]);
    }
  }
  return best.length ? unique(best, tokens) : hasHost()
    ? resolveSeedTarget(census, requirement, meaningFiles().map((path) => [path, readText(path)])) : null;
}

/** Bind a concrete canonical surface and role to its actually declared registry constant.
 * The constant spelling follows the seed registry generator; no seed or prompt is privileged.
 * Equal evidence for different targets is unresolved. */
export function resolveSeedTarget(census, requirement, sources) {
  return resolveSeedTargetWithOwners(requirement, sources, (_path, name) =>
    modulesDeclaring(census, name).map(module => [module, moduleSymbol(module, name)])
      .filter(([, symbol]) => symbol.kind === 'const' || symbol.kind === 'static')
      .map(([module, symbol]) => target(module, symbol)));
}

function resolveSeedTargetWithOwners(requirement, sources, declaredOwners) {
  const tokens = tokensOf(requirement).map((token) => token.toLowerCase());
  const roleWords = tokens.map(singular);
  let bestLength = 0;
  const best = new Map();
  for (const [path, source] of sources) {
    const stem = path.split('/').at(-1)?.replace(/\.lino$/u, '');
    if (!path.endsWith('.lino') || !/^[A-Za-z0-9_-]+$/u.test(stem)) continue;
    const name = stem.toUpperCase().replaceAll('-', '_') + '_LINO';
    const candidates = declaredOwners(path, name);
    if (!candidates.length) continue;
    for (const meaning of parseLexiconText(source)) {
      for (const surface of meaningWords(meaning)) {
        if (surface.includes('…')) continue;
        const parts = tokensOf(surface).map((token) => token.toLowerCase());
        // A role must have independent request evidence outside the matched surface.
        if (!parts.length || !tokens.some((_, at) =>
          parts.every((part, offset) => tokens[at + offset] === part) &&
          meaning.roles.some((role) => role.split(/[_-]/u).some((part) =>
            roleWords.some((word, index) => (index < at || index >= at + parts.length) && word === singular(part.toLowerCase())))))) continue;
        if (parts.length > bestLength) { bestLength = parts.length; best.clear(); }
        if (parts.length === bestLength) for (const candidate of candidates) best.set(candidate.module_path + ':' + candidate.symbol, candidate);
      }
    }
  }
  return best.size === 1 ? best.values().next().value : null;
}

/** A current generated declaration lookup stays separate from the committed census. */
function resolveGeneratedSeedTarget(census, requirement) {
  if (!hasHost() || typeof host().generatedSeedOwnerDeclarations !== 'function'
    || explicitModuleScope(census, requirement) !== census) return null;
  const tokens = tokensOf(requirement);
  const words = tokens.map(token => singular(token.toLowerCase()));
  // An unresolved existing declaration is ambiguity, not permission to guess another owner.
  if (tokens.some(token => looksLikeDeclaredName(token) && modulesDeclaring(census, token).length)) return null;
  if (census.modules.some(module => module.symbols.some(symbol => {
    if (symbol.kind !== 'const' && symbol.kind !== 'static') return false;
    const parts = symbol.name.split('_').filter(Boolean).map(part => singular(part.toLowerCase()));
    return parts.length >= 2 && parts.every(part => words.includes(part));
  }))) return null;
  const owners = host().generatedSeedOwnerDeclarations();
  const declarationTarget = owner => ({module_path: owner.module_path, symbol: owner.symbol, kind: owner.kind});
  const uniqueOwners = candidates => {
    const targets = [...new Map(candidates.map(owner => [owner.module_path + ':' + owner.symbol, declarationTarget(owner)])).values()];
    if (targets.length === 1) return targets[0];
    const scores = targets.map(candidate => [pathWords(candidate.module_path).filter(word => words.includes(word)).length, candidate]);
    const best = Math.max(...scores.map(([score]) => score));
    const winners = scores.filter(([score]) => score === best);
    return winners.length === 1 ? winners[0][1] : null;
  };
  for (const token of tokens) {
    if (!looksLikeDeclaredName(token)) continue;
    const candidates = owners.filter(owner => owner.symbol === token);
    if (candidates.length) return uniqueOwners(candidates);
  }
  let bestLength = 0, best = [];
  for (const owner of owners) {
    const parts = owner.symbol.split('_').filter(Boolean).map(part => singular(part.toLowerCase()));
    if (parts.length < 2 || !parts.every(part => words.includes(part))) continue;
    if (parts.length > bestLength) {bestLength = parts.length; best = [owner];}
    else if (parts.length === bestLength) best.push(owner);
  }
  if (best.length) return uniqueOwners(best);
  return resolveSeedTargetWithOwners(requirement, meaningFiles().map(path => [path, readText(path)]),
    (path, name) => owners.filter(owner => owner.seed_path === path && owner.symbol === name).map(declarationTarget));
}

function unique(candidates, tokens) {
  if (candidates.length === 1) return target(...candidates[0]);
  const words = tokens.map((token) => singular(token.toLowerCase()));
  const scored = candidates.map(([module, symbol]) => [
    pathWords(module.path).filter((word) => words.includes(word)).length, module, symbol,
  ]);
  if (!scored.length) return null;
  const top = Math.max(...scored.map(([score]) => score));
  const winners = scored.filter(([score]) => score === top);
  return winners.length === 1 ? target(winners[0][1], winners[0][2]) : null;
}

function target(module, symbol) {
  return { module_path: module.path, symbol: symbol.name, kind: symbol.kind };
}

function pathWords(path) {
  return path.split(/[/_.]/u)
    .filter((part) => part !== '' && part !== 'src' && part !== 'rs' && part !== 'mod')
    .map((part) => singular(part.toLowerCase()));
}

function tokensOf(text) {
  const out = [];
  let current = '';
  for (const character of text) {
    if (isAlphanumeric(character) || character === '_') current += character;
    else {
      if (current) out.push(current);
      current = '';
    }
  }
  if (current) out.push(current);
  return out;
}

function looksLikeDeclaredName(token) {
  return token.length >= 2 && /[A-Z]/.test(token) && /^[A-Z0-9_]+$/.test(token);
}

function singular(word) {
  return new TextEncoder().encode(word).length > 3 && word.endsWith('s') && !word.endsWith('ss') ? word.slice(0, -1) : word;
}
