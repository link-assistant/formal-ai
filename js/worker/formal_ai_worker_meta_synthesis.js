// The meta reasoner's synthesis half (see formal_ai_worker_meta_reasoner.js):
// typed program enumeration over the instruction set in
// data/seed/meta-reasoning.lino, parameter and threshold inference from the
// request's examples, rendering, and verification of the rendered source.
// Domain-free: every operation, filter and probe is seed data.

/**
 * The type of a runtime value in the instruction set's vocabulary.
 * @param {*} value
 * @returns {string}
 */
function metaTypeOf(value) {
  if (typeof value === "string") return "text";
  if (typeof value === "number") return "number";
  if (Array.isArray(value)) {
    if (value.length && value.every((item) => typeof item === "number")) return "list_number";
    if (value.length && value.every((item) => typeof item === "string")) return "list_text";
    return "list_any";
  }
  return "unknown";
}

/**
 * The type a primitive produces from a concrete input type, or null.
 * @param {object} primitive
 * @param {string} type
 * @returns {string|null}
 */
function metaApply(primitive, type) {
  if (primitive.from === type) return primitive.to === "list_any" ? type : primitive.to;
  if (primitive.from === "list_any" && type.startsWith("list_")) return primitive.to === "list_any" ? type : primitive.to;
  return null;
}

/**
 * Compiled primitive functions, keyed by id and parameter.
 * @param {object} primitive
 * @param {*} parameter
 * @returns {function}
 */
function metaCompile(primitive, parameter) {
  const source = primitive.code.split("{k}").join(String(parameter));
  const cached = metaCompiled.get(source);
  if (cached) return cached;
  // eslint-disable-next-line no-new-func -- the instruction set is seed data.
  const fn = new Function(`return (${source});`)();
  metaCompiled.set(source, fn);
  return fn;
}

/**
 * The measures a list of `element` can be filtered by: every primitive from
 * the element type to a number, and a number element itself.
 * @param {string} element
 * @returns {Array<object>}
 */
function metaMeasures(element) {
  const seed = metaSeed();
  if (!seed.measureCache) seed.measureCache = new Map();
  if (seed.measureCache.has(element)) return seed.measureCache.get(element);
  const out = [];
  if (element === "number") out.push({ id: "value", from: "number", to: "number", doc: "", code: "(input) => input", infer: "", environment: "" });
  // A measure is any short parameter-free program from the element to a
  // number ("the lines of a file": read the file, split its lines, count).
  let frontier = [{ ids: [], codes: [], type: element, environment: "" }];
  for (let size = 1; size <= META_BOUNDS.measureLength; size += 1) {
    const next = [];
    for (const partial of frontier) {
      for (const primitive of seed.primitives) {
        if (primitive.infer) continue;
        const type = metaApply(primitive, partial.type);
        if (!type) continue;
        const grown = {
          ids: partial.ids.concat(primitive.id),
          codes: partial.codes.concat(primitive.code),
          type,
          environment: partial.environment || primitive.environment || "",
        };
        if (type === "number") {
          const code = `(input) => ${grown.codes.reduce((inner, step) => `(${step})(${inner})`, "input")}`;
          out.push({ id: grown.ids.join("∘"), parts: grown.ids, from: element, to: "number", doc: "", code, infer: "", environment: grown.environment });
        } else {
          next.push(grown);
        }
      }
    }
    frontier = next;
  }
  seed.measureCache.set(element, out);
  return out;
}

/**
 * All typed programs from a type, shortest first, each step a primitive or a
 * primitive mapped over a list.
 * @param {string} fromType
 * @param {number} length
 * @returns {Array<{steps: Array<object>, type: string}>}
 */
