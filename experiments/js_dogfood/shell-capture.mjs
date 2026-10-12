// Preserve shell output as regular-file bytes before rendering bounded results.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {closeSync, mkdtempSync, openSync, readSync, rmSync, statSync, unlinkSync, writeSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

const DEFAULT_PREVIEW_BYTES = 256 * 1024;
const BLOCK_BYTES = 32 * 1024;

/** Freeze the bytes observed at command completion; inherited live FDs cannot alter receipts. */
function snapshot(file, destination, maximum) {
  const bytes = statSync(file).size;
  const input = openSync(file, 'r'), output = openSync(destination, 'wx', 0o600);
  const digest = createHash('sha256'), buffer = Buffer.alloc(BLOCK_BYTES);
  const chunks = [], head = Buffer.alloc(Math.ceil(maximum / 2));
  let offset = 0, headBytes = 0, tail = Buffer.alloc(0);
  try {
    while (offset < bytes) {
      const size = readSync(input, buffer, 0, Math.min(buffer.length, bytes - offset), offset);
      if (!size) throw new Error('shell capture changed before its observed snapshot completed');
      const chunk = buffer.subarray(0, size);
      digest.update(chunk);
      let written = 0;
      while (written < size) written += writeSync(output, chunk, written, size - written);
      if (bytes <= maximum) chunks.push(Buffer.from(chunk));
      else {
        const first = Math.min(size, head.length - headBytes);
        if (first) { chunk.copy(head, headBytes, 0, first); headBytes += first; }
        tail = Buffer.concat([tail, chunk]).subarray(-Math.floor(maximum / 2));
      }
      offset += size;
    }
  } finally { closeSync(input); closeSync(output); }
  const raw = bytes <= maximum ? Buffer.concat(chunks) : null;
  const exactUtf8 = raw !== null && Buffer.from(raw.toString('utf8')).equals(raw);
  return {
    path: destination, bytes, sha256: digest.digest('hex'), complete: true,
    text: raw?.toString('utf8') ?? null, exactUtf8,
    preview: raw ? raw.toString('utf8') : head.subarray(0, headBytes).toString('utf8')
      + '\n[output preview truncated; complete observed bytes retained]\n' + tail.toString('utf8'),
  };
}

/** Bounded previews are transport metadata, never stdout byte certificates. */
export function shellCapture(command, {cwd, timeoutMs = 60000, maxOutputBytes = DEFAULT_PREVIEW_BYTES} = {}) {
  if (typeof command !== 'string') throw new TypeError('shell command must be a string');
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) throw new RangeError('shell timeout must be positive');
  if (!Number.isSafeInteger(maxOutputBytes) || maxOutputBytes < 2) throw new RangeError('shell preview limit must be at least two bytes');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-shell-capture-'));
  const stdoutLive = join(directory, 'stdout.live'), stderrLive = join(directory, 'stderr.live');
  let output = null, error = null, retain = false;
  try {
    output = openSync(stdoutLive, 'wx', 0o600); error = openSync(stderrLive, 'wx', 0o600);
    const result = spawnSync('/bin/sh', ['-c', command], {
      cwd, stdio: ['ignore', output, error], timeout: timeoutMs,
      killSignal: 'SIGKILL', detached: process.platform !== 'win32',
    });
    if (result.error?.code === 'ETIMEDOUT' && result.pid && process.platform !== 'win32') {
      try { process.kill(-result.pid, 'SIGKILL'); }
      catch (failure) { if (failure.code !== 'ESRCH') throw failure; }
    }
    closeSync(output); output = null; closeSync(error); error = null;
    const stdout = snapshot(stdoutLive, join(directory, 'stdout.raw'), maxOutputBytes);
    const stderr = snapshot(stderrLive, join(directory, 'stderr.raw'), maxOutputBytes);
    unlinkSync(stdoutLive); unlinkSync(stderrLive);
    const truncated = stdout.bytes > maxOutputBytes || stderr.bytes > maxOutputBytes;
    const invalidUtf8 = (!stdout.exactUtf8 && stdout.text !== null) || (!stderr.exactUtf8 && stderr.text !== null);
    if (truncated || invalidUtf8 || result.error) {
      retain = true;
      const capture = ({text, exactUtf8, preview, ...receipt}) => ({...receipt, preview});
      const timedOut = result.error?.code === 'ETIMEDOUT';
      return JSON.stringify({
        is_error: true,
        error: timedOut ? 'shell command timed out' : result.error?.message
          ?? (truncated ? 'shell output preview is incomplete' : 'shell output is not exact UTF-8 text'),
        exit_code: typeof result.status === 'number' ? result.status : null,
        signal: result.signal ?? null, timeout_ms: timedOut ? timeoutMs : null,
        command_output_complete: !result.error && !result.signal,
        capture_scope: 'bytes observed when the shell process stopped',
        stdout_capture: capture(stdout), stderr_capture: capture(stderr),
      });
    }
    if (result.signal) return 'Output: '+stdout.text+'\nError: '+stderr.text+'\nSignal: '+result.signal;
    if (typeof result.status !== 'number') throw new Error('shell process returned no exit status');
    if (result.status === 0 && stderr.bytes === 0) return 'Output: '+stdout.text+'\nExit Code: 0';
    return 'Output: '+stdout.text+'\nError: '+stderr.text+'\nExit Code: '+result.status;
  } finally {
    if (output !== null) closeSync(output);
    if (error !== null) closeSync(error);
    if (!retain) rmSync(directory, {recursive: true, force: true});
  }
}
