// Preserve bounded, fully observed shell bytes before rendering previews.
import {boundedCommandSync} from '../../js/server/bounded-command.mjs';
import {createHash} from 'node:crypto';
import {mkdtempSync, rmSync, openSync, closeSync, writeSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

// Prior regular-file stdio made Node writes synchronous even before abrupt process.exit.
// Restore that producer behavior only inside explicitly marked capture children.
if (process.env.FORMAL_AI_DOGFOOD_BLOCKING_STDIO === '1') {
  for (const stream of [process.stdout, process.stderr]) stream._handle?.setBlocking?.(true);
}

function captureEnvironment(environment) {
  const inherited = process.env.NODE_OPTIONS ?? '';
  const supplied = environment?.NODE_OPTIONS ?? '';
  const previous = supplied === inherited ? inherited : [inherited, supplied].filter(Boolean).join(' ');
  const preload = '--import=' + JSON.stringify(import.meta.url);
  return {...(environment ?? process.env), FORMAL_AI_DOGFOOD_BLOCKING_STDIO: '1',
    NODE_OPTIONS: previous.includes(preload) ? previous : [previous, preload].filter(Boolean).join(' ')};
}

const DEFAULT_PREVIEW_BYTES = 256 * 1024;
const MAXIMUM_CAPTURE_BYTES = 64 * 1024 * 1024;

/** One shared raw-capture writer; failed progress cannot spin or certify missing bytes. */
export function writeCaptureBytes(file, bytes) {
  const descriptor = openSync(file, 'wx', 0o600);
  try {
    let offset = 0;
    while (offset < bytes.length) {
      const remaining = bytes.length - offset;
      const count = writeSync(descriptor, bytes, offset, remaining);
      if (!Number.isSafeInteger(count) || count <= 0 || count > remaining) {
        throw new Error('InvalidCaptureWriteProgress');
      }
      offset += count;
    }
  } finally { closeSync(descriptor); }
}

/** Freeze the actual bounded bytes; previews never certify omitted command output. */
function snapshot(raw, destination, maximum, complete) {
  writeCaptureBytes(destination, raw);
  const bytes = raw.length;
  const text = bytes <= maximum ? raw.toString('utf8') : null;
  const exactUtf8 = text !== null && Buffer.from(text).equals(raw);
  return {
    path: destination, bytes, sha256: createHash('sha256').update(raw).digest('hex'), complete,
    text, exactUtf8,
    preview: text ?? raw.subarray(0, Math.ceil(maximum / 2)).toString('utf8')
      + '\n[output preview truncated; complete observed bytes retained]\n'
      + raw.subarray(-Math.floor(maximum / 2)).toString('utf8'),
  };
}

/** Bounded previews are transport metadata, never stdout byte certificates. */
export function shellCapture(command, {cwd, timeoutMs = 60000, maxOutputBytes = DEFAULT_PREVIEW_BYTES,
  environment, onProcessObservation} = {}) {
  if (typeof command !== 'string') throw new TypeError('shell command must be a string');
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0 || timeoutMs > 2147482647) throw new RangeError('shell timeout must be positive');
  if (!Number.isSafeInteger(maxOutputBytes) || maxOutputBytes < 2) {
    throw new RangeError('shell preview limit must be at least two bytes');
  }
  if (onProcessObservation !== undefined && typeof onProcessObservation !== 'function') {
    throw Error('shell_process_observer_required');
  }
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-shell-capture-'));
  let retain = false;
  try {
    const result = boundedCommandSync('/bin/sh', ['-c', command], {
      cwd, env: captureEnvironment(environment), timeout: timeoutMs,
      stdoutLimitBytes: MAXIMUM_CAPTURE_BYTES, stderrLimitBytes: MAXIMUM_CAPTURE_BYTES,
    });
    const complete = !result.error && result.cleanup_complete;
    const stdout = snapshot(result.stdout, join(directory, 'stdout.raw'), maxOutputBytes, complete);
    const stderr = snapshot(result.stderr, join(directory, 'stderr.raw'), maxOutputBytes, complete);
    if (onProcessObservation !== undefined) {
      onProcessObservation({command, cwd: cwd ?? process.cwd(), pid: result.pid ?? null,
        status: result.status, signal: result.signal ?? null,
        error: result.error ? {name: result.error.name, code: result.error.code ?? null,
          message: result.error.message} : null, stdout, stderr});
    }
    const truncated = stdout.bytes > maxOutputBytes || stderr.bytes > maxOutputBytes;
    const invalidUtf8 = (!stdout.exactUtf8 && stdout.text !== null) || (!stderr.exactUtf8 && stderr.text !== null);
    if (truncated || invalidUtf8 || result.error || !result.cleanup_complete) {
      retain = true;
      const capture = ({text, exactUtf8, preview, ...receipt}) => ({...receipt, preview});
      const timedOut = result.timed_out === true;
      return JSON.stringify({
        is_error: true,
        error: timedOut ? 'shell command timed out' : result.error?.message
          ?? (!result.cleanup_complete ? 'shell command cleanup incomplete'
            : truncated ? 'shell output preview is incomplete' : 'shell output is not exact UTF-8 text'),
        exit_code: typeof result.status === 'number' ? result.status : null,
        signal: result.signal ?? null, timeout_ms: timedOut ? timeoutMs : null,
        command_output_complete: complete && !result.signal,
        capture_scope: 'bytes observed when the shell process stopped',
        stdout_capture: capture(stdout), stderr_capture: capture(stderr),
      });
    }
    if (result.signal) return 'Output: '+stdout.text+'\nError: '+stderr.text+'\nSignal: '+result.signal;
    if (typeof result.status !== 'number') throw new Error('shell process returned no exit status');
    if (result.status === 0 && stderr.bytes === 0) return 'Output: '+stdout.text+'\nExit Code: 0';
    return 'Output: '+stdout.text+'\nError: '+stderr.text+'\nExit Code: '+result.status;
  } finally {
    if (!retain) rmSync(directory, {recursive: true, force: true});
  }
}
