// ---------------------------------------------------------------------------
// code_refactoring
// ---------------------------------------------------------------------------

/**
 * Parse one handler expression: `x => expr`, `(x) => expr`, or
 * `function (x) { body }`.
 * @param {string} handler handler source
 * @returns {object|null} {param, body}, or null for any other shape
 */
function codeRefactoringParseHandler(handler) {
  const trimmed = handler.trim();
  const arrowAt = trimmed.indexOf("=>");
  if (arrowAt !== -1) {
    const param = codeTaskTrimChar(codeTaskTrimChar(trimmed.slice(0, arrowAt).trim(), "("), ")").trim();
    const body = trimmed.slice(arrowAt + 2).trim();
    if (param === "" || body === "") return null;
    return { param: param, body: body };
  }
  if (!trimmed.startsWith("function")) return null;
  const rest = trimmed.slice(8);
  const open = rest.indexOf("(");
  if (open === -1) return null;
  const span = codeTaskParenSpan(rest, open);
  const brace = rest.indexOf("{", span[1]);
  if (brace === -1) return null;
  let depth = 0;
  for (let index = brace; index < rest.length; index += 1) {
    if (rest[index] === "{") {
      depth += 1;
    } else if (rest[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        const body = rest.slice(brace + 1, index).trim();
        if (span[0].trim() === "" || body === "") return null;
        return { param: span[0].trim(), body: body };
      }
    }
  }
  return null;
}

/**
 * Parse the promise chain out of whitespace-flattened code.
 * @param {string} flat code with whitespace runs collapsed to one space
 * @returns {object|null} {head, thens, catchHandler}, or null
 */
function codeRefactoringParseChain(flat) {
  const firstThen = flat.indexOf(".then(");
  if (firstThen === -1) return null;
  const head = codeTaskTrimEndChar(flat.slice(0, firstThen).trim(), ";").trim();
  if (head === "") return null;
  const thens = [];
  let catchHandler = null;
  let cursor = firstThen;
  for (;;) {
    const thenAt = flat.indexOf(".then(", cursor);
    const catchAt = flat.indexOf(".catch(", cursor);
    let at = -1;
    let isThen = true;
    if (thenAt !== -1 && catchAt !== -1 && thenAt < catchAt) {
      at = thenAt;
    } else if (catchAt !== -1) {
      at = catchAt;
      isThen = false;
    } else if (thenAt !== -1) {
      at = thenAt;
    } else {
      break;
    }
    const span = codeTaskParenSpan(flat, at + (isThen ? 6 : 7) - 1);
    const handler = codeRefactoringParseHandler(span[0]);
    if (handler === null) return null;
    if (isThen) {
      thens.push(handler);
    } else {
      catchHandler = handler;
    }
    cursor = span[1];
  }
  if (thens.length === 0) return null;
  return { head: head, thens: thens, catchHandler: catchHandler };
}

/**
 * Render the async/await rewrite; with `executable` the awaits are desugared to
 * `__await(…)` calls so the worker can run the twin synchronously.
 * @param {object} chain parsed chain
 * @param {boolean} executable whether to render the runnable twin
 * @returns {string} the rewritten source
 */
function codeRefactoringRenderAsync(chain, executable) {
  const lines = [executable ? "function run() {" : "async function run() {"];
  const hasCatch = chain.catchHandler !== null;
  if (hasCatch) lines.push("  try {");
  const indent = hasCatch ? "    " : "  ";
  let previous = chain.head;
  for (const handler of chain.thens) {
    const awaited = executable ? "__await(" + previous + ")" : "await " + previous;
    lines.push(indent + "const " + handler.param + " = " + awaited + ";");
    previous = handler.body;
  }
  lines.push(indent + (executable ? "__await(" + previous + ")" : "await " + previous) + ";");
  if (hasCatch) {
    lines.push("  } catch (" + chain.catchHandler.param + ") {");
    lines.push("    " + chain.catchHandler.body);
    lines.push("  }");
  }
  lines.push("}");
  return lines.join("\n");
}

