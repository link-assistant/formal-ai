// Current-window tool records; source status is owned by the provider boundary.
import { Capability } from '../capability.mjs';
import { classifyTool } from '../capability_router.mjs';
import { plainText } from '../content.mjs';
import { commandArgument } from '../tool_result.mjs';
import { jsonText } from '../plan.mjs';

/** Mirrors `fn tool_result_records`: `{capability, arguments, content}` per current-turn result. */
export function toolResultRecords(messages) {
  let turnStart = 0;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role.toLowerCase() === 'user') {
      turnStart = index + 1;
      break;
    }
  }
  const records = [];
  for (let index = turnStart; index < messages.length; index += 1) {
    const message = messages[index];
    if (message.role.toLowerCase() !== 'tool') continue;
    const call = toolResultCall(messages, index, turnStart);
    let capability = null;
    let args = null;
    if (call) {
      capability = classifyTool(call.function.name);
      try {
        args = JSON.parse(call.function.arguments);
      } catch {
        args = null;
      }
    }
    records.push({ capability, arguments: args, content: plainText(message.content), is_error: Boolean(message.is_error || message.isError) });
  }
  return records;
}

/** Mirrors fn tool_result_call in file_read/records.rs: current-window call and tool identity. */
function toolResultCall(messages, index, start) {
  const message = messages[index];
  if (message.tool_call_id == null) return null;
  for (let prior = index - 1; prior >= start; prior -= 1) {
    const call = (messages[prior].tool_calls ?? []).findLast((candidate) => candidate.id === message.tool_call_id);
    if (call) return !message.name || message.name.toLowerCase() === call.function.name.toLowerCase() ? call : null;
  }
  return null;
}

const stringField = (value, key) =>
  (value && typeof value === 'object' && !Array.isArray(value) && typeof value[key] === 'string' ? value[key] : null);

/** Mirrors fn read_record_for_path in file_read/records.rs: conflicting path fields cannot bind. */
export function readRecordForPath(records, path) {
  return records.findLast((record) => {
    if (record.capability !== Capability.Read || record.arguments === null) return false;
    const fields = ['path', 'filePath', 'file_path', 'absolute_path']
      .filter((key) => Object.hasOwn(record.arguments, key)).map((key) => record.arguments[key]);
    return fields.length > 0 && fields.every((field) => typeof field === 'string' && field.trim() !== '' && field === fields[0])
      && samePath(fields[0], path);
  }) ?? null;
}
/** Mirrors fn read_result_for_path in file_read/records.rs. */
export function readResultForPath(records, path) {
  return readRecordForPath(records, path)?.content ?? null;
}

/** Mirrors `fn grep_result_for_path`. */
export function grepResultForPath(records, path, pattern) {
  const found = records.find((record) => {
    if (record.capability !== Capability.Grep) return false;
    const recorded = pathArgument(record.arguments);
    return recorded !== null && samePath(recorded, path) && stringField(record.arguments, 'pattern') === pattern;
  });
  return found ? found.content : null;
}

/** Mirrors `fn same_path`. */
export function samePath(recorded, planned) {
  if (recorded === planned) return true;
  const wanted = trimStartMatchesStr(planned, './');
  const actual = trimStartMatchesStr(recorded, './');
  if (!actual.endsWith(wanted)) return false;
  const prefix = actual.slice(0, actual.length - wanted.length);
  return prefix === '' || prefix.endsWith('/');
}

/** `str::trim_start_matches(&str)`. */
function trimStartMatchesStr(text, prefix) {
  let rest = text;
  while (prefix && rest.startsWith(prefix)) rest = rest.slice(prefix.length);
  return rest;
}

/** Mirrors `fn run_record_for_command`. */
export function runRecordForCommand(records, command) {
  const found = records.find((record) => record.capability === Capability.Run
    && commandArgument(jsonText(record.arguments)) === command);
  return found ? found.content : null;
}

/** Mirrors `fn path_argument`. */
function pathArgument(args) {
  for (const key of ['filePath', 'path', 'file_path', 'absolute_path']) {
    const value = stringField(args, key);
    if (value !== null) return value;
  }
  return null;
}
