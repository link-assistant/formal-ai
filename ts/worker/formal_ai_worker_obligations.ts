// The obligation report of a catalog `write_program` turn (issue #1166
// R1166-4), the browser twin of rust/src/intent_formalization/obligations.rs
// (`formalize_request`, `request_carries_work_obligations`, `request_demands`,
// `bound_output_literals`, `obligation_gap_lines`), the clause-level part of
// rust/src/obligation_ledger.rs and obligation_ledger/derivation.rs
// (`clauses_with_spans`, `ObligationNode::build`, `derive_expectation`) and the
// one-level checkable split of rust/src/task_decomposition.rs. JavaScript
// twins: js/agentic/crate/intent_formalization_obligations.mjs,
// obligation_ledger.mjs and task_decomposition.mjs. The plan reader
// `derive_expectation` consults is formal_ai_worker_obligation_plans.js.
//
// Vocabulary is the seed lexicon (formal_ai_worker_13.js). Two data/meta
// documents are not served to the worker, so their decisions are mirrored
// here and held equal to the files by
// rust/tests/web/issue-1166-worker-obligation-gaps.test.mjs: the expectation
// rules of data/meta/obligation-evidence-contract.lino, and the approval of
// the `missing_operation_contract` strategy in
// data/meta/task-decomposition-strategies.lino.

// `obligation_expectation_rules`: when, mode, expectation, otherwise, reason.
function obligationExpectationRules() {
  return [
    ["general_plan_mode", "literal_file", "file_bytes", "", ""],
    ["general_plan_mode", "command_output", "output_hash", "command_exit", ""],
    ["general_plan_mode", "repository_work_item", "underivable", "", "planned_not_executed_has_no_verification_command"],
    ["generated_check", "", "symbolic_check", "", ""],
    ["no_artifact", "", "underivable", "", "no_artifact_in_clause"],
    ["unbound_output_literal", "", "underivable", "", "output_literal_not_bound_by_executor"],
  ].map(([when, mode, expectation, otherwise, reason]) => ({ when, mode, expectation, otherwise, reason }));
}

// Whether the shipped strategy ledger approves a strategy activated by a
// missing operation contract (`TaskStrategyLedger::shipped`), so `plans_for`
// answers exactly when `missing_operation_contract` does.
function obligationMissingContractStrategyApproved() {
  return true;
}

const OBLIGATION_SPLIT_DEPTH_BOUND = 4;
const OBLIGATION_OBSERVABLE = new Set(["file_bytes", "output_hash", "command_exit", "symbolic_check"]);

// ------------------------------------------------------- quoted segments

const OBLIGATION_QUOTE_PAIRS = [
  ["```", "```"], ["'", "'"], ["\"", "\""], ["`", "`"], ["«", "»"],
  ["“", "”"], ["‘", "’"], ["「", "」"], ["『", "』"], ["《", "》"],
];

function obligationClosingDelimiter(text, cursor, close) {
  let from = cursor;
  for (;;) {
    const closeAt = text.indexOf(close, from);
    if (closeAt < 0) return null;
    const afterClose = closeAt + close.length;
    const apostrophe = close === "'"
      && oblIsAsciiAlphanumeric(Array.from(text.slice(0, closeAt)).pop())
      && oblIsAsciiAlphanumeric(Array.from(text.slice(afterClose, afterClose + 2))[0]);
    if (!apostrophe) return closeAt;
    from = afterClose;
  }
}

// Mirrors `fn quoted_segment_spans`: `{text, start, end}` per segment.
function obligationQuotedSegmentSpans(text) {
  const result = [];
  let cursor = 0;
  while (cursor < text.length) {
    let best = null;
    for (const [open, close] of OBLIGATION_QUOTE_PAIRS) {
      let from = cursor;
      for (;;) {
        const openAt = text.indexOf(open, from);
        if (openAt < 0) break;
        const wordQuote = open === "'" && oblIsAsciiAlphanumeric(Array.from(text.slice(0, openAt)).pop());
        if (!wordQuote && obligationClosingDelimiter(text, openAt + open.length, close) !== null) {
          if (best === null || openAt < best[0] || (openAt === best[0] && open.length > best[1].length)) best = [openAt, open, close];
          break;
        }
        from = openAt + open.length;
      }
    }
    if (!best) break;
    const [openAt, open, close] = best;
    const contentEnd = obligationClosingDelimiter(text, openAt + open.length, close);
    if (contentEnd === null) break;
    result.push({ text: text.slice(openAt + open.length, contentEnd), start: openAt, end: contentEnd + close.length });
    cursor = contentEnd + close.length;
  }
  return result;
}

