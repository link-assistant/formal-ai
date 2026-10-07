// Pattern inference over 1D sequences and 2D grids (issue #531, R403/R405).
//
// Browser twin of rust/src/solver_handlers/pattern_inference.rs and the
// substrate it runs: rust/src/sequences/{store,symbols,compression,
// patterns_1d,grid_2d,inference}.rs. A prompt that names pattern curiosity (a
// surface of a `verifiable_expectation_pattern` meaning in the lexicon) and
// carries a run of at least three atoms -- "find the pattern in 1 2 1 2 1 2",
// "what comes next in 7 7 7 7", a newline-separated grid -- is parsed into
// deduplicated link points, compressed by the Re-Pair-style pair replacement,
// classified (constant, repetition, period, palindrome; grid symmetries), and
// reported through the `pattern_*` templates of
// data/seed/multilingual-responses-pattern.lino. A bare "what is a pattern?"
// carries no data and falls through. tryVerifiableTask calls this first, as the
// native `try_verifiable_task_with_online` calls `try_pattern_inference`.

/**
 * A fresh link store: doublets addressed from 1, the composite index keyed by
 * "source,target". Mirrors `SequenceStore::new`.
 * @returns {{links: Array<Array<number>>, index: Map<string, number>}}
 */
function patternStoreNew() {
  return { links: [], index: new Map() };
}

/**
 * Allocate a self-referential point. Mirrors `SequenceStore::create_point`;
 * points stay out of the composite index so `(a, a)` is a distinct link.
 * @param {{links: Array<Array<number>>, index: Map<string, number>}} store
 * @returns {number}
 */
function patternStorePoint(store) {
  const address = store.links.length + 1;
  store.links.push([address, address]);
  return address;
}

/**
 * The address of the `(source, target)` doublet, created on first use.
 * Mirrors `SequenceStore::get_or_create`.
 * @param {{links: Array<Array<number>>, index: Map<string, number>}} store
 * @param {number} source
 * @param {number} target
 * @returns {number}
 */
function patternStorePair(store, source, target) {
  const key = `${source},${target}`;
  const existing = store.index.get(key);
  if (existing !== undefined) return existing;
  const address = store.links.length + 1;
  store.links.push([source, target]);
  store.index.set(key, address);
  return address;
}

/**
 * The points a link expands to, left to right. Mirrors `SequenceStore::expand`.
 * @param {{links: Array<Array<number>>, index: Map<string, number>}} store
 * @param {number} address
 * @returns {Array<number>}
 */
function patternStoreExpand(store, address) {
  const output = [];
  const pending = [address];
  while (pending.length > 0) {
    const link = pending.pop();
    const doublet = store.links[link - 1];
    if (doublet[0] === link && doublet[1] === link) {
      output.push(link);
    } else {
      pending.push(doublet[1]);
      pending.push(doublet[0]);
    }
  }
  return output;
}

/**
 * Non-overlapping occurrences of `source target`, scanning left to right.
 * Mirrors `count_non_overlapping` in compression.rs.
 * @param {Array<number>} sequence
 * @param {number} source
 * @param {number} target
 * @returns {number}
 */
function patternCountPair(sequence, source, target) {
  let count = 0;
  let index = 0;
  while (index + 1 < sequence.length) {
    if (sequence[index] === source && sequence[index + 1] === target) {
      count += 1;
      index += 2;
    } else {
      index += 1;
    }
  }
  return count;
}

/**
 * The adjacent pair with the most non-overlapping occurrences (at least two),
 * ties broken by the smallest `(source, target)`. Mirrors `most_frequent_pair`.
 * @param {Array<number>} sequence
 * @returns {{source: number, target: number, count: number}|null}
 */
function patternMostFrequentPair(sequence) {
  let best = null;
  const seen = new Set();
  for (let index = 0; index + 1 < sequence.length; index += 1) {
    const source = sequence[index];
    const target = sequence[index + 1];
    const key = `${source},${target}`;
    if (seen.has(key)) continue;
    seen.add(key);
    const count = patternCountPair(sequence, source, target);
    if (count < 2) continue;
    const smaller = best !== null && (best.source < source || (best.source === source && best.target < target));
    if (best !== null && (best.count > count || (best.count === count && smaller))) continue;
    best = { source: source, target: target, count: count };
  }
  return best;
}

