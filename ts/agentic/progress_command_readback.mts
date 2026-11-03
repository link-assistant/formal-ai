// Exact commanded readback custody. The caller constructs the owned read-only command.
import {Capability} from './capability.mjs';
import {commandArgument} from './tool_result.mjs';
import {argumentValuesEqual, canonicalCreation} from './qualified_tool_observation.mjs';
import {isWorkspaceCreationTool} from './capability_router.mjs';
import {agenticMessage} from './messages.mjs';
import {quoteShellWord} from './portable/string_contracts.mjs';
import {ownedDeclaredCreateFrame} from './planner/owned_goals.mjs';
import {clausesWithSpans} from './crate/obligation_ledger.mjs';

/** One seeded command builder shared by execution and freshness classification. */
export function ownedCreationReadbackCommand(target) {
  return agenticMessage('literal-create-shell-readback', {path: quoteShellWord(target)});
}
function safeRelativeCreationPath(path) {
  return typeof path === 'string' && path.length > 0 && !path.startsWith('/')
    && !/^[A-Za-z]:/u.test(path) && !/[\0\r\n\\]/u.test(path)
    && path.split('/').every(part => part !== '' && part !== '.' && part !== '..');
}
function validCreationHostRoot(root) {
  return typeof root === 'string' && root.startsWith('/') && !/[\0\r\n\\]/u.test(root)
    && (root === '/' || safeRelativeCreationPath(root.slice(1)));
}
export function projectedCreationPath(relative, hostRoot) {
  return safeRelativeCreationPath(relative) && validCreationHostRoot(hostRoot)
    ? (hostRoot === '/' ? '/' : hostRoot + '/') + relative : null;
}
export function creationPathMatches(logical, observed, hostRoot) {
  return typeof logical === 'string' && typeof observed === 'string'
    && (logical === observed || projectedCreationPath(logical, hostRoot) === observed
      || projectedCreationPath(observed, hostRoot) === logical);
}
export function workspaceRelativeCreationPath(observed, hostRoot) {
  if (typeof observed !== 'string' || !validCreationHostRoot(hostRoot)) return null;
  const prefix = hostRoot === '/' ? '/' : hostRoot + '/';
  if (!observed.startsWith(prefix)) return null;
  const relative = observed.slice(prefix.length);
  return projectedCreationPath(relative, hostRoot) === observed ? relative : null;
}
export function readOnlyCapability(capability) {
  return [Capability.Read, Capability.ReadMany, Capability.Search, Capability.Fetch,
    Capability.Grep, Capability.Glob, Capability.ListDir, Capability.AskUser].includes(capability);
}
function qualifiedReceipt(progress, attempt, command, qualify) {
  const raw = attempt.raw_receipt ?? null, frame = qualify(progress, command);
  const qualified = typeof raw === 'string' && frame?.binding === 'exact'
    && frame.receiptPresent === true && frame.detail === raw && frame.declaredTool === attempt.tool;
  return {raw, qualified, succeeded: qualified && frame.succeeded === true && attempt.succeeded === true};
}
function scalarCommand(argumentsText) {
  try {
    const parsedArguments = JSON.parse(argumentsText);
    return parsedArguments !== null && typeof parsedArguments === 'object' && !Array.isArray(parsedArguments)
      && Object.keys(parsedArguments).length === 1 && typeof parsedArguments.command === 'string'
      ? parsedArguments.command : null;
  } catch { return null; }
}
/** Known framed reads may coexist; unknown or unbound shell actions still invalidate proof. */
export function readOnlyAttempt(progress, attempt, qualify, pathFromArguments) {
  if (readOnlyCapability(attempt.capability)) return true;
  if (attempt.capability !== Capability.Run || typeof qualify !== 'function'
    || typeof pathFromArguments !== 'function') return false;
  const command = scalarCommand(attempt.arguments);
  if (command === null) return false;
  const known = progress.attempts.some((write, index) => {
    if (!write.succeeded || write.arguments === null) return false;
    const path = write.capability === Capability.Write ? pathFromArguments(write.arguments) : null;
    if (path !== null && command === ownedCreationReadbackCommand(path)) return true;
    const creation = write.capability === Capability.Edit && isWorkspaceCreationTool(write.tool)
      ? canonicalCreation(write.arguments) : null;
    const observedPath = path ?? creation?.path ?? null;
    const mutation = observedPath === null ? null : progress.creationMutationFor(observedPath);
    return mutation?.qualified && mutation.succeeded && mutation.index === index
      && [mutation.path, mutation.observedPath, mutation.workspaceRelativePath]
        .some(target => typeof target === 'string' && command === ownedCreationReadbackCommand(target));
  });
  // A failed qualified cat is still read-only; it never becomes successful evidence.
  return known && qualifiedReceipt(progress, attempt, command, qualify).qualified;
}