// Mirrors `fn text_outside_quoted_segments`.
function obligationTextOutsideQuotes(prompt) {
  let outside = "";
  let cursor = 0;
  for (const segment of obligationQuotedSegmentSpans(prompt)) {
    if (segment.start < cursor) continue;
    outside += `${prompt.slice(cursor, segment.start)} `;
    cursor = segment.end;
  }
  return outside + prompt.slice(cursor);
}

// ---------------------------------------------------- task decomposition

function obligationIsRepositoryWorkItem(token) {
  const trimmed = oblTrimEndMatches(token, oblCharIn(".。!?"));
  let path = null;
  if (trimmed.startsWith("https://github.com/")) path = trimmed.slice("https://github.com/".length);
  else if (token.startsWith("http://github.com/")) path = token.slice("http://github.com/".length);
  if (path === null) return false;
  const segments = path.split("/");
  return segments.length === 4 && (segments[2] === "issues" || segments[2] === "pull")
    && Array.from(segments[3]).every(oblIsAsciiDigit);
}

// Mirrors `fn concrete_target`: whether the task names one.
function obligationHasConcreteTarget(task) {
  return oblSplitWhitespace(task).some((token) => {
    const cleaned = oblTrimMatches(token, oblCharIn("<>()[]{},;:\"'"));
    if (cleaned === "" || obligationIsRepositoryWorkItem(cleaned)) return false;
    const dot = cleaned.lastIndexOf(".");
    const fileLike = cleaned.includes("/") || (dot > 0 && dot < cleaned.length - 1);
    return fileLike || cleaned.includes("_")
      || (cleaned.includes("-") && Array.from(cleaned).every((c) => oblIsAlphanumeric(c) || c === "-"));
  });
}

// Mirrors `fn missing_operation_contract`.
function obligationMissingOperationContract(task) {
  const normalized = task.toLowerCase();
  const either = (role) => lexiconMentionsRole(role, normalized) || oblMentionsRoleRaw(role, normalized);
  return oblSplitWhitespace(task).some(obligationIsRepositoryWorkItem)
    || either("decomposable_task_noun")
    || (either("software_authoring_action") && !obligationHasConcreteTarget(task));
}

// Mirrors `fn is_checkable`.
function obligationIsCheckable(text) {
  const normalized = normalizePrompt(text);
  return lexiconMentionsRole("observable_task_action", normalized)
    && !lexiconMentionsRole("unobservable_task_action", normalized)
    && !obligationMissingOperationContract(text);
}

const obligationPlansFor = (text) => obligationMissingContractStrategyApproved() && obligationMissingOperationContract(text);

function obligationPushTrimmed(out, candidate) {
  const trimmed = oblTrim(candidate);
  if (trimmed !== "") out.push(trimmed);
}

function obligationSplitSentences(text) {
  const chars = Array.from(text);
  const out = [];
  let current = "";
  chars.forEach((character, index) => {
    current += character;
    const next = chars[index + 1];
    if ("?!。！？".includes(character) || (character === "." && (next === undefined || oblIsWhitespace(next)))) {
      obligationPushTrimmed(out, current);
      current = "";
    }
  });
  obligationPushTrimmed(out, current);
  return out;
}

function obligationSplitClauses(sentence) {
  const fold = (text) => text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
  const out = [];
  for (const chunk of sentence.split(/[,;，；、]/u)) {
    let pieces = [chunk];
    for (const marker of wordsForRole("clause_continuation_marker")) {
      pieces = pieces.flatMap((piece) => {
        if (containsCjk(marker)) return piece.split(marker).map(oblTrim);
        const toks = oblSplitWhitespace(piece);
        if (!toks.some((token) => fold(token) === fold(marker))) return [piece];
        const parts = [[]];
        for (const token of toks) {
          if (fold(token) === fold(marker)) parts.push([]);
          else parts[parts.length - 1].push(token);
        }
        return parts.map((part) => part.join(" "));
      });
    }
    for (const piece of pieces) obligationPushTrimmed(out, piece);
  }
  return out;
}