/**
 * Run one program against recording stubs: every free identifier is a stub
 * whose calls are traced; promises are modelled synchronously (a stub is a
 * fulfilled thenable of itself); call number `rejectCall` (1-based, 0 = none)
 * returns a rejected thenable instead.
 * @param {string} source program text
 * @param {number} rejectCall which call rejects
 * @returns {Array<string>} the call trace
 */
function codeRefactoringTraceRun(source, rejectCall) {
  const trace = [];
  const state = { calls: 0 };
  const describe = function (value) {
    if (value !== null && (typeof value === "object" || typeof value === "function")) {
      if (value.__stubLabel) return value.__stubLabel;
      if (value.__plainWrapper) return describe(value.__value);
      if (value instanceof Error) return "Error(" + value.message + ")";
      return typeof value === "function" ? "<fn>" : "<object>";
    }
    return JSON.stringify(value) === undefined ? String(value) : JSON.stringify(value);
  };
  const isThenable = function (value) {
    return value !== null && (typeof value === "object" || typeof value === "function") &&
      (Boolean(value.__stubLabel) || Boolean(value.__rejected) || Boolean(value.__plainWrapper));
  };
  let makeStub = null;
  let makeRejected = null;
  const settle = function (value) {
    if (isThenable(value)) return value;
    const wrapper = {
      __plainWrapper: true,
      __value: value,
      then: function (onFulfilled) {
        if (typeof onFulfilled !== "function") return wrapper;
        try {
          return settle(onFulfilled(value));
        } catch (error) {
          return makeRejected(error);
        }
      },
      catch: function () { return wrapper; },
    };
    return wrapper;
  };
  makeRejected = function (error) {
    const rejected = {
      __rejected: true,
      __error: error,
      then: function (onFulfilled, onRejected) {
        if (typeof onRejected !== "function") return rejected;
        try {
          return settle(onRejected(error));
        } catch (inner) {
          return makeRejected(inner);
        }
      },
      catch: function (onRejected) {
        if (typeof onRejected !== "function") return rejected;
        try {
          return settle(onRejected(error));
        } catch (inner) {
          return makeRejected(inner);
        }
      },
    };
    return rejected;
  };
  makeStub = function (label) {
    const call = function (args, prefix) {
      const parts = [];
      for (const arg of args) parts.push(describe(arg));
      const callLabel = prefix + label + "(" + parts.join(", ") + ")";
      state.calls += 1;
      trace.push(callLabel);
      if (state.calls === rejectCall) return makeRejected(makeStub("error<" + callLabel + ">"));
      return makeStub(callLabel);
    };
    const target = function () {};
    let proxy = null;
    proxy = new Proxy(target, {
      get: function (unused, prop) {
        if (typeof prop === "symbol") {
          if (prop === Symbol.toPrimitive) return function () { return label; };
          return undefined;
        }
        switch (prop) {
          case "__stubLabel":
            return label;
          case "then":
            return function (onFulfilled) {
              if (typeof onFulfilled !== "function") return proxy;
              try {
                return settle(onFulfilled(proxy));
              } catch (error) {
                return makeRejected(error);
              }
            };
          case "catch":
            return function () { return proxy; };
          default:
            return makeStub(label + "." + prop);
        }
      },
      apply: function (unused, thisArg, args) { return call(args, ""); },
      construct: function (unused, args) { return call(args, "new "); },
    });
    return proxy;
  };
  const awaitValue = function (value) {
    if (value !== null && typeof value === "object" && value.__rejected) throw value.__error;
    if (value !== null && typeof value === "object" && value.__plainWrapper) return value.__value;
    return value;
  };
  const environment = new Proxy({}, {
    has: function (unused, prop) {
      return typeof prop === "string" && prop !== "__await";
    },
    get: function (unused, prop) {
      if (typeof prop === "symbol") return undefined;
      return makeStub(prop);
    },
  });
  try {
    const runner = new Function("__env", "__await", "with (__env) {\n" + source + "\n}");
    runner(environment, awaitValue);
  } catch (error) {
    trace.push("threw " + describe(error && error.__stubLabel ? error : error));
  }
  return trace;
}

