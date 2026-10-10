// Reading a turn's tool results back out of the transcript (issue #468): a
// port of rust/src/agentic_coding/progress.rs.
//
// `Progress` keeps the Rust field names (`run_outputs`, `fetched_pages`, ...);
// its methods are camelCase. A `ToolAttempt` is `{capability, succeeded,
// detail, arguments, tool}` with `arguments` / `tool` null when unknown.

import { plainText, rustLines } from './content.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { wordsForRole } from './crate/seed_meanings.mjs';
import { agenticMessage } from './messages.mjs';
import { Capability } from './capability.mjs';
import { writeArguments } from './plan.mjs';
import { fill } from './work_item_steps.mjs';
import { classifyTool } from './capability_router.mjs';
import { commandArgument, failureMessage, normalizedPayload, sourceReadObservation } from './tool_result.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { applyPatchInput } from './crate/protocol_responses_apply_patch.mjs';
import {
  eqIgnoreAsciiCase, isObject, parseJson, splitWhitespace, trim, trimMatches,
} from './crate/rust_str.mjs';

/** Mirrors `ToolAttempt::is_work_item_read`. */
export function isWorkItemRead(attempt) {
  if (attempt.capability !== Capability.Run || attempt.arguments === null) return false;
  const command = commandArgument(attempt.arguments);
  return command !== null && isWorkItemReadCommand(command);
}

/** Mirrors `struct Progress` in rust/src/agentic_coding/progress.rs. */
export class Progress {
  constructor() {
    this.completed = [];
    this.attempts = [];
    this.fetched_text = null;
    this.fetched_pages = [];
    this.attempted_fetches = [];
    this.attempted_work_item_reads = [];
    this.failed_work_item_reads = [];
    this.search_output = null;
    this.run_outputs = [];
    this.run_observations = [];
    this.search_result = null;
  }

  /** Mirrors `Progress::scan`. */
  static scan(messages) {
    const progress = new Progress();
    const currentTurn = evidenceWindowStart(messages);
    for (let index = currentTurn; index < messages.length; index += 1) {
      const message = messages[index];
      if (message.role.toLowerCase() !== 'tool') continue;
      const capability = resultCapability(messages, index);
      if (capability === null) continue;
      const raw = plainText(message.content);
      const call = resultToolCall(messages, index, currentTurn);
      const path = call === null ? null : sourceReadPath(call.function.arguments);
      const boundRead = capability === Capability.Read && call !== null && path !== null
        && classifyTool(call.function.name) === Capability.Read
        && (!message.name || eqIgnoreAsciiCase(message.name, call.function.name));
      const observation = boundRead ? sourceReadObservation(raw, Boolean(message.is_error || message.isError),
        message.source_read ?? message.sourceRead ?? null, path) : null;
      const failure = observation !== null ? observation.error
        : capability === Capability.Fetch && standaloneHttpFailure(raw) ? raw
          : capability === Capability.Read || (capability === Capability.Fetch && (parseJson(raw) !== undefined || !legacyFetchNotice(raw)))
          ? (message.is_error || message.isError ? raw : null)
          : failureMessage(raw, Boolean(message.is_error || message.isError), capability !== Capability.Run);
      progress.attempts.push({
        capability,
        source_read: observation,
        succeeded: failure === null,
        detail: failure ?? raw,
        arguments: call ? call.function.arguments : null,
        tool: message.name ?? (call ? call.function.name : null),
      });
      if (capability === Capability.Fetch) {
        const payload = raw;
        const fetchUrl = call ? argumentUrl(call.function.arguments) : null;
        if (fetchUrl !== null && !progress.attempted_fetches.includes(fetchUrl)) {
          progress.attempted_fetches.push(fetchUrl);
        }
        if (failure !== null && fetchUrl !== null) progress.failed_work_item_reads.push([fetchUrl, failure]);
        if (failure === null && payload !== null && trim(payload)) {
          if (fetchUrl !== null) progress.fetched_pages.push([fetchUrl, payload]);
          progress.fetched_text = payload;
        }
      }
      if (capability === Capability.Search) {
        const payload = normalizedPayload(raw);
        progress.search_result = payload ?? '';
        if (payload !== null && trim(payload)) progress.search_output = payload;
      }
      if (capability === Capability.Run) {
        const command = call ? commandArgument(call.function.arguments) : null;
        if (command !== null) progress.run_observations.push([command, raw]);
        const url = command !== null ? workItemReadUrl(command) : null;
        if (url !== null) {
          if (!progress.attempted_work_item_reads.includes(url)) progress.attempted_work_item_reads.push(url);
          const reason = workItemReadFailureReason(command, raw, failure);
          if (reason !== null) {
            progress.failed_work_item_reads.push([url, reason]);
          } else if (failure === null) {
            const text = normalizedPayload(raw);
            if (text !== null && trim(text)) progress.fetched_pages.push([url, withoutExitSentinel(text)]);
          }
        }
        progress.run_outputs.push(raw);
      }
      progress.completed.push(capability);
    }
    return progress;
  }