/**
 * Compress `sequence` by repeatedly replacing its most frequent pair with the
 * pair's link. Mirrors `compress`: `{original, sequence, steps}`.
 * @param {{links: Array<Array<number>>, index: Map<string, number>}} store
 * @param {Array<number>} sequence
 * @returns {{original: Array<number>, sequence: Array<number>, steps: Array<{source: number, target: number, replacement: number, occurrences: number}>}}
 */
function patternCompress(store, sequence) {
  let current = sequence.slice();
  const steps = [];
  for (let best = patternMostFrequentPair(current); best !== null; best = patternMostFrequentPair(current)) {
    const replacement = patternStorePair(store, best.source, best.target);
    const next = [];
    let index = 0;
    while (index < current.length) {
      if (index + 1 < current.length && current[index] === best.source && current[index + 1] === best.target) {
        next.push(replacement);
        index += 2;
      } else {
        next.push(current[index]);
        index += 1;
      }
    }
    current = next;
    steps.push({ source: best.source, target: best.target, replacement: replacement, occurrences: best.count });
  }
  return { original: sequence.slice(), sequence: current, steps: steps };
}

/**
 * Whether expanding the compressed sequence reproduces the original exactly.
 * Mirrors `CompressionResult::is_lossless`.
 * @param {{links: Array<Array<number>>, index: Map<string, number>}} store
 * @param {{original: Array<number>, sequence: Array<number>}} compression
 * @returns {boolean}
 */
function patternCompressionIsLossless(store, compression) {
  const expanded = [];
  for (const link of compression.sequence) {
    for (const point of patternStoreExpand(store, link)) expanded.push(point);
  }
  return expanded.length === compression.original.length &&
    expanded.every((point, index) => point === compression.original[index]);
}

/**
 * Format a non-negative `value` with `digits` fractional digits, ties to even,
 * as Rust's `{:.N}` formats an `f64`.
 * @param {number} value
 * @param {number} digits
 * @returns {string}
 */
function patternFormatFixed(value, digits) {
  const scaled = value * 10 ** digits;
  const floor = Math.floor(scaled);
  const fraction = scaled - floor;
  let whole = floor;
  if (fraction > 0.5 || (fraction === 0.5 && floor % 2 !== 0)) whole = floor + 1;
  const text = String(whole).padStart(digits + 1, "0");
  return digits === 0 ? text : `${text.slice(0, text.length - digits)}.${text.slice(text.length - digits)}`;
}

/**
 * The smallest block whose repetition tiles the whole sequence, or null.
 * Mirrors `detect_repetition` in patterns_1d.rs.
 * @param {Array<number>} sequence
 * @returns {{period: number, repetitions: number}|null}
 */
function patternRepetition(sequence) {
  const length = sequence.length;
  for (let period = 1; period < length; period += 1) {
    if (length % period !== 0) continue;
    if (sequence.every((element, index) => element === sequence[index % period])) {
      return { period: period, repetitions: length / period };
    }
  }
  return null;
}

/**
 * The smallest period shorter than the sequence, or null. Mirrors `detect_period`.
 * @param {Array<number>} sequence
 * @returns {number|null}
 */
function patternPeriod(sequence) {
  for (let period = 1; period < sequence.length; period += 1) {
    let holds = true;
    for (let index = period; index < sequence.length && holds; index += 1) {
      holds = sequence[index] === sequence[index - period];
    }
    if (holds) return period;
  }
  return null;
}

/**
 * Whether the sequence reads the same both ways. Mirrors `detect_palindrome`.
 * @param {Array<number>} sequence
 * @returns {boolean}
 */
function patternPalindrome(sequence) {
  const last = sequence.length - 1;
  for (let left = 0; left < last - left; left += 1) {
    if (sequence[left] !== sequence[last - left]) return false;
  }
  return true;
}

/**
 * Whether `candidate` is `original` read backwards. Mirrors `is_reverse`.
 * @param {Array<number>} original
 * @param {Array<number>} candidate
 * @returns {boolean}
 */
function patternIsReverse(original, candidate) {
  const last = original.length - 1;
  return original.length === candidate.length && candidate.every((element, index) => element === original[last - index]);
}

