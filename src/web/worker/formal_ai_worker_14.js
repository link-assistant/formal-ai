// Worker module 15 of 21. Loaded by ../formal_ai_worker.js.
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
  const mentionsProgramRequest = lexiconMentionsRole(
    ROLE_PROGRAM_REQUEST,
    normalized,
  );
  const asksForProgram =
    lexiconMentionsRole(ROLE_PROGRAM_KIND, normalized) &&
    mentionsProgramRequest;
  const asksForKnownLanguageProgram =
    Boolean(language) &&
    mentionsProgramRequest &&
    (Boolean(WRITE_PROGRAM_LANGUAGES[language]) || codingOracleKnowsLanguage(language));
  if (!task && !asksForProgram && !asksForKnownLanguageProgram) return null;
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
    unsupported: (language, task, languages, tasks) =>
      `I can route \`write_program(language, task)\`, but I do not have a template for ` +
      `language \`${language}\` and task \`${task}\`. ` +
      `Supported languages: ${languages}. Supported tasks: ${tasks}.`,
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
    unsupported: (language, task, languages, tasks) =>
      `Я могу выполнить \`write_program(language, task)\`, но у меня нет шаблона для ` +
      `языка \`${language}\` и задачи \`${task}\`. ` +
      `Поддерживаемые языки: ${languages}. Поддерживаемые задачи: ${tasks}.`,
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
    unsupported: (language, task, languages, tasks) =>
      `मैं \`write_program(language, task)\` रूट कर सकता हूँ, लेकिन भाषा \`${language}\` और ` +
      `कार्य \`${task}\` के लिए मेरे पास कोई टेम्पलेट नहीं है। ` +
      `समर्थित भाषाएँ: ${languages}. समर्थित कार्य: ${tasks}.`,
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
    unsupported: (language, task, languages, tasks) =>
      `我可以路由 \`write_program(language, task)\`，但我没有语言 \`${language}\` 和任务 ` +
      `\`${task}\` 的模板。支持的语言：${languages}。支持的任务：${tasks}。`,
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

