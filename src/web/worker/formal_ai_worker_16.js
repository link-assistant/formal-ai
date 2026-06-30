// Worker module 17 of 21. Loaded by ../formal_ai_worker.js.
function writeProgramDecompositionParts(modifier) {
  if (modifier === "reverse_sort") {
    return {
      operation: "sort",
      operationModifier: "descending",
      target: "program:last.output_order",
      targetKind: "program_output",
    };
  }
  if (modifier === "cancel_reverse_sort") {
    // Issue #386: the inverse of reverse_sort — cancel the descending order
    // over the same program-output target.
    return {
      operation: "cancel",
      operationModifier: "reverse_sort",
      target: "program:last.output_order",
      targetKind: "program_output",
    };
  }
  if (modifier === "path_argument") {
    return {
      operation: "accept",
      operationModifier: "path_argument",
      target: "program:last.input",
      targetKind: "program_input",
    };
  }
  return {
    operation: "modify",
    operationModifier: null,
    target: "program:last",
    targetKind: "program_artifact",
  };
}

function writeProgramPrimaryModifier(modifiers) {
  if (!Array.isArray(modifiers) || modifiers.length === 0) return null;
  return modifiers.includes("reverse_sort") ? "reverse_sort" : modifiers[0];
}

function writeProgramCandidateRuleId(plan, modifier) {
  if (!plan || !modifier) return "";
  const traces = Array.isArray(plan.traces) ? plan.traces : [];
  for (let index = traces.length - 1; index >= 0; index -= 1) {
    const ruleId = String(traces[index].ruleId || "");
    if (ruleId.includes(modifier)) return ruleId;
  }
  return `${modifier}_${plan.baseTask || "program"}`;
}

function writeProgramSynthesisRequest(context, prompt, modifier) {
  const parts = writeProgramDecompositionParts(modifier);
  const lines = ["rule_synthesis_request"];
  lines.push("  issue #359");
  lines.push("  impulse current_turn");
  lines.push("  artifact program:last");
  lines.push(`  artifact_language ${context.language || "missing"}`);
  lines.push(`  base_task ${context.task || "missing"}`);
  lines.push("  bare_imperative true");
  lines.push(`  operation ${parts.operation}`);
  if (parts.operationModifier) {
    lines.push(`  operation_modifier ${parts.operationModifier}`);
  }
  lines.push(`  target ${parts.target}`);
  lines.push(`  target_kind ${parts.targetKind}`);
  lines.push(`  source_text ${prompt}`);
  return lines.join("\n");
}

function writeProgramSynthesisCandidate(candidateId, context, plan, modifier) {
  const parts = writeProgramDecompositionParts(modifier);
  const lines = ["rule_synthesis_candidate"];
  lines.push(`  id ${candidateId}`);
  lines.push("  source constructed_from_operation_vocabulary");
  lines.push(`  base_task ${context.task || "missing"}`);
  lines.push(`  modifier ${modifier}`);
  lines.push(`  operation ${parts.operation}`);
  if (parts.operationModifier) {
    lines.push(`  operation_modifier ${parts.operationModifier}`);
  }
  lines.push(`  target ${parts.target}`);
  lines.push(`  resolved_task ${(plan && plan.resolvedTask) || "missing"}`);
  return lines.join("\n");
}

function templateHasDescendingOrder(code) {
  const compact = String(code || "")
    .toLowerCase()
    .split(/\s+/)
    .join("");
  return [
    "sort_by(|a,b|b.cmp(a))",
    "reverse=true",
    ".sort().reverse()",
    "sort.sort(sort.reverse",
    "compare_desc",
    "rbegin(),names.rend()",
    "comparator.reverseorder()",
    "orderbydescending",
    "sort.reverse",
  ].some((marker) => compact.includes(marker));
}

function writeProgramVerificationTrace(candidateId, plan, template, modifiers) {
  if (!plan || !Array.isArray(modifiers) || modifiers.length === 0) return null;
  const planCheck =
    programPlanWasModified(plan) && Array.isArray(plan.traces) && plan.traces.length > 0;
  // Issue #386: verify the rendered program actually matches the operation. A
  // reverse_sort must leave the output descending; its inverse,
  // cancel_reverse_sort, must leave NO descending order — otherwise the cancel
  // silently failed to remove the sort. Modifiers that touch no ordering pass.
  const cancelsSort = modifiers.includes("cancel_reverse_sort");
  const reversesSort = modifiers.includes("reverse_sort");
  const descending = templateHasDescendingOrder(template);
  const renderCheck = cancelsSort ? !descending : reversesSort ? descending : true;
  const passed = planCheck && renderCheck;
  const expectedOrder = reversesSort && !cancelsSort ? "c.txt,b.txt,a.txt" : "a.txt,b.txt,c.txt";
  return [
    "rule_verification",
    `  candidate ${candidateId}`,
    "  fixture list_files_output_order",
    "  input a.txt,b.txt,c.txt",
    `  expected_order ${expectedOrder}`,
    `  lowering_check ${planCheck ? "passed" : "failed"}`,
    `  render_check ${renderCheck ? "passed" : "failed"}`,
    `  status ${passed ? "passed" : "failed"}`,
  ].join("\n");
}

function writeProgramDiagnosticBundle({
  prompt,
  initiallyDetected,
  coreference,
  detected,
  modifiers,
  lowered,
  plan,
  template,
}) {
  const steps = [];
  const trace = [];
  const coreferenceTrace = coreference && coreference.trace;
  if (!initiallyDetected && coreferenceTrace) {
    const detail = "selected_rule initial unknown reason no_seed_route next try_rule_synthesis";
    steps.push({ step: "route_attempt", detail });
    trace.push(`selected_rule:${detail}`);
  } else {
    const detail = "selected_rule write_program reason seed_route";
    steps.push({ step: "route_attempt", detail });
    trace.push(`selected_rule:${detail}`);
  }

  if (coreferenceTrace) {
    const detail = `write_program_coreference_rewrite\n  ${coreferenceTrace}`;
    steps.push({ step: "coreference_binding", detail });
    trace.push(`write_program_coreference_rewrite:${coreferenceTrace}`);
  }

  const safeModifiers = Array.isArray(modifiers) ? modifiers : [];
  if (safeModifiers.length > 0) {
    const operationHits = safeModifiers.join(",");
    const detail = `rule_synthesis_operation_vocabulary\n  ${operationHits}`;
    steps.push({ step: "modifier_detection", detail });
    trace.push(`rule_synthesis_operation_vocabulary:${operationHits}`);
  }

  const primaryModifier = writeProgramPrimaryModifier(safeModifiers);
  if (primaryModifier && lowered && programPlanWasModified(lowered)) {
    const context = {
      task: (detected && detected.task) || lowered.baseTask || "missing",
      language: (detected && detected.language) || "missing",
    };
    const candidateId = writeProgramCandidateRuleId(lowered, primaryModifier);
    const request = writeProgramSynthesisRequest(context, prompt, primaryModifier);
    const candidate = writeProgramSynthesisCandidate(
      candidateId,
      context,
      lowered,
      primaryModifier,
    );
    const construction = `${request}\n${candidate}`;
    steps.push({ step: "rule_construction", detail: construction });
    trace.push(`rule_synthesis_request:${request}`);
    trace.push(`rule_synthesis_candidate:${candidate}`);

    const verification = writeProgramVerificationTrace(
      candidateId,
      lowered,
      template,
      safeModifiers,
    );
    if (verification) {
      steps.push({ step: "rule_verification", detail: verification });
      trace.push(`rule_verification:${verification}`);
    }
  }

  if (plan) {
    const detail = `write_program_plan\n${plan}`;
    steps.push({ step: "program_plan", detail });
    trace.push(`write_program_plan:${plan}`);
  }

  return { steps, trace };
}