// Mirrors `fn segment`: quoted literals are masked while splitting.
function obligationSegment(task) {
  const base = 0xe000;
  const slots = 0x1900;
  const slotOf = (character) => {
    const index = character.codePointAt(0) - base;
    return index >= 0 && index < slots ? index : null;
  };
  let masked = task;
  const literals = [];
  if (!Array.from(task).some((character) => slotOf(character) !== null)) {
    masked = "";
    let cursor = 0;
    for (const span of obligationQuotedSegmentSpans(task)) {
      if (literals.length >= slots) break;
      masked += task.slice(cursor, span.start) + String.fromCodePoint(base + literals.length);
      literals.push(task.slice(span.start, span.end));
      cursor = span.end;
    }
    masked += task.slice(cursor);
  }
  const sentences = obligationSplitSentences(masked);
  const pieces = sentences.length > 1 ? sentences : obligationSplitClauses(masked);
  return pieces.map((piece) => Array.from(piece).map((character) => {
    const index = slotOf(character);
    return index !== null && index < literals.length ? literals[index] : character;
  }).join(""));
}

function obligationHeadAction(text) {
  if (containsCjk(text)) return null;
  const first = oblSplitWhitespace(text)[0];
  if (first === undefined) return null;
  return lexiconMentionsRole("observable_task_action", normalizePrompt(first)) ? first : null;
}

// Mirrors `fn split_once_checkable`.
function obligationSplitOnceCheckable(task) {
  const segments = obligationSegment(task);
  if (segments.length < 2) return [];
  const head = obligationHeadAction(segments[0]);
  const distributed = head === null ? segments : segments.map((text, index) =>
    (index === 0 || obligationHeadAction(text) !== null || containsCjk(text) || oblSplitWhitespace(text).length < 3
      ? text : `${head} ${text}`));
  const join = (left, right) => `${left}${containsCjk(left) || containsCjk(right) ? "，" : ", "}${right}`;
  const out = [];
  let pending = null;
  for (const text of distributed) {
    const joined = pending === null ? text : join(pending, text);
    pending = null;
    if (obligationIsCheckable(text) || obligationPlansFor(text)) out.push(joined);
    else pending = joined;
  }
  if (pending !== null) {
    if (out.length) out[out.length - 1] = join(out[out.length - 1], pending);
    else out.push(pending);
  }
  return out.length < 2 ? [] : out;
}

// ------------------------------------------------------ the ledger tree

// Mirrors `fn derive_expectation`: `{kind, reason}`.
function obligationDeriveExpectation(clause) {
  const rules = obligationExpectationRules();
  const mode = obligationPlanMode(clause);
  if (mode !== null) {
    const rule = rules.find((candidate) => candidate.when === "general_plan_mode" && candidate.mode === mode);
    if (!rule) return { kind: "underivable", reason: `no_rule_for_plan_mode_${mode}` };
    // Only a literal-file plan states content, so only it declares a digest;
    // every other mode falls back to the rule's `otherwise` shape.
    const shape = mode !== "literal_file" && rule.otherwise ? rule.otherwise : rule.expectation;
    return OBLIGATION_OBSERVABLE.has(shape) ? { kind: shape, reason: "" } : { kind: "underivable", reason: rule.reason };
  }
  const generated = rules.find((rule) => rule.when === "generated_check");
  if (obligationIsCheckable(clause) && (!generated || generated.expectation === "symbolic_check")) {
    return { kind: "symbolic_check", reason: "" };
  }
  const noArtifact = rules.find((rule) => rule.when === "no_artifact");
  return { kind: "underivable", reason: noArtifact ? noArtifact.reason : "no_artifact_in_clause" };
}

// Rust `format!("{:?}", option)` for an `Option<String>`.
function obligationDebugOption(value) {
  if (value === null) return "None";
  let out = "\"";
  for (const character of value) {
    const code = character.codePointAt(0);
    const escapes = { "\"": "\\\"", "\\": "\\\\", "\n": "\\n", "\r": "\\r", "\t": "\\t", "\0": "\\0" };
    if (escapes[character]) out += escapes[character];
    else if (code < 0x20 || code === 0x7f) out += `\\u{${code.toString(16)}}`;
    else out += character;
  }
  return `Some(${out}")`;
}

function obligationLeaf(parent, clause, span, depth) {
  const trimmed = oblTrim(clause);
  return {
    node_id: stableBehaviorRuleId("obligation", `${obligationDebugOption(parent)}:${depth}:${trimmed}`),
    clause: trimmed,
    span,
    depth,
    expectation: obligationDeriveExpectation(trimmed),
    children: [],
  };
}