/**
 * The left shift that rotates `original` onto `candidate` (0 when equal), or
 * null when `candidate` is no rotation of it. Mirrors `detect_translation`.
 * @param {Array<number>} original
 * @param {Array<number>} candidate
 * @returns {number|null}
 */
function patternTranslation(original, candidate) {
  const length = original.length;
  if (length !== candidate.length) return null;
  for (let shift = 0; shift < Math.max(length, 1); shift += 1) {
    if (candidate.every((element, index) => element === original[(index + shift) % length])) return shift;
  }
  return null;
}

/**
 * The dominant structure, most specific first. Mirrors `classify_sequence`:
 * kind is one of empty, constant, repetition, periodic, aperiodic.
 * @param {Array<number>} sequence
 * @returns {{kind: string, period: number, repetitions: number}}
 */
function patternClassify(sequence) {
  if (sequence.length === 0) return { kind: "empty", period: 0, repetitions: 0 };
  if (sequence.every((element) => element === sequence[0])) return { kind: "constant", period: 0, repetitions: 0 };
  const repetition = patternRepetition(sequence);
  if (repetition !== null) return { kind: "repetition", period: repetition.period, repetitions: repetition.repetitions };
  const period = patternPeriod(sequence);
  if (period !== null) return { kind: "periodic", period: period, repetitions: 0 };
  return { kind: "aperiodic", period: 0, repetitions: 0 };
}

/**
 * The full 1D report. Mirrors `infer_sequence_patterns`.
 * @param {{links: Array<Array<number>>, index: Map<string, number>}} store
 * @param {Array<number>} sequence
 * @returns {{length: number, distinct: number, classification: {kind: string, period: number, repetitions: number}, palindrome: boolean, period: number|null, repetition: {period: number, repetitions: number}|null, compression: {original: Array<number>, sequence: Array<number>, steps: Array<object>}}}
 */
function patternInferSequence(store, sequence) {
  return {
    length: sequence.length,
    distinct: new Set(sequence).size,
    classification: patternClassify(sequence),
    palindrome: patternPalindrome(sequence),
    period: patternPeriod(sequence),
    repetition: patternRepetition(sequence),
    compression: patternCompress(store, sequence),
  };
}

/**
 * Whether the report found any structure. Mirrors `SequencePatternReport::has_structure`.
 * @param {{palindrome: boolean, period: number|null, repetition: object|null, compression: {steps: Array<object>}, classification: {kind: string}}} report
 * @returns {boolean}
 */
function patternSequenceHasStructure(report) {
  return report.palindrome || report.period !== null || report.repetition !== null ||
    report.compression.steps.length > 0 || report.classification.kind === "constant";
}

/**
 * The compressed length over the original length. Mirrors `compression_ratio`.
 * @param {{original: Array<number>, sequence: Array<number>}} compression
 * @returns {number}
 */
function patternCompressionRatio(compression) {
  if (compression.original.length === 0) return 1.0;
  return compression.sequence.length / compression.original.length;
}

/**
 * A seed template with its `{name}` slots filled. Mirrors `render_pattern`.
 * @param {string} intent
 * @param {string} language
 * @param {Object<string, string>} values
 * @returns {string}
 */
function patternRender(intent, language, values) {
  let rendered = String(answerFor(intent, language) || "");
  for (const name of Object.keys(values)) {
    rendered = rendered.split(`{${name}}`).join(values[name]);
  }
  return rendered;
}

/**
 * The localized structure lines of a 1D report. Mirrors `SequencePatternReport::summary_in`.
 * @param {object} report
 * @param {string} language
 * @returns {string}
 */