function metaPrograms(fromType, length) {
  const primitives = metaSeed().primitives;
  let frontier = [{ steps: [], type: fromType }];
  const out = [];
  for (let size = 1; size <= length; size += 1) {
    const next = [];
    for (const program of frontier) {
      for (const primitive of primitives) {
        const direct = metaApply(primitive, program.type);
        if (direct) next.push({ steps: program.steps.concat({ primitive, mapped: false }), type: direct });
        if (program.type.startsWith("list_")) {
          const element = program.type.slice(5);
          const mapped = metaApply(primitive, element);
          if (mapped && !mapped.startsWith("list_")) next.push({ steps: program.steps.concat({ primitive, mapped: true }), type: `list_${mapped}` });
        }
      }
      if (program.type.startsWith("list_")) {
        for (const measure of metaMeasures(program.type.slice(5))) {
          for (const filter of metaSeed().filters) next.push({ steps: program.steps.concat({ primitive: measure, mapped: false, filter }), type: program.type });
        }
      }
    }
    for (const program of next) out.push(program);
    frontier = next;
  }
  return out;
}

/**
 * A step's operations for evidence scoring.
 * @param {object} step
 * @returns {Array<string>}
 */
function metaStepOperations(step) {
  if (step.filter) return step.primitive.id === "value" ? [step.filter.id] : [step.primitive.id, step.filter.id];
  return step.mapped ? [step.primitive.id, "map_each"] : [step.primitive.id];
}

/**
 * True when a step takes the program's parameter.
 * @param {object} step
 * @returns {boolean}
 */
function metaStepIsParametric(step) {
  return Boolean(step.filter || step.primitive.infer);
}

/**
 * Run a program on one input; a parametric last step takes `parameter`.
 * @param {Array<object>} steps
 * @param {*} input
 * @param {*} parameter
 * @returns {*}
 */
function metaRun(steps, input, parameter) {
  let value = input;
  for (const step of steps) {
    const fn = metaCompile(step.primitive, step.primitive.infer ? parameter : null);
    if (step.filter) {
      const test = metaCompile({ code: step.filter.test }, null);
      value = value.filter((item) => test(fn(item), parameter));
    } else {
      value = step.mapped ? value.map((item) => fn(item)) : fn(value);
    }
  }
  return value;
}

/**
 * Infer the parameter of a program's parametric last step: an arithmetic
 * parameter from the first example, a filter threshold from all of them.
 * @param {Array<object>} steps
 * @param {Array<{input: *, output: *}>} examples
 * @returns {*}
 */
function metaInferParameter(steps, examples) {
  const last = steps[steps.length - 1];
  const example = examples[0];
  if (steps.slice(0, -1).some(metaStepIsParametric)) return undefined;
  if (last.filter) return metaInferThreshold(steps, examples);
  if (!last.primitive.infer) return null;
  const before = metaRun(steps.slice(0, -1), example.input, null);
  // eslint-disable-next-line no-new-func -- the inference rule is seed data.
  const infer = new Function(`return (${last.primitive.infer});`)();
  const pair = last.mapped ? [before[0], Array.isArray(example.output) ? example.output[0] : undefined] : [before, example.output];
  if (pair[0] === undefined || pair[1] === undefined) return undefined;
  const parameter = infer(pair[0], pair[1]);
  return typeof parameter === "number" && Number.isFinite(parameter) ? parameter : undefined;
}

/**
 * The threshold separating the kept elements from the dropped ones across
 * every example, or undefined when the output is not a filtering of the
 * input or no threshold separates them.
 * @param {Array<object>} steps
 * @param {Array<{input: *, output: *}>} examples
 * @returns {number|undefined}
 */
function metaInferThreshold(steps, examples) {
  const last = steps[steps.length - 1];
  const measure = metaCompile(last.primitive, null);
  const kept = [];
  const dropped = [];
  for (const example of examples) {
    const before = metaRun(steps.slice(0, -1), example.input, null);
    if (!Array.isArray(before) || !Array.isArray(example.output)) return undefined;
    let cursor = 0;
    for (const item of before) {
      if (cursor < example.output.length && JSON.stringify(item) === JSON.stringify(example.output[cursor])) {
        kept.push(measure(item));
        cursor += 1;
      } else {
        dropped.push(measure(item));
      }
    }
    if (cursor !== example.output.length) return undefined;
  }
  if (!kept.length || !dropped.length) return undefined;
  const test = metaCompile({ code: last.filter.test }, null);
  for (const candidate of dropped.concat(kept).sort((a, b) => a - b)) {
    if (kept.every((value) => test(value, candidate)) && dropped.every((value) => !test(value, candidate))) return candidate;
  }
  return undefined;
}