// Mirrors `fn clauses_with_spans`: `[clause, [start, end]]` in UTF-8 bytes.
function obligationClausesWithSpans(request) {
  const cues = wordsForRole("enumeration_cue");
  if (!cues.length) return [[oblTrim(request), [0, oblUtf8Len(request)]]];
  const unspaced = (character) => {
    const cp = character.codePointAt(0);
    return (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff)
      || (cp >= 0xac00 && cp <= 0xd7af) || (cp >= 0xf900 && cp <= 0xfaff);
  };
  const startsWithCue = (rest, cue) => {
    if (!rest.startsWith(cue)) return false;
    if (Array.from(cue).some(unspaced)) return true;
    const next = Array.from(rest.slice(cue.length))[0];
    return next === undefined || !oblIsAlphanumeric(next);
  };
  const opensAClause = (index) => {
    const lineEnd = oblTrimEndMatches(request.slice(0, index), oblCharIn(" \t\r"));
    const before = oblTrimEnd(lineEnd);
    return before === "" || lineEnd.endsWith("\n")
      || [".", "!", "?", ";", ":", "。", "！", "？", "।", "॥"].some((mark) => before.endsWith(mark));
  };
  const boundaries = [0];
  for (let index = 0; index < request.length; index += request.codePointAt(index) > 0xffff ? 2 : 1) {
    if (index === 0 || !opensAClause(index)) continue;
    const rest = request.slice(index).toLowerCase();
    if (cues.some((cue) => startsWithCue(rest, cue))) boundaries.push(index);
  }
  boundaries.push(request.length);
  const unique = boundaries.filter((value, at) => at === 0 || value !== boundaries[at - 1]);
  const out = [];
  for (let at = 0; at + 1 < unique.length; at += 1) {
    const raw = request.slice(unique[at], unique[at + 1]);
    const trimmed = oblTrim(raw);
    if (trimmed === "") continue;
    const start = oblUtf8Len(request.slice(0, unique[at] + raw.indexOf(trimmed)));
    out.push([trimmed, [start, start + oblUtf8Len(trimmed)]]);
  }
  return out;
}

function obligationExpand(node) {
  if (OBLIGATION_OBSERVABLE.has(node.expectation.kind) || node.depth >= OBLIGATION_SPLIT_DEPTH_BOUND) return;
  const pieces = obligationSplitOnceCheckable(node.clause);
  if (pieces.length < 2) return;
  let cursor = 0;
  const children = [];
  for (const piece of pieces) {
    const trimmed = oblTrim(piece);
    if (trimmed === "") continue;
    const offset = node.clause.slice(cursor).indexOf(trimmed);
    let span = node.span;
    if (offset >= 0) {
      const start = node.span[0] + oblUtf8Len(node.clause.slice(0, cursor + offset));
      cursor += offset + trimmed.length;
      span = [start, start + oblUtf8Len(trimmed)];
    }
    const child = obligationLeaf(node.node_id, trimmed, span, Math.min(255, node.depth + 1));
    obligationExpand(child);
    children.push(child);
  }
  if (children.length >= 2) node.children = children;
}

// The clause-level nodes `formalize_request` reads from `ObligationNode::build`.
function obligationClauseNodes(request) {
  const root = obligationLeaf(null, request, [0, oblUtf8Len(request)], 0);
  const clauses = obligationClausesWithSpans(request);
  if (clauses.length < 2) {
    obligationExpand(root);
  } else {
    root.children = clauses.map(([clause, span]) => {
      const child = obligationLeaf(root.node_id, clause, span, 1);
      obligationExpand(child);
      return child;
    });
  }
  return root.children.length > 0 ? root.children : [root];
}

// --------------------------------------------------- the obligation graph

const OBLIGATION_AUTHORING_ROLES = [
  ["code_style_obligation", "code_style"],
  ["file_naming_obligation", "file_naming"],
  ["ci_badge_obligation", "ci_badge"],
];

// Mirrors `fn authoring_kind`.
function obligationAuthoringKind(clause) {
  const normalized = normalizePrompt(obligationTextOutsideQuotes(clause));
  if (lexiconMentionsRole("ci_workflow_request", normalized)) return "ci_workflow";
  if (lexiconMentionsRole("program_request", normalized) || lexiconMentionsRole("coding_request_verb", normalized)) return "program_file";
  const lower = normalized.toLowerCase();
  const found = OBLIGATION_AUTHORING_ROLES.find(([role]) => oblMentionsRoleRaw(role, lower));
  return found ? found[1] : null;
}

