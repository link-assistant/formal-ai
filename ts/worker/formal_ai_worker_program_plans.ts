// Substitution rules that lower program plans, write-program parameters and
// recovery, coding guidance, program explanations and the blueprint capability table.
// Loaded by ../formal_ai_worker.js.
function applySubstitutionRule(linkSet, rule, event, sequence) {
  if (!rule.events.includes(event)) return null;
  const required = rule.conditions.slice();
  for (const action of rule.actions) required.push(action.remove);
  const links = sortedLinksFromSet(linkSet);
  const bindings = findBindings(links, required, 0, {});
  if (!bindings) return null;
  // Pre-instantiate every mutation so a partial rewrite never mutates the set.
  const ops = [];
  for (const action of rule.actions) {
    const remove = instantiatePattern(action.remove, bindings);
    if (remove === null) return null;
    const adds = [];
    for (const addPattern of action.add) {
      const add = instantiatePattern(addPattern, bindings);
      if (add === null) return null;
      adds.push(add);
    }
    ops.push({ remove, adds });
  }
  const before = new Set(linkSet);
  const removed = [];
  const added = [];
  for (const op of ops) {
    const removeKey = linkKey(op.remove);
    if (linkSet.has(removeKey)) {
      linkSet.delete(removeKey);
      removed.push(op.remove);
    }
    for (const add of op.adds) {
      const addKey = linkKey(add);
      if (!linkSet.has(addKey)) {
        linkSet.add(addKey);
        added.push(add);
      }
    }
  }
  if (linkSet.size === before.size && [...linkSet].every((key) => before.has(key))) {
    return null;
  }
  return { sequence, ruleId: rule.id, event, bindings, removed, added };
}

function applyFirstSubstitutionRule(linkSet, ruleSet, event, sequence) {
  for (const rule of ruleSet.rules) {
    if (!rule.events.includes(event)) continue;
    const trace = applySubstitutionRule(linkSet, rule, event, sequence);
    if (trace) return trace;
  }
  return null;
}

const DEFAULT_MAX_SUBSTITUTIONS = 64;

function applySubstitutionRules(initialLinks, ruleSet, event, maxApplications) {
  const limit = maxApplications || DEFAULT_MAX_SUBSTITUTIONS;
  const linkSet = new Set(initialLinks.map(linkKey));
  const traces = [];
  let terminatedByGuard = false;
  while (traces.length < limit) {
    const trace = applyFirstSubstitutionRule(linkSet, ruleSet, event, traces.length);
    if (!trace) {
      return { links: sortedLinksFromSet(linkSet), traces, terminatedByGuard };
    }
    traces.push(trace);
  }
  const probe = new Set(linkSet);
  terminatedByGuard =
    applyFirstSubstitutionRule(probe, ruleSet, event, traces.length) !== null;
  return { links: sortedLinksFromSet(linkSet), traces, terminatedByGuard };
}

// --- Program-plan pipeline (mirror of src/program_plan.rs) ------------------

// Issue #386: every declared (cancelOp, baseOp) inverse relationship, taken
// from the `inverse` field on a modifier operation. Mirrors
// `OperationVocabulary::inverse_pairs` in `src/seed/operation_vocabulary.rs`.
function inversePairsFromOperations() {
  const pairs = [];
  for (const operation of operationVocabulary()) {
    if (operation.inverse) pairs.push([operation.slug, operation.inverse]);
  }
  return pairs;
}

function cloneLinkPattern(pattern) {
  return {
    from: Object.assign({}, pattern.from),
    to: Object.assign({}, pattern.to),
  };
}

// Issue #386: derive subtractive ("cancel") rules from the additive base rules
// plus the declared (cancelOp, baseOp) inverse pairs — the JS mirror of
// `derive_inverse_rules` in `src/program_plan.rs`. For every base rule that
// fires on `request:modifier -> baseOp` with a single-link task rewrite, emit
// its inverse: fire on `request:modifier -> cancelOp` and swap the rewrite's
// removed and added task links. "Cancel the sort" becomes the exact,
// automatically-maintained inverse of "sort" — pure data, no new control flow.
function deriveInverseRules(baseRules, inversePairs) {
  const derived = [];
  for (const [cancelOp, baseOp] of inversePairs) {
    for (const rule of baseRules) {
      const conditionIndex = rule.conditions.findIndex(
        (condition) =>
          literalPatternValue(condition.from) === MODIFIER_NODE &&
          literalPatternValue(condition.to) === baseOp,
      );
      if (conditionIndex === -1) continue;
      // A well-defined inverse exists only for a single-link additive rewrite.
      if (rule.actions.length !== 1) continue;
      const action = rule.actions[0];
      if (!Array.isArray(action.add) || action.add.length !== 1) continue;
      const added = action.add[0];
      const conditions = rule.conditions.map((condition, index) =>
        index === conditionIndex
          ? parseLinkPattern(`${MODIFIER_NODE} -> ${cancelOp}`)
          : cloneLinkPattern(condition),
      );
      derived.push({
        id: `${cancelOp}__${rule.id}`,
        order: rule.order,
        events: rule.events.slice(),
        conditions,
        actions: [{ remove: cloneLinkPattern(added), add: [cloneLinkPattern(action.remove)] }],
      });
    }
  }
  return derived;
}