function tryWriteProgram(prompt, history, responseLanguage, composition) {
  let detected = writeProgramParameters(prompt);
  const initiallyDetected = detected;
  const coreference = detected ? null : rewriteBareProgramCoreference(prompt, history);
  if (!detected && !coreference) return null;
  if (coreference) detected = coreference.parameters;
  // Issue #324: recover task/language from the conversation when a follow-up
  // modification names neither (and apply any path-argument modifier).
  const { language, task, plan, modifiers, lowered } = recoverWriteProgramParameters(
    detected,
    prompt,
    history,
  );
  // Issue #324: answer in the language of the request (falls back to en).
  const i18n = writeProgramStrings(responseLanguage);
  const template = language && task ? WRITE_PROGRAM_TEMPLATES[task]?.[language] : null;
  const diagnostics = writeProgramDiagnosticBundle({
    prompt,
    initiallyDetected,
    coreference,
    detected,
    modifiers,
    lowered,
    plan,
    template,
  });
  if (!template) {
    // Issue #340 (R7): before the unsupported dead end, try a composite
    // blueprint. When the request decomposes into a recognized recipe for a
    // language we ship a curated program for, return that program with its plan
    // and an honest "not run" report (mirrors `try_program_blueprint` in
    // `src/solver_handlers/program_blueprint.rs`).
    const normalizedForBlueprint = normalizeProgramPrompt(prompt);
    const blueprintLanguage =
      language || programLanguageFromPrompt(normalizedForBlueprint);
    const blueprint = blueprintLanguage
      ? selectBlueprint(normalizedForBlueprint, blueprintLanguage)
      : null;
    if (blueprint) {
      return blueprintWriteProgramAnswer(
        blueprint,
        blueprintLanguage,
        responseLanguage,
        composition,
      );
    }
    // Issue #412 (R6): before the unsupported dead end, try the coding oracle —
    // the cached external knowledge bases (Hello World Collection, Rosetta Code,
    // …) answer for languages the verified catalog does not template (Kotlin,
    // Swift, PHP, …). Mirrors `try_write_program_from_oracle` in
    // `src/solver_handler_oracle.rs`.
    const oracleTask = task || programTaskFromPrompt(normalizeProgramPrompt(prompt));
    const oracleAnswer = language ? codingOracleAnswer(oracleTask, language) : null;
    if (oracleAnswer) return oracleAnswer;
    return {
      intent: "write_program_unsupported",
      content: i18n.unsupported(
        language || "missing",
        task || "missing",
        Object.keys(WRITE_PROGRAM_LANGUAGES).join(", "),
        Object.keys(WRITE_PROGRAM_TASKS).join(", "),
      ),
      confidence: 0.4,
      evidence: [
        "response:write_program:unsupported",
        `program_parameter:language:${language || "missing"}`,
        `program_parameter:task:${task || "missing"}`,
      ],
      steps: diagnostics.steps,
      trace: diagnostics.trace,
    };
  }
  const languageInfo = WRITE_PROGRAM_LANGUAGES[language];
  const taskInfo = WRITE_PROGRAM_TASKS[task];
  const expectedOutput = writeProgramExpectedOutput(task, languageInfo, taskInfo);
  // The sandbox can only execute self-contained JavaScript; a snippet that pulls
  // in Node APIs (e.g. the list-files `require("fs")`) cannot run here (#312).
  const ranInSandbox =
    language === "javascript" && !/\brequire\s*\(|\bimport\b/.test(template);
  const lines = [];
  lines.push(i18n.intro(languageInfo.name, taskInfo.label));
  lines.push("");
  lines.push("```" + languageInfo.fence);
  lines.push(template);
  lines.push("```");
  lines.push("");
  lines.push(
    ...writeProgramExecutionLines(language, task, template, expectedOutput, i18n),
  );
  // Issue #330 (R9): teach a novice — append a plain-language explanation of how
  // the code works, then step-by-step instructions for testing it. Follow-up
  // edits (when the dialog already showed code) drop the verbose setup steps.
  const priorCode = historyHasPriorCode(history);
  lines.push("");
  lines.push(programExplanationSection(task, responseLanguage));
  lines.push("");
  lines.push(programTestInstructions(languageInfo, responseLanguage, priorCode));
  const content = applyInlineHelloWorldOutputReplacement(prompt, task, lines.join("\n"));
  return {
    intent: "write_program",
    content,
    confidence: 0.9,
    evidence: [
      `response:write_program:${task}:${language}`,
      `program_parameter:language:${language}`,
      `program_parameter:task:${task}`,
      `program_parameters:write_program(language=${language}:task=${task})`,
      task === "hello_world"
        ? `legacy_intent:hello_world_${language}`
        : `legacy_intent:write_program_${task}_${language}`,
      `execution_status:${language}:${ranInSandbox ? "ran" : "unavailable"}`,
      // Issue #324 R4/R6: surface the substitution plan when a follow-up
      // modification rewrote the task (mirrors the Rust `write_program_plan`
      // event in `src/solver.rs`).
      ...(plan ? [`write_program_plan:${task}`] : []),
      ...(coreference
        ? [`write_program_coreference_rewrite:${task || "missing"}:${language || "missing"}`]
        : []),
    ],
    steps: diagnostics.steps,
    trace: diagnostics.trace.length ? diagnostics.trace : undefined,
  };
}

function tryHistorical(prompt, history) {
  const normalized = normalizePrompt(prompt);
  // Issue #386: a conversation-summary request is recognised by composing
  // meaning roles (see isSummarizePrompt) across every supported language, so
  // we test it before the empty-normalized bail-out below.
  if (isSummarizePrompt(normalized)) {
    return trySummarizeConversation(history);
  }
  if (!normalized) return null;
  if (normalized === "what is my name" || normalized === "what s my name") {
    const hit = tryRecallName(history);
    if (hit) return hit;
  }
  // Issue #529: recognise "what was written in the previous message?" across
  // every supported language via the conversation_recall_previous_message seed
  // role before the English-only last-question phrases below.
  const previousMessage = tryRecallPreviousMessage(prompt, history);
  if (previousMessage) return previousMessage;
  if (
    normalized === "what was my previous question" ||
    normalized === "what was the previous question" ||
    normalized === "what was my last question"
  ) {
    return tryRecallLastQuestion(history);
  }
  return null;
}

// Issue #386 research comparison-table roles — mirror the ROLE_COMPARISON_*
// and ROLE_RESEARCH_* consts in src/seed/roles.rs. The strong trigger, the weak
// table-noun/difference-cue pair, the research-prompt signals (bare markers plus
// prefix surfaces), and the per-column criterion words all live in
// data/seed/meanings-research-table.lino (loaded into MEANINGS_LINO), so
// the handler references the concepts, not raw words in one language.
const ROLE_COMPARISON_TABLE_TRIGGER = "comparison_table_trigger";
const ROLE_COMPARISON_TABLE_NOUN = "comparison_table_noun";
const ROLE_COMPARISON_DIFFERENCE_CUE = "comparison_difference_cue";
const ROLE_RESEARCH_PROMPT_SIGNAL = "research_prompt_signal";
const ROLE_RESEARCH_CRITERION = "research_criterion";

// True when the prompt asks for a comparison drawn as a table: a strong
// comparison_table_trigger, or the weak pair of a comparison_table_noun and a
// comparison_difference_cue — each token-bounded across every supported
// language. Mirrors is_comparison_table_request in
// src/solver_handlers/research_table.rs.
function isComparisonTableRequest(normalized) {
  return (
    lexiconMentionsRole(ROLE_COMPARISON_TABLE_TRIGGER, normalized) ||
    (lexiconMentionsRole(ROLE_COMPARISON_TABLE_NOUN, normalized) &&
      lexiconMentionsRole(ROLE_COMPARISON_DIFFERENCE_CUE, normalized))
  );
}

// True when `prompt` was itself a research request — the prior turn a
// comparison-table follow-up reuses for its topics. research_prompt_signal
// carries bare markers (matched token-bounded anywhere) and prefix surfaces
// (matched when the prompt opens with the literal before the … slot); both live
// in the seed data. Mirrors looks_like_research_prompt in
// src/solver_handlers/research_table.rs.
function looksLikeResearchPrompt(prompt) {
  const normalized = normalizePrompt(prompt);
  if (lexiconMentionsRole(ROLE_RESEARCH_PROMPT_SIGNAL, normalized)) return true;
  return roleWordForms(ROLE_RESEARCH_PROMPT_SIGNAL)
    .filter((form) => form.slot === "prefix")
    .some((form) => normalized.startsWith(form.before));
}

function stripResearchListMarker(line) {
  const value = String(line || "").trim();
  if (!value) return "";
  if (/^[-*+]\s+/u.test(value)) {
    return value.replace(/^[-*+]\s+/u, "").trim();
  }
  if (/^\d+[.):]\s*/u.test(value)) {
    return value.replace(/^\d+[.):]\s*/u, "").trim();
  }
  return "";
}