  /** Mirrors `Progress::attempted_fetch_of`. */
  attemptedFetchOf(url) {
    return this.attempted_fetches.includes(url)
      || this.attempts.some((attempt) => attempt.capability === Capability.Fetch
        && (attempt.arguments === null || argumentUrl(attempt.arguments) === null));
  }

  /** Mirrors `Progress::attempted_work_item_read_of`. */
  attemptedWorkItemReadOf(url) {
    return this.attempted_work_item_reads.includes(url);
  }

  /** Mirrors `Progress::failed_work_item_read_of`. */
  failedWorkItemReadOf(url) {
    if (this.fetched_pages.some(([fetched]) => fetched === url)) return null;
    for (let index = this.failed_work_item_reads.length - 1; index >= 0; index -= 1) {
      const [failed, reason] = this.failed_work_item_reads[index];
      if (failed === url) return reason;
    }
    return null;
  }

  /** Mirrors `Progress::work_item_read_attempts`. */
  workItemReadAttempts(url) {
    const reasons = this.failed_work_item_reads.filter(([failed]) => failed === url);
    let nextReason = 0;
    const lines = [];
    for (const attempt of this.attempts) {
      if (attempt.capability === Capability.Run) {
        const command = attempt.arguments === null ? null : commandArgument(attempt.arguments);
        if (command === null) continue;
        const read = issueViewCommandUrl(command) ?? restReadUrl(command);
        if (read === null || read !== url) continue;
        const outcome = trim(rustLines(attempt.detail)[0] ?? '');
        const reason = nextReason < reasons.length ? reasons[nextReason++][1] : null;
        lines.push(reason === null
          ? fill('read_attempt_run_line', [['{command}', command], ['{outcome}', outcome]])
          : fill('read_attempt_run_reason_line', [['{command}', command], ['{reason}', reason], ['{outcome}', outcome]]));
      } else if (attempt.capability === Capability.Fetch) {
        const fetches = this.attempts.filter((candidate) => candidate.capability === Capability.Fetch).length;
        const fetched = attempt.arguments === null ? null : argumentUrl(attempt.arguments);
        const forThisUrl = fetched === url || (fetched === null && fetches === 1);
        if (forThisUrl) {
          const outcome = trim(rustLines(attempt.detail)[0] ?? '');
          lines.push(fill('read_attempt_fetch_line', [['{url}', url], ['{outcome}', outcome]]));
          if (!attempt.succeeded && fetched === url && nextReason < reasons.length) nextReason++;
        }
      }
    }
    return lines;
  }

  /** Mirrors `Progress::has_run`. */
  hasRun(command) {
    return this.run_observations.some(([ran]) => ran === command);
  }