let cachedProgramPlanRules = null;
function programPlanRules() {
  if (!cachedProgramPlanRules) {
    const set = parseSubstitutionRules(PROGRAM_PLAN_RULES_LINO);
    const derived = deriveInverseRules(set.rules, inversePairsFromOperations());
    set.rules = set.rules.concat(derived);
    set.rules.sort((left, right) =>
      left.order - right.order ||
      (left.id < right.id ? -1 : left.id > right.id ? 1 : 0),
    );
    cachedProgramPlanRules = set;
  }
  return cachedProgramPlanRules;
}

function lowerProgramPlanWithRules(ruleSet, baseTask, modifiers) {
  const initial = [{ from: TASK_NODE, to: baseTask }];
  for (const modifier of modifiers) initial.push({ from: MODIFIER_NODE, to: modifier });
  const { links, traces, terminatedByGuard } = applySubstitutionRules(
    initial,
    ruleSet,
    "manual",
  );
  const resolvedLink = links.find((link) => link.from === TASK_NODE);
  const resolvedTask = resolvedLink ? resolvedLink.to : baseTask;
  return {
    baseTask,
    modifiers: modifiers.slice(),
    resolvedTask,
    links,
    traces,
    terminatedByGuard,
  };
}

function lowerProgramPlan(baseTask, modifiers) {
  return lowerProgramPlanWithRules(programPlanRules(), baseTask, modifiers);
}

function resolveProgramTask(baseTask, modifiers) {
  return lowerProgramPlan(baseTask, modifiers).resolvedTask;
}

function programPlanWasModified(plan) {
  return plan.resolvedTask !== plan.baseTask;
}

// Render the plan graph and its substitution trace as Links Notation so the
// worker can surface the reasoning transparently (issue #324 R6), mirroring
// `ProgramPlan::links_notation` in `src/program_plan.rs`.
function programPlanLinksNotation(plan) {
  const lines = ["program_plan"];
  lines.push(`  base_task ${plan.baseTask}`);
  lines.push(`  resolved_task ${plan.resolvedTask}`);
  for (const modifier of plan.modifiers) lines.push(`  modifier ${modifier}`);
  lines.push("  substitution_graph");
  for (const link of plan.links) lines.push(`    link ${link.from} -> ${link.to}`);
  lines.push("  substitution_trace_report");
  lines.push("    event manual");
  lines.push(`    terminated_by_guard ${plan.terminatedByGuard ? "true" : "false"}`);
  for (const trace of plan.traces) {
    lines.push(`    trace ${trace.ruleId}`);
    lines.push(`      sequence ${trace.sequence}`);
    lines.push(`      rule_id ${trace.ruleId}`);
    for (const name of Object.keys(trace.bindings).sort()) {
      lines.push(`      binding ${name}=${trace.bindings[name]}`);
    }
    for (const link of trace.removed) lines.push(`      removed ${link.from} -> ${link.to}`);
    for (const link of trace.added) lines.push(`      added ${link.from} -> ${link.to}`);
  }
  return lines.join("\n");
}

