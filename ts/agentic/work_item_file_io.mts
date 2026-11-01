// Intended owner: js/agentic/work_item_file_io.mjs.
// An explicit transport observation adapter. It retains physical Run provenance;
// it does not synthesize ChatMessage Read/Edit receipts or infer process success.
import { Progress } from './progress.mjs';
import { Capability } from './capability.mjs';
import { shellCommandTool, toolFor, isWorkspaceCreationTool } from './capability_router.mjs';
import { readText } from './host.mjs';
import { clientWorkingDirectory, plainText } from './content.mjs';
import { commandArgument, reportedExitCode, observedPayload, incompleteReceipt } from './tool_result.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { quoteShellWord } from './portable/string_contracts.mjs';
import { planOne, jsonText, writeArguments } from './plan.mjs';
import { readPolicyBlocksPlan } from './file_read/ownership.mjs';
const PROGRAM = 'data/meta/work-item-file-io-program.cjs';
const canonical = value => JSON.stringify(Object.fromEntries(Object.keys(value).sort().map(key => [key, value[key]])));
const encode = text => {
  const bytes = new TextEncoder().encode(text);
  let binary = '';
  for (let index = 0; index < bytes.length; index += 8192) binary += String.fromCharCode(...bytes.subarray(index, index + 8192));
  return btoa(binary);
};
function decode(text) {
  if (typeof text !== 'string') throw Error('FileIoBase64Required');
  const bytes = Uint8Array.from(atob(text), character => character.charCodeAt(0));
  const value = new TextDecoder('utf-8', {
    fatal: true
  }).decode(bytes);
  if (encode(value) !== text) throw Error('FileIoBase64Noncanonical');
  return value;
}
function program() {
  const source = readText(PROGRAM);
  if (typeof source !== 'string' || source.length > 32768) throw Error('FileIoProgramUnavailable');
  return source;
}
const relative = path => typeof path === 'string' && path.length > 0 && path.length <= 4096 && !path.startsWith('/') && !path.includes('\\') && !path.includes('\0') && path.split('/').every(part => part !== '' && part !== '.' && part !== '..');
const rootPath = root => typeof root === 'string' && root.startsWith('/') && !root.endsWith('/') && root.split('/').slice(1).every(part => part !== '' && part !== '.' && part !== '..');
export function canonicalFileIoPath(path, root) {
  if (relative(path)) return path;
  if (!rootPath(root) || typeof path !== 'string' || !path.startsWith(root + '/')) return null;
  const suffix = path.slice(root.length + 1);
  return relative(suffix) ? suffix : null;
}
export function fileIoCallerRoot(messages) {
  return clientWorkingDirectory(messages.filter(message => ['system', 'developer', 'user'].includes(message.role.toLowerCase())));
}
function requestValue(operation, root, path, before, after) {
  if (!['root', 'read', 'create', 'replace'].includes(operation)) throw Error('FileIoOperationUnsupported');
  if (operation !== 'root' && (!rootPath(root) || !relative(path))) throw Error('FileIoOwnedPathRequired');
  return {
    afterBase64: after === null ? null : encode(after),
    beforeSha256: before === null ? null : sha256Hex(before),
    operation,
    path,
    root,
    schema: 'formal-ai-owned-file-request/v1'
  };
}
export function fileIoCommand(operation, root, path, before = null, after = null) {
  const request = requestValue(operation, root, path, before, after),
    payload = encode(canonical(request));
  const command = 'node -e ' + quoteShellWord(program()) + ' ' + quoteShellWord(payload);
  if (command.length > 196608) throw Error('FileIoCommandLimit');
  return command;
}
function searchSpecification(directory, identifiers) {
  if (!relative(directory) || directory.split('/').some(part => part.startsWith('.')) || !Array.isArray(identifiers) ||
         !identifiers.length || identifiers.length > 8 || identifiers.some(identifier => typeof identifier !== 'string' ||
         !/^[A-Za-z_][A-Za-z0-9_]{0,127}$/u.test(identifier)) || new Set(identifiers).size !== identifiers.length) throw Error('FileIoSearchScopeRequired');
  return {
    directory,
    identifiers: [...identifiers].sort()
  };
}
function validSearchRequest(value) {
  try {
    const specification = searchSpecification(value.directory, value.identifiers);
    return value.schema === 'formal-ai-owned-search-request/v1' && value.operation === 'search' && Object.keys(value).sort().join('|') === 'directory|identifiers|operation|root|schema' && rootPath(value.root) && JSON.stringify(specification.identifiers) === JSON.stringify(value.identifiers);
  } catch {
    return false;
  }
}
/** Mirrors `command` in rust/src/agentic_coding/work_item_file_io/search.rs. */
export function fileIoSearchCommand(root, directory, identifiers) {
  if (!rootPath(root)) throw Error('FileIoOwnedPathRequired');
  const specification = searchSpecification(directory, identifiers);
  const request = {
    schema: 'formal-ai-owned-search-request/v1',
    operation: 'search',
    root,
    ...specification
  };
  const command = 'node -e ' + quoteShellWord(program()) + ' ' + quoteShellWord(encode(canonical(request)));
  if (command.length > 196608) throw Error('FileIoCommandLimit');
  return command;
}
function searchReceiptFor(request, output) {
  if (typeof output !== 'string' || output.length > 262144) return null;
  const exact = /^FORMAL_AI_FILE_IO_V1 ([A-Za-z0-9+/=]+)\n?$/u.exec(output);
  if (!exact) return null;
  try {
    const text = decode(exact[1]),
      receipt = JSON.parse(text),
      value = request.value;
    if (canonical(receipt) !== text || receipt.schema !== 'formal-ai-owned-search-receipt/v1' || receipt.operation !== 'search' ||
         receipt.requestIdentity !== request.identity || receipt.root !== value.root || receipt.directory !== value.directory ||
         JSON.stringify(receipt.identifiers) !== JSON.stringify(value.identifiers) || receipt.state !== 'observed-search' ||
         typeof receipt.complete !== 'boolean' || !Array.isArray(receipt.files) || receipt.files.length > 8 ||
         !Array.isArray(receipt.gaps) || receipt.gaps.length > 32 || receipt.complete && receipt.gaps.length !== 0 ||
         !Number.isInteger(receipt.visitedFiles) || receipt.visitedFiles < 0 || receipt.visitedFiles > 1025 ||
         !Number.isInteger(receipt.visitedDirectories) || receipt.visitedDirectories < 0 || receipt.visitedDirectories > 129 ||
         !Number.isInteger(receipt.bytesRead) || receipt.bytesRead < 0 || receipt.bytesRead > 16777216 || receipt.scope !== 'bounded-nonhidden-rust-source-candidates; text matches are not bindings; fresh Read and CST validation required') return null;
    const paths = new Set();
    for (const file of receipt.files) {
      if (!relative(file.path) || !file.path.startsWith(value.directory + '/') || !file.path.endsWith('.rs') ||
         file.path.split('/').some(part => part.startsWith('.')) || paths.has(file.path) || !/^[a-f0-9]{64}$/u.test(file.sha256) ||
         !Number.isInteger(file.byteLength) || file.byteLength < 0 || file.byteLength > 131072 || !Array.isArray(file.identifiers) ||
         !file.identifiers.length || new Set(file.identifiers).size !== file.identifiers.length || file.identifiers.some(identifier => !value.identifiers.includes(identifier))) return null;
      paths.add(file.path);
    }
    return receipt;
  } catch {
    return null;
  }
}
function searchObservation(frame, request, completed) {
  const receipt = searchReceiptFor(request, completed.output),
    code = completed.parts?.code ?? null;
  const succeeded = code === 0 && receipt !== null && !completed.gap && !completed.pending;
  return {
    directory: request.value.directory,
    identifiers: request.value.identifiers,
    root: request.value.root,
    requestIdentity: request.identity,
    status: completed.pending ? 'pending' : code === 0 ? 'reported-success' : code === null ? 'unknown' : 'reported-failure',
    complete: succeeded && receipt.complete,
    files: succeeded ? receipt.files : [],
    receipt: succeeded ? receipt : null,
    error: completed.gap ?? (completed.pending ? null : !succeeded ? 'FileIoSearchObservationUnavailable' : null),
    transport: {
      kind: 'registered-run-search',
      actualTool: frame.tool,
      callId: frame.id,
      command: request.command,
      actualExitCode: code,
      pendingSession: completed.pending?.session ?? null,
      completedAt: completed.completedAt,
      originalReceipt: frame.raw,
      root: request.value.root
    }
  };
}
export function fileIoSearchStep(messages, tools, directory, identifiers, task = '') {
  let specification;
  try {
    specification = searchSpecification(directory, identifiers);
  } catch (error) {
    return {
      gap: String(error.message)
    };
  }
  if (readPolicyBlocksPlan(task, planOne('read', jsonText({
    filePath: directory
  })))) return {
    gap: 'ReadPolicyRefused'
  };
  const progress = WorkItemFileProgress.scan(messages),
    pending = fileIoPendingStep(progress, tools);
  if (pending) return pending;
  const run = shellCommandTool(tools);
  if (!run) return {
    gap: 'FileIoRunToolUnavailable'
  };
  try {
    const command = progress.root ? fileIoSearchCommand(progress.root, specification.directory, specification.identifiers) : fileIoCommand('root', null, null);
    return {
      step: planOne(run, jsonText({
        command,
        max_output_tokens: 65536,
        yield_time_ms: 1000
      }))
    };
  } catch (error) {
    return {
      gap: String(error.message)
    };
  }
}
function ownedCommand(command) {
  if (typeof command !== 'string' || command.length > 196608) return null;
  let source;
  try {
    source = program();
  } catch {
    return null;
  }
  const prefix = 'node -e ' + quoteShellWord(source) + ' ';
  if (!command.startsWith(prefix)) return null;
  const encoded = command.slice(prefix.length);
  if (!/^'[A-Za-z0-9+/=]*'$/u.test(encoded)) return null;
  try {
    const input = decode(encoded.slice(1, -1)),
      value = JSON.parse(input);
    if (canonical(value) !== input) return null;
    if (validSearchRequest(value)) return {
      value,
      identity: sha256Hex(input),
      command
    };
    if (Object.keys(value).sort().join('|') !== 'afterBase64|beforeSha256|operation|path|root|schema' || value.schema !== 'formal-ai-owned-file-request/v1' || !['root', 'read', 'create', 'replace'].includes(value.operation)) return null;
    if (value.operation === 'root') {
      if ([value.root, value.path, value.beforeSha256, value.afterBase64].some(item => item !== null)) return null;
    } else if (!rootPath(value.root) || !relative(value.path)) return null;
    return {
      value,
      identity: sha256Hex(input),
      command
    };
  } catch {
    return null;
  }
}
function processParts(raw) {
  let object;
  try {
    object = JSON.parse(raw);
  } catch {}
  if (object && typeof object === 'object' && !Array.isArray(object)) {
    const output = typeof object.output === 'string' ? object.output : typeof object.stdout === 'string' ? object.stdout : null;
    const code = Number.isInteger(object.exit_code) ? object.exit_code : Number.isInteger(object.exitCode) ? object.exitCode : null;
    const session = Number.isSafeInteger(object.session_id) && object.session_id > 0 ? object.session_id : null;
    if (output !== null) return {
      output,
      code,
      session
    };
  }
  return {
    output: observedPayload(raw),
    code: reportedExitCode(raw),
    session: null
  };
}
function incompleteCleanup(raw) {
  try {
    const value = JSON.parse(raw);
    return value?.cleanup_complete === false || value?.cleanupComplete === false;
  } catch {
    return false;
  }
}
function receiptFor(request, output) {
  if (typeof output !== 'string' || output.length > 262144) return null;
  const exact = /^FORMAL_AI_FILE_IO_V1 ([A-Za-z0-9+/=]+)\n?$/u.exec(output);
  if (!exact) return null;
  try {
    const text = decode(exact[1]),
      receipt = JSON.parse(text);
    if (canonical(receipt) !== text || receipt.schema !== 'formal-ai-owned-file-receipt/v1' || receipt.requestIdentity !== request.identity || receipt.operation !== request.value.operation || receipt.path !== request.value.path || request.value.operation !== 'root' && receipt.root !== request.value.root) return null;
    return receipt;
  } catch {
    return null;
  }
}
function argumentsObject(attempt) {
  try {
    return JSON.parse(attempt.arguments);
  } catch {
    return null;
  }
}
function canonicalNativeAttempt(attempt, root) {
  if (![Capability.Read, Capability.Edit, Capability.Write].includes(attempt.capability)) return attempt;
  const argumentsValue = argumentsObject(attempt);
  if (!argumentsValue) return attempt;
  const names = ['path', 'filePath', 'file_path', 'absolute_path'].filter(name => Object.hasOwn(argumentsValue, name));
  const paths = names.map(name => canonicalFileIoPath(argumentsValue[name], root));
  if (!paths.length || paths.some(path => path === null || path !== paths[0])) return {
    ...attempt,
    arguments: null,
    physicalArguments: attempt.arguments,
    transport: {
      kind: 'client-file-tool',
      root,
      canonicalPath: null,
      actualTool: attempt.tool,
      physicalArguments: attempt.arguments,
      reason: 'AmbiguousOrUnownedPathAliases'
    }
  };
  const normalized = {
    ...argumentsValue
  };
  for (const name of names) normalized[name] = paths[0];
  // Keep established controller aliases available without inventing a tool call.
  normalized.filePath = paths[0];
  return {
    ...attempt,
    arguments: jsonText(normalized),
    physicalArguments: attempt.arguments,
    transport: {
      kind: 'client-file-tool',
      root,
      canonicalPath: paths[0],
      actualTool: attempt.tool
    }
  };
}
function rawFrames(messages) {
  const calls = new Map(),
    duplicates = new Set(),
    frames = [];
  for (let index = evidenceWindowStart(messages); index < messages.length; index++) {
    const message = messages[index];
    if (message.role.toLowerCase() === 'assistant') for (const call of message.tool_calls ?? []) {
      if (calls.has(call.id)) duplicates.add(call.id);
      calls.set(call.id, {
        call,
        index
      });
    }
    if (message.role.toLowerCase() === 'tool' && typeof message.tool_call_id === 'string') {
      const declaration = calls.get(message.tool_call_id);
      if (frames.some(frame => frame.id === message.tool_call_id)) {
        duplicates.add(message.tool_call_id);
        continue;
      }
      if (!declaration || duplicates.has(message.tool_call_id)) continue;
      if (message.name && message.name !== declaration.call.function.name) continue;
      frames.push({
        id: message.tool_call_id,
        tool: declaration.call.function.name,
        argumentsValue: declaration.call.function.arguments,
        raw: plainText(message.content),
        error: message.is_error === true || message.isError === true,
        index
      });
    }
  }
  return frames.filter(frame => !duplicates.has(frame.id));
}
function completedRun(frame, frames) {
  let parts = processParts(frame.raw),
    output = parts.output ?? '',
    polls = 0,
    completedAt = frame.index;
  if (frame.error) return {
    gap: 'FileIoProviderReportedFailure',
    parts,
    output,
    polls,
    completedAt
  };
  if ((incompleteReceipt(frame.raw) || incompleteCleanup(frame.raw)) && parts.session === null) return {
    gap: 'FileIoIncompleteProcessReceipt',
    parts,
    output,
    polls,
    completedAt
  };
  while (parts.code === null && parts.session !== null) {
    const pollsFor = frames.filter(next => next.index > completedAt && next.tool.split('__').at(-1).split(/[.:/]/u).at(-1) === 'write_stdin').filter(next => {
      let argumentsValue;
      try {
        argumentsValue = JSON.parse(next.argumentsValue);
      } catch {
        return false;
      }
      return argumentsValue.session_id === parts.session && (argumentsValue.chars === undefined || argumentsValue.chars === '');
    });
    const next = pollsFor[0];
    if (!next) return {
      pending: {
        session: parts.session,
        polls
      },
      output,
      completedAt
    };
    if (++polls > 3) return {
      gap: 'FileIoPollingBudgetExceeded',
      output,
      completedAt
    };
    parts = processParts(next.raw);
    if (next.error || (incompleteReceipt(next.raw) || incompleteCleanup(next.raw)) && parts.session === null) return {
      gap: 'FileIoPollingFailed',
      parts,
      output,
      completedAt: next.index
    };
    output += parts.output ?? '';
    completedAt = next.index;
    if (output.length > 262144) return {
      gap: 'FileIoOutputLimit',
      output: '',
      completedAt
    };
  }
  return {
    parts,
    output,
    polls,
    completedAt
  };
}
export class WorkItemFileProgress {
  constructor(attempts, root, pending = null, gap = null, transportDiagnostics = [], searches = []) {
    this.searches = searches;
    this.attempts = attempts;
    this.root = root;
    this.pending = pending;
    this.gap = gap;
    this.transportDiagnostics = transportDiagnostics;
  }
  static scan(messages) {
    const original = Progress.scan(messages),
      frames = rawFrames(messages),
      owned = frames.map(frame => ({
        frame,
        request: ownedCommand(commandArgument(frame.argumentsValue))
      })).filter(item => item.request);
    let root = fileIoCallerRoot(messages),
      pending = null,
      gap = null;
    if (root !== null && root !== undefined && !rootPath(root)) gap = 'FileIoCallerRootUnsupported';
    const events = [],
      searches = [],
      transportDiagnostics = [];
    for (const {
      frame,
      request
    } of owned) {
      const completed = completedRun(frame, frames);
      transportDiagnostics.push({
        kind: 'registered-process',
        actualTool: frame.tool,
        callId: frame.id,
        requestIdentity: request.identity,
        operation: request.value.operation,
        path: request.value.path,
        actualExitCode: completed.parts?.code ?? null,
        pendingSession: completed.pending?.session ?? null,
        polls: completed.polls ?? completed.pending?.polls ?? null,
        reason: completed.gap ?? null
      });
      if (request.value.operation === 'search') searches.push(searchObservation(frame, request, completed));
      if (completed.pending) {
        pending = {
          ...completed.pending,
          originalTool: frame.tool,
          requestIdentity: request.identity
        };
        continue;
      }
      if (completed.gap) {
        gap = completed.gap;
        continue;
      }
      if (request.value.operation === 'search') {
        if (root && request.value.root !== root) gap = 'FileIoRootContextContradicted';
        continue;
      }
      const receipt = receiptFor(request, completed.output),
        code = completed.parts?.code;
      if (code === 0 && receipt && request.value.operation === 'root' && receipt.state === 'observed-root' && rootPath(receipt.root)) {
        if (root !== null && root !== undefined && root !== receipt.root) gap = 'FileIoRootContextContradicted';else root = receipt.root;
        continue;
      }
      if (request.value.operation === 'root') {
        gap = 'FileIoRootObservationUnavailable';
        continue;
      }
      if (root && request.value.root !== root) {
        gap = 'FileIoRootContextContradicted';
        continue;
      }
      const argumentsValue = {
          filePath: request.value.path
        },
        operation = request.value.operation;
      const capability = operation === 'read' ? Capability.Read : operation === 'create' ? Capability.Write : Capability.Edit;
      const transport = {
        kind: 'registered-run-file-io',
        actualTool: frame.tool,
        callId: frame.id,
        requestIdentity: request.identity,
        command: request.command,
        exitCode: code,
        completedAt: completed.completedAt,
        originalReceipt: frame.raw
      };
      if (operation !== 'read') {
        let after;
        try {
          after = decode(request.value.afterBase64);
        } catch {
          gap = 'FileIoCandidateEncoding';
          continue;
        }
        if (operation === 'create') argumentsValue.content = after;else {
          argumentsValue.newString = after;
          argumentsValue.beforeSha256 = request.value.beforeSha256;
        }
      }
      let source_read = null,
        succeeded = false;
      if (operation === 'read') {
        source_read = {
          status: code === 0 ? 'reported-success' : code === null ? 'unknown' : 'reported-failure',
          complete: false,
          absent: false,
          source: null,
          error: code === null ? 'FileIoProcessStatusUnknown' : code !== 0 ? 'FileIoProcessFailed' : 'FileIoReceiptInvalid'
        };
        if (code === 0 && receipt?.state === 'absent') {
          source_read.absent = true;
          source_read.error = 'ENOENT';
        }
        if (code === 0 && receipt?.state === 'observed-source') try {
          const source = decode(receipt.contentBase64),
            bytes = new TextEncoder().encode(source);
          if (bytes.length <= 131072 && bytes.length === receipt.byteLength && sha256Hex(source) === receipt.sha256) {
            source_read = {
              status: 'reported-success',
              complete: true,
              absent: false,
              source,
              error: null
            };
            succeeded = true;
          }
        } catch {}
      } else succeeded = code === 0 && receipt?.state === 'effect-applied' && receipt.beforeSha256 === request.value.beforeSha256 && receipt.afterSha256 === sha256Hex(decode(request.value.afterBase64));
      events.push({
        at: completed.completedAt,
        attempt: {
          capability,
          arguments: jsonText(argumentsValue),
          succeeded,
          source_read,
          detail: frame.raw,
          tool: frame.tool,
          transport
        }
      });
      if (code === null) gap = 'FileIoProcessStatusUnknown';
    }
    // Native tool attempts preserve genuine scanner receipts. Run attempts are
    // replaced only by authenticated registered-operation observations above.
    const nonOwned = original.attempts.filter(attempt => !ownedCommand(commandArgument(attempt.arguments)));
    const native = nonOwned.map(attempt => canonicalNativeAttempt(attempt, root));
    // Raw scanner order equals tool-result order. Reassociate exact call argumentsValue to
    // real frame indices; duplicate declarations never gain file-IO authority.
    const used = new Set();
    for (const attempt of native) {
      const frame = frames.find((frame, index) => !used.has(index) && frame.tool === attempt.tool && frame.argumentsValue === attempt.physicalArguments);
      const index = frame ? frames.indexOf(frame) : frames.findIndex((frame, index) => !used.has(index) && frame.tool === attempt.tool && frame.argumentsValue === attempt.arguments);
      if (index >= 0) {
        used.add(index);
        events.push({
          at: frames[index].index,
          attempt
        });
      }
    }
    events.sort((left, right) => left.at - right.at);
    for (let index = 0; index < events.length; index++) {
      const attempt = events[index].attempt,
        argumentsValue = argumentsObject(attempt);
      if (attempt.transport?.kind !== 'registered-run-file-io' || attempt.capability !== Capability.Edit) continue;
      const prior = events.slice(0, index).reverse().find(event => event.attempt.capability === Capability.Read &&
         argumentsObject(event.attempt)?.filePath === argumentsValue.filePath && event.attempt.source_read?.complete === true &&
         event.attempt.source_read.error === null)?.attempt.source_read.source;
      if (typeof prior === 'string' && sha256Hex(prior) === argumentsValue.beforeSha256) attempt.arguments = jsonText({
        ...argumentsValue,
        oldString: prior
      });else {
        attempt.succeeded = false;
        gap = 'FileIoObservedPreimageUnavailable';
      }
    }
    return new WorkItemFileProgress(events.map(event => event.attempt), root, pending, gap, transportDiagnostics, searches);
  }
  searchFor(directory, identifiers) {
    let specification;
    try {
      specification = searchSpecification(directory, identifiers);
    } catch {
      return null;
    }
    return [...this.searches].reverse().find(observation => observation.directory === specification.directory && JSON.stringify(observation.identifiers) === JSON.stringify(specification.identifiers)) ?? null;
  }
  sourceReadFor(path) {
    return [...this.attempts].reverse().find(attempt => attempt.capability === Capability.Read && argumentsObject(attempt)?.filePath === path)?.source_read ?? null;
  }
}
export function fileIoPendingStep(progress, tools) {
  if (progress.gap) return {
    gap: progress.gap
  };
  if (!progress.pending) return null;
  if (progress.pending.polls >= 3) return {
    gap: 'FileIoPollingBudgetExceeded'
  };
  const poll = tools.find(name => name.split('__').at(-1).split(/[.:/]/u).at(-1) === 'write_stdin');
  return poll ? {
    step: planOne(poll, jsonText({
      session_id: progress.pending.session,
      chars: '',
      max_output_tokens: 65536,
      yield_time_ms: 1000
    }))
  } : {
    gap: 'FileIoPollingToolUnavailable'
  };
}
function runStep(messages, tools, operation, path, before = null, after = null) {
  const progress = WorkItemFileProgress.scan(messages),
    pending = fileIoPendingStep(progress, tools);
  if (pending) return pending;
  const run = shellCommandTool(tools);
  if (!run) return {
    gap: 'FileIoRunToolUnavailable'
  };
  try {
    const command = progress.root ? fileIoCommand(operation, progress.root, path, before, after) : fileIoCommand('root', null, null);
    return {
      step: planOne(run, jsonText({
        command,
        max_output_tokens: 65536,
        yield_time_ms: 1000
      }))
    };
  } catch (error) {
    return {
      gap: String(error.message)
    };
  }
}
export function fileIoReadStep(messages, tools, path, task = '') {
  // Query existing Read policy before lowering to a Run transport. This query
  // is never emitted or inserted into the client transcript.
  if (readPolicyBlocksPlan(task, planOne('read', jsonText({
    filePath: path
  })))) return {
    gap: 'ReadPolicyRefused'
  };
  const progress = WorkItemFileProgress.scan(messages),
    pending = fileIoPendingStep(progress, tools);
  if (pending) return pending;
  const read = toolFor(tools, Capability.Read);
  return read ? {
    step: planOne(read, jsonText({
      filePath: path
    }))
  } : runStep(messages, tools, 'read', path);
}
export function fileIoEffectStep(messages, tools, path, before, after) {
  if (typeof after !== 'string' || new TextEncoder().encode(after).length > 131072) return {
    gap: 'FileIoCandidateLimit'
  };
  const progress = WorkItemFileProgress.scan(messages),
    pending = fileIoPendingStep(progress, tools);
  if (pending) return pending;
  if (before === null) {
    const write = toolFor(tools, Capability.Write);
    if (write) return {
      step: planOne(write, writeArguments(path, after))
    };
    return runStep(messages, tools, 'create', path, null, after);
  }
  const edit = toolFor(tools, Capability.Edit);
  if (edit && !isWorkspaceCreationTool(edit)) return {
    step: planOne(edit, jsonText({
      filePath: path,
      oldString: before,
      newString: after
    }))
  };
  return runStep(messages, tools, 'replace', path, before, after);
}