  /** Mirrors `Progress::repeated_call`: the attempt `call` repeats, or null. */
  repeatedCall(call) {
    let index = -1;
    for (let at = this.attempts.length - 1; at >= 0; at -= 1) {
      const attempt = this.attempts[at];
      if (!attempt.succeeded) continue;
      const sameTool = (attempt.tool !== null && eqIgnoreAsciiCase(attempt.tool, call.tool))
        || (attempt.tool === null && classifyTool(call.tool) === attempt.capability);
      if (sameTool && (attempt.arguments === call.arguments || sameRunOperand(attempt, call))) {
        index = at;
        break;
      }
    }
    if (index < 0) return null;
    const later = this.attempts.slice(index + 1);
    const progressed = later.some((attempt, offset) =>
      !this.attempts.slice(0, index + offset + 1).some((earlier) => sameAttempt(earlier, attempt)));
    return progressed ? null : this.attempts[index];
  }

  /** Mirrors `Progress::latest_failure_of_tool`. */
  latestFailureOfTool(tool) {
    for (let index = this.attempts.length - 1; index >= 0; index -= 1) {
      const attempt = this.attempts[index];
      if (!attempt.succeeded && attempt.tool === tool) return attempt;
    }
    return null;
  }

  /** Mirrors `Progress::identical_failures_of`. */
  identicalFailuresOf(tool) {
    const latest = this.latestFailureOfTool(tool);
    if (latest === null) return 0;
    return this.attempts.filter((attempt) =>
      !attempt.succeeded && attempt.tool === tool && attempt.detail === latest.detail).length;
  }

  /** Mirrors `Progress::done`. */
  done(capability) {
    return this.completed.includes(capability);
  }

  /** Mirrors `Progress::count`. */
  count(capability) {
    return this.completed.filter((done) => done === capability).length;
  }