/**
 * Render a program as the JavaScript the answer shows and the verifier runs.
 * @param {Array<object>} steps
 * @param {*} parameter
 * @returns {string}
 */
function metaRender(steps, parameter) {
  const lines = ["function solution(input) {", "  let value = input;"];
  for (const step of steps) {
    const code = step.primitive.code.split("{k}").join(String(parameter));
    if (step.filter) {
      const test = step.filter.test.replace(/^\(measure, threshold\) => /u, "");
      lines.push(`  value = value.filter((item) => { const measure = (${code})(item); const threshold = ${parameter}; return ${test}; }); // ${step.filter.id} by ${step.primitive.id}`);
      continue;
    }
    lines.push(step.mapped ? `  value = value.map(${code}); // each: ${step.primitive.id}` : `  value = (${code})(value); // ${step.primitive.id}`);
  }
  lines.push("  return value;", "}");
  return lines.join("\n");
}

/**
 * Run rendered source on every example; the verifier never trusts the
 * enumerator's own evaluation.
 * @param {string} source
 * @param {Array<object>} examples
 * @returns {{passed: number, total: number, failures: Array<object>}}
 */
function metaVerify(source, examples) {
  // eslint-disable-next-line no-new-func -- running the composed answer is the verification.
  const solution = new Function(`${source}\nreturn solution;`)();
  const failures = [];
  for (const example of examples) {
    let actual;
    try {
      actual = solution(example.input);
    } catch (error) {
      actual = `error: ${error.message}`;
    }
    if (JSON.stringify(actual) !== JSON.stringify(example.output)) failures.push({ input: example.input, expected: example.output, actual });
  }
  return { passed: examples.length - failures.length, total: examples.length, failures };
}

/**
 * Evidence for a program: the grounded score of every operation it uses.
 * @param {Array<object>} steps
 * @param {Map<string, number>} evidence
 * @returns {number}
 */
function metaEvidenceScore(steps, evidence) {
  let score = 0;
  for (const operation of new Set(steps.flatMap(metaStepOperations))) score += evidence.get(operation) || 0;
  return score;
}

/**
 * Coverage of the request: each grounded word credits once, with the best
 * of its hypotheses the program uses.
 * @param {Array<object>} steps
 * @param {Array<Array<{operation: string, score: number}>>} words
 * @returns {number}
 */
function metaCoverageScore(steps, words) {
  // A filter's measure serves the filter, not a word of the request.
  const used = new Set(steps.flatMap((step) => (step.filter ? [step.filter.id] : metaStepOperations(step))));
  let score = 0;
  for (const hypotheses of words) {
    let best = 0;
    for (const hypothesis of hypotheses) if (used.has(hypothesis.operation) && hypothesis.score > best) best = hypothesis.score;
    score += best;
  }
  return score;
}

/**
 * Search for a program whose difference from the goal (failing examples) is
 * zero: shortest first, then most evidenced.
 * @param {Array<object>} examples
 * @param {Map<string, number>} evidence
 * @param {object} trace
 * @returns {object|null}
 */