function patternSequenceSummary(report, language) {
  const lines = [patternRender("pattern_sequence_count", language, {
    length: String(report.length),
    distinct: String(report.distinct),
  })];
  const classification = report.classification;
  switch (classification.kind) {
    case "repetition":
      lines.push(patternRender("pattern_sequence_repetition", language, {
        period: String(classification.period),
        repetitions: String(classification.repetitions),
      }));
      break;
    case "periodic":
      lines.push(patternRender("pattern_sequence_periodic", language, { period: String(classification.period) }));
      break;
    default:
      lines.push(patternRender(`pattern_sequence_${classification.kind}`, language, {}));
  }
  if (report.palindrome && report.length > 1) {
    lines.push(patternRender("pattern_sequence_palindrome", language, {}));
  }
  if (report.compression.steps.length > 0) {
    lines.push(patternRender("pattern_sequence_compression", language, {
      pairs: String(report.compression.steps.length),
      percent: patternFormatFixed(patternCompressionRatio(report.compression) * 100.0, 0),
    }));
  } else {
    lines.push(patternRender("pattern_sequence_no_compression", language, {}));
  }
  return lines.join("\n");
}

/**
 * The grid after a spatial transform. Mirrors `Grid::apply`; transform is one
 * of rotate_cw, rotate_180, rotate_ccw, reflect_horizontal, reflect_vertical,
 * transpose, anti_transpose.
 * @param {{rows: number, cols: number, cells: Array<number>}} grid
 * @param {string} transform
 * @returns {{rows: number, cols: number, cells: Array<number>}}
 */
function patternGridApply(grid, transform) {
  const at = (row, col) => grid.cells[row * grid.cols + col];
  const cells = [];
  switch (transform) {
    case "rotate_cw":
      for (let col = 0; col < grid.cols; col += 1) for (let row = grid.rows - 1; row >= 0; row -= 1) cells.push(at(row, col));
      return { rows: grid.cols, cols: grid.rows, cells: cells };
    case "rotate_ccw":
      for (let col = grid.cols - 1; col >= 0; col -= 1) for (let row = 0; row < grid.rows; row += 1) cells.push(at(row, col));
      return { rows: grid.cols, cols: grid.rows, cells: cells };
    case "rotate_180":
      return { rows: grid.rows, cols: grid.cols, cells: grid.cells.slice().reverse() };
    case "reflect_horizontal":
      for (let row = 0; row < grid.rows; row += 1) for (let col = grid.cols - 1; col >= 0; col -= 1) cells.push(at(row, col));
      return { rows: grid.rows, cols: grid.cols, cells: cells };
    case "reflect_vertical":
      for (let row = grid.rows - 1; row >= 0; row -= 1) for (let col = 0; col < grid.cols; col += 1) cells.push(at(row, col));
      return { rows: grid.rows, cols: grid.cols, cells: cells };
    case "transpose":
      for (let col = 0; col < grid.cols; col += 1) for (let row = 0; row < grid.rows; row += 1) cells.push(at(row, col));
      return { rows: grid.cols, cols: grid.rows, cells: cells };
    case "anti_transpose":
      for (let col = grid.cols - 1; col >= 0; col -= 1) for (let row = grid.rows - 1; row >= 0; row -= 1) cells.push(at(row, col));
      return { rows: grid.cols, cols: grid.rows, cells: cells };
    default:
      return { rows: grid.rows, cols: grid.cols, cells: grid.cells.slice() };
  }
}

/**
 * Whether two grids have the same shape and cells.
 * @param {{rows: number, cols: number, cells: Array<number>}} left
 * @param {{rows: number, cols: number, cells: Array<number>}} right
 * @returns {boolean}
 */
function patternGridEquals(left, right) {
  return left.rows === right.rows && left.cols === right.cols &&
    left.cells.every((cell, index) => cell === right.cells[index]);
}

/**
 * The non-identity transforms that leave the grid unchanged, in the native
 * order. Mirrors `Grid::invariant_transforms`.
 * @param {{rows: number, cols: number, cells: Array<number>}} grid
 * @returns {Array<string>}
 */
function patternGridInvariantTransforms(grid) {
  const square = grid.rows === grid.cols;
  const turns = ["rotate_cw", "rotate_ccw", "transpose", "anti_transpose"];
  return ["rotate_cw", "rotate_180", "rotate_ccw", "reflect_horizontal", "reflect_vertical", "transpose", "anti_transpose"]
    .filter((transform) => (square || !turns.includes(transform)) && patternGridEquals(patternGridApply(grid, transform), grid));
}

/**
 * The first transform (identity included) that maps `grid` onto `other`: the
 * analogy "the output is the input rotated / reflected". Mirrors
 * `Grid::transform_onto`; null when none does.
 * @param {{rows: number, cols: number, cells: Array<number>}} grid
 * @param {{rows: number, cols: number, cells: Array<number>}} other
 * @returns {string|null}
 */