  #latestSuccess(capability) {
    for (let index = this.attempts.length - 1; index >= 0; index -= 1) {
      const attempt = this.attempts[index];
      if (attempt.capability === capability && attempt.succeeded) return index;
    }
    return -1;
  }

  /** Mirrors `Progress::latest_successful_output`. */
  latestSuccessfulOutput(capability) {
    const index = this.#latestSuccess(capability);
    return index < 0 ? null : this.attempts[index].detail;
  }

  /** Mirrors `Progress::latest_successful_arguments`. */
  latestSuccessfulArguments(capability) {
    const index = this.#latestSuccess(capability);
    return index < 0 ? null : this.attempts[index].arguments;
  }

  /** Mirrors `Progress::latest_success_unstalled`. */
  latestSuccessUnstalled(capability) {
    const index = this.#latestSuccess(capability);
    if (index < 0) return false;
    return this.attempts.slice(index + 1).every((attempt) => attempt.succeeded);
  }

  /** Mirrors `Progress::run_count_for`. */
  runCountFor(command) {
    return this.attempts.filter((attempt) => runAttemptMatches(attempt, command)).length;
  }

  /** Mirrors `Progress::successful_run_count_for`. */
  successfulRunCountFor(command) {
    return this.attempts.filter((attempt) => attempt.succeeded && runAttemptMatches(attempt, command)).length;
  }

  /** Mirrors `Progress::latest_successful_run_output_for`. */
  latestSuccessfulRunOutputFor(command) {
    for (let index = this.attempts.length - 1; index >= 0; index -= 1) {
      const attempt = this.attempts[index];
      if (attempt.succeeded && runAttemptMatches(attempt, command)) return attempt.detail;
    }
    return null;
  }

  /** Mirrors `Progress::latest_run_output_for`. */
  latestRunOutputFor(command) {
    for (let index = this.run_observations.length - 1; index >= 0; index -= 1) {
      const [attempted, output] = this.run_observations[index];
      if (attempted === command) return output;
    }
    return null;
  }

  /** Mirrors Progress::source_read_for: the latest exact current-window source observation. */
  sourceReadFor(path) {
    for (let index = this.attempts.length - 1; index >= 0; index -= 1) {
      const attempt = this.attempts[index];
      if (attempt.capability === Capability.Read && attempt.arguments !== null
        && sourceReadPath(attempt.arguments) === path) return attempt.source_read ?? null;
    }
    return null;
  }

  /** Mirrors `Progress::successful_read_output_for`. */
  successfulReadOutputFor(path) {
    for (let index = this.attempts.length - 1; index >= 0; index -= 1) {
      const attempt = this.attempts[index];
      if (attempt.capability === Capability.Read && attempt.succeeded
        && attempt.arguments !== null && argumentTargets(attempt.arguments, path)) return attempt.detail;
    }
    return null;
  }

  /** Mirrors `Progress::attempted_write_for`. */
  attemptedWriteFor(path) {
    return this.attempts.some((attempt) => attempt.capability === Capability.Write
      && attempt.arguments !== null && argumentTargets(attempt.arguments, path));
  }

  /** Mirrors `Progress::successful_write_for`. */
  successfulWriteFor(path) {
    return this.attempts.some((attempt) => attempt.capability === Capability.Write && attempt.succeeded
      && attempt.arguments !== null && argumentTargets(attempt.arguments, path));
  }

  /** Mirrors `Progress::latest_successful_write_index`. */
  latestSuccessfulWriteIndex(path) {
    for (let index = this.attempts.length - 1; index >= 0; index -= 1) {
      const attempt = this.attempts[index];
      if (attempt.capability === Capability.Write && attempt.succeeded
        && attempt.arguments !== null && argumentTargets(attempt.arguments, path)) return index;
    }
    return null;
  }

  /** Mirrors `Progress::attempts_after_latest_write`: observation window for the current write. */
  attemptsAfterLatestWrite(path) {
    const index = this.latestSuccessfulWriteIndex(path);
    return index === null ? null : this.attempts.slice(index + 1);
  }

  /** Mirrors `Progress::successful_write_content_for`. */
  successfulWriteContentFor(path) {
    for (let index = this.attempts.length - 1; index >= 0; index -= 1) {
      const attempt = this.attempts[index];
      if (attempt.capability !== Capability.Write || !attempt.succeeded || attempt.arguments === null) continue;
      if (!argumentTargets(attempt.arguments, path)) continue;
      const content = argumentContent(attempt.arguments);
      if (content !== null) return content;
    }
    return null;
  }

  /** Mirrors `Progress::last`. */
  last() {
    return this.completed.length ? this.completed[this.completed.length - 1] : null;
  }

  /** Mirrors `Progress::latest_failure`. */
  latestFailure() {
    const attempt = this.attempts[this.attempts.length - 1];
    return attempt && !attempt.succeeded ? attempt : null;
  }

  /** Mirrors `Progress::previous_attempt`. */
  previousAttempt() {
    return this.attempts.length >= 2 ? this.attempts[this.attempts.length - 2] : null;
  }

  /** Mirrors `Progress::failed_write_count_for`. */
  failedWriteCountFor(path) {
    return this.attempts.filter((attempt) => attempt.capability === Capability.Write && !attempt.succeeded
      && attempt.arguments !== null && argumentTargets(attempt.arguments, path)).length;
  }

  /** Mirrors `Progress::search_result`. */
  searchResult() {
    return this.search_result;
  }
}

/** Mirrors `Progress::same_attempt`. */
function sameAttempt(earlier, later) {
  const sameTool = earlier.tool !== null && later.tool !== null
    ? eqIgnoreAsciiCase(earlier.tool, later.tool)
    : earlier.capability === later.capability;
  return sameTool && earlier.arguments === later.arguments;
}

/** Mirrors `Progress::same_run_operand`. */
function sameRunOperand(attempt, call) {
  const attemptedCommand = attempt.arguments === null ? null : commandArgument(attempt.arguments);
  const plannedCommand = commandArgument(call.arguments);
  if (attemptedCommand !== null && plannedCommand === attemptedCommand) return true;
  const attemptedUrl = attempt.arguments === null ? null : argumentUrl(attempt.arguments);
  return attemptedUrl !== null && argumentUrl(call.arguments) === attemptedUrl;
}

function firstStringField(argumentsText, keys) {
  const value = parseJson(argumentsText);
  if (!isObject(value)) return null;
  for (const key of keys) if (typeof value[key] === 'string') return value[key];
  return null;
}