function writeProgramParameters(prompt) {
  const normalized = normalizeProgramPrompt(prompt);
  let task = programTaskFromPrompt(normalized);
  const language = programLanguageFromPrompt(normalized);
  // Issue #386: recognise "write a <program>" by *meaning*, not a hardcoded
  // per-language word list — a program_kind artifact (program / script / code /
  // function / class) requested by a program_request verb (write / create / … / build).
  // The surface words live once in data/seed/meanings.lino; this code knows the
  // concepts. Mirrors write_program_parameters in src/intent_formalization.rs.
  const mentionsProgramRequest = lexiconMentionsRole(ROLE_PROGRAM_REQUEST, normalized);
  const asksForProgram = lexiconMentionsRole(ROLE_PROGRAM_KIND, normalized) && mentionsProgramRequest;
  const asksForKnownLanguageProgram =
    Boolean(language) &&
    mentionsProgramRequest &&
    (Boolean(WRITE_PROGRAM_LANGUAGES[language]) || codingOracleKnowsLanguage(language));
  // Issue #1021 (R1021-31): "мне нужен код" names code and nothing else, so it is a
  // request with no parameters whatever the asking verb (namesCodeAndNothingElse).
  const asksForBareCode = !task && !language && namesCodeAndNothingElse(normalized);
  // Catalog aliases bind coding tasks. Without a program artefact or language,
  // a container-scoped enumeration asks for a workspace action.
  const enumeratesContainer = ["capability_container_scope", "capability_act_enumerate"]
    .every((role) => lexiconMentionsRole(role, normalized) || lexiconMentionsRoleSubstring(role, normalized));
  if (!language && !lexiconMentionsRole(ROLE_PROGRAM_KIND, normalized) && enumeratesContainer) return null;
  if (!task && !asksForProgram && !asksForKnownLanguageProgram && !asksForBareCode) return null;
  // Issue #358: modification phrases in the same turn lower the base task
  // through the data-backed substitution pipeline.
  if (task) {
    const modifiers = detectedProgramModifiers(normalized);
    task = resolveProgramTask(task, modifiers);
  }
  return { language, task };
}

function looksLikeBareProgramArtifactFollowUp(normalized) {
  // Issue #386: a bare follow-up modifies an existing program artifact when the
  // prompt evidences a program_artifact meaning *and* a program_modification
  // meaning. The surface words live once in the seed; this code knows concepts.
  return (
    lexiconMentionsRole(ROLE_PROGRAM_ARTIFACT, normalized) &&
    lexiconMentionsRole(ROLE_PROGRAM_MODIFICATION, normalized)
  );
}

function activeProgramContext(history) {
  let task = null;
  let language = null;
  if (!Array.isArray(history)) return null;
  for (let index = history.length - 1; index >= 0; index -= 1) {
    const turn = history[index];
    const content = turn && (turn.content || turn.text || turn.message);
    if (!content) continue;
    const prior = writeProgramParameters(content);
    if (!prior) continue;
    if (!task && prior.task) task = prior.task;
    if (!language && prior.language) language = prior.language;
    if (task && language) return { task, language };
  }
  return null;
}

function rewriteBareProgramCoreference(prompt, history) {
  const normalized = normalizeProgramPrompt(prompt);
  if (!looksLikeBareProgramArtifactFollowUp(normalized)) return null;
  const context = activeProgramContext(history);
  if (!context) return null;
  return {
    parameters: { task: context.task, language: context.language },
    trace: `referent=active_program_artifact task=${context.task} language=${context.language}`,
  };
}

// Issue #324: a follow-up such as "Сделай так, чтобы программа принимала путь
// как аргумент" routes to write_program but names neither a task nor a
// language - both came from a previous turn. Recover the missing parameters
// from the most recent prior turn that named them and apply any data-defined
// modifier present in the follow-up. Mirrors `recover_write_program_rule` in
// `src/intent_formalization.rs`.
function recoverWriteProgramParameters(parameters, prompt, history) {
  let task = parameters.task || null;
  let language = parameters.language || null;
  if ((!task || !language) && Array.isArray(history)) {
    for (let index = history.length - 1; index >= 0; index -= 1) {
      const turn = history[index];
      const content = turn && (turn.content || turn.text || turn.message);
      if (!content) continue;
      const prior = writeProgramParameters(content);
      if (!prior) continue;
      if (!task && prior.task) task = prior.task;
      if (!language && prior.language) language = prior.language;
      if (task && language) break;
    }
  }
  const normalized = normalizeProgramPrompt(prompt);
  // Issue #324 R4/R6: lower the recovered task through the substitution
  // pipeline when the follow-up carries a modifier, and surface the resulting
  // plan as Links Notation (mirrors `recover_write_program_rule` in
  // `src/intent_formalization.rs`, which sets `WriteProgramRecovery::plan`).
  let plan = null;
  let modifiers = [];
  let lowered = null;
  if (task) {
    modifiers = detectedProgramModifiers(normalized);
    if (modifiers.length) {
      lowered = lowerProgramPlan(task, modifiers);
      if (programPlanWasModified(lowered)) plan = programPlanLinksNotation(lowered);
      task = lowered.resolvedTask;
    }
  }
  return { task, language, plan, modifiers, lowered };
}