function cleanResearchText(value) {
  return String(value || "")
    .trim()
    .replace(/^[`"':;,.?!\s]+/u, "")
    .replace(/[`"':;,.?!\s]+$/u, "")
    .replace(/\s+/g, " ")
    .trim();
}

function extractResearchTopics(prompt) {
  const topics = [];
  for (const line of String(prompt || "").split(/\r?\n/u)) {
    const item = cleanResearchText(stripResearchListMarker(line));
    if (!item || looksLikeResearchPrompt(item)) continue;
    if (!topics.some((topic) => topic.toLowerCase() === item.toLowerCase())) {
      topics.push(item);
    }
    if (topics.length >= 8) break;
  }
  if (topics.length === 0 && String(prompt || "").includes(":")) {
    const item = cleanResearchText(String(prompt).split(":").slice(1).join(":"));
    if (item) topics.push(item);
  }
  return topics;
}

const RESEARCH_TABLE_DEFAULT_CRITERIA = [
  "key_differences",
  "use_cases",
  "advantages",
  "disadvantages",
];

const RESEARCH_TABLE_CRITERION_LABELS = {
  key_differences: "Key differences",
  use_cases: "Use cases",
  advantages: "Advantages",
  disadvantages: "Disadvantages",
};

function pushUniqueCriterion(criteria, criterion) {
  if (!criteria.includes(criterion)) criteria.push(criterion);
}

// Add every comparison column the text names. Walks the research_criterion
// meanings in declaration order (which fixes the column order) and adds a
// criterion when any of its surface words occurs as a raw substring — the same
// substring contract the legacy code used, so space-guarded stems like 'pro '
// and ' con ' still avoid matching inside 'process'/'control'. The trigger words
// live in the seed data; only the language-independent slug keys each column.
// Mirrors append_criteria_from_text in src/solver_handlers/research_table.rs.
function appendResearchCriteriaFromText(text, criteria) {
  const normalized = normalizePrompt(text);
  for (const meaning of meaningsWithRole(ROLE_RESEARCH_CRITERION)) {
    if (
      RESEARCH_TABLE_CRITERION_LABELS[meaning.slug] &&
      meaning.words.some((word) => word && normalized.includes(word))
    ) {
      pushUniqueCriterion(criteria, meaning.slug);
    }
  }
}

function extractResearchCriteria(prompt) {
  const criteria = [];
  for (const line of String(prompt || "").split(/\r?\n/u)) {
    const item = stripResearchListMarker(line);
    if (item) appendResearchCriteriaFromText(item, criteria);
  }
  if (criteria.length === 0) appendResearchCriteriaFromText(prompt, criteria);
  return criteria.length > 0 ? criteria : RESEARCH_TABLE_DEFAULT_CRITERIA.slice();
}

function researchTableCell(topic, criterion) {
  const normalized = normalizePrompt(topic);
  if (normalized.includes("machine learning algorithm")) {
    if (criterion === "key_differences") {
      return "Broad family of data-driven methods; includes supervised, unsupervised, and reinforcement approaches.";
    }
    if (criterion === "use_cases") {
      return "Classification, regression, clustering, recommendation, anomaly detection, and forecasting.";
    }
    if (criterion === "advantages") {
      return "Flexible toolkit; often efficient on structured data; many models are easier to inspect than deep nets.";
    }
    if (criterion === "disadvantages") {
      return "Model choice, preprocessing, and feature design can dominate results; overfitting remains a risk.";
    }
  }
  if (normalized.includes("deep learning") && normalized.includes("traditional ml")) {
    if (criterion === "key_differences") {
      return "Deep learning learns layered representations; traditional ML often relies more on explicit feature engineering.";
    }
    if (criterion === "use_cases") {
      return "Deep learning fits images, speech, and language at scale; traditional ML fits many tabular and smaller-data tasks.";
    }
    if (criterion === "advantages") {
      return "Deep learning scales with data and reduces manual features; traditional ML is usually faster and more interpretable.";
    }
    if (criterion === "disadvantages") {
      return "Deep learning needs more data/compute and is harder to explain; traditional ML may underfit unstructured signals.";
    }
  }
  if (normalized.includes("neural network")) {
    if (criterion === "key_differences") {
      return "Built from weighted layers, activations, losses, and optimization; provides the base mechanism for deep learning.";
    }
    if (criterion === "use_cases") {
      return "Pattern recognition, embeddings, sequence modeling, vision, speech, and nonlinear function approximation.";
    }
    if (criterion === "advantages") {
      return "Captures nonlinear relationships and can be trained end-to-end for complex perception tasks.";
    }
    if (criterion === "disadvantages") {
      return "Requires tuning and regularization; decisions can be opaque; training can be unstable on poor data.";
    }
  }
  if (criterion === "key_differences") {
    return "Use the prior search sources to identify what distinguishes this topic from the others.";
  }
  if (criterion === "use_cases") {
    return "Summarize the practical settings where the Step 1 sources apply this topic.";
  }
  if (criterion === "advantages") {
    return "Extract strengths reported by the prior search sources before treating them as verified.";
  }
  return "Extract limitations reported by the prior search sources before treating them as verified.";
}

function escapeResearchTableCell(value) {
  return String(value || "").replace(/\|/g, "\\|").replace(/\n/g, " ");
}

function renderResearchComparisonTable(topics, criteria) {
  const lines = [
    "Research comparison table (draft; verify claims against the Step 1 source links).",
    "",
    `| Topic | ${criteria.map((criterion) => RESEARCH_TABLE_CRITERION_LABELS[criterion]).join(" | ")} |`,
    `| --- | ${criteria.map(() => "---").join(" | ")} |`,
  ];
  for (const topic of topics) {
    lines.push(
      `| ${escapeResearchTableCell(topic)} | ${criteria
        .map((criterion) => escapeResearchTableCell(researchTableCell(topic, criterion)))
        .join(" | ")} |`,
    );
  }
  return lines.join("\n");
}

function compactResearchLogValue(value) {
  return String(value || "").split(/\s+/u).filter(Boolean).join(" ");
}

function tryResearchComparisonTable(prompt, normalized, history = []) {
  if (!isComparisonTableRequest(normalized)) return null;
  const priorSearch = lastHistoryTurn(history, "user");
  if (!priorSearch || !looksLikeResearchPrompt(priorSearch)) return null;
  const topics = extractResearchTopics(priorSearch);
  if (topics.length < 2) return null;
  const criteria = extractResearchCriteria(prompt);
  if (criteria.length === 0) return null;
  return {
    intent: "research_comparison_table",
    content: renderResearchComparisonTable(topics, criteria),
    confidence: 0.78,
    evidence: [
      `research_table:prior_search:${compactResearchLogValue(priorSearch)}`,
      ...topics.map((topic) => `research_table:topic:${topic}`),
      ...criteria.map((criterion) => `research_table:criterion:${criterion}`),
    ],
  };
}

const RESEARCH_RESULT_FOLLOWUP_PROMPTS = new Set([
  "result",
  "the result",
  "what result",
  "what is result",
  "what s the result",
  "what is the result",
  "what was the result",
  "what are the results",
  "what were the results",
  "show the result",
  "show the results",
  "give me the result",
  "give me the results",
  "what is the answer",
  "what was the answer",
  "what did you find",
  "what did we find",
  "what is the outcome",
  "what was the outcome",
]);

function isResearchResultFollowup(normalized) {
  return RESEARCH_RESULT_FOLLOWUP_PROMPTS.has(normalizePrompt(normalized));
}

function classifyPriorResearchAnswer(answer) {
  const normalized = normalizePrompt(answer || "");
  if (
    normalized.includes("no cors enabled web search results") ||
    normalized.includes("no cors readable web search results") ||
    normalized.includes("no usable cors search results") ||
    normalized.includes("не получены результаты веб поиска") ||
    normalized.includes("未获取到") ||
    normalized.includes("कोई खोज परिणाम नहीं")
  ) {
    return "no_results";
  }
  if (
    normalized.includes("all cors readable search providers are disabled") ||
    normalized.includes("all cors enabled search providers are disabled") ||
    normalized.includes("все cors совместимые поисковые провайдеры отключены") ||
    normalized.includes("所有支持 cors 的搜索提供方都已禁用") ||
    normalized.includes("सभी cors समर्थित खोज प्रदाता अक्षम हैं")
  ) {
    return "all_providers_disabled";
  }
  if (
    normalized.includes("web search requested") ||
    normalized.includes("open research question detected") ||
    normalized.includes("search providers that can be queried") ||
    normalized.includes("verify claims against")
  ) {
    return "search_plan_only";
  }
  return "open_research";
}

function researchPromptPreview(value) {
  const compact = compactResearchLogValue(value);
  const chars = Array.from(compact);
  return chars.length <= 240 ? compact : `${chars.slice(0, 237).join("")}...`;
}

function renderResearchResultFollowup(priorSearch, status) {
  const preview = researchPromptPreview(priorSearch);
  if (status === "no_results") {
    return `The result of the previous research step is: no CORS-readable web search results were returned. I do not have verified source data to complete the requested analysis, calculation, table, or sources list yet.\n\nPrior research task: \`${preview}\`\n\nNext step: rerun the search with narrower queries or provide source links; then I can calculate the requested impact from those sources.`;
  }
  if (status === "all_providers_disabled") {
    return `The result of the previous research step is: web search could not run because the CORS-readable search providers were disabled. No verified research result was produced yet.\n\nPrior research task: \`${preview}\`\n\nNext step: enable a search provider or provide source links; then I can complete the requested analysis from those sources.`;
  }
  if (status === "search_plan_only") {
    return `The previous turn only set up the research/search step. It did not produce a final result, calculation, or sourced executive summary yet.\n\nPrior research task: \`${preview}\`\n\nNext step: run the source search and use the returned sources to finish the calculation.`;
  }
  return `There is no verified final research result in the conversation yet. The prior turn was a research request, but I do not see a completed source-backed answer to report.\n\nPrior research task: \`${preview}\`\n\nNext step: run the search or provide source links; then I can produce the requested result.`;
}

function tryResearchResultFollowup(prompt, normalized, history = []) {
  if (!isResearchResultFollowup(normalized)) return null;
  const priorSearch = lastHistoryTurn(history, "user");
  if (!priorSearch || !looksLikeResearchPrompt(priorSearch)) return null;
  const status = classifyPriorResearchAnswer(lastHistoryTurn(history, "assistant"));
  return {
    intent: "research_result_followup",
    content: renderResearchResultFollowup(priorSearch, status),
    confidence: 0.76,
    evidence: [
      `research_result_followup:prior_search:${compactResearchLogValue(priorSearch)}`,
      `research_result_followup:status:${status}`,
    ],
  };
}

// Issue #386: a conversation-summary request is recognised by composing
// meaning roles, not by matching raw words per language. The universal
// algorithm is identical for every language: the prompt either carries a
// complete standalone conversation-summary phrasing, an objectless courtesy
// frame ("can you summarize", "подведи итог"), a summary directive together
// with an explicit conversation reference, or it is itself a bare summary
// directive. The prompt is re-normalised first so the boundary-aware matcher
// sees punctuation collapsed to spaces (idempotent here, since `normalized`
// is already normalised). Mirror of asks_for_conversation_summary in
// src/solver_handlers/mod.rs.
function isSummarizePrompt(normalized) {
  const cleaned = normalizePrompt(normalized);
  return (
    lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_PHRASE, cleaned) ||
    lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_COURTESY, cleaned) ||
    (lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_DIRECTIVE, cleaned) &&
      lexiconMentionsRole(ROLE_CONVERSATION_REFERENCE, cleaned)) ||
    summaryDirectiveLeads(cleaned)
  );
}