/** Mirrors `fn argument_path`. */
/** Mirrors fn source_read_path: conflicting naming fields do not bind source bytes. */
function sourceReadPath(argumentsText) {
  const value = parseJson(argumentsText);
  if (!isObject(value)) return null;
  const fields = ['path', 'filePath', 'file_path'].filter((key) => Object.hasOwn(value, key)).map((key) => value[key]);
  return fields.length > 0 && fields.every((field) => typeof field === 'string' && trim(field) !== '' && field === fields[0]) ? fields[0] : null;
}

function argumentPath(argumentsText) {
  return firstStringField(argumentsText, ['path', 'filePath', 'file_path']);
}

/** Mirrors `fn argument_content`. */
function argumentContent(argumentsText) {
  return firstStringField(argumentsText, ['content', 'contents', 'text', 'new_string']);
}

/** Mirrors `fn write_matches`. */
export function writeMatches(argumentsText, path, content) {
  if (argumentTargets(argumentsText, path) && argumentContent(argumentsText) === content) return true;
  const patch = applyPatchInput(writeArguments(path, content));
  return patch !== null && trim(argumentsText) === trim(patch);
}

/** `Path::components` reduced to comparable strings. */
function pathComponents(path) {
  const out = [];
  if (path.startsWith('/')) out.push('/');
  path.split('/').forEach((part) => {
    if (!part) return;
    if (part === '.' && !(out.length === 0 && !path.startsWith('/'))) return;
    out.push(part);
  });
  return out;
}

/** `Path::ends_with`. */
function pathEndsWith(path, child) {
  const whole = pathComponents(path);
  const tail = pathComponents(child);
  if (tail.length > whole.length) return false;
  return tail.every((part, index) => whole[whole.length - tail.length + index] === part);
}

/** Mirrors `fn argument_targets`. */
function argumentTargets(argumentsText, path) {
  const observed = argumentPath(argumentsText);
  return observed !== null && (observed === path || (!path.startsWith('/') && pathEndsWith(observed, path)));
}

/** Mirrors `fn result_capability`. */
export function resultCapability(messages, index) {
  const message = messages[index];
  if (message.name) {
    const capability = classifyTool(message.name);
    if (capability !== null) return capability;
  }
  const call = resultToolCall(messages, index);
  return call ? classifyTool(call.function.name) : null;
}

/** Mirrors `fn result_tool_call`. */
function resultToolCall(messages, index, start = evidenceWindowStart(messages)) {
  const id = messages[index].tool_call_id;
  if (id === null || id === undefined) return null;
  for (let at = index - 1; at >= start; at -= 1) {
    const call = (messages[at].tool_calls || []).find((candidate) => candidate.id === id);
    if (call) return call;
  }
  return null;
}

/** Mirrors `fn argument_url`. */
function argumentUrl(argumentsText) {
  const url = firstStringField(argumentsText, ['url']);
  return url !== null && trim(url) ? url : null;
}

/** Mirrors `fn run_attempt_matches`. */
function runAttemptMatches(attempt, command) {
  return attempt.capability === Capability.Run && attempt.arguments !== null
    && commandArgument(attempt.arguments) === command;
}

/** Mirrors `fn is_issue_view_command`. */
function isIssueViewCommand(command) {
  const words = splitWhitespace(command);
  return words[0] === 'gh' && (words[1] === 'issue' || words[1] === 'pr') && words[2] === 'view';
}

/** Mirrors `fn is_work_item_read_command`. */
function isWorkItemReadCommand(command) {
  return isIssueViewCommand(command)
    || ((command.startsWith('gh api ') || command.startsWith('curl ')) && restReadUrl(command) !== null);
}

/** Mirrors `fn work_item_read_url`. */
function workItemReadUrl(command) {
  return issueViewCommandUrl(command) ?? restReadUrl(command);
}

/** Mirrors `fn issue_view_command_url`. */
function issueViewCommandUrl(command) {
  const words = splitWhitespace(command);
  if (words[0] !== 'gh') return null;
  if ((words[1] !== 'issue' && words[1] !== 'pr') || words[2] !== 'view') return null;
  return words[3] === undefined ? null : repositoryWorkReference(words[3]);
}