function patternGridTransformOnto(grid, other) {
  const transforms = ["identity", "rotate_cw", "rotate_180", "rotate_ccw", "reflect_horizontal", "reflect_vertical", "transpose", "anti_transpose"];
  return transforms.find((transform) => patternGridEquals(patternGridApply(grid, transform), other)) || null;
}

/**
 * The seed template intents of the symmetries the grid exhibits, in report
 * order; diagonal symmetries count on square grids only. Mirrors `Grid::symmetries`.
 * @param {{rows: number, cols: number, cells: Array<number>}} grid
 * @returns {Array<string>}
 */
function patternGridSymmetries(grid) {
  const square = grid.rows === grid.cols;
  const symmetries = [];
  for (const entry of [
    ["reflect_horizontal", "pattern_grid_horizontal", true],
    ["reflect_vertical", "pattern_grid_vertical", true],
    ["rotate_180", "pattern_grid_rotation", true],
    ["transpose", "pattern_grid_diagonal", square],
    ["anti_transpose", "pattern_grid_anti_diagonal", square],
  ]) {
    if (entry[2] && patternGridEquals(patternGridApply(grid, entry[0]), grid)) symmetries.push(entry[1]);
  }
  return symmetries;
}

/**
 * The localized lines of a grid report. Mirrors `GridPatternReport::summary_in`;
 * the symmetry list separator mirrors inference.rs (`、` in Chinese).
 * @param {{rows: number, cols: number, cells: Array<number>}} grid
 * @param {object} rowMajor
 * @param {string} language
 * @returns {string}
 */
function patternGridSummary(grid, rowMajor, language) {
  const symmetries = patternGridSymmetries(grid).map((intent) => patternRender(intent, language, {}));
  const lines = [patternRender("pattern_grid_shape", language, { rows: String(grid.rows), cols: String(grid.cols) })];
  if (symmetries.length === 0) {
    lines.push(patternRender("pattern_grid_no_symmetry", language, {}));
  } else {
    lines.push(patternRender("pattern_grid_symmetry", language, {
      symmetries: symmetries.join(language === "zh" ? "、" : ", "),
    }));
  }
  lines.push(patternSequenceSummary(rowMajor, language));
  return lines.join("\n");
}

/**
 * Whether a cleaned token is an atom: ASCII digits, or one uppercase ASCII
 * letter (so the English "a" and "i" never join a run). Mirrors `is_atom_token`.
 * @param {string} token
 * @returns {boolean}
 */
function patternIsAtom(token) {
  return /^[0-9]+$/.test(token) || /^[A-Z]$/.test(token);
}

/**
 * The whitespace-separated tokens of `text`, each trimmed of surrounding
 * non-alphanumeric characters. Mirrors `split_whitespace().map(clean_token)`.
 * @param {string} text
 * @returns {Array<string>}
 */
function patternTokens(text) {
  return text.split(/\p{White_Space}+/u)
    .filter((raw) => raw.length > 0)
    .map((raw) => raw.replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, ""));
}

/**
 * The longest contiguous run of at least three atoms, or an empty list.
 * Mirrors `longest_atom_run`.
 * @param {string} text
 * @returns {Array<string>}
 */
function patternLongestRun(text) {
  let best = [];
  let current = [];
  for (const token of patternTokens(text)) {
    if (patternIsAtom(token)) {
      current.push(token);
    } else {
      if (current.length > best.length) best = current;
      current = [];
    }
  }
  if (current.length > best.length) best = current;
  return best.length >= 3 ? best : [];
}

/**
 * A grid of two or more lines holding the same number (at least two) of atoms,
 * as `{rows, cols, tokens}` in row-major order, or null. Mirrors `parse_grid`.
 * @param {string} prompt
 * @returns {{rows: number, cols: number, tokens: Array<string>}|null}
 */
function patternParseGrid(prompt) {
  const rows = prompt.split("\n")
    .map((line) => patternTokens(line.replace(/\r$/, "")).filter(patternIsAtom))
    .filter((tokens) => tokens.length > 0);
  if (rows.length < 2) return null;
  const cols = rows[0].length;
  if (cols < 2 || rows.some((row) => row.length !== cols)) return null;
  return { rows: rows.length, cols: cols, tokens: rows.flat() };
}

