// Render retained child diagnostics only; no native invocation or authority grant.
import { writeSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
const chunkBytes = 64 * 1024,
  jsonLineBytes = 1024 * 1024;
function stderr(bytes) {
  for (let offset = 0; offset < bytes.length;) {
    let count;
    try { count = writeSync(2, bytes, offset, bytes.length - offset); }
    catch (error) { error.diagnosticBytesWritten = offset; throw error; }
    if (count <= 0) throw Error('diagnostic_stderr_closed');
    offset += count;
  }
}
export function failedNativeProcess(result) {
  return Boolean(result.error || result.signal || result.status !== 0);
}
export function reportNativeProcessFailure(stage, result, {
  cargoJson = false,
  write = stderr,
  wait = () => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 2),
  now = Date.now
} = {}) {
  if (!['build', 'producer'].includes(stage)) throw Error('invalid_diagnostic_stage');
  if (!failedNativeProcess(result)) return false;
  const deadline = now() + 2000;
  let transportClosed = false;
  const emit = value => {
    if (transportClosed) return;
    const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value);
    for (let offset = 0; offset < bytes.length; offset += chunkBytes) {
      let pending = bytes.subarray(offset, offset + chunkBytes);
      for (;;) {
        try { write(pending); break; }
        catch (error) {
          const written = error.diagnosticBytesWritten ?? 0;
          if (!Number.isSafeInteger(written) || written < 0 || written > pending.length) { transportClosed = true; return; }
          pending = pending.subarray(written);
          if ((error.code === 'EAGAIN' || error.code === 'EWOULDBLOCK') && now() < deadline) { wait(); continue; }
          transportClosed = true;
          return; // Raw bytes were retained before reporting; never replace the native exit.
        }
      }
    }
  };
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderrBytes = result.stderr ?? Buffer.alloc(0);
  if (!Buffer.isBuffer(stdout) || !Buffer.isBuffer(stderrBytes)) throw Error('captured_diagnostic_bytes_required');
  const token = randomUUID();
  emit('::stop-commands::' + token + '\n');
  try {
    const errorCode = typeof result.error?.code === 'string' && /^[A-Z0-9_]{1,64}$/u.test(result.error.code) ? result.error.code : result.error ? 'unknown' : null;
    emit('Native ' + stage + ' failed: ' + JSON.stringify({
      exitCode: result.status ?? null,
      signal: result.signal ?? null,
      errorCode,
      stdoutBytes: stdout.length,
      stderrBytes: stderrBytes.length
    }) + '\n');
    emit('--- captured ' + stage + ' stdout' + (cargoJson ? ' (Cargo rendered diagnostics and unchanged other rows)' : '') + ' ---\n');
    if (cargoJson) {
      const rows = [];
      let start = 0;
      while (start < stdout.length) {
        const newline = stdout.indexOf(10, start),
          end = newline < 0 ? stdout.length : newline + 1,
          line = stdout.subarray(start, end);
        let rendered = null;
        if (line.length <= jsonLineBytes) {
          try {
            const row = JSON.parse(line.toString('utf8'));
            if (row.reason === 'compiler-message' && typeof row.message?.rendered === 'string') rendered = row.message.rendered;
          } catch {/* Preserve malformed/non-JSON output below. */}
        }
        rows.push({line, rendered});
        start = end;
      }
      for (const row of rows) if (row.rendered !== null) emit(row.rendered);
      emit('--- prioritized captured ' + stage + ' stderr ---\n');
      emit(stderrBytes);
      for (const row of rows) if (row.rendered === null) emit(row.line);
    } else emit(stdout);
    emit('\n--- captured ' + stage + ' stderr ---\n');
    emit(stderrBytes);
    emit('\n--- end captured ' + stage + ' diagnostics; raw bytes and hashes retained in artifact ---\n');
  } finally {
    emit('::' + token + '::\n');
  }
  return true;
}