// Issue #324: a request in a given language must be answered in that language.
// These mirror the localized framing produced by the Rust engine
// (`write_program_intro`, `unsupported_write_program_answer`,
// `execution_report`). Only the natural-language prose is localized; the code
// and the Links Notation trace stay canonical. `en` is the fallback.
const WRITE_PROGRAM_I18N = {
  en: {
    intro: (name, label) => `Here is a minimal ${name} ${label} program:`,
    ranInSandbox: "Execution status: ran in the demo's Web Worker sandbox.",
    outputLabel: "Output:",
    noOutput: "(no output)",
    sandboxFailed: (message) => `Execution status: failed in sandbox - ${message}.`,
    notRun: (language, reason) =>
      `Execution status: not run - ${reason}.`,
    copyInstruction: (language) =>
      `Copy the snippet into a ${language} environment to verify.`,
    noFilesystem: (language) =>
      `the browser sandbox has no filesystem access for this ${language} program`,
    noToolchain: (language) => `the browser sandbox cannot invoke a ${language} toolchain`,
    sampleDirectory: (files) =>
      `The sample output below is for a clean directory containing exactly ${markdownFileList(
        files,
        "and",
      )} and no extra files:`,
    expectedOutput: "Expected output after verification:",
  },
  ru: {
    intro: (name, label) => `Вот минимальная программа на языке ${name} (${label}):`,
    ranInSandbox: "Статус выполнения: запущено в песочнице Web Worker демо.",
    outputLabel: "Вывод:",
    noOutput: "(нет вывода)",
    sandboxFailed: (message) => `Статус выполнения: сбой в песочнице - ${message}.`,
    notRun: (language, reason) =>
      `Статус выполнения: не запущено - ${reason}.`,
    copyInstruction: (language) =>
      `Скопируйте фрагмент в среду ${language}, чтобы проверить.`,
    noFilesystem: (language) =>
      `у браузерной песочницы нет доступа к файловой системе для этой программы на ${language}`,
    noToolchain: (language) =>
      `браузерная песочница не может вызвать инструментарий ${language}`,
    sampleDirectory: (files) =>
      `Ниже показан вывод для чистого каталога, содержащего ровно ${markdownFileList(
        files,
        "и",
      )}, и никаких других файлов:`,
    expectedOutput: "Ожидаемый вывод после проверки:",
  },
  hi: {
    intro: (name, label) => `यहाँ ${name} में एक न्यूनतम प्रोग्राम है (${label}):`,
    ranInSandbox: "निष्पादन स्थिति: डेमो के Web Worker सैंडबॉक्स में चला।",
    outputLabel: "आउटपुट:",
    noOutput: "(कोई आउटपुट नहीं)",
    sandboxFailed: (message) => `निष्पादन स्थिति: सैंडबॉक्स में विफल - ${message}.`,
    notRun: (language, reason) =>
      `निष्पादन स्थिति: नहीं चला - ${reason}.`,
    copyInstruction: (language) =>
      `सत्यापित करने के लिए स्निपेट को ${language} वातावरण में कॉपी करें।`,
    noFilesystem: (language) =>
      `इस ${language} प्रोग्राम के लिए ब्राउज़र सैंडबॉक्स में फ़ाइल सिस्टम तक पहुँच नहीं है`,
    noToolchain: (language) =>
      `ब्राउज़र सैंडबॉक्स ${language} टूलचेन को आमंत्रित नहीं कर सकता`,
    sampleDirectory: (files) =>
      `नीचे दिया गया नमूना आउटपुट ऐसी साफ डायरेक्टरी के लिए है जिसमें ठीक ${markdownFileList(
        files,
        "और",
      )} हों और कोई अतिरिक्त फाइल न हो:`,
    expectedOutput: "सत्यापन के बाद अपेक्षित आउटपुट:",
  },
  zh: {
    intro: (name, label) => `这是一个最小的 ${name} 程序（${label}）：`,
    ranInSandbox: "执行状态：已在演示的 Web Worker 沙箱中运行。",
    outputLabel: "输出：",
    noOutput: "（无输出）",
    sandboxFailed: (message) => `执行状态：沙箱中失败 - ${message}。`,
    notRun: (language, reason) =>
      `执行状态：未运行 - ${reason}。`,
    copyInstruction: (language) => `将代码片段复制到 ${language} 环境中以验证。`,
    noFilesystem: (language) => `浏览器沙箱无法为此 ${language} 程序访问文件系统`,
    noToolchain: (language) => `浏览器沙箱无法调用 ${language} 工具链`,
    sampleDirectory: (files) =>
      `下面的示例输出适用于一个只包含 ${markdownFileList(files, "和")} 且没有其他文件的干净目录：`,
    expectedOutput: "验证后的预期输出：",
  },
};

// Issue #699 batch 3: the dead end that recited the template catalogue back at
// the user is gone from both engines. When every synthesis route misses, the worker names
// the gap from seed data, exactly like `src/program_skill_gap.rs`. The route
// slugs are language-neutral, so adding a route updates every language at once.
const PROGRAM_SYNTHESIS_ROUTES = [
  "catalog",
  "blueprint_recipes",
  "coding_oracle",
  "seed_idiom_composer",
];
const MISSING_PROGRAM_PARAMETER = "missing";

