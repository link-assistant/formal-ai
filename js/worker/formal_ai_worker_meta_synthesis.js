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
 * A parameter as the JavaScript literal its code is given: a number as
 * written, a text or a list of values as JSON.
 * @param {*} value
 * @returns {string}
 */
function metaLiteral(value) {
  return typeof value === "string" || Array.isArray(value) ? JSON.stringify(value) : String(value);
}
/**
 * An operation's code with its parameter slots filled: `{k}` takes the whole
 * parameter, `{k0}`, `{k1}` ... the values of a list parameter in order.
 * @param {string} code
 * @param {*} parameter
 * @returns {string}
 */
function metaSubstitute(code, parameter) {
  const values = Array.isArray(parameter) ? parameter : [parameter];
  let out = String(code).split("{k}").join(metaLiteral(parameter));
  for (const [index, value] of values.entries()) out = out.split(`{k${index}}`).join(metaLiteral(value));
  return out;
}
/**
 * Compiled primitive functions, keyed by id and parameter.
 * @param {object} primitive
 * @param {*} parameter
 * @returns {function}
 */
function metaCompile(primitive, parameter) {
  const source = metaSubstitute(primitive.code, parameter);
  const cached = metaCompiled.get(source);
  if (cached) return cached;
  // eslint-disable-next-line no-new-func -- the instruction set is seed data.
  const fn = new Function(`return (${source});`)();
  metaCompiled.set(source, fn);
  return fn;
}
/**
 * The measures a list of `element` can be filtered by: every primitive from
 * the element type to a number, and a number element itself. A text measure
 * is the element's content: the element itself when it is text, or what one
 * operation reads from it ("the files that contain": read the file).
 * @param {string} element
 * @param {string} [type] the measure's result type, number by default
 * @returns {Array<object>}
 */
function metaMeasures(element, type) {
  const seed = metaSeed();
  if (!seed.measureCache) seed.measureCache = new Map();
  const key = `${element} ${type || "number"}`;
  if (seed.measureCache.has(key)) return seed.measureCache.get(key);
  const out = [];
  if (type === "text") {
    if (element === "text") out.push({ id: "value", from: "text", to: "text", doc: "", code: "(input) => input", infer: "", environment: "", takes: [] });
    for (const primitive of seed.primitives) {
      if (primitive.from !== element || primitive.to !== "text" || element === "text" || primitive.infer || primitive.effect || primitive.takes.length) continue;
      out.push({ ...primitive, parts: [primitive.id] });
    }
    seed.measureCache.set(key, out);
    return out;
  }
  if (element === "number") out.push({ id: "value", from: "number", to: "number", doc: "", code: "(input) => input", infer: "", environment: "", takes: [] });
  // A measure is any short parameter-free program from the element to a
  // number ("the lines of a file": read the file, split its lines, count).
  let frontier = [{ ids: [], codes: [], type: element, environment: "" }];
  for (let size = 1; size <= META_BOUNDS.measureLength; size += 1) {
    const next = [];
    for (const partial of frontier) {
      for (const primitive of seed.primitives) {
        if (primitive.infer || primitive.effect || primitive.takes.length) continue;
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
          out.push({ id: grown.ids.join("∘"), parts: grown.ids, from: element, to: "number", doc: "", code, infer: "", environment: grown.environment, takes: [] });
        } else {
          next.push(grown);
        }
      }
    }
    frontier = next;
  }
  seed.measureCache.set(key, out);
  return out;
}
/**
 * All typed programs from a type, shortest first, each step a primitive or a
 * primitive mapped over a list.
 * @param {string} fromType
 * @param {number} length
 * @param {Set<string>|null} [mustUse] when given, only the programs using one
 *   of these operations (see `metaStepUsesAny`) are returned
 * @returns {Array<{steps: Array<object>, type: string}>}
 */