const WORK_REFERENCE_TRIM = new Set(Array.from('<>()[]{},;."\'。，、；：（）「」«»।'));

/**
 * Mirrors `fn repository_work_reference` in
 * rust/src/agentic_coding/general_planner.rs (a local copy: that module is
 * ported elsewhere).
 */
export function repositoryWorkReference(request) {
  for (const token of splitWhitespace(request)) {
    const url = trimMatches(token, (character) => WORK_REFERENCE_TRIM.has(character));
    let path = null;
    if (url.startsWith('https://github.com/')) path = url.slice('https://github.com/'.length);
    else if (url.startsWith('http://github.com/')) path = url.slice('http://github.com/'.length);
    if (path === null) continue;
    const segments = path.split('/');
    if (segments.length === 4 && segments[0] && segments[1] && (segments[2] === 'issues' || segments[2] === 'pull')
      && /^[0-9]*$/.test(segments[3])) return url;
  }
  return null;
}

/** Mirrors `fn rest_read_url`. */
function restReadUrl(command) {
  const at = command.indexOf('repos/');
  if (at < 0) return null;
  const segments = command.slice(at + 'repos/'.length).split('/');
  const [owner, repo, kindSegment, numberSegment] = segments;
  if (!owner || !repo || kindSegment === undefined) return null;
  if (kindSegment !== 'issues' && kindSegment !== 'pulls') return null;
  if (numberSegment === undefined) return null;
  const number = /^[0-9]*/.exec(numberSegment)[0];
  if (!number) return null;
  const kind = kindSegment === 'pulls' ? 'pull' : 'issues';
  return `https://github.com/${owner}/${repo}/${kind}/${number}`;
}

/** Mirrors `fn exit_sentinel`. */
function exitSentinel(raw) {
  let last = null;
  for (const line of rustLines(raw)) {
    const trimmed = trim(line);
    if (!trimmed.startsWith('__formal_ai_exit=')) continue;
    const digits = trimmed.slice('__formal_ai_exit='.length);
    if (/^[+-]?[0-9]+$/.test(digits)) {
      const value = Number(digits);
      if (value >= -2147483648 && value <= 2147483647) last = value;
    }
  }
  return last;
}

/** Mirrors `fn without_exit_sentinel`. */
function withoutExitSentinel(text) {
  return rustLines(text).filter((line) => !trim(line).startsWith('__formal_ai_exit=')).join('\n');
}

/** Mirrors `fn work_item_read_failure_reason`. */
function workItemReadFailureReason(command, raw, echoedFailure) {
  if (echoedFailure !== null) return echoedFailure;
  const status = exitSentinel(raw);
  if (status !== null && status !== 0) return fill('read_failure_exit_status', [['{status}', String(status)]]);
  const text = withoutExitSentinel(raw);
  const curl = command.startsWith('curl ');
  const shaped = curl ? Boolean(trim(text)) && !trim(text).startsWith('{') : text.includes('\n\n');
  if (shaped) return null;
  return agenticMessage(curl ? 'progress_rest_answer_not_issue_text' : 'progress_output_not_work_item_shape');
}

/** Mirrors fn legacy_fetch_notice: source prose is not provider failure merely for mentioning one. */
function legacyFetchNotice(raw) {
  const normalized = normalizePrompt(raw);
  return wordsForRole('tool_result_failure_signal').some((surface) => {
    const prefix = normalizePrompt(surface);
    if (!normalized.startsWith(prefix)) return false;
    const next = Array.from(normalized.slice(prefix.length))[0];
    return next === undefined || !/[\p{Alphabetic}\p{N}]/u.test(next);
  });
}

/** A whole single-line protocol failure is transport status, not an article quotation. */
function standaloneHttpFailure(raw) {
  return /^HTTP\/[0-9]+(?:\.[0-9]+)?[ \t]+[45][0-9]{2}:?(?:[ \t]+[^\x00-\x08\x0a-\x1f\x7f\u0085\u2028\u2029\ufeff]*)?$/u.test(raw.replace(/^[ \t\r\n]+|[ \t\r\n]+$/gu, ''));
}