function metaSynthesizeFromExamples(examples, evidence, trace) {
  const fromType = metaTypeOf(examples[0].input);
  const toType = metaTypeOf(examples[0].output);
  trace.emit("goal", `program ${fromType} → ${toType} with difference 0 over ${examples.length} example(s)`);
  let evaluated = 0;
  let rejected = 0;
  for (let length = 1; length <= META_BOUNDS.programLength; length += 1) {
    const programs = metaPrograms(fromType, length)
      .filter((program) => program.steps.length === length && (program.type === toType || (toType === "list_any" && program.type.startsWith("list_"))))
      .sort((a, b) => metaEvidenceScore(b.steps, evidence) - metaEvidenceScore(a.steps, evidence));
    const passing = [];
    for (const program of programs) {
      if (evaluated >= META_BOUNDS.candidateBudget) break;
      evaluated += 1;
      let parameter;
      try {
        parameter = metaInferParameter(program.steps, examples);
      } catch {
        parameter = undefined;
      }
      if (parameter === undefined) continue;
      let difference = 0;
      let counterexample = null;
      for (const example of examples) {
        let actual;
        try {
          actual = metaRun(program.steps, example.input, parameter);
        } catch {
          actual = undefined;
        }
        if (JSON.stringify(actual) !== JSON.stringify(example.output)) {
          difference += 1;
          if (!counterexample) counterexample = { input: example.input, expected: example.output, actual };
        }
      }
      if (difference === 0) {
        passing.push({ steps: program.steps, parameter, evidence: metaEvidenceScore(program.steps, evidence) });
      } else {
        rejected += 1;
        if (rejected <= META_BOUNDS.rejectionsTraced && metaEvidenceScore(program.steps, evidence) > 0) {
          trace.emit("counterexample", `${program.steps.map(metaStepLabel).join(" ∘ ")}: ${JSON.stringify(counterexample.input)} gave ${JSON.stringify(counterexample.actual)}, expected ${JSON.stringify(counterexample.expected)} (difference ${difference})`);
        }
      }
    }
    trace.emit("search", `length ${length}: ${programs.length} typed candidate(s), ${passing.length} with difference 0`);
    if (passing.length) {
      passing.sort((a, b) => b.evidence - a.evidence);
      const best = passing[0];
      const ties = passing.filter((item) => item.evidence === best.evidence);
      if (ties.length > 1) trace.emit("tie", `${ties.length} equally evidenced programs; kept ${ties.slice(0, 3).map((item) => item.steps.map(metaStepLabel).join(" ∘ ")).join(" | ")}`);
      return { steps: best.steps, parameter: best.parameter, alternatives: ties.length - 1, evaluated };
    }
  }
  trace.emit("impasse", `no program up to length ${META_BOUNDS.programLength} reaches difference 0 (${evaluated} evaluated)`);
  return null;
}

/**
 * A short label for one program step.
 * @param {object} step
 * @returns {string}
 */
function metaStepLabel(step) {
  if (step.filter) return `${step.filter.id}(${step.primitive.id})`;
  return step.mapped ? `each(${step.primitive.id})` : step.primitive.id;
}

/**
 * Without examples, the goal is a program that uses one operation from every
 * grounded word's hypothesis group, preferring a type-preserving program.
 * @param {Array<Array<string>>} groups
 * @param {Map<string, number>} evidence
 * @param {object} trace
 * @param {number|null} parameter a number the request states, bound to a parametric step
 * @param {Array<Array<object>>} words every grounded word's hypotheses
 * @param {Array<string>} inputTypes argument types the request's data words name
 * @param {Array<object>} clauses the request's clauses (see metaClauses)
 * @returns {object|null}
 */