function obligationPrintEvidenced(lower) {
  const meaning = findMeaning("print_stdout");
  return Boolean(meaning) && meaningEvidencedIn(meaning, lower);
}

// Mirrors `fn formalize_request` with its `coreference_pass`: the clause
// nodes, each with its `kind` and output `literal` (or null).
function obligationFormalizeRequest(text) {
  const seen = [];
  const graph = [];
  for (const node of obligationClauseNodes(text)) {
    const lower = normalizePrompt(obligationTextOutsideQuotes(node.clause)).toLowerCase();
    const quoted = obligationQuotedSegmentSpans(node.clause);
    let kind = null;
    let literal = null;
    if ((obligationPrintEvidenced(lower) || oblMentionsRoleRaw("output_obligation_verb", lower))
      && quoted.length > 0 && quoted[quoted.length - 1].text !== "") {
      kind = "output_literal";
      literal = quoted[quoted.length - 1].text;
    } else {
      kind = obligationAuthoringKind(node.clause);
    }
    if (kind === "output_literal" && seen.includes(literal)) continue;
    if (kind === "output_literal") seen.push(literal);
    graph.push({ node, kind, literal });
  }
  return graph;
}

// Mirrors `fn bound_output_literals`.
function obligationBoundOutputLiterals(text) {
  const clauseStarts = obligationClausesWithSpans(text).map(([, span]) => {
    let bytes = 0;
    let index = 0;
    for (const character of text) {
      if (bytes >= span[0]) break;
      bytes += oblUtf8Len(character);
      index += character.length;
    }
    return index;
  });
  let previousEnd = 0;
  const outputs = [];
  for (const literal of obligationQuotedSegmentSpans(text)) {
    const clauseStart = clauseStarts.filter((start) => start <= literal.start).reduce((max, start) => Math.max(max, start), 0);
    const introduction = text.slice(Math.max(previousEnd, clauseStart), literal.start).split(/[\n.;。]/u).pop();
    previousEnd = literal.end;
    if (obligationPrintEvidenced(introduction.toLowerCase()) && !outputs.includes(literal.text)) outputs.push(literal.text);
  }
  return outputs;
}

// Mirrors `fn obligation_gap_lines`: the gap report, then the unbound output
// literals, one `obligation <id> span <start>:<end> underivable <reason>` each.
function obligationGapLines(text, graph = obligationFormalizeRequest(text)) {
  const line = (node, reason) => `obligation ${node.node_id} span ${node.span[0]}:${node.span[1]} underivable ${reason}`;
  const gaps = graph.filter(({ node }) => node.expectation.kind === "underivable").map(({ node }) => line(node, node.expectation.reason));
  const bound = obligationBoundOutputLiterals(text);
  const unbound = obligationExpectationRules().find((rule) => rule.when === "unbound_output_literal");
  for (const { node, kind, literal } of graph) {
    if (kind === "output_literal" && !bound.includes(literal)) gaps.push(line(node, unbound ? unbound.reason : "unbound_output_literal"));
  }
  return gaps;
}

// Mirrors `fn request_carries_work_obligations` (`ci_workflow` true adds
// `request_demands(prompt, CiWorkflow)`, the native `WriteProgram` gate).
function obligationCarriesWork(graph, ciWorkflow = false) {
  const authoring = graph.map(({ node }) => obligationAuthoringKind(node.clause)).filter(Boolean);
  const has = (kind) => graph.some((entry) => entry.kind === kind) || authoring.includes(kind);
  return (graph.some((entry) => entry.kind === "output_literal") && authoring.length > 0)
    || has("file_naming") || has("ci_badge") || (ciWorkflow && has("ci_workflow"));
}
const obligationCatalogGateOpen = (graph) => obligationCarriesWork(graph, true);

// The `obligation_gap` events `record_obligation_gaps` appends for a catalog
// `write_program` request, in report order; none when the gate is closed.
function solverObligationGapEvents(prompt) {
  const graph = obligationFormalizeRequest(prompt);
  if (!obligationCatalogGateOpen(graph)) return [];
  return obligationGapLines(prompt, graph).map((gap) => solverEvent("obligation_gap", gap));
}
