// The meta reasoner's decomposition and memory (see
// formal_ai_worker_meta_reasoner.js).
//
// Decomposition: a message holding several artifact requests ("Write a
// function that ... . Write a function that ...") is split into sub-goals,
// each solved by the same loop with shared knowledge, and the answer composes
// their results. Memory: chunks learned by explanation-based learning leave
// the worker as an `append` memory operation in links notation and come back
// through the `memory` statements the app passes to solve, so a meaning
// discovered once is reused in later sessions without a lookup.

/**
 * The separate artifact requests of a message: sentences that each ask for
 * an artifact or carry their own examples.
 * @param {string} prompt
 * @returns {Array<string>}
 */
function metaSubRequests(prompt) {
  const markers = metaCueMarkers("artifact");
  const sentences = String(prompt || "")
    .split(/(?<=[.!?])\s+(?=\p{Lu})|\n+/u)
    .map((sentence) => sentence.trim())
    .filter(Boolean);
  const requests = sentences.filter((sentence) => {
    const lowered = ` ${sentence.toLowerCase()} `;
    return markers.some((marker) => lowered.includes(marker)) || metaExamples(sentence, metaValueLiterals(sentence)).length > 0;
  });
  return requests.length > 1 ? requests : [String(prompt || "")];
}

/**
 * Solve every sub-request as its own goal and compose the results. The
 * composite is solved when every sub-goal is.
 * @param {string} prompt
 * @param {Array<string>} requests
 * @param {string} language
 * @param {object} settings
 * @returns {Promise<object>}
 */
async function metaReasonComposite(prompt, requests, language, settings) {
  const trace = metaTrace();
  trace.emit("impulse", prompt);
  trace.emit("decompose", `${requests.length} sub-goals: ${requests.map((request, index) => `${index + 1}) ${request}`).join(" ")}`);
  const subgoals = [];
  for (const request of requests) {
    // eslint-disable-next-line no-await-in-loop -- sub-goals share knowledge in order.
    const sub = await metaReason(request, language, settings);
    subgoals.push(sub);
    trace.emit("subgoal", `${request} → ${sub.goal} ${sub.status}${sub.program ? ` (${sub.program.steps.join(" ∘ ")})` : ""}`);
  }
  const solved = subgoals.every((sub) => sub.status === "solved" && sub.program);
  const any = subgoals.some((sub) => sub.program);
  const result = {
    goal: "decompose",
    language,
    examples: [],
    definitions: [],
    unknowns: subgoals.flatMap((sub) => sub.unknowns),
    needs: [],
    program: null,
    verification: null,
    status: solved ? "solved" : any ? "partial" : "open",
    subgoals,
    trace,
    lookups: subgoals.flatMap((sub) => sub.lookups || []),
  };
  trace.emit(solved ? "goal_achieved" : "partial", metaNote("subgoals_programmed", { done: subgoals.filter((sub) => sub.program).length, total: subgoals.length }));
  result.derivationLino = metaDerivationLino(result);
  return result;
}

/**
 * The composed answer: each sub-request with its own answer.
 * @param {object} result
 * @param {boolean} [allowOpen]
 * @returns {object|null}
 */
function metaCompositeAnswer(result, allowOpen) {
  const language = result.language === "ru" ? "ru" : "en";
  const parts = [metaResponse("decomposed", language, { count: result.subgoals.length })];
  const evidence = ["meta_reasoner:derivation", ...result.trace.events.map((event) => `meta:${event.kind}:${event.detail}`)];
  let answered = 0;
  for (const [index, sub] of result.subgoals.entries()) {
    const answer = metaAnswer(sub, allowOpen);
    if (answer) answered += 1;
    parts.push("", `### ${metaResponse("subrequest", language, { index: index + 1, request: sub.trace.events[0].detail })}`, "", answer ? answer.content : metaResponse("open_unknowns", language, {
      terms: sub.unknowns.filter((unknown) => unknown.status === "open").map((unknown) => unknown.word).join(", ") || "—",
      question: metaResponse("decide_question", language, {}),
    }));
  }
  if (!answered) return null;
  const verified = result.subgoals.every((sub) => sub.verification && !sub.verification.failures.length);
  return {
    intent: verified ? "meta_reasoned_program" : "meta_reasoned_program_unverified",
    content: parts.join("\n"),
    confidence: verified ? 0.9 : 0.55,
    evidence,
  };
}

/**
 * Load learned chunks from the app's memory statements
 * (`meta_learned_chunk` records), so earlier discoveries are recalled.
 * @param {Array<string>} memory
 * @returns {number} chunks loaded
 */
function metaImportLearned(memory) {
  let loaded = 0;
  for (const value of Array.isArray(memory) ? memory : []) {
    const statement = String(value || "");
    if (!statement.includes("meta_learned_chunk")) continue;
    for (const node of parseLinoTree(statement).children) {
      if (!node || node.name !== "meta_learned_chunk") continue;
      const field = (name) => {
        const child = node.children.find((item) => item.name === name);
        return child ? String(child.value) : "";
      };
      const word = field("word");
      const operation = field("operation");
      if (!word || !operation || metaLearnedChunks.has(word)) continue;
      metaLearnedChunks.set(word, { operation, score: Number(field("score")) || 0.5, via: field("via") || "memory" });
      loaded += 1;
    }
  }
  return loaded;
}

/**
 * The chunks learned since the last call, as one `append` memory operation
 * for the app to persist, or null when nothing new was learned.
 * @returns {{action: string, kind: string, statement: string}|null}
 */
function metaTakeLearned() {
  if (!metaNewChunks.size) return null;
  const lines = [];
  for (const [word, chunk] of Array.from(metaNewChunks).sort((a, b) => a[0].localeCompare(b[0]))) {
    lines.push("meta_learned_chunk", `  word ${JSON.stringify(word)}`, `  operation ${chunk.operation}`,
      `  score ${chunk.score}`, `  via ${JSON.stringify(String(chunk.via || ""))}`);
  }
  metaNewChunks.clear();
  return { action: "append", kind: "meta_learned_chunk", statement: lines.join("\n") };
}

/**
 * Attach newly learned chunks to an answer as its memory operation, unless
 * the answer already carries one (that write goes first; the chunks wait for
 * the next turn).
 * @param {object} answer
 * @returns {object}
 */
function metaAttachLearned(answer) {
  if (!answer || answer.memoryOperation) return answer;
  const operation = metaTakeLearned();
  return operation ? { ...answer, memoryOperation: operation } : answer;
}