// Issue #906: `missing` is an internal sentinel and must never reach the
// requester — substituting it into the localized wording produced replies that
// named a gap "in language `missing`". Which of the four dead ends a request
// reached decides the wording, the intent, the event and the response link.
// Mirrors `Shape` in src/program_skill_gap.rs.
const PROGRAM_DEAD_ENDS = {
  skill_gap: {
    name: "write_program_skill_gap_name",
    answer: "write_program_skill_gap",
    intent: "write_program_skill_gap",
    event: "skill_gap",
    link: "response:write_program:skill_gap",
  },
  task_unspecified: {
    name: "write_program_skill_gap_name_task_unspecified",
    answer: "write_program_skill_gap",
    intent: "write_program_skill_gap",
    event: "skill_gap",
    link: "response:write_program:skill_gap",
  },
  language_unspecified: {
    name: "write_program_language_unspecified_name",
    answer: "write_program_language_unspecified",
    intent: "write_program_language_unspecified",
    event: "unspecified_parameter",
    link: "response:write_program:language_unspecified",
  },
  request_unspecified: {
    name: "write_program_request_unspecified_name",
    answer: "write_program_request_unspecified",
    intent: "write_program_request_unspecified",
    event: "unspecified_parameter",
    link: "response:write_program:request_unspecified",
  },
};

// Classify a (task, language) pair — mirrors `program_skill_gap::shape`.
function programDeadEnd(task, language) {
  if (task) return language ? PROGRAM_DEAD_ENDS.skill_gap : PROGRAM_DEAD_ENDS.language_unspecified;
  return language ? PROGRAM_DEAD_ENDS.task_unspecified : PROGRAM_DEAD_ENDS.request_unspecified;
}

// The English name is the gap's identity (it travels in the evidence trail);
// the localized name is what the reader sees inside the reply.
function programSkillGapName(task, language, responseLanguage) {
  const shape = programDeadEnd(task, language);
  const template = answerFor(shape.name, responseLanguage) || answerFor(shape.name, "en") || "";
  return template.replace("{task}", task || MISSING_PROGRAM_PARAMETER)
    .replace("{language}", language || MISSING_PROGRAM_PARAMETER);
}
function programSkillGapAnswer(task, language, responseLanguage) {
  const shape = programDeadEnd(task, language);
  const template = answerFor(shape.answer, responseLanguage) || answerFor(shape.answer, "en") || "";
  return template
    .replace("{gap}", programSkillGapName(task, language, responseLanguage))
    .replace("{routes}", PROGRAM_SYNTHESIS_ROUTES.join(", "))
    .replace("{task}", task || MISSING_PROGRAM_PARAMETER)
    .replace("{language}", language || MISSING_PROGRAM_PARAMETER);
}

function writeProgramStrings(language) {
  return WRITE_PROGRAM_I18N[language] || WRITE_PROGRAM_I18N.en;
}

function markdownFileList(files, conjunction) {
  const quoted = files.map((file) => `\`${file}\``);
  if (quoted.length <= 1) return quoted.join("");
  return `${quoted.slice(0, -1).join(", ")} ${conjunction} ${quoted[quoted.length - 1]}`;
}

function listFilesTaskDirection(task) {
  switch (task) {
    case "list_files":
    case "list_files_arg":
      return "ascending";
    case "list_files_reverse_sort":
    case "list_files_arg_reverse_sort":
      return "descending";
    default:
      return "";
  }
}

function listFilesSampleFiles(languageInfo) {
  return ["README.md", "data.txt", languageInfo.saveAs].sort();
}

function writeProgramExpectedOutput(task, languageInfo, taskInfo) {
  const direction = listFilesTaskDirection(task);
  if (!direction) return taskInfo.output;
  const files = listFilesSampleFiles(languageInfo);
  if (direction === "descending") files.reverse();
  return files.join("\n");
}

