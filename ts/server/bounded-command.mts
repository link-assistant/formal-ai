/** Binary-safe finite child supervision. Zero stream limits discard output. */
export async function boundedCommand(file, argumentsValue = [], options = {}) {
  const { spawn } = await import('node:child_process');
  const started = process.hrtime.bigint();
  const timeout = options.timeout;
  const stdoutLimit = options.stdoutLimitBytes ?? options.maxBuffer ?? 1024 * 1024;
  const stderrLimit = options.stderrLimitBytes ?? options.maxBuffer ?? 1024 * 1024;
  const grace = 250;
  if (!Number.isFinite(timeout) || timeout <= 0 || timeout > 2147482647
      || !Number.isSafeInteger(stdoutLimit) || stdoutLimit < 0
      || !Number.isSafeInteger(stderrLimit) || stderrLimit < 0) {
    throw new TypeError('invalid_command_bounds');
  }
  return new Promise((resolve) => {
    let child;
    let status = null;
    let signal = null;
    let failure = null;
    let timedOut = false;
    let exited = false;
    let stdinComplete = options.input == null;
    let stdoutComplete = stdoutLimit === 0;
    let stderrComplete = stderrLimit === 0;
    let settled = false;
    let cleanupTimer;
    let deadlineTimer;
    const stdout = [];
    const stderr = [];
    let stdoutBytes = 0;
    let stderrBytes = 0;
    const complete = () => exited && stdinComplete && stdoutComplete && stderrComplete;
    const makeError = (code) => Object.assign(new Error(code), { code });
    const kill = () => {
      if (!child?.pid) return;
      if (process.platform !== 'win32') {
        try { process.kill(-child.pid, 'SIGKILL'); } catch {}
      }
      try { child.kill('SIGKILL'); } catch {}
    };
    const finish = (cleanupComplete) => {
      if (settled) return;
      settled = true;
      kill();
      if (cleanupComplete && options.childSignal) Atomics.store(options.childSignal, 0, 0);
      clearTimeout(deadlineTimer);
      clearTimeout(cleanupTimer);
      if (!cleanupComplete) { failure ??= makeError('ECLEANUP'); child?.unref(); }
      child?.stdin?.destroy();
      child?.stdout?.destroy();
      child?.stderr?.destroy();
      let out = Buffer.concat(stdout, stdoutBytes);
      let err = Buffer.concat(stderr, stderrBytes);
      if (options.encoding && options.encoding !== 'buffer') {
        out = out.toString(options.encoding);
        err = err.toString(options.encoding);
      }
      resolve({ pid: child?.pid ?? 0, status, signal, stdout: out, stderr: err,
        output: [null, out, err], error: failure ?? undefined,
        timed_out: timedOut, cleanup_complete: cleanupComplete,
        elapsedNs: process.hrtime.bigint() - started });
    };
    const check = () => {
      if (complete()) finish(true);
    };
    const stop = (error) => {
      if (settled) return;
      if (complete() && !failure) { finish(true); return; }
      failure ??= error;
      kill();
      child?.stdin?.destroy();
      stdinComplete = true;
      cleanupTimer ??= setTimeout(() => finish(complete()), grace);
      check();
    };
    try {
      child = spawn(file, argumentsValue, {
        cwd: options.cwd, env: options.env, windowsHide: true,
        detached: process.platform !== 'win32',
        stdio: [options.input == null ? 'ignore' : 'pipe',
          stdoutLimit === 0 ? 'ignore' : 'pipe', stderrLimit === 0 ? 'ignore' : 'pipe'],
      });
      if (options.childSignal) Atomics.store(options.childSignal, 0, child.pid ?? 0);
    } catch (error) {
      failure = error;
      exited = stdinComplete = stdoutComplete = stderrComplete = true;
      finish(true);
      return;
    }
    const stream = (pipe, isStdout, limit) => {
      if (!pipe) return;
      pipe.on('data', (chunk) => {
        if (settled) return;
        const used = isStdout ? stdoutBytes : stderrBytes;
        const accepted = Math.min(limit - used, chunk.length);
        if (accepted > 0) {
          const copy = Buffer.from(chunk.subarray(0, accepted));
          if (isStdout) { stdout.push(copy); stdoutBytes += accepted; }
          else { stderr.push(copy); stderrBytes += accepted; }
        }
        if (accepted !== chunk.length) stop(makeError('ENOBUFS'));
      });
      pipe.once('end', () => {
        if (isStdout) stdoutComplete = true;
        else stderrComplete = true;
        check();
      });
      pipe.once('error', (error) => stop(error));
    };
    stream(child.stdout, true, stdoutLimit);
    stream(child.stderr, false, stderrLimit);
    child.once('error', (error) => {
      failure ??= error;
      if (!child.pid) {
        exited = stdinComplete = stdoutComplete = stderrComplete = true;
        finish(true);
      } else stop(error);
    });
    child.once('exit', (code, observedSignal) => {
      status = code;
      signal = observedSignal;
      exited = true;
      check();
    });
    deadlineTimer = setTimeout(() => {
      if (complete()) { finish(true); return; }
      timedOut = true;
      stop(makeError('ETIMEDOUT'));
    }, Math.max(0, timeout - Number(process.hrtime.bigint() - started) / 1e6));
    if (child.stdin) {
      child.stdin.once('error', (error) => stop(error));
      child.stdin.end(options.input, () => { stdinComplete = true; check(); });
    }
    check();
  });
}