/**
 * Deduplicated points for tokens: equal tokens share the point first allocated
 * for them. Mirrors `intern` plus `to_points` (`SymbolTable::scalar`).
 * @param {{links: Array<Array<number>>, index: Map<string, number>}} store
 * @param {Array<string>} tokens
 * @returns {Array<number>}
 */
function patternPoints(store, tokens) {
  const points = new Map();
  return tokens.map((token) => {
    if (!points.has(token)) points.set(token, patternStorePoint(store));
    return points.get(token);
  });
}

/**
 * The next token when the structure defines one (constant, repetition, bare
 * period), or null. Mirrors `predict_next`.
 * @param {Array<string>} tokens
 * @param {{kind: string, period: number}} classification
 * @returns {string|null}
 */
function patternPredictNext(tokens, classification) {
  switch (classification.kind) {
    case "constant":
      return tokens[0];
    case "repetition":
      return tokens[tokens.length % classification.period];
    case "periodic":
      return tokens[tokens.length - classification.period];
    default:
      return null;
  }
}

/**
 * The answer record of a pattern report.
 * @param {string} body
 * @param {Array<string>} log
 * @param {boolean} structured
 * @returns {{intent: string, content: string, confidence: number, evidence: Array<string>, toolCalls: Array<object>}}
 */
function patternAnswer(body, log, structured) {
  return {
    intent: "pattern_inference",
    content: body,
    confidence: structured ? 0.85 : 0.6,
    evidence: log.concat(["response:pattern_inference"]),
    toolCalls: [],
  };
}

/**
 * Answer a concrete pattern-inference request over a sequence or grid, or null.
 * Mirrors `try_pattern_inference`: the report language is the forced response
 * language when a replay set one, English otherwise.
 * @param {string} prompt
 * @returns {object|null}
 */
function tryPatternInference(prompt) {
  const text = String(prompt || "");
  const lowered = text.toLowerCase();
  if (!meaningsWithRole("verifiable_expectation_pattern").some((meaning) => meaningEvidencedIn(meaning, lowered))) {
    return null;
  }
  // A list the numeric-list pipeline serves completely asks for a
  // transformation of the list, not an observation about it (issue #1021).
  if (tryNumericList(text, [])) return null;
  const language = FORCED_RESPONSE_LANGUAGE || "en";
  const log = FORCED_RESPONSE_LANGUAGE ? [`language_to:${language}`] : [];
  const store = patternStoreNew();
  const grid = patternParseGrid(text);
  if (grid !== null) {
    const shaped = { rows: grid.rows, cols: grid.cols, cells: patternPoints(store, grid.tokens) };
    const rowMajor = patternInferSequence(store, shaped.cells);
    const symmetric = patternGridInvariantTransforms(shaped);
    const body = patternRender("pattern_grid_report", language, { structure: patternGridSummary(shaped, rowMajor, language) });
    log.push("pattern_inference:kind:grid", `pattern_inference:dimensions:${grid.rows}x${grid.cols}`,
      `pattern_inference:symmetries:${symmetric.length}`);
    const structured = patternGridSymmetries(shaped).length > 0 || patternSequenceHasStructure(rowMajor);
    return patternAnswer(body, log, structured);
  }
  const tokens = patternLongestRun(text);
  if (tokens.length === 0) return null;
  const report = patternInferSequence(store, patternPoints(store, tokens));
  let body = patternRender("pattern_sequence_report", language, {
    rendered: tokens.join(" "),
    structure: patternSequenceSummary(report, language),
  });
  const next = patternPredictNext(tokens, report.classification);
  if (next !== null) body += `\n${patternRender("pattern_sequence_next", language, { next: next })}`;
  log.push("pattern_inference:kind:sequence", `pattern_inference:length:${report.length}`,
    `pattern_inference:distinct:${report.distinct}`,
    `pattern_inference:compression_ratio:${patternFormatFixed(patternCompressionRatio(report.compression), 2)}`);
  return patternAnswer(body, log, patternSequenceHasStructure(report));
}