function metaPrograms(fromType, length, mustUse = null) {
  // Call-local descriptors cannot retain a different request, seed or worker realm.
  const transitionsByType = new Map();
  const primitives = metaSeed().primitives;
  let frontier = [mustUse === null ? { steps: [], type: fromType } : { steps: [], type: fromType, uses: false }];
  const out = [];
  for (let size = 1; size <= length; size += 1) {
    const next = [];
    // A last-length program using none of `mustUse` is never built.
    const extend = (program, step, type) => {
      const uses = mustUse === null ? null : program.uses || metaStepUsesAny(step, mustUse);
      if (mustUse !== null && size === length && !uses) return;
      const grown = { steps: program.steps.concat(step), type };
      if (mustUse !== null) grown.uses = uses;
      next.push(grown);
    };
    for (const program of frontier) {
      let typedTransitions = transitionsByType.get(program.type);
      if (typedTransitions === undefined) {
        typedTransitions = [];
        const recordTransition = (step, type) => typedTransitions.push({ step, type });

        for (const primitive of primitives) {
          const direct = metaApply(primitive, program.type);
          if (direct) recordTransition({ primitive, mapped: false }, direct);
          if (program.type.startsWith("list_")) {
            const element = program.type.slice(5);
            const mapped = metaApply(primitive, element);
            if (mapped && !mapped.startsWith("list_")) recordTransition({ primitive, mapped: true }, `list_${mapped}`);
          }
        }
        // A file is edited in place by any text transformation: read it,
        // transform the text, write it back ("rewrite_file").
        if (program.type === "path" || program.type === "list_path") {
          for (const primitive of primitives) {
            if (primitive.from !== "text" || primitive.to !== "text" || primitive.infer || primitive.takes.length) continue;
            recordTransition({ primitive, mapped: program.type === "list_path", rewrite: true }, program.type);
          }
        }
        if (program.type.startsWith("list_")) {
          for (const measure of metaMeasures(program.type.slice(5))) {
            for (const filter of metaSeed().filters) {
              if (filter.measure === "number") recordTransition({ primitive: measure, mapped: false, filter }, program.type);
            }
          }
          // A filter comparing the element's content with a value.
          for (const filter of metaSeed().filters) {
            if (filter.measure === "number") continue;
            for (const measure of metaMeasures(program.type.slice(5), filter.measure)) recordTransition({ primitive: measure, mapped: false, filter }, program.type);
          }
          // A selector picks one element by its measure ("the longest word").
          for (const measure of metaMeasures(program.type.slice(5))) {
            for (const select of metaSeed().selectors) recordTransition({ primitive: measure, mapped: false, select }, program.type.slice(5));
          }
        }

        transitionsByType.set(program.type, typedTransitions);
      }
      for (const transition of typedTransitions) extend(program, { ...transition.step }, transition.type);
    }
    for (const program of next) if (mustUse === null || program.uses) out.push(program);
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
  if (step.rewrite) return step.mapped ? [step.primitive.id, "rewrite_file", "map_each"] : [step.primitive.id, "rewrite_file"];
  const chooser = step.filter || step.select;
  if (chooser) return step.primitive.id === "value" ? [chooser.id] : [step.primitive.id, chooser.id];
  return step.mapped ? [step.primitive.id, "map_each"] : [step.primitive.id];
}

/**
 * Whether a step uses one of `operations` -- in `metaStepOperations(step)` or,
 * for a filter or selector, its measure's parts -- without building a list.
 */
function metaStepUsesAny(step, operations) {
  const chooser = step.filter || step.select;
  if (!(chooser && step.primitive.id === "value") && operations.has(step.primitive.id)) return true;
  if (chooser) return operations.has(chooser.id) || (step.primitive.parts || []).some((part) => operations.has(part));
  return (step.rewrite && operations.has("rewrite_file")) || (step.mapped && operations.has("map_each"));
}

/**
 * A step's operations with a filter's or selector's measure folded into it:
 * the measure serves the filter, not a word of the request.
 * @param {object} step
 * @returns {Array<string>}
 */
function metaStepMainOperations(step) {
  const chooser = step.filter || step.select;
  return chooser ? [chooser.id] : metaStepOperations(step);
}

/**
 * True when a step takes the program's parameter.
 * @param {object} step
 * @returns {boolean}
 */
function metaStepIsParametric(step) {
  return Boolean(step.filter || step.primitive.infer || (step.primitive.takes || []).length);
}

/**
 * The types of the values a parametric step reads: a filter's threshold
 * (its measure type), an inferred number, or the values an operation takes.
 * @param {object} step
 * @returns {Array<string>}
 */
function metaStepParameterTypes(step) {
  if (step.filter) return [step.filter.measure];
  if (step.primitive.infer) return ["number"];
  return (step.primitive.takes || []).slice();
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
    const fn = metaCompile(step.primitive, step.primitive.infer || (step.primitive.takes || []).length ? parameter : null);
    if (step.select) {
      const test = metaCompile({ code: step.select.test }, null);
      value = value.reduce((best, item) => (test(fn(item), fn(best)) ? item : best));
    } else if (step.rewrite) {
      const rewrite = metaCompile({ code: metaRewriteCode(step.primitive.code) }, null);
      value = step.mapped ? value.map((item) => rewrite(item)) : rewrite(value);
    } else if (step.filter) {
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
  if (last.filter) return last.filter.measure === "number" ? metaInferThreshold(steps, examples) : undefined;
  if ((last.primitive.takes || []).length) return undefined;
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
    const code = metaSubstitute(step.primitive.code, parameter);
    if (step.select) {
      lines.push(`  value = value.reduce((best, item) => ((${step.select.test})((${code})(item), (${code})(best)) ? item : best)); // ${step.select.id} by ${step.primitive.id}`);
      continue;
    }
    if (step.rewrite) {
      const rewrite = metaRewriteCode(code);
      lines.push(step.mapped ? `  value = value.map(${rewrite}); // each: rewrite_file by ${step.primitive.id}` : `  value = (${rewrite})(value); // rewrite_file by ${step.primitive.id}`);
      continue;
    }
    if (step.filter) {
      const test = step.filter.test.replace(/^\(measure, threshold\) => /u, "");
      lines.push(`  value = value.filter((item) => { const measure = (${code})(item); const threshold = ${metaLiteral(parameter)}; return ${test}; }); // ${step.filter.id} by ${step.primitive.id}`);
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
  const used = new Set(steps.flatMap(metaStepMainOperations));
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
  trace.emit("goal", metaNote("examples_goal", { from: fromType, to: toType, count: examples.length }));
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
      if (ties.length > 1) trace.emit("tie", metaNote("tie", { count: ties.length, kept: ties.slice(0, 3).map((item) => item.steps.map(metaStepLabel).join(" ∘ ")).join(" | ") }));
      return { steps: best.steps, parameter: best.parameter, alternatives: ties.length - 1, evaluated };
    }
  }
  trace.emit("impasse", metaNote("no_program", { length: META_BOUNDS.programLength, evaluated }));
  return null;
}
/**
 * A short label for one program step.
 * @param {object} step
 * @returns {string}
 */
function metaStepLabel(step) {
  if (step.rewrite) return step.mapped ? `each(rewrite(${step.primitive.id}))` : `rewrite(${step.primitive.id})`;
  if (step.filter || step.select) return `${(step.filter || step.select).id}(${step.primitive.id})`;
  return step.mapped ? `each(${step.primitive.id})` : step.primitive.id;
}
/**
 * Without examples, the goal is a program that uses one operation from every
 * grounded word's hypothesis group, preferring a type-preserving program.
 * @param {Array<Array<string>>} groups
 * @param {Map<string, number>} evidence
 * @param {object} trace
 * @param {Array<{value: *, type: string}>} values the values the request names, in order, bound to a parametric step
 * @param {Array<Array<object>>} words every grounded word's hypotheses
 * @param {Array<string>} inputTypes argument types the request's data words name
 * @param {Array<object>} clauses the request's clauses (see metaClauses)
 * @param {{weakWords?: Array<Array<object>>, weakViews?: Array<Array<object>>, universal?: boolean, measureWords?: Array<Array<object>>|null}} [hints]
 *   words tied across many operations, the views tied plural nouns name,
 *   whether the request quantifies
 *   universally ("every file"), and the bound's unit word naming a measure
 * @returns {object|null}
 */
function metaSynthesizeFromMeaning(groups, evidence, trace, values, words, inputTypes, clauses, hints) {
  const { weakWords = [], weakViews = [], universal = false, measureWords = null } = hints || {};
  trace.emit("goal", `program serving ${groups.map((group) => `{${group.join("|")}}`).join(" ")}`);
  const types = ["text", "list_number", "list_text", "number", "path"];
  const operations = metaSeed().primitives;
  // The last clause's head is the outermost operation, so its result type is
  // the program's.
  const lastHead = clauses.length ? clauses[clauses.length - 1].head : groups[0];
  const headTypes = lastHead.map((id) => (operations.find((primitive) => primitive.id === id) || {}).to).filter(Boolean);
  let best = null;
  const allWords = words.concat(weakWords);
  const strongest = new Map();
  for (const hypotheses of allWords) {
    for (const hypothesis of hypotheses) strongest.set(hypothesis.operation, Math.max(strongest.get(hypothesis.operation) || 0, hypothesis.score));
  }
  const headOperations = new Set(lastHead);
  for (const fromType of types) {
    // Only programs using the head operation are enumerated at full length.
    for (const program of metaPrograms(fromType, META_BOUNDS.programLength - 1, headOperations)) {
      const parametric = program.steps.filter(metaStepIsParametric);
      if (parametric.length > (values.length ? 1 : 0)) continue;
      const bound = parametric.length ? metaBindValues(metaStepParameterTypes(parametric[0]), values) : null;
      if (bound === undefined) continue;
      // A measure's parts serve the words they ground ("80 characters").
      const used = new Set(program.steps.flatMap((step) => metaStepOperations(step).concat(step.filter || step.select ? step.primitive.parts || [] : [])));
      if (!lastHead.some((operation) => used.has(operation))) continue;
      const uncovered = groups.filter((group) => !group.some((operation) => used.has(operation))).length;
      // A filter's measure is internal to the filter: it is not charged as
      // an ungrounded operation; its grounded parts only break ties.
      const ungrounded = program.steps.flatMap(metaStepMainOperations).filter((operation) => !evidence.get(operation)).length;
      const main = new Set(program.steps.flatMap(metaStepMainOperations));
      const parts = new Set(program.steps.filter((step) => step.filter || step.select).flatMap((step) => step.primitive.parts || [step.primitive.id]));
      let measureEvidence = 0;
      // A word the main program already explains (it is that operation's
      // strongest word) does not name the measure; a word whose meaning the
      // main program owes to another word may ("prints ... lines").
      for (const hypotheses of measureWords || allWords) {
        if (!measureWords && hypotheses.some((hypothesis) => main.has(hypothesis.operation) && hypothesis.score >= strongest.get(hypothesis.operation) * 0.99)) continue;
        measureEvidence += Math.max(0, ...hypotheses.filter((hypothesis) => parts.has(hypothesis.operation)).map((hypothesis) => hypothesis.score));
      }
      const measureSize = parts.size;
      const order = metaClauseOrder(program, fromType, clauses);
      const statedInput = inputTypes.length && !inputTypes.includes(fromType) ? 1 : 0;
      // A rewrite writes the head's result back to the file: it fits there.
      const rewritesHead = program.steps.some((step) => step.rewrite && lastHead.includes(step.primitive.id));
      const headFits = rewritesHead || headTypes.some((type) => type === program.type || (type === "list_any" && program.type.startsWith("list_"))) ? 0 : 1;
      // Grounded evidence decides first (every operation asked for, none
      // invented), then the request's own composition order, then shape.
      // "every file" asks for iteration: a program never holding a list
      // does not quantify over anything.
      const unquantified = universal && !order.types.some((type) => type.startsWith("list_")) ? 1 : 0;
      const rank = [ungrounded, uncovered, -metaCoverageScore(program.steps, words), -measureEvidence,
        order.violations, order.objectMismatch, unquantified, statedInput,
        headFits, -metaCoverageScore(program.steps, weakViews), program.type === fromType ? 0 : 1, program.steps.length, -metaCoverageScore(program.steps, weakWords), measureSize];
      let better = !best;
      for (let position = 0; !better && position < rank.length; position += 1) {
        if (rank[position] !== best.rank[position]) {
          better = rank[position] < best.rank[position];
          break;
        }
      }
      if (better) {
        best = { steps: program.steps, fromType, rank, parameter: bound, uncovered };
      }
    }
  }
  if (!best) trace.emit("impasse", metaNote("no_head_program", {}));
  else if (best.uncovered) trace.emit("evidence", metaNote("uncovered", { count: best.uncovered }));
  return best;
}
/**
 * The request's values a parametric step reads, by type in the order the
 * request names them, or undefined when one is missing. One value is bound
 * as itself, several as a list.
 * @param {Array<string>} types
 * @param {Array<{value: *, type: string}>} values
 * @returns {*}
 */
function metaBindValues(types, values) {
  const used = new Set();
  const out = [];
  for (const type of types) {
    const at = values.findIndex((item, index) => !used.has(index) && item.type === type);
    if (at < 0) return undefined;
    used.add(at);
    out.push(values[at].value);
  }
  return out.length === 1 ? out[0] : out;
}
/**
 * How far a program departs from the request's composition: coordinated
 * clauses apply in order; within a clause the head acts last, after its
 * object's modifiers; the head consumes the type its object noun names.
 * Representation changes (split / join) are free.
 * @param {{steps: Array<object>}} program
 * @param {string} fromType
 * @param {Array<object>} clauses
 * @returns {{violations: number, objectMismatch: number, types: Array<string>}}
 */
function metaClauseOrder(program, fromType, clauses) {
  const inputs = [];
  let type = fromType;
  for (const step of program.steps) {
    inputs.push(step.rewrite ? "text" : step.mapped ? type.slice(5) : type);
    type = step.select ? type.slice(5) : step.filter || step.rewrite ? type : step.mapped ? `list_${step.primitive.to}` : metaApply(step.primitive, type);
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
  return { violations, objectMismatch, types: inputs.concat(type) };
}
/**
 * The in-place edit of one file by a text transformation: the seeded
 * combinator `rewrite_file` applied to an operation's code.
 * @param {string} code a text -> text function source
 * @returns {string}
 */
function metaRewriteCode(code) {
  const combinator = metaSeed().combinators.find((item) => item.id === "rewrite_file");
  return String(combinator ? combinator.code : "").split("{f}").join(code);
}