function writeProgramExecutionLines(language, task, code, output, strings) {
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
  const reason =
    language === "javascript" ? i18n.noFilesystem(language) : i18n.noToolchain(language);
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

// Issue #330 (R9): a localized plain-language "How it works" paragraph so the
// demo teaches a novice instead of returning an unexplained snippet. Mirrors
// `coding::guidance::program_explanation`; `__fallback` is the neutral wording
// used for any task without a bespoke explanation yet.
const PROGRAM_EXPLANATIONS = {
  hello_world: {
    en: "The program prints the text `Hello, world!` to standard output and then exits.",
    ru: "Программа выводит текст `Hello, world!` в стандартный вывод и завершается.",
    hi: "प्रोग्राम मानक आउटपुट पर `Hello, world!` टेक्स्ट छापता है और फिर समाप्त हो जाता है।",
    zh: "程序将文本 `Hello, world!` 打印到标准输出，然后退出。",
  },
  count_to_three: {
    en: "The program prints the numbers 1, 2, and 3 — each on its own line — and then exits.",
    ru: "Программа выводит числа 1, 2 и 3 — каждое на отдельной строке — и завершается.",
    hi: "प्रोग्राम संख्याएँ 1, 2 और 3 — हर एक अलग पंक्ति में — छापता है और फिर समाप्त हो जाता है।",
    zh: "程序打印数字 1、2 和 3 —— 每个数字单独一行 —— 然后退出。",
  },
  list_files: {
    en:
      "The program reads the entries of the current directory, keeps only the regular " +
      "files, collects their names into a list, sorts the list alphabetically, and " +
      "prints each name on its own line.",
    ru:
      "Программа читает содержимое текущего каталога, оставляет только обычные файлы, " +
      "собирает их имена в список, сортирует список по алфавиту и печатает каждое имя " +
      "на отдельной строке.",
    hi:
      "प्रोग्राम वर्तमान निर्देशिका की प्रविष्टियाँ पढ़ता है, केवल सामान्य फ़ाइलें रखता है, उनके " +
      "नाम एक सूची में एकत्र करता है, सूची को वर्णानुक्रम में क्रमबद्ध करता है, और हर नाम को " +
      "अलग पंक्ति में छापता है।",
    zh:
      "程序读取当前目录的条目，只保留普通文件，将它们的名称收集到一个列表中，" +
      "按字母顺序排序，然后将每个名称打印在单独一行。",
  },
  list_files_arg: {
    en:
      "The program takes the directory path from the first command-line argument " +
      "(falling back to the current directory when none is given), reads that " +
      "directory's entries, keeps only the regular files, sorts their names " +
      "alphabetically, and prints each name on its own line.",
    ru:
      "Программа берёт путь к каталогу из первого аргумента командной строки (если " +
      "аргумент не задан, используется текущий каталог), читает содержимое этого " +
      "каталога, оставляет только обычные файлы, сортирует их имена по алфавиту и " +
      "печатает каждое имя на отдельной строке.",
    hi:
      "प्रोग्राम पहले कमांड-लाइन तर्क से निर्देशिका पथ लेता है (कोई तर्क न होने पर वर्तमान " +
      "निर्देशिका का उपयोग करता है), उस निर्देशिका की प्रविष्टियाँ पढ़ता है, केवल सामान्य " +
      "फ़ाइलें रखता है, उनके नामों को वर्णानुक्रम में क्रमबद्ध करता है, और हर नाम को अलग पंक्ति " +
      "में छापता है।",
    zh:
      "程序从第一个命令行参数获取目录路径（未提供参数时使用当前目录），读取该目录的条目，" +
      "只保留普通文件，按字母顺序排序它们的名称，然后将每个名称打印在单独一行。",
  },
  list_files_reverse_sort: {
    en:
      "The program reads the entries of the current directory, keeps only the regular " +
      "files, collects their names into a list, sorts the list in reverse alphabetical " +
      "order, and prints each name on its own line.",
    ru:
      "Программа читает содержимое текущего каталога, оставляет только обычные файлы, " +
      "собирает их имена в список, сортирует список в обратном алфавитном порядке и " +
      "печатает каждое имя на отдельной строке.",
    hi:
      "प्रोग्राम वर्तमान निर्देशिका की प्रविष्टियाँ पढ़ता है, केवल सामान्य फ़ाइलें रखता है, उनके " +
      "नाम एक सूची में एकत्र करता है, सूची को उल्टे वर्णानुक्रम में क्रमबद्ध करता है, और हर नाम " +
      "को अलग पंक्ति में छापता है।",
    zh:
      "程序读取当前目录的条目，只保留普通文件，将它们的名称收集到一个列表中，" +
      "按反向字母顺序排序，然后将每个名称打印在单独一行。",
  },
  list_files_arg_reverse_sort: {
    en:
      "The program takes the directory path from the first command-line argument " +
      "(falling back to the current directory when none is given), reads that " +
      "directory's entries, keeps only the regular files, sorts their names in " +
      "reverse alphabetical order, and prints each name on its own line.",
    ru:
      "Программа берёт путь к каталогу из первого аргумента командной строки (если " +
      "аргумент не задан, используется текущий каталог), читает содержимое этого " +
      "каталога, оставляет только обычные файлы, сортирует их имена в обратном " +
      "алфавитном порядке и печатает каждое имя на отдельной строке.",
    hi:
      "प्रोग्राम पहले कमांड-लाइन तर्क से निर्देशिका पथ लेता है (कोई तर्क न होने पर वर्तमान " +
      "निर्देशिका का उपयोग करता है), उस निर्देशिका की प्रविष्टियाँ पढ़ता है, केवल सामान्य " +
      "फ़ाइलें रखता है, उनके नामों को उल्टे वर्णानुक्रम में क्रमबद्ध करता है, और हर नाम को " +
      "अलग पंक्ति में छापता है।",
    zh:
      "程序从第一个命令行参数获取目录路径（未提供参数时使用当前目录），读取该目录的条目，" +
      "只保留普通文件，按反向字母顺序排序它们的名称，然后将每个名称打印在单独一行。",
  },
  fizzbuzz: {
    en:
      "The program loops over the numbers 1 to 15. For each number it prints `FizzBuzz` " +
      "when the number is divisible by both 3 and 5, `Fizz` when it is divisible by 3, " +
      "`Buzz` when it is divisible by 5, and otherwise the number itself — each on its " +
      "own line.",
    ru:
      "Программа перебирает числа от 1 до 15. Для каждого числа она печатает `FizzBuzz`, " +
      "если оно делится и на 3, и на 5; `Fizz`, если делится на 3; `Buzz`, если делится " +
      "на 5; иначе само число — каждое на отдельной строке.",
    hi:
      "प्रोग्राम 1 से 15 तक की संख्याओं पर लूप करता है। हर संख्या के लिए वह `FizzBuzz` छापता है " +
      "जब वह 3 और 5 दोनों से विभाज्य हो, `Fizz` जब वह 3 से विभाज्य हो, `Buzz` जब वह 5 से " +
      "विभाज्य हो, अन्यथा स्वयं संख्या — हर एक अलग पंक्ति में।",
    zh:
      "程序遍历数字 1 到 15。对于每个数字，当它同时能被 3 和 5 整除时打印 `FizzBuzz`，" +
      "能被 3 整除时打印 `Fizz`，能被 5 整除时打印 `Buzz`，否则打印数字本身 —— 每个单独一行。",
  },
  factorial: {
    en:
      "The program multiplies together the numbers 1 through 5 (1×2×3×4×5), which is the " +
      "factorial of 5, and prints the result, 120.",
    ru:
      "Программа перемножает числа от 1 до 5 (1×2×3×4×5) — это факториал 5 — и печатает " +
      "результат, 120.",
    hi:
      "प्रोग्राम 1 से 5 तक की संख्याओं को आपस में गुणा करता है (1×2×3×4×5), जो 5 का फैक्टोरियल " +
      "है, और परिणाम 120 छापता है।",
    zh: "程序将 1 到 5 的数字相乘（1×2×3×4×5），这就是 5 的阶乘，并打印结果 120。",
  },
  reverse_string: {
    en:
      "The program takes the string `hello`, reverses the order of its characters, and " +
      "prints the result, `olleh`.",
    ru:
      "Программа берёт строку `hello`, переставляет её символы в обратном порядке и " +
      "печатает результат — `olleh`.",
    hi: "प्रोग्राम स्ट्रिंग `hello` लेता है, उसके अक्षरों का क्रम उलटता है, और परिणाम `olleh` छापता है।",
    zh: "程序取字符串 `hello`，将其字符顺序反转，并打印结果 `olleh`。",
  },
  sum_to_ten: {
    en:
      "The program adds together the integers from 1 to 10 (1 + 2 + … + 10) and prints " +
      "the total, 55.",
    ru: "Программа складывает целые числа от 1 до 10 (1 + 2 + … + 10) и печатает сумму — 55.",
    hi: "प्रोग्राम 1 से 10 तक के पूर्णांकों को जोड़ता है (1 + 2 + … + 10) और कुल योग 55 छापता है।",
    zh: "程序将 1 到 10 的整数相加（1 + 2 + … + 10），并打印总和 55。",
  },
  fibonacci: {
    en:
      "The program defines a recursive `fibonacci` function (F(1)=F(2)=1, " +
      "F(n)=F(n-1)+F(n-2)) and prints the 10th term, 55.",
    ru: "Программа определяет рекурсивную функцию `fibonacci` (F(1)=F(2)=1, F(n)=F(n-1)+F(n-2)) и печатает 10-й член — 55.",
    hi: "प्रोग्राम एक पुनरावर्ती `fibonacci` फ़ंक्शन परिभाषित करता है (F(1)=F(2)=1, F(n)=F(n-1)+F(n-2)) और 10वाँ पद 55 छापता है।",
    zh: "程序定义了一个递归的 `fibonacci` 函数（F(1)=F(2)=1，F(n)=F(n-1)+F(n-2)），并打印第 10 项 55。",
  },
  __fallback: {
    en: "The program performs the requested task and prints its result to standard output.",
    ru: "Программа выполняет запрошенную задачу и печатает результат в стандартный вывод.",
    hi: "प्रोग्राम अनुरोधित कार्य करता है और परिणाम को मानक आउटपुट पर छापता है।",
    zh: "程序执行所请求的任务，并将结果打印到标准输出。",
  },
};

function programExplanation(task, language) {
  const byTask = PROGRAM_EXPLANATIONS[task] || PROGRAM_EXPLANATIONS.__fallback;
  return byTask[language] || byTask.en;
}

function programExplanationSection(task, language) {
  const heading =
    { ru: "Как это работает:", hi: "यह कैसे काम करता है:", zh: "工作原理：" }[language] ||
    "How it works:";
  return `${heading}\n${programExplanation(task, language)}`;
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
function programTestInstructions(languageInfo, language, priorCodeResponse) {
  const saveAs = languageInfo.saveAs;
  const runCommand = languageInfo.runCommand;
  const checkCommand = languageInfo.checkCommand;

  if (priorCodeResponse) {
    return (
      {
        ru:
          `Проверьте обновлённую программу так же, как и раньше: сохраните код в файл ` +
          `\`${saveAs}\` и снова выполните \`${runCommand}\`.`,
        hi:
          `अपडेट किए गए प्रोग्राम को पहले की तरह ही जाँचें: कोड को \`${saveAs}\` फ़ाइल में सहेजें ` +
          `और फिर से \`${runCommand}\` चलाएँ।`,
        zh:
          `像之前一样测试更新后的程序：将代码保存到文件 \`${saveAs}\`，然后再次运行 ` +
          `\`${runCommand}\`。`,
      }[language] ||
      `Test the updated program the same way as before: save the code to \`${saveAs}\` ` +
        `and run \`${runCommand}\` again.`
    );
  }

  const heading =
    {
      ru: "Как проверить это самостоятельно:",
      hi: "इसे स्वयं कैसे जाँचें:",
      zh: "如何自行测试：",
    }[language] || "How to test it yourself:";

  const setupHint = languageInfo.setupHint;
  const steps = [];
  steps.push(
    {
      ru: `Установите инструментарий: ${setupHint}.`,
      hi: `टूलचेन इंस्टॉल करें: ${setupHint}।`,
      zh: `安装工具链：${setupHint}。`,
    }[language] || `Install ${setupHint}.`,
  );
  steps.push(
    {
      ru: `Сохраните приведённый выше код в файл \`${saveAs}\`.`,
      hi: `ऊपर दिए गए कोड को \`${saveAs}\` फ़ाइल में सहेजें।`,
      zh: `将上面的代码保存到文件 \`${saveAs}\`。`,
    }[language] || `Save the code above to a file named \`${saveAs}\`.`,
  );
  if (checkCommand) {
    steps.push(
      {
        ru: `Проверьте, что код компилируется: \`${checkCommand}\`.`,
        hi: `जाँचें कि कोड संकलित होता है: \`${checkCommand}\`।`,
        zh: `检查代码能否编译：\`${checkCommand}\`。`,
      }[language] || `Check that it compiles: \`${checkCommand}\`.`,
    );
  }
  steps.push(
    {
      ru: `Запустите программу: \`${runCommand}\`.`,
      hi: `प्रोग्राम चलाएँ: \`${runCommand}\`।`,
      zh: `运行程序：\`${runCommand}\`。`,
    }[language] || `Run it: \`${runCommand}\`.`,
  );
  steps.push(
    {
      ru: "Сравните вывод с разделом ожидаемого вывода выше.",
      hi: "आउटपुट की तुलना ऊपर दिए गए अपेक्षित आउटपुट से करें।",
      zh: "将输出与上面的预期输出部分进行比较。",
    }[language] || "Compare the output with the expected output shown above.",
  );

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

// Curated programs (verbatim copies of the Rust raw-string consts). They are
// hand-written and reviewed, not sandbox-executed (they need network access /
// external libraries), so the execution report stays honest.
const BLUEPRINT_RUST_HTTP_JSON_STATS = `//! Fetch JSON from a URL and report the mean and median of every number in it.
//!
//! Cargo.toml dependencies:
//!   reqwest = { version = "0.12", features = ["blocking", "json"] }
//!   serde_json = "1"

use std::env;
use std::error::Error;

use serde_json::Value;

/// Recursively collect every numeric value out of a decoded JSON document,
/// regardless of how deeply it is nested inside arrays or objects.
fn collect_numbers(value: &Value, numbers: &mut Vec<f64>) {
    match value {
        Value::Number(number) => {
            if let Some(as_float) = number.as_f64() {
                numbers.push(as_float);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_numbers(item, numbers)),
        Value::Object(map) => map.values().for_each(|item| collect_numbers(item, numbers)),
        _ => {}
    }
}

/// Arithmetic mean of the samples (the caller guarantees a non-empty slice).
fn mean(samples: &[f64]) -> f64 {
    samples.iter().sum::<f64>() / samples.len() as f64
}

/// Median of the samples; averages the two middle values when the count is even.
fn median(samples: &mut [f64]) -> f64 {
    samples.sort_by(|left, right| left.partial_cmp(right).expect("no NaN in input"));
    let middle = samples.len() / 2;
    if samples.len() % 2 == 0 {
        (samples[middle - 1] + samples[middle]) / 2.0
    } else {
        samples[middle]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Read the target URL from the first command-line argument.
    let url = env::args()
        .nth(1)
        .ok_or("usage: stats <url-returning-json>")?;

    // 2. Make the HTTP GET request and parse the JSON body. Both steps can fail,
    //    so \`?\` propagates any network or decoding error up to \`main\`.
    let document: Value = reqwest::blocking::get(&url)?.json()?;

    // 3. Gather every number from the decoded document.
    let mut numbers = Vec::new();
    collect_numbers(&document, &mut numbers);
    // region:error_handling
    // Guard against an empty data set before computing statistics.
    if numbers.is_empty() {
        return Err("the JSON response contained no numbers".into());
    }
    // endregion:error_handling

    // 4. Compute and print the statistics.
    println!("count:  {}", numbers.len());
    println!("mean:   {:.4}", mean(&numbers));
    println!("median: {:.4}", median(&mut numbers));
    Ok(())
}
`;