/**
 * Run the original chain and its executable twin in every scenario (all calls
 * succeed, then each call in turn rejects) and compare the call traces.
 * @param {string} flat whitespace-flattened original code
 * @param {object} chain parsed chain
 * @returns {object} {status, scenarios, calls, detail}
 */
function codeRefactoringExecutionCheck(flat, chain) {
  const bodies = [chain.head];
  for (const handler of chain.thens) bodies.push(handler.body);
  if (chain.catchHandler !== null) bodies.push(chain.catchHandler.body);
  for (const body of bodies) {
    if (/\b(while|for|do)\b/.test(body)) {
      return { status: "skipped", scenarios: 0, calls: 0, detail: "a handler body contains a loop" };
    }
  }
  if (typeof Proxy !== "function") {
    return { status: "skipped", scenarios: 0, calls: 0, detail: "no Proxy in this runtime" };
  }
  const twin = codeRefactoringRenderAsync(chain, true) + "\ntry { run(); } catch (__unhandled) {}";
  const baseline = codeRefactoringTraceRun(flat, 0);
  const calls = baseline.length;
  const limit = Math.min(calls, 8);
  let scenarios = 0;
  for (let reject = 0; reject <= limit; reject += 1) {
    const original = codeRefactoringTraceRun(flat, reject);
    const rewritten = codeRefactoringTraceRun(twin, reject);
    scenarios += 1;
    if (original.join("\n") !== rewritten.join("\n")) {
      return {
        status: "mismatch",
        scenarios: scenarios,
        calls: calls,
        detail: (reject === 0 ? "all calls succeed" : "call " + reject + " rejects") +
          ": original [" + original.join("; ") + "] vs rewrite [" + rewritten.join("; ") + "]",
      };
    }
  }
  return { status: "matched", scenarios: scenarios, calls: calls, detail: baseline.join("; ") };
}

/**
 * Rewrite a promise chain with async/await, then run both versions.
 * Mirrors `handle_code_refactoring` in rust/src/solver_handlers/code_refactoring.rs.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleCodeRefactoring(prompt, normalized) {
  if (!codeTaskCued("code_refactoring", "", prompt, normalized)) return null;
  const code = codeTaskCodeBlock(prompt);
  const log = codeTaskLog();
  codeTaskLogAppend(log, "code_refactoring:request", "cued");
  const flat = code === null ? "" : codeTaskWords(code).join(" ");
  const chain = code === null ? null : codeRefactoringParseChain(flat);
  if (chain === null) {
    codeTaskLogAppend(log, "code_refactoring:refusal", "chain=none");
    return codeTaskAnswer(log, "code_refactoring", "response:code_refactoring",
      codeTaskTemplate("code_refactoring_refusal", []), 0.4);
  }
  codeTaskLogAppend(log, "code_refactoring:chain", "then=" + chain.thens.length);
  let body = codeTaskTemplate("code_refactoring_async", [["code", codeRefactoringRenderAsync(chain, false)]]);
  const check = codeRefactoringExecutionCheck(flat, chain);
  codeTaskLogAppend(log, "code_refactoring:execution", check.status);
  switch (check.status) {
    case "matched":
      body += "\n\n" + codeTaskTemplate("code_refactoring_execution_matched", [
        ["scenarios", String(check.scenarios)],
        ["calls", String(check.calls)],
        ["trace", check.detail],
      ]);
      break;
    case "mismatch":
      body += "\n\n" + codeTaskTemplate("code_refactoring_execution_mismatch", [["detail", check.detail]]);
      break;
    default:
      body += "\n\n" + codeTaskTemplate("code_refactoring_execution_skipped", [["detail", check.detail]]);
  }
  return codeTaskAnswer(log, "code_refactoring", "response:code_refactoring", body,
    check.status === "mismatch" ? 0.4 : 0.7);
}

/**
 * Browser binding for the `code_refactoring` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryCodeRefactoring(prompt) {
  return handleCodeRefactoring(prompt, prompt.toLowerCase());
}