/** Only the exact successful latest mutation may lend its subsequent complete Read. */
export function postMutationReadStateFor(progress, path, capability, argumentsValue, qualifyTool, qualifyCommand, pathFromArguments) {
  let read = null;
  for (let index = progress.attempts.length - 1; index >= 0; index--) {
    const attempt = progress.attempts[index];
    if (readOnlyAttempt(progress, attempt, qualifyCommand, pathFromArguments)) {
      if (read === null && attempt.capability === Capability.Read && attempt.arguments !== null
        && pathFromArguments(attempt.arguments) === path) read = attempt;
      continue;
    }
    let actual; try { actual = JSON.parse(attempt.arguments); } catch { return {current:false,attempt:null}; }
    const frame = qualifyTool(progress, capability, argumentsValue);
    const current = attempt.capability === capability && argumentValuesEqual(actual, argumentsValue)
      && attempt.succeeded && frame?.binding === 'exact' && frame.receiptPresent
      && frame.declaredTool === attempt.tool && frame.succeeded && frame.detail === attempt.detail;
    return {current,attempt:current ? read : null};
  }
  return {current:false,attempt:null};
}

/** The actual complete preimage before the latest target mutation, without intervening effects. */
export function sourceReadBeforeLatestMutationFor(progress, path, capability, qualifyCommand, pathFromArguments) {
  const index = progress.attempts.findLastIndex(attempt => attempt.capability === capability
    && attempt.arguments !== null && pathFromArguments(attempt.arguments) === path);
  for (let prior = index - 1; prior >= 0; prior--) {
    const attempt = progress.attempts[prior];
    if (!readOnlyAttempt(progress, attempt, qualifyCommand, pathFromArguments)) return null;
    if (attempt.capability === Capability.Read && attempt.arguments !== null
      && pathFromArguments(attempt.arguments) === path) {
      const observed = attempt.source_read;
      return attempt.succeeded && observed?.error === null && observed.complete ? attempt : null;
    }
  }
  return null;
}

/** Current target Read, stopping at any observed potential effect. */
export function freshSourceReadAttemptFor(progress, path, qualifyCommand, pathFromArguments) {
  for (let index = progress.attempts.length - 1; index >= 0; index--) {
    const attempt = progress.attempts[index];
    if (!readOnlyAttempt(progress, attempt, qualifyCommand, pathFromArguments)) return null;
    if (attempt.capability === Capability.Read && attempt.arguments !== null
      && pathFromArguments(attempt.arguments) === path) return attempt;
  }
  return null;
}