import { Worker, MessageChannel, receiveMessageOnPort, workerData } from 'node:worker_threads';

/** Preserve synchronous callers while supervising child I/O in the same async kernel. */
export function boundedCommandSync(file, argumentsValue = [], options = {}) {
  if (!Number.isFinite(options.timeout) || options.timeout <= 0 || options.timeout > 2147482647) {
    throw new TypeError('invalid_command_bounds');
  }
  const startedNs = process.hrtime.bigint();
  const wake = new Int32Array(new SharedArrayBuffer(8));
  const { port1, port2 } = new MessageChannel();
  const inherited = [];
  for (let index = 0; index < process.execArgv.length; index += 1) {
    const argument = process.execArgv[index];
    if (/^(?:--(?:require|import|loader|experimental-loader)(?:=|$)|-r$)/u.test(argument)) {
      inherited.push(argument);
      if (!argument.includes('=') && index + 1 < process.execArgv.length) {
        index += 1;
        inherited.push(process.execArgv[index]);
      }
    }
  }
  const worker = new Worker(new URL(import.meta.url), {
    workerData: { boundedCommand: true, file, argumentsValue, options, startedNs, wake, port: port2 },
    transferList: [port2], execArgv: inherited,
    resourceLimits: { maxOldGenerationSizeMb: 64 },
  });
  // Preserve preload/security NODE_OPTIONS. The environment may override the heap hint.
  worker.unref();
  worker.on('error', () => {});
  const waited = Atomics.wait(wake, 1, 0, options.timeout + 1000);
  const message = receiveMessageOnPort(port1)?.message;
  port1.close();
  void worker.terminate().catch(() => {});
  if (waited === 'timed-out' || !message) {
    const identifier = Atomics.load(wake, 0);
    if (identifier > 1) {
      if (process.platform !== 'win32') {
        try { process.kill(-identifier, 'SIGKILL'); } catch {}
      }
      try { process.kill(identifier, 'SIGKILL'); } catch {}
    }
    const error = Object.assign(new Error('ECLEANUP'), { code: 'ECLEANUP' });
    return { pid: identifier, status: null, signal: null,
      stdout: Buffer.alloc(0), stderr: Buffer.alloc(0), output: [null, Buffer.alloc(0), Buffer.alloc(0)],
      error, timed_out: true, cleanup_complete: false, elapsedNs: process.hrtime.bigint() - startedNs };
  }
  const result = message.result;
  if (message.error) throw Object.assign(new Error(message.error.message), message.error);
  if (typeof result.stdout !== 'string') result.stdout = Buffer.from(result.stdout);
  if (typeof result.stderr !== 'string') result.stderr = Buffer.from(result.stderr);
  result.output = [null, result.stdout, result.stderr];
  if (result.error) result.error = Object.assign(new Error(result.error.message), result.error);
  return result;
}

if (workerData?.boundedCommand === true) {
  try {
    const spentMs = Number(process.hrtime.bigint() - workerData.startedNs) / 1e6;
    const remaining = workerData.options.timeout - spentMs;
    const result = remaining > 0
      ? await boundedCommand(workerData.file, workerData.argumentsValue,
        { ...workerData.options, timeout: remaining, childSignal: workerData.wake })
      : { pid: 0, status: null, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0),
        error: Object.assign(new Error('ETIMEDOUT'), { code: 'ETIMEDOUT' }),
        timed_out: true, cleanup_complete: true };
    result.elapsedNs = process.hrtime.bigint() - workerData.startedNs;
    if (result.error) result.error = {
      message: result.error.message, code: result.error.code,
      errno: result.error.errno, syscall: result.error.syscall, path: result.error.path,
    };
    const transfers = [];
    for (const name of ['stdout', 'stderr']) {
      if (typeof result[name] !== 'string') {
        result[name] = Uint8Array.from(result[name]);
        transfers.push(result[name].buffer);
      }
    }
    result.output = [null, result.stdout, result.stderr];
    workerData.port.postMessage({ result }, transfers);
  } catch (error) {
    workerData.port.postMessage({ error: { message: error.message, code: error.code } });
  } finally {
    Atomics.store(workerData.wake, 1, 1);
    Atomics.notify(workerData.wake, 1);
    workerData.port.close();
  }
}