function metaSynthesizeFromMeaning(groups, evidence, trace, parameter, words, inputTypes, clauses) {
  trace.emit("goal", `program serving ${groups.map((group) => `{${group.join("|")}}`).join(" ")}`);
  const types = ["text", "list_number", "list_text", "number", "path"];
  const operations = metaSeed().primitives;
  // The last clause's head is the outermost operation, so its result type is
  // the program's.
  const lastHead = clauses.length ? clauses[clauses.length - 1].head : groups[0];
  const headTypes = lastHead.map((id) => (operations.find((primitive) => primitive.id === id) || {}).to).filter(Boolean);
  let best = null;
  for (const fromType of types) {
    for (const program of metaPrograms(fromType, META_BOUNDS.programLength - 1)) {
      const parametric = program.steps.filter(metaStepIsParametric).length;
      if (parametric > (parameter === null ? 0 : 1)) continue;
      const used = new Set(program.steps.flatMap(metaStepOperations));
      if (!lastHead.some((operation) => used.has(operation))) continue;
      const uncovered = groups.filter((group) => !group.some((operation) => used.has(operation))).length;
      // A filter's measure is internal to the filter: it is not charged as
      // an ungrounded operation; its grounded parts only break ties.
      const ungrounded = program.steps
        .flatMap((step) => (step.filter ? [step.filter.id] : metaStepOperations(step)))
        .filter((operation) => !evidence.get(operation)).length;
      const main = new Set(program.steps.flatMap((step) => (step.filter ? [step.filter.id] : metaStepOperations(step))));
      const parts = new Set(program.steps.filter((step) => step.filter).flatMap((step) => step.primitive.parts || [step.primitive.id]));
      let measureEvidence = 0;
      for (const hypotheses of words) {
        if (hypotheses.some((hypothesis) => main.has(hypothesis.operation))) continue;
        measureEvidence += Math.max(0, ...hypotheses.filter((hypothesis) => parts.has(hypothesis.operation)).map((hypothesis) => hypothesis.score));
      }
      const measureSize = parts.size;
      const order = metaClauseOrder(program, fromType, clauses);
      const statedInput = inputTypes.length && !inputTypes.includes(fromType) ? 1 : 0;
      const headFits = headTypes.some((type) => type === program.type || (type === "list_any" && program.type.startsWith("list_"))) ? 0 : 1;
      // Grounded evidence decides first (every operation asked for, none
      // invented), then the request's own composition order, then shape.
      const rank = [ungrounded, uncovered, -metaCoverageScore(program.steps, words), -measureEvidence, order.violations, order.objectMismatch, statedInput, headFits, program.type === fromType ? 0 : 1, program.steps.length, measureSize];
      let better = !best;
      for (let position = 0; !better && position < rank.length; position += 1) {
        if (rank[position] !== best.rank[position]) {
          better = rank[position] < best.rank[position];
          break;
        }
      }
      if (better) {
        best = { steps: program.steps, fromType, rank, parameter: parametric ? parameter : null, uncovered };
      }
    }
  }
  if (!best) trace.emit("impasse", "no typed program serves the last clause's head");
  else if (best.uncovered) trace.emit("evidence", `${best.uncovered} grounded word group(s) left unexplained by the chosen program`);
  return best;
}

/**
 * How far a program departs from the request's composition: coordinated
 * clauses apply in order; within a clause the head acts last, after its
 * object's modifiers; the head consumes the type its object noun names.
 * Representation changes (split / join) are free.
 * @param {{steps: Array<object>}} program
 * @param {string} fromType
 * @param {Array<object>} clauses
 * @returns {{violations: number, objectMismatch: number}}
 */
function metaClauseOrder(program, fromType, clauses) {
  const inputs = [];
  let type = fromType;
  for (const step of program.steps) {
    inputs.push(step.mapped ? type.slice(5) : type);
    type = step.filter ? type : step.mapped ? `list_${step.primitive.to}` : metaApply(step.primitive, type);
  }
  const positionOf = (ids) => program.steps.findIndex((step) => metaStepOperations(step).some((id) => ids.includes(id) && !metaIsView(id)));
  let violations = 0;
  let objectMismatch = 0;
  let previousHead = -1;
  for (const clause of clauses) {
    const head = positionOf(clause.head);
    if (head < 0) continue;
    if (head < previousHead) violations += 1;
    previousHead = head;
    for (const [position, step] of program.steps.entries()) {
      const ids = metaStepOperations(step).filter((id) => !metaIsView(id));
      if (position > head && ids.some((id) => clause.others.includes(id))) violations += 1;
    }
    if (clause.objectType && inputs[head] !== clause.objectType && !(clause.objectType.startsWith("list_") && inputs[head] === "list_any")) objectMismatch += 1;
  }
  return { violations, objectMismatch };
}