function writeProgramExecutionLines(language, task, code, output, strings, responseLanguage) {
  const i18n = strings || WRITE_PROGRAM_I18N.en;
  // Issue #312: the list-files snippet reads the real filesystem through Node's
  // `fs`/`require`, which the browser Web Worker sandbox does not provide, and
  // its output depends on the directory contents. Never claim it "ran" here -
  // detect the Node API use and report the documented sample-directory output
  // instead, so the demo stays honest.
  const needsNodeApis = /\brequire\s*\(|\bimport\b/.test(code);
  if (language === "javascript" && !needsNodeApis) {
    const logs = [];
    try {
      const runner = new Function("console", `"use strict"; ${code}`);
      runner({ log: (...args) => logs.push(args.join(" ")) });
      return [
        i18n.ranInSandbox,
        i18n.outputLabel,
        "```text",
        logs.join("\n") || i18n.noOutput,
        "```",
      ];
    } catch (error) {
      return [i18n.sandboxFailed(error.message || String(error))];
    }
  }
  const runtimeProbe = typeof browserExecutionProbe === "function"
    ? browserExecutionProbe(language)
    : { status: "unavailable" };
  const reason = language === "javascript"
    ? i18n.noFilesystem(language)
    : runtimeProbe.status === "available_to_download" || runtimeProbe.status === "ready"
      ? browserRuntimeMessage(responseLanguage || "en")
      : i18n.noToolchain(language);
  const lines = [i18n.notRun(language, reason), "", i18n.copyInstruction(language), ""];
  if (listFilesTaskDirection(task)) {
    lines.push(i18n.sampleDirectory(listFilesSampleFiles(WRITE_PROGRAM_LANGUAGES[language])));
  } else {
    lines.push(i18n.expectedOutput);
  }
  lines.push("```text", output, "```");
  return lines;
}

function inlineHelloWorldReplacement(prompt) {
  const normalized = normalizePrompt(prompt);
  if (!isReplaceTextPrompt(normalized)) return "";
  const quoted = quotedTextSegments(prompt);
  if (!quoted.length) return "";
  const replacement =
    quoted.length >= 2 && (normalized.includes("replace") || normalized.includes("замен"))
      ? quoted[1]
      : quoted[quoted.length - 1];
  return replacement.trim() ? replacement : "";
}

function applyInlineHelloWorldOutputReplacement(prompt, task, content) {
  if (task !== "hello_world") return content;
  const replacement = inlineHelloWorldReplacement(prompt);
  return replacement ? String(content).split("Hello, world!").join(replacement) : content;
}

// Issue #330 (R9): the guidance sentences — "How it works" explanations and
// "How to test it yourself" steps — are the `coding_guidance` records of
// data/seed/coding-guidance.lino in every response language (R379), the same
// records rust/src/coding/guidance.rs reads.
let CODING_GUIDANCE_RECORDS = null;

/**
 * The `coding_guidance` records by id, read once the seed is loaded.
 * @returns {Map<string, object>}
 */
function codingGuidanceRecords() {
  if (CODING_GUIDANCE_RECORDS !== null) return CODING_GUIDANCE_RECORDS;
  const text = seedRawText(SEED_RAW, "coding-guidance.lino");
  if (!text) return new Map();
  const field = (node, name) => {
    const child = node.children.find((entry) => entry.name === name);
    return child ? String(child.value || "") : "";
  };
  CODING_GUIDANCE_RECORDS = new Map(
    parseLinoTree(text).children
      .filter((node) => field(node, "record_type") === "coding_guidance")
      .map((node) => [field(node, "id"), node]),
  );
  return CODING_GUIDANCE_RECORDS;
}

/**
 * The guidance sentence `id` in `language`, falling back to English, each
 * `{name}` slot filled in one pass; a record seed does not carry is the empty
 * string. Mirrors `guidance` in rust/src/coding/guidance.rs.
 * @param {string} id
 * @param {string} language
 * @param {Object<string, string>} [values]
 * @returns {string}
 */
function codingGuidance(id, language, values = {}) {
  const record = codingGuidanceRecords().get(id);
  if (!record) return "";
  const surface = (slug) => {
    if (slug === "id" || slug === "record_type") return "";
    const child = record.children.find((entry) => entry.name === slug && entry.children.length === 0);
    return child ? String(child.value || "") : "";
  };
  const template = surface(language) || surface("en");
  return template.replace(/\{([^{}]*)\}/gu, (slot, name) =>
    Object.prototype.hasOwnProperty.call(values, name) ? values[name] : slot);
}

function programExplanation(task, language) {
  return codingGuidance(task, language) || codingGuidance("default_explanation", language);
}

function programExplanationSection(task, language) {
  return `${codingGuidance("how_it_works_heading", language)}\n${programExplanation(task, language)}`;
}

// Issue #330 (R9): did an earlier assistant turn already present a fenced code
// block? When it did, follow-up code edits omit the verbose setup steps and show
// a concise "test it the same way" note instead. Mirrors
// `coding::guidance::history_has_prior_code`.
function historyHasPriorCode(history) {
  return (Array.isArray(history) ? history : []).some(
    (turn) =>
      turn &&
      String(turn.role || "").toLowerCase() === "assistant" &&
      String(turn.content || "").includes("```"),
  );
}

// Issue #330 (R9): step-by-step, novice-friendly instructions for testing the
// program, localized for every response language. When the dialog already
// walked the user through running code, the verbose setup is replaced by a
// short "test it the same way" note. Mirrors
// `coding::guidance::program_test_instructions`.
function programTestInstructions(languageInfo, language, priorCodeResponse, task) {
  const saveAs = languageInfo.saveAs;
  const runCommand = programRunCommandLine(task, languageInfo.runCommand); // stdin fixture piped in
  const checkCommand = languageInfo.checkCommand;

  const values = { save_as: saveAs, run_command: runCommand };
  if (priorCodeResponse) return codingGuidance("prior_code_note", language, values);

  const heading = codingGuidance("how_to_test_heading", language);
  const steps = [
    codingGuidance("step_install", language, { setup_hint: languageInfo.setupHint }),
    codingGuidance("step_save", language, values),
  ];
  if (checkCommand) steps.push(codingGuidance("step_check", language, { check_command: checkCommand }));
  steps.push(codingGuidance("step_run", language, values));
  steps.push(codingGuidance("step_compare", language));

  const numbered = steps.map((step, index) => `${index + 1}. ${step}`).join("\n");
  return `${heading}\n${numbered}`;
}

// ---------------------------------------------------------------------------
// Issue #340 (R7): composite-program *blueprints*. A `write_program` request can
// name a language we support but a *task* the verified template catalog cannot
// resolve to a single, sandbox-runnable program — e.g. "make an HTTP GET, parse
// the JSON, compute the mean and median". Before this, such a request fell
// through to `write_program_unsupported` (a dead end). A blueprint closes that
// gap while staying honest: it decomposes the prompt into recognized
// capabilities, matches a curated recipe, and returns the full program with its
// decomposition plan, library prerequisites, and an honest "not run" report
// (these programs need external libraries / network access the sandbox cannot
// provide, so they can never claim "compiled and ran").
//
// This is a binary-matching mirror of `src/coding/blueprint.rs`; the parity
// experiment (`experiments/issue-340-worker-parity.mjs`) keeps the two copies in
// lockstep.

// A recognized programming capability — one "sub-task" a composite request can
// decompose into. Detection is keyword-based and script-aware, mirroring
// `CAPABILITIES` in `src/coding/blueprint.rs`.
const BLUEPRINT_CAPABILITIES = [
  {
    slug: "http_request",
    label: "Make an HTTP request",
    keywords: [
      "http",
      "https",
      "url",
      "get request",
      "http get",
      "fetch",
      "download",
      "запрос",
      "ссылк",
      "загруз",
      "स्थानांतरण",
      "अनुरोध",
      "请求",
      "下载",
      "网址",
    ],
  },
  {
    slug: "json_parse",
    label: "Parse the JSON response",
    keywords: [
      "json",
      "parse",
      "parses",
      "parsing",
      "deserialize",
      "разбор",
      "разобрать",
      "парсинг",
      "जेसन",
      "पार्स",
      "解析",
    ],
  },
  {
    slug: "statistics",
    label: "Calculate statistics (mean, median)",
    keywords: [
      "statistics",
      "statistic",
      "mean",
      "average",
      "median",
      "статистик",
      "среднее",
      "медиан",
      "औसत",
      "माध्यिका",
      "सांख्यिकी",
      "统计",
      "平均",
      "中位数",
    ],
  },
  {
    slug: "output_results",
    label: "Output the results",
    keywords: [
      "output",
      "print",
      "outputs",
      "display",
      "report",
      "вывод",
      "вывести",
      "печат",
      "आउटपुट",
      "छाप",
      "输出",
      "打印",
      "显示",
    ],
  },
  {
    slug: "error_handling",
    label: "Handle errors",
    keywords: [
      "error handling",
      "error-handling",
      "errors",
      "error",
      "exception",
      "ошибк",
      "обработк",
      "त्रुटि",
      "错误",
      "异常",
    ],
  },
  {
    slug: "comments",
    label: "Document the code with comments",
    keywords: [
      "comments",
      "comment",
      "commented",
      "documented",
      "комментар",
      "टिप्पणि",
      "注释",
      "评论",
    ],
  },
  {
    slug: "web_research",
    label: "Research current source data",
    keywords: [
      "search",
      "research",
      "sources",
      "source",
      "look up",
      "current",
      "average",
      "поиск",
      "источник",
      "источники",
      "искать",
      "खोज",
      "स्रोत",
      "वर्तमान",
      "搜索",
      "来源",
    ],
  },
  {
    slug: "city_costs",
    label: "Compare city living costs",
    keywords: [
      "living costs",
      "cost of living",
      "average rent",
      "rent",
      "moscow",
      "berlin",
      "new york",
      "city",
      "cities",
      "аренда",
      "стоимость жизни",
      "москва",
      "берлин",
      "нью-йорк",
      "जीवन यापन",
      "लागत",
      "किराया",
      "मास्को",
      "बर्लिन",
      "न्यूयॉर्क",
      "租金",
      "生活成本",
    ],
  },
  {
    slug: "visa_requirements",
    label: "Check visa requirements",
    keywords: ["visa", "visa-free", "russian citizens", "requirements"],
  },
  {
    slug: "flight_costs",
    label: "Estimate flight costs",
    keywords: [
      "flight costs",
      "flight cost",
      "flight",
      "from moscow",
      "next 3 months",
      "destinations",
    ],
  },
  {
    slug: "travel_planner_class",
    label: "Build a travel-planner class",
    keywords: [
      "travel planner",
      "travelplanner",
      "itinerary",
      "generate_itinerary",
      "add_destination",
      "destination",
      "trip",
      "class",
    ],
  },
  {
    slug: "budget_flags",
    label: "Flag destinations over budget",
    keywords: [
      "budget < estimated cost",
      "budget warning",
      "estimated cost",
      "prioritize",
      "visa-free access",
      "flag",
      "budget",
    ],
  },
  {
    slug: "sample_itinerary",
    label: "Generate a sample itinerary",
    keywords: ["sample output", "sample itinerary", "7-day", "7 day", "$2000", "$2,000"],
  },
  {
    slug: "budget_rule",
    label: "Apply the 50/30/20 budget rule",
    keywords: [
      "50/30/20",
      "budget rule",
      "monthly income",
      "income",
      "needs",
      "wants",
      "savings",
      "бюджет",
      "доход",
      "сбереж",
      "बजट",
      "आय",
      "बचत",
      "预算",
      "收入",
    ],
  },
  {
    slug: "compound_savings",
    label: "Project compound savings",
    keywords: [
      "annual return",
      "return",
      "8%",
      "10 years",
      "$3000",
      "100,000",
      "100000",
      "years to save",
      "накопить",
      "доходность",
      "वार्षिक रिटर्न",
      "रिटर्न",
      "साल",
      "收益",
      "年",
    ],
  },
  {
    slug: "markdown_report",
    label: "Export a Markdown comparison report",
    keywords: [
      "markdown",
      "formatted markdown",
      "report",
      "comparison table",
      "table",
      "export",
      "отчет",
      "отчёт",
      "таблица",
      "экспорт",
      "मार्कडाउन",
      "रिपोर्ट",
      "तालिका",
      "निर्यात",
      "报告",
      "表格",
      "导出",
    ],
  },
  {
    slug: "source_text",
    label: "Read the program's own source code as text",
    keywords: [
      "own source",
      "source code as text",
      "source code",
      "source text",
      "itself",
    ],
  },
  {
    slug: "source_metrics",
    label: "Count functions, loops, conditionals, and comments",
    keywords: ["counts", "count", "functions", "loops", "conditionals", "comments"],
  },
  {
    slug: "complexity_score",
    label: "Calculate a cyclomatic-complexity score",
    keywords: ["complexity score", "cyclomatic", "complexity"],
  },
  {
    slug: "json_report",
    label: "Output the metrics as JSON",
    keywords: ["json report", "json", "metrics"],
  },
  {
    slug: "self_response_analysis",
    label: "Analyze the assistant response with the same metrics",
    keywords: ["your own response", "own response", "reasoning text", "same metrics"],
  },
  {
    slug: "complexity_comparison",
    label: "Compare code complexity with reasoning-text complexity",
    keywords: ["compare", "more complex", "which is more complex"],
  },
  {
    slug: "crypto_prices",
    label: "Fetch current crypto prices",
    keywords: [
      "crypto",
      "btc",
      "eth",
      "ton",
      "usdt",
      "current prices",
      "public api",
    ],
  },
  {
    slug: "portfolio_holdings",
    label: "Model portfolio holdings",
    keywords: ["portfolio", "holdings"],
  },
  {
    slug: "portfolio_calculations",
    label: "Calculate total value, 24h changes, and weights",
    keywords: [
      "total value",
      "value in usd",
      "24h change",
      "weight distribution",
    ],
  },
  {
    slug: "alert_logic",
    label: "Notify when an asset drops more than 5%",
    keywords: ["alert", "notify", "drops"],
  },
  {
    slug: "mock_api",
    label: "Mock the public API endpoint",
    keywords: ["mock", "mock endpoint"],
  },
];