// A bare summary directive standing alone is itself a request to summarize the
// running conversation ("summarize", "резюме", "总结", ...). For whitespace-
// delimited scripts the directive must be the whole prompt; for CJK (no word
// spaces) a leading directive suffices, mirroring the historical `^总结`
// anchor and keeping compounds like "工作总结" (a work summary) out. Mirror of
// summary_directive_leads in src/solver_handlers/mod.rs.
function summaryDirectiveLeads(cleaned) {
  return wordsForRole(ROLE_CONVERSATION_SUMMARY_DIRECTIVE).some((word) =>
    containsCjk(word) ? cleaned.startsWith(word) : cleaned === word,
  );
}

function trimUrlToken(token) {
  return String(token || "")
    .replace(/^[<>()\[\]{}"'`«»]+/u, "")
    .replace(/[<>()\[\]{}"'`«»]+$/u, "")
    .replace(/[.,!?;:…]+$/u, "");
}

function looksLikeHostname(value) {
  const host = String(value || "").trim();
  if (!host.includes(".") || host.startsWith(".") || host.endsWith(".")) {
    return false;
  }
  const labels = host.split(".");
  if (labels.some((label) => !label)) return false;
  const tld = labels[labels.length - 1] || "";
  if (tld.length < 2) return false;
  return labels.every(
    (label) =>
      /^[a-z0-9-]+$/i.test(label) &&
      !label.startsWith("-") &&
      !label.endsWith("-"),
  );
}

function normalizeUrlCandidate(candidate) {
  const text = String(candidate || "").trim();
  if (!text || /\s/.test(text) || text.includes("@")) return null;
  const lower = text.toLowerCase();
  let url = "";
  if (lower.startsWith("http://") || lower.startsWith("https://")) {
    url = text;
  } else {
    const hostCandidate = text.split(/[/?#]/, 1)[0] || "";
    if (lower.startsWith("www.") || looksLikeHostname(hostCandidate)) {
      url = `https://${text}`;
    }
  }
  if (!url) return null;
  let parsed;
  try {
    parsed = new URL(url);
  } catch (_error) {
    return null;
  }
  if (!looksLikeHostname(parsed.hostname)) return null;
  return parsed.href.replace(/\/$/, "");
}

function firstUrlCandidate(prompt) {
  const tokens = String(prompt || "").split(/\s+/);
  for (const token of tokens) {
    const trimmed = trimUrlToken(token);
    const url = normalizeUrlCandidate(trimmed);
    if (url) return { raw: trimmed, url };
  }
  return null;
}

// Issue #386: the web intents are recognised by *meaning*, not a hardcoded
// per-language phrase list. The surface words live once in
// data/seed/meanings-web-navigation.lino as the `http_fetch` and `url_navigate`
// meanings (loaded into MEANINGS_LINO); mirror the ROLE_HTTP_FETCH /
// ROLE_URL_NAVIGATE consts in src/seed/meanings.rs.
const ROLE_HTTP_FETCH = "http_fetch";
const ROLE_URL_NAVIGATE = "url_navigate";

function startsWithAny(haystack, prefixes) {
  return prefixes.some((prefix) => haystack.startsWith(prefix));
}

// Does a meaning carrying `role` evidence one of the prompt's lowercased forms?
// Buckets every surface form of the role by its derived slot exactly as
// role_evidences_web_intent does in src/solver_handlers/web_requests.rs:
//   - "prefix": form.before must *begin* one of the prompt forms ("fetch …" →
//     "fetch google.com"); form.before keeps the trailing space.
//   - "bare": form.text must appear *anywhere* in one of the prompt forms
//     ("запрос к" → "сделать запрос к google.com").
// `forms` are the lowercased prompt views matched against (the normalized
// prompt and the raw lowercased prompt). The result is a pure OR over every
// (form, surface) pair, so bucket order only affects short-circuiting — the
// surface words are never named here (issue #386).
function roleEvidencesWebIntent(role, forms) {
  const wordForms = roleWordForms(role);
  const matchesPrefix = wordForms
    .filter((form) => form.slot === "prefix")
    .some((form) => forms.some((text) => text.startsWith(form.before)));
  if (matchesPrefix) return true;
  return wordForms
    .filter((form) => form.slot === "bare")
    .some((form) => forms.some((text) => text.includes(form.text)));
}

function isHttpFetchPrompt(prompt, normalized) {
  const raw = String(prompt || "").trimStart().toLowerCase();
  return roleEvidencesWebIntent(ROLE_HTTP_FETCH, [normalized, raw]);
}

function isUrlNavigatePrompt(prompt, normalized, rawCandidate) {
  const raw = String(prompt || "").trimStart().toLowerCase();
  if (raw.startsWith(String(rawCandidate || "").toLowerCase())) {
    // Bare URL — treat as navigation, not a request to fetch.
    return true;
  }
  return roleEvidencesWebIntent(ROLE_URL_NAVIGATE, [normalized, raw]);
}

function extractHttpFetchUrl(prompt, normalized) {
  const candidate = firstUrlCandidate(prompt);
  if (!candidate || !isHttpFetchPrompt(prompt, normalized)) {
    return null;
  }
  return candidate.url;
}

function extractUrlNavigateUrl(prompt, normalized) {
  const candidate = firstUrlCandidate(prompt);
  if (!candidate || !isUrlNavigatePrompt(prompt, normalized, candidate.raw)) {
    return null;
  }
  return candidate.url;
}

function cleanSearchQuery(value) {
  return String(value || "")
    .trim()
    .replace(/^[<>()\[\]{}"'`«»]+/u, "")
    .replace(/[<>()\[\]{}"'`«»]+$/u, "")
    .replace(/[.,!?;:…]+$/u, "")
    .replace(/\s+/g, " ")
    .trim();
}

function stripSearchPrefix(prompt, prefix) {
  const text = String(prompt || "").trim();
  if (text.toLowerCase().startsWith(prefix)) {
    return validSearchQuery(text.slice(prefix.length));
  }
  return "";
}

function stripSearchSuffix(prompt, suffix) {
  const text = cleanSearchQuery(prompt);
  if (text.toLowerCase().endsWith(suffix)) {
    return validSearchQuery(text.slice(0, text.length - suffix.length));
  }
  return "";
}

function stripSearchCircumfix(prompt, prefix, suffix) {
  const text = cleanSearchQuery(prompt);
  const lower = text.toLowerCase();
  if (lower.startsWith(prefix) && lower.endsWith(suffix)) {
    return validSearchQuery(
      text.slice(prefix.length, text.length - suffix.length),
    );
  }
  return "";
}

// Issue #386: every surface cue the web-search recogniser reasons about — the
// explicit command prefixes, the action/source/signal vocabulary, the topic
// connectives, the query noise, the follow-up instruction verbs and clause
// boundaries, and the research/enumeration vocabulary — is sourced from the
// language-independent meaning lexicon (data/seed/meanings-web-search*.lino,
// meanings-web-research.lino, meanings-web-followup.lino, embedded in
// MEANINGS_LINO). The code references those meanings by their semantic
// *role* and by the *slot* each word form occupies (prefix / suffix / bare),
// never by raw words baked into a per-language list — adding a language or a
// synonym is a pure data edit. Mirrors WebSearchMarkers / markers() in
// src/solver_handlers/web_search_intent.rs.
const ROLE_WEB_SEARCH_EXPLICIT_PREFIX = "web_search_explicit_prefix";
const ROLE_WEB_SEARCH_ACTION = "web_search_action";
const ROLE_WEB_SEARCH_STRONG_ACTION = "web_search_strong_action";
const ROLE_WEB_SEARCH_SIGNAL = "web_search_signal";
const ROLE_WEB_SEARCH_TOPIC_MARKER = "web_search_topic_marker";
const ROLE_WEB_SEARCH_IMPERATIVE_LEAD = "web_search_imperative_lead";
const ROLE_WEB_SEARCH_QUERY_LEADING_NOISE = "web_search_query_leading_noise";
const ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE = "web_search_query_trailing_noise";
const ROLE_WEB_SEARCH_SOURCE_ONLY = "web_search_source_only";
const ROLE_WEB_SEARCH_NEWS_SUBJECT = "web_search_news_subject";
const ROLE_WEB_SEARCH_NEWS_RECENCY = "web_search_news_recency";
const ROLE_WEB_SEARCH_RECORDS_SUBJECT = "web_search_records_subject";
const ROLE_WEB_SEARCH_PUBLIC_EVENT_SUBJECT = "web_search_public_event_subject";
// Mention of web search inside a *prior* conversation turn (raw lowercased
// substring of the turn text, not the normalised prompt). Mirrors
// ROLE_WEB_SEARCH_HISTORY_SIGNAL in src/seed/roles.rs.
const ROLE_WEB_SEARCH_HISTORY_SIGNAL = "web_search_history_signal";
const ROLE_FOLLOWUP_INSTRUCTION_VERB = "followup_instruction_verb";
const ROLE_CLAUSE_CONTINUATION_MARKER = "clause_continuation_marker";
const ROLE_RESEARCH_QUESTION_OPENER = "research_question_opener";
const ROLE_TERM_INFORMATION_REQUEST_OPENER = "term_information_request_opener";
const ROLE_RESEARCH_SUPERLATIVE_MODIFIER = "research_superlative_modifier";
const ROLE_RESEARCH_EVIDENCE_DOMAIN = "research_evidence_domain";
const ROLE_RESEARCH_EVALUATION_DOMAIN = "research_evaluation_domain";
const ROLE_ENUMERATION_REQUEST_OPENER = "enumeration_request_opener";
const ROLE_ENUMERATION_CONSTRAINT = "enumeration_constraint";

// Issue #386 proof + who-is roles — mirror ROLE_PROOF_* / ROLE_WHO_QUESTION_*
// in src/seed/roles.rs. Surfaces live in data/seed/meanings-proof.lino and the
// who_is_question meaning in data/seed/meanings-intent.lino. The proof_directive
// bare verbs and proof_claim_scaffold prefixes share the `prove` meaning,
// separated by slot. (The worker's proof engine does not branch on the
// Goedel/determinism concepts, so those roles are referenced only by Rust.)
const ROLE_PROOF_DIRECTIVE = "proof_directive";
const ROLE_PROOF_REQUEST_LEAD = "proof_request_lead";
const ROLE_PROOF_MARKER = "proof_marker";
const ROLE_PROOF_CLAIM_SCAFFOLD = "proof_claim_scaffold";
const ROLE_WHO_QUESTION_LEAD = "who_question_lead";
const ROLE_WHO_QUESTION_TAIL = "who_question_tail";

// Issue #386 policy role — mirrors ROLE_CIRCULAR_JOKE_PHRASE in
// src/seed/roles.rs. Surfaces (the «купи слона» idiom and its buy-an-elephant
// calque in every supported language) live in data/seed/meanings-policy.lino,
// read here by tryKupiSlona as raw substrings. (The physical_action_trigger
// role in that file is read only by the Rust solver, which screens content
// policy first; the worker has no such handler, so it is not mirrored.)
const ROLE_CIRCULAR_JOKE_PHRASE = "circular_joke_phrase";

// Issue #386 calculator-rate roles — mirror ROLE_EXCHANGE_RATE_REFERENCE,
// ROLE_CURRENCY_USD_REFERENCE and ROLE_CALCULATION_BASIS_REFERENCE in
// src/seed/roles.rs. Surfaces (exchange-rate, US-dollar and calculation-basis
// forms in every supported language) live in
// data/seed/meanings-calculator.lino, read here by tryCalculatorRateBasis as
// raw substrings — the JS mirror of asks_for_usd_rate_basis.
const ROLE_EXCHANGE_RATE_REFERENCE = "exchange_rate_reference";
const ROLE_CURRENCY_USD_REFERENCE = "currency_usd_reference";
const ROLE_CALCULATION_BASIS_REFERENCE = "calculation_basis_reference";

// Issue #386 compound-interest / currency-conversion roles — mirror the
// ROLE_INVESTMENT_CUE, ROLE_INTEREST_CUE, ROLE_COMPOUNDING_ACTION_CUE,
// ROLE_COMPOUNDING_FREQUENCY_CUE, ROLE_LIVE_RATE_FRESHNESS_CUE,
// ROLE_YEAR_UNIT_CUE, ROLE_CONVERSION_ACTION_CUE, ROLE_FINAL_AMOUNT_REFERENCE,
// ROLE_CURRENCY_EUR_REFERENCE and ROLE_CURRENCY_RUB_REFERENCE consts in
// src/seed/roles.rs. Surfaces live in data/seed/meanings-finance.lino (the
// investment / interest / compounding / frequency / live-rate / year /
// conversion / final-amount forms) and data/seed/meanings-calculator.lino (the
// euro and ruble currency references), read by parseCompoundInterestRequest,
// parseCompoundsPerYear, parseCompoundYears, targetCurrencyFromText,
// asksForWebRate, parseFinalAmountConversionRequest and currencyCodeFromWord —
// the JS mirror of the compound_interest.rs recognizers.
const ROLE_INVESTMENT_CUE = "investment_cue";
const ROLE_INTEREST_CUE = "interest_cue";
const ROLE_COMPOUNDING_ACTION_CUE = "compounding_action_cue";
const ROLE_COMPOUNDING_FREQUENCY_CUE = "compounding_frequency_cue";
const ROLE_LIVE_RATE_FRESHNESS_CUE = "live_rate_freshness_cue";
const ROLE_YEAR_UNIT_CUE = "year_unit_cue";
const ROLE_CONVERSION_ACTION_CUE = "conversion_action_cue";
const ROLE_FINAL_AMOUNT_REFERENCE = "final_amount_reference";
const ROLE_CURRENCY_EUR_REFERENCE = "currency_eur_reference";
const ROLE_CURRENCY_RUB_REFERENCE = "currency_rub_reference";

// Issue #386 definition-merge roles — mirror the ROLE_DEFINITION_MERGE_ACTION,
// ROLE_DEFINITION_ARTIFACT_REQUEST, ROLE_DEFINITION_MERGE_MARKER and
// ROLE_DEFINITION_MERGE_TAIL_BOUNDARY consts in src/seed/roles.rs. Surfaces
// live in data/seed/meanings-definition-merge.lino, read by
// extractDefinitionMergeTerm and trimDefinitionMergeTail — the JS mirror of the
// definition_merge.rs recognizers.
const ROLE_DEFINITION_MERGE_ACTION = "definition_merge_action";
const ROLE_DEFINITION_ARTIFACT_REQUEST = "definition_artifact_request";
const ROLE_DEFINITION_MERGE_MARKER = "definition_merge_marker";
const ROLE_DEFINITION_MERGE_TAIL_BOUNDARY = "definition_merge_tail_boundary";

// Issue #386 meta-explanation roles — mirror ROLE_ASSISTANT_SELF_REFERENCE and
// ROLE_ARCHITECTURE_CONCEPT in src/seed/roles.rs. Surfaces (the assistant's
// self-reference pronouns and the architecture concepts — language model,
// neural network, the project's local rules — in every supported language) live
// in data/seed/meanings-meta.lino, read here by isArchitectureQuestion as raw
// substrings, the JS mirror of is_architecture_question. (The why/how-you-work
// recognizers live only in the Rust solver_handler; the worker has no
// tryMetaExplanation, so the answer_rationale_lead, causal_interrogative,
// prior_answer_reference, assistant_mechanism_inquiry and operating_principle
// roles in that file are not mirrored here.)
const ROLE_ASSISTANT_SELF_REFERENCE = "assistant_self_reference";
const ROLE_ARCHITECTURE_CONCEPT = "architecture_concept";

// Issue #386 documentation-handler roles — mirror ROLE_EXPLANATION_REQUEST_LEAD,
// ROLE_WEB_MEDIUM and ROLE_CODE_METHOD_NOUN in src/seed/roles.rs. Their surfaces
// live in data/seed/meanings-docs.lino (explanation leads, the method noun) and
// data/seed/meanings-web-search.lino (the web_medium surfaces on
// reference_internet). isExplanationRequest, isExplicitWebSearchPrompt and the
// join+method branch of isPandasDataFrameJoinPrompt read them instead of naming
// any interrogative, medium or per-language method word in code here.
const ROLE_EXPLANATION_REQUEST_LEAD = "explanation_request_lead";
const ROLE_WEB_MEDIUM = "web_medium";
const ROLE_CODE_METHOD_NOUN = "code_method_noun";

// The literal lead-in (form.before, the text before the … slot) of every
// prefix-slot form of a role, in lexicon declaration order. A meaning's roles
// apply to all its forms, so keep only the slot we asked for. Mirrors
// prefix_literals.
function prefixLiterals(role) {
  return roleWordForms(role)
    .filter((form) => form.slot === "prefix")
    .map((form) => form.before);
}
// The literal tail (form.after) of every suffix-slot form of a role. Mirrors
// suffix_literals.
function suffixLiterals(role) {
  return roleWordForms(role)
    .filter((form) => form.slot === "suffix")
    .map((form) => form.after);
}
// The literal pair around every circumfix-slot form of a role. Mirrors
// circumfix_literals.
function circumfixLiterals(role) {
  return roleWordForms(role)
    .filter((form) => form.slot === "circumfix")
    .map((form) => ({ before: form.before, after: form.after }));
}
// The surface text of every bare-slot form of a role (drop any prefix/suffix
// surfaces the same meaning also owns). Mirrors bare_literals.
function bareLiterals(role) {
  return roleWordForms(role)
    .filter((form) => form.slot === "bare")
    .map((form) => form.text);
}
// The distinct surface words of a role, trimmed + lowercased for equality
// comparison against a cleaned query. Mirrors source_literals (words_for_role).
function sourceLiterals(role) {
  const seen = new Set();
  const out = [];
  for (const form of roleWordForms(role)) {
    const key = form.text.trim().toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      out.push(key);
    }
  }
  return out;
}

// Build (once) the marker projection from the meaning lexicon, then cache it —
// roleWordForms walks the whole lexicon, so memoize like the Rust OnceLock.
let WEB_SEARCH_MARKERS_CACHE = null;
function webSearchMarkers() {
  if (WEB_SEARCH_MARKERS_CACHE) return WEB_SEARCH_MARKERS_CACHE;
  WEB_SEARCH_MARKERS_CACHE = {
    explicitPrefixes: prefixLiterals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
    explicitSuffixes: suffixLiterals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
    explicitCircumfixes: circumfixLiterals(ROLE_WEB_SEARCH_EXPLICIT_PREFIX),
    actionMarkers: bareLiterals(ROLE_WEB_SEARCH_ACTION),
    strongActionMarkers: bareLiterals(ROLE_WEB_SEARCH_STRONG_ACTION),
    signalMarkers: bareLiterals(ROLE_WEB_SEARCH_SIGNAL),
    topicAfterMarkers: prefixLiterals(ROLE_WEB_SEARCH_TOPIC_MARKER),
    topicBeforeMarkers: suffixLiterals(ROLE_WEB_SEARCH_TOPIC_MARKER),
    imperativeLeadMarkers: prefixLiterals(ROLE_WEB_SEARCH_IMPERATIVE_LEAD),
    leadingNoise: prefixLiterals(ROLE_WEB_SEARCH_QUERY_LEADING_NOISE),
    trailingNoise: suffixLiterals(ROLE_WEB_SEARCH_QUERY_TRAILING_NOISE),
    sourceOnly: sourceLiterals(ROLE_WEB_SEARCH_SOURCE_ONLY),
    newsSubjectMarkers: bareLiterals(ROLE_WEB_SEARCH_NEWS_SUBJECT),
    newsRecencyMarkers: bareLiterals(ROLE_WEB_SEARCH_NEWS_RECENCY),
    recordsSubjectMarkers: bareLiterals(ROLE_WEB_SEARCH_RECORDS_SUBJECT),
    publicEventSubjectMarkers: bareLiterals(ROLE_WEB_SEARCH_PUBLIC_EVENT_SUBJECT),
    followupVerbs: bareLiterals(ROLE_FOLLOWUP_INSTRUCTION_VERB),
    continuationMarkers: bareLiterals(ROLE_CLAUSE_CONTINUATION_MARKER),
    termInformationPrefixes: prefixLiterals(ROLE_TERM_INFORMATION_REQUEST_OPENER),
    researchQuestionPrefixes: prefixLiterals(ROLE_RESEARCH_QUESTION_OPENER),
    researchModifiers: bareLiterals(ROLE_RESEARCH_SUPERLATIVE_MODIFIER),
    researchEvidenceDomains: bareLiterals(ROLE_RESEARCH_EVIDENCE_DOMAIN),
    researchEvaluationDomains: bareLiterals(ROLE_RESEARCH_EVALUATION_DOMAIN),
    enumerationPrefixes: prefixLiterals(ROLE_ENUMERATION_REQUEST_OPENER),
    enumerationConstraintMarkers: bareLiterals(ROLE_ENUMERATION_CONSTRAINT),
  };
  return WEB_SEARCH_MARKERS_CACHE;
}

// A request to filter the user's OWN contributed facts ("facts I contributed",
// "my facts") is conversation search, not a web search. Mirrors
// is_personal_fact_filter_request in src/solver_handlers/web_search_intent.rs.
function isPersonalFactFilterRequest(normalized) {
  const text = String(normalized || "");
  return (
    text.includes("facts i have contributed") ||
    text.includes("facts ive contributed") ||
    text.includes("facts i contributed") ||
    text.includes("my facts")
  );
}

function containsSearchMarker(normalized, marker) {
  const text = String(normalized || "");
  if (marker.startsWith(" ") || marker.endsWith(" ")) {
    return ` ${text} `.includes(marker);
  }
  return text.includes(marker);
}

function containsAnySearchMarker(normalized, markers) {
  return markers.some((marker) => containsSearchMarker(normalized, marker));
}

function stripSearchNoisePrefix(value, prefix) {
  const text = cleanSearchQuery(value);
  return text.toLowerCase().startsWith(prefix)
    ? cleanSearchQuery(text.slice(prefix.length))
    : text;
}

function stripSearchNoiseSuffix(value, suffix) {
  const text = cleanSearchQuery(value);
  return text.toLowerCase().endsWith(suffix)
    ? cleanSearchQuery(text.slice(0, text.length - suffix.length))
    : text;
}

// Sentence-ending punctuation that can introduce a follow-up instruction
// clause — ASCII plus the fullwidth/ideographic forms a CJK prompt uses.
// Mirrors is_sentence_boundary.
const SEARCH_SENTENCE_BOUNDARY = new Set([
  ".",
  "?",
  "!",
  ";",
  ":",
  "\u3002",
  "\uff1f",
  "\uff01",
  "\uff1b",
  "\uff1a",
]);

// ASCII-only lowercase: folds A–Z and nothing else, so the result keeps the same
// length (in UTF-16 code units) as the input and computed offsets stay aligned.
// Mirrors Rust str::to_ascii_lowercase (a full toLowerCase could change length,
// e.g. 'İ' -> 'i̇', and misalign the cut offsets).
function asciiLowercase(value) {
  return String(value || "").replace(/[A-Z]/g, (character) =>
    String.fromCharCode(character.charCodeAt(0) + 32),
  );
}

// Is the single Unicode code point `code` a letter or number? Mirrors Rust
// char::is_alphanumeric closely enough for token-boundary detection.
function isSearchAlnum(code) {
  return /[\p{L}\p{N}]/u.test(String.fromCodePoint(code));
}

// Does `index` begin a token in `text` (the preceding code point is non-
// alphanumeric, or there is none)? Surrogate-pair aware. Mirrors is_token_start.
function isSearchTokenStart(text, index) {
  if (index <= 0) return true;
  const i = index - 1;
  let code = text.charCodeAt(i);
  if (code >= 0xdc00 && code <= 0xdfff && i > 0) {
    const high = text.charCodeAt(i - 1);
    if (high >= 0xd800 && high <= 0xdbff) {
      code = (high - 0xd800) * 0x400 + (code - 0xdc00) + 0x10000;
    }
  }
  return !isSearchAlnum(code);
}

// Does `index` end a token in `text` (the following code point is non-
// alphanumeric, or there is none)? Mirrors is_token_end.
function isSearchTokenEnd(text, index) {
  if (index >= text.length) return true;
  return !isSearchAlnum(text.codePointAt(index));
}

// Whether `haystack` ends with `marker` as a whole token. CJK markers match as
// bare substrings; space-delimited markers need a preceding whitespace (or for
// the whole string to be exactly the marker). Mirrors ends_with_token.
function searchEndsWithToken(haystack, marker) {
  if (containsCjk(marker)) return haystack.endsWith(marker);
  if (haystack === marker) return true;
  if (!haystack.endsWith(marker)) return false;
  const head = haystack.slice(0, haystack.length - marker.length);
  return /\s$/u.test(head);
}

// If the text immediately before `verbStart` is a follow-up boundary, return the
// code-unit offset at which to cut (the start of the boundary run); otherwise
// null. Mirrors boundary_before.
function searchBoundaryBefore(text, verbStart, markers) {
  const head = text.slice(0, verbStart).trimEnd();
  if (head.length === 0) return null;
  if (SEARCH_SENTENCE_BOUNDARY.has(head[head.length - 1])) return head.length;
  // Walk back over a run of clause-continuation markers ("and", "then",
  // "and then"); the cut falls at the start of the run.
  let cursor = head;
  let matched = false;
  for (;;) {
    const trimmed = cursor.trimEnd();
    let rest = null;
    for (const marker of markers.continuationMarkers) {
      if (searchEndsWithToken(trimmed, marker)) {
        rest = trimmed.slice(0, trimmed.length - marker.length);
        break;
      }
    }
    if (rest === null) break;
    cursor = rest;
    matched = true;
  }
  return matched ? cursor.trimEnd().length : null;
}

// Drop a trailing follow-up instruction clause ("… and summarize who won",
// "… . Compare their patents") from a query. A universal boundary algorithm, not
// a list of memorised fragments: a follow-up clause is one of the lexicon's
// followup_instruction_verb surfaces sitting immediately after a *boundary* —
// sentence punctuation or a run of clause_continuation_marker words — and the
// query is cut at the start of the earliest such boundary. A bare verb with no
// boundary before it is part of the topic and left untouched. Mirrors
// truncate_search_instruction_tail in src/solver_handlers/web_search_intent.rs.