// Creation custody is separate from broad Edit/Write classification.
/** Only complete owned clauses of the scanner's standing request may commute. */
export function ownedCreationDeclarations(request, hostRoot = null) {
  if (typeof request !== 'string') return [];
  const declarations = clausesWithSpans(request).flatMap(([clause, span]) => {
    const frame = ownedDeclaredCreateFrame(clause);
    return frame === null ? [] : [Object.freeze({...frame, span: Object.freeze([...span])})];
  });
  const ambiguous = declarations.some((declaration, index) => declarations.some((other, at) =>
    at < index && creationPathMatches(declaration.target, other.target, hostRoot)));
  return ambiguous ? [] : Object.freeze(declarations);
}
function declaredCreation(declarations, mutation, hostRoot) {
  return declarations.some(declaration => creationPathMatches(declaration.target, mutation.path, hostRoot)
    && declaration.content === mutation.content);
}
function disjointCreationTargets(left, right) {
  const parts = path => path.replaceAll('\\', '/').split('/');
  const valid = path => !/^[A-Za-z]:/u.test(path) && parts(path).every(part => part !== '.' && part !== '..');
  const key = path => parts(path).filter(Boolean).join('/').toLowerCase();
  const rooted = path => path.startsWith('/') || path.startsWith('\\');
  return valid(left) && valid(right) && rooted(left) === rooted(right) && key(left) !== key(right);
}
function declaredDisjointMutation(progress, frames, mutation, attempt, index, declarations, pathFromArguments, hostRoot) {
  if (!declaredCreation(declarations, mutation, hostRoot)) return false;
  const candidates = frames.filter(frame => boundAttempt(progress, frame)?.attempt === attempt);
  if (candidates.length !== 1) return false;
  const creation = creationArguments(candidates[0], pathFromArguments);
  if (creation === null) return false;
  const currentPath = mutation.observedPath ?? mutation.path;
  const left = projectedCreationPath(currentPath, hostRoot) ?? currentPath;
  const right = projectedCreationPath(creation.path, hostRoot) ?? creation.path;
  if (!disjointCreationTargets(left, right)) return false;
  const other = creationMutationFor(progress, frames, creation.path, pathFromArguments, hostRoot);
  return other?.qualified && other.succeeded && other.index === index
    && declaredCreation(declarations, other, hostRoot);
}
function creationArguments(frame, pathFromArguments) {
  if (!isWorkspaceCreationTool(frame.declaredTool)) return null;
  if (frame.capability === Capability.Edit) return canonicalCreation(frame.arguments);
  if (frame.capability !== Capability.Write) return null;
  const path = pathFromArguments(frame.arguments), value = frame.argumentsValue;
  if (path === null || value === null || typeof value !== 'object' || Array.isArray(value)) return null;
  const contents = ['content', 'contents', 'text', 'new_string']
    .filter(key => Object.hasOwn(value, key)).map(key => value[key]);
  return contents.length > 0 && contents.every(content => typeof content === 'string' && content === contents[0])
    ? {path, content: contents[0]} : null;
}
function argumentValue(raw) {
  try { return JSON.parse(raw); } catch { return raw; }
}
function boundAttempt(progress, frame) {
  const matches = progress.attempts.map((attempt, index) => ({attempt, index})).filter(({attempt}) =>
    attempt.capability === frame.capability && attempt.tool === frame.declaredTool
    && argumentValuesEqual(argumentValue(attempt.arguments), frame.argumentsValue)
    && (attempt.raw_receipt ?? attempt.detail) === frame.detail);
  return matches.length === 1 ? matches[0] : null;
}
export function creationMutationFor(progress, frames, path, pathFromArguments, hostRoot = null) {
  for (const frame of [...frames].reverse()) {
    const creation = creationArguments(frame, pathFromArguments);
    if (creation === null || !creationPathMatches(path, creation.path, hostRoot)) continue;
    const bound = boundAttempt(progress, frame);
    const qualified = frame.binding === 'exact' && frame.receiptPresent && !frame.duplicate && bound !== null;
    return Object.freeze({...creation, path, observedPath: creation.path,
      workspaceRelativePath: workspaceRelativeCreationPath(creation.path, hostRoot),
      frame, capability: frame.capability, tool: frame.declaredTool,
      arguments: frame.arguments, argumentsValue: frame.argumentsValue, receiptPresent: frame.receiptPresent,
      attempt: bound?.attempt ?? null, index: bound?.index ?? null, qualified,
      succeeded: qualified && frame.succeeded && bound.attempt.succeeded});
  }
  return null;
}
export function creationPreflightFor(progress, frames, path, pathFromArguments, hostRoot = null) {
  const unknown = attempted => ({attempted, observation: null});
  if (frames.some(frame => !frame.receiptPresent && !readOnlyCapability(frame.capability))) return unknown(true);
  for (const frame of [...frames].reverse()) {
    if (!readOnlyCapability(frame.capability)) return unknown(false);
    if (frame.capability !== Capability.Read || !creationPathMatches(path, pathFromArguments(frame.arguments), hostRoot)) continue;
    const bound = boundAttempt(progress, frame);
    return frame.binding === 'exact' && frame.receiptPresent && !frame.duplicate && bound !== null
      ? {attempted: true, observation: bound.attempt.source_read ?? null} : unknown(true);
  }
  return unknown(false);
}
export function creationReadbackFor(progress, frames, mutation, command, qualify, pathFromArguments, declarations = [], hostRoot = null) {
  const empty = current => ({current, readAttempt: null, commandReceipt: null, pendingRead: false, needsFreshRead: false});
  if (!mutation?.succeeded || mutation.index === null || !frames.includes(mutation.frame)
    || progress.attempts[mutation.index] !== mutation.attempt
    || frames.some(frame => !frame.receiptPresent && !readOnlyCapability(frame.capability))) return empty(false);
  const laterFrames = frames.slice(frames.indexOf(mutation.frame) + 1);
  const state = empty(true);
  state.pendingRead = laterFrames.some(frame => frame.capability === Capability.Read
    && creationPathMatches(mutation.path, pathFromArguments(frame.arguments), hostRoot) && !frame.receiptPresent);
  for (let index = progress.attempts.length - 1; index > mutation.index; index--) {
    const attempt = progress.attempts[index];
    if (attempt.capability === Capability.Run && commandArgument(attempt.arguments) === command) {
      state.commandReceipt ??= qualifiedReceipt(progress, attempt, command, qualify);
      if (!state.commandReceipt.qualified) return empty(false);
      continue;
    }
    if (!readOnlyAttempt(progress, attempt, qualify, pathFromArguments)) {
      if (declaredDisjointMutation(progress, frames, mutation, attempt, index, declarations, pathFromArguments, hostRoot)) continue;
      const invalidators = frames.filter(frame => frame.binding === 'exact' && frame.receiptPresent
        && !frame.duplicate && boundAttempt(progress, frame)?.attempt === attempt);
      return {...state, current: false, needsFreshRead: state.readAttempt === null
        && state.commandReceipt === null && !state.pendingRead && invalidators.length === 1};
    }
    if (state.readAttempt === null && attempt.capability === Capability.Read
      && creationPathMatches(mutation.path, pathFromArguments(attempt.arguments), hostRoot)) state.readAttempt = attempt;
  }
  return state;
}
