"use strict";

// Shared Electron process boundary. command-stream is ESM-only, while the host
// intentionally remains CommonJS, so load it once and expose a small
// promise-based adapter that keeps argv execution, shell
// execution, streaming, cancellation, and exit diagnostics in one place.

function defaultComponentLoader() {
  return import("command-stream");
}

function errorText(error) {
  return error && error.message ? error.message : String(error || "command failed");
}

function createCommandRunner(options = {}) {
  const loadComponent = options.loadComponent || defaultComponentLoader;
  let componentPromise = null;

  function component() {
    if (!componentPromise) componentPromise = Promise.resolve().then(loadComponent);
    return componentPromise;
  }

  async function start(command, args = [], runOptions = {}) {
    const { ProcessRunner } = await component();
    const shell = runOptions.shell === true;
    const spec = shell
      ? { mode: "shell", command: String(command) }
      : { mode: "exec", file: String(command), args: args.map(String) };
    const stdoutChunks = [];
    const stderrChunks = [];
    let cancelled = Boolean(runOptions.signal && runOptions.signal.aborted);
    const markCancelled = () => { cancelled = true; };
    if (runOptions.signal) runOptions.signal.addEventListener("abort", markCancelled, { once: true });
    const processRunner = new ProcessRunner(spec, {
      cwd: runOptions.cwd,
      env: runOptions.env,
      stdin: runOptions.stdin === undefined ? "ignore" : runOptions.stdin,
      mirror: false,
      capture: true,
      signal: runOptions.signal,
      killSignal: runOptions.killSignal || "SIGTERM",
    });

    processRunner.on("stdout", (chunk) => {
      stdoutChunks.push(Buffer.from(chunk));
      if (typeof runOptions.onStdout === "function") runOptions.onStdout(chunk);
    });
    processRunner.on("stderr", (chunk) => {
      stderrChunks.push(Buffer.from(chunk));
      if (typeof runOptions.onStderr === "function") runOptions.onStderr(chunk);
    });

    const completion = processRunner.start().then(
      (result) => ({
        ...result,
        code: typeof result.code === "number" ? result.code : 1,
        stdout: stdoutChunks.length
          ? Buffer.concat(stdoutChunks).toString("utf8")
          : String(result.stdout || ""),
        stderr: stderrChunks.length
          ? Buffer.concat(stderrChunks).toString("utf8")
          : String(result.stderr || ""),
        cancelled,
        component: "command-stream",
      }),
      (error) => ({
        code: typeof error.code === "number" ? error.code : 1,
        stdout: stdoutChunks.length ? Buffer.concat(stdoutChunks).toString("utf8") : String(error.stdout || ""),
        stderr: stderrChunks.length ? Buffer.concat(stderrChunks).toString("utf8") : String(error.stderr || errorText(error)),
        cancelled,
        component: "command-stream",
        error,
      }),
    ).finally(() => {
      if (runOptions.signal) runOptions.signal.removeEventListener("abort", markCancelled);
    });

    return {
      process: processRunner,
      completion,
      kill: (signal) => {
        cancelled = true;
        return processRunner.kill(signal);
      },
      get pid() {
        return processRunner.child && processRunner.child.pid;
      },
      get killed() {
        return Boolean(cancelled || processRunner.finished);
      },
    };
  }

  async function run(command, args = [], runOptions = {}) {
    const handle = await start(command, args, runOptions);
    return handle.completion;
  }

  function runTool(toolOptions = {}) {
    if (toolOptions.isolation === "docker") {
      const dockerArgs = [
        ...(toolOptions.dockerPrefixArgs || []),
        "run",
        "--rm",
        String(toolOptions.image),
        "sh",
        "-c",
        String(toolOptions.command),
      ];
      return run(toolOptions.dockerBinary || "docker", dockerArgs, toolOptions);
    }
    return run(String(toolOptions.command || ""), [], { ...toolOptions, shell: true });
  }

  return { start, run, runTool };
}

module.exports = { createCommandRunner };
