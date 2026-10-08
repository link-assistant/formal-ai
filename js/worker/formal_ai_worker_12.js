// Worker module 13 of 21. Loaded by ../formal_ai_worker.js.
function pushInstallationCommand(commands, candidate, provenance = INSTALL_PROVENANCE_CODE_SPAN) {
  const command = String(candidate || "").trim();
  if (!command || !looksLikeInstallationCommand(command, provenance)) return;
  if (!commands.includes(command)) commands.push(command);
}

function collectInstallationInlineCommands(source, commands) {
  const text = String(source || "");
  let inTick = false;
  let candidate = "";
  for (const character of text) {
    if (character === "`") {
      if (inTick) {
        // Inline code spans are author-marked code: trust the shape.
        pushInstallationCommand(commands, candidate.trim(), INSTALL_PROVENANCE_CODE_SPAN);
        candidate = "";
        inTick = false;
      } else {
        inTick = true;
      }
      continue;
    }
    if (inTick) candidate += character;
  }
}

function collectInstallationBulletCommands(source, commands) {
  for (const line of String(source || "").split(/\r?\n/)) {
    let trimmed = line.trim();
    trimmed = trimmed.replace(/^[-*+\d]+[.) ]*/, "").trim();
    if (trimmed.startsWith("`") && trimmed.endsWith("`") && trimmed.length > 2) {
      // The whole bullet is a single code span: code provenance.
      pushInstallationCommand(commands, trimmed.slice(1, -1), INSTALL_PROVENANCE_CODE_SPAN);
    } else {
      // Raw document line with no code markup: prove it structurally.
      pushInstallationCommand(commands, trimmed, INSTALL_PROVENANCE_BARE_LINE);
    }
  }
}

function collectInstallationScriptCommands(source, commands) {
  for (const line of String(source || "").split(/\r?\n/)) {
    const trimmed = normalizeInstallationScriptLine(line);
    // Lines inside a shell/PowerShell fence are code by construction.
    if (!shouldSkipInstallationScriptLine(trimmed))
      pushInstallationCommand(commands, trimmed, INSTALL_PROVENANCE_CODE_SPAN);
  }
}

// Translate a single verb token into an action category. Keyed on the verb
// itself (not the surrounding tool), so the same lexicon serves every program.
// Returns the marker "run" for generic launcher verbs so the caller can prefer
// a more concrete object.
function classifyInstallationVerb(token) {
  switch (token) {
    case "clone":
      return "Clone the repository";
    case "cd":
    case "chdir":
    case "pushd":
      return "Enter the project directory";
    case "install":
    case "add":
    case "ci":
    case "restore":
    case "sync":
    case "bootstrap":
    case "vendor":
    case "i":
      return "Install dependencies";
    case "test":
    case "check":
    case "lint":
    case "doctor":
    case "verify":
    case "validate":
    case "version":
    case "pytest":
    case "jest":
    case "mocha":
    case "vitest":
    case "tox":
      return "Run the verification command";
    case "build":
    case "compile":
    case "configure":
    case "make":
    case "package":
    case "dist":
    case "bundle":
    case "cmake":
    case "gradle":
    case "ninja":
    case "msbuild":
      return "Build the project";
    case "run":
    case "serve":
    case "start":
    case "up":
    case "exec":
    case "dev":
    case "launch":
    case "watch":
      return "run";
    default:
      return null;
  }
}

// Structural view of a command: the program (last path segment of the
// executable), the ordered non-flag argument tokens, and whether a version/help
// probe flag is present.
function parseInstallationCommand(command) {
  let tokens = String(command || "").trim().split(/\s+/).filter(Boolean);
  while (tokens.length && (tokens[0] === "sudo" || tokens[0] === "env" || tokens[0] === "command")) {
    tokens = tokens.slice(1);
  }
  const rawProgram = tokens.shift() || "";
  let program = rawProgram.split("/").pop().toLowerCase();

  let rest = tokens;
  if ((program === "python" || program === "python3" || program === "py") && rest[0] === "-m" && rest[1]) {
    program = rest[1].toLowerCase();
    rest = rest.slice(2);
  }

  const args = [];
  let isProbe = false;
  for (const token of rest) {
    const bare = token.replace(/^['"]+/, "").replace(/['"]+$/, "");
    if (["--version", "-v", "-V", "--help", "-h"].includes(bare)) {
      isProbe = true;
      continue;
    }
    if (bare.startsWith("-")) continue;
    args.push(bare.toLowerCase());
  }
  return { program, args, isProbe };
}

// Derive a human-readable step description from the parsed verb/object of the
// command rather than matching the whole string against a substring table.
function describeInstallationCommand(command) {
  const parsed = parseInstallationCommand(command);
  if (parsed.isProbe) return "Verify the installation";

  let genericRun = false;
  for (const argument of parsed.args) {
    const action = classifyInstallationVerb(argument);
    if (action === "run") {
      genericRun = true;
    } else if (action) {
      return action;
    }
  }
  const programAction = classifyInstallationVerb(parsed.program);
  if (programAction === "run") {
    genericRun = true;
  } else if (programAction) {
    return programAction;
  }
  if (genericRun) return "Start the application";

  // Fall back to a description synthesized from the program/verb so unseen but
  // well-formed commands still read meaningfully.
  if (parsed.args.length) return `Run the ${parsed.program} ${parsed.args[0]} step`;
  return `Run ${parsed.program}`;
}

function extractInstallationSteps(source, sourceFormat) {
  const commands = [];
  if (sourceFormat === INSTALL_FORMAT_MARKDOWN) {
    for (const block of installationFencedBlocks(source)) {
      if (isInstallationShellFence(block.info) || isInstallationPowerShellFence(block.info)) {
        collectInstallationScriptCommands(block.body, commands);
      }
    }
    collectInstallationInlineCommands(source, commands);
    collectInstallationBulletCommands(source, commands);
  } else {
    collectInstallationScriptCommands(source, commands);
  }
  return commands.map((command, index) => ({
    id: `S${index + 1}`,
    description: describeInstallationCommand(command),
    command,
  }));
}

function extractInstallationProject(prompt) {
  const source = String(prompt || "");
  const lower = source.toLowerCase();
  const marker = " for ";
  const start = lower.indexOf(marker);
  if (start < 0) return "the project";
  const tail = source.slice(start + marker.length);
  const stopMatch = tail.match(/[\s,:;\n]/);
  const stop = stopMatch ? stopMatch.index : tail.length;
  const project = tail.slice(0, stop).trim();
  return project.includes("/") || project.includes("-") ? project : "the project";
}

function installationMeaningKey(conversion) {
  const parts = [`source=${conversion.sourceFormat}`, `project=${conversion.project}`];
  for (const target of conversion.targetFormats) parts.push(`target=${target}`);
  for (const step of conversion.steps) parts.push(`command=${step.command}`);
  return parts.join(";");
}

function installationEvidence(conversion) {
  const evidence = [
    "formalization:install_steps_ir",
    `meaning:${stableBehaviorRuleId("installation_conversion_request", installationMeaningKey(conversion))}`,
    ...metaAlgorithmConstructionEvidence("installation_conversion"),
    `installation_conversion:source_format:${conversion.sourceFormat}`,
    `installation_conversion:project:${conversion.project}`,
  ];
  for (const target of conversion.targetFormats) {
    evidence.push(`installation_conversion:target_format:${target}`);
  }
  for (const step of conversion.steps) {
    evidence.push(`installation_conversion:step:${step.id}:${step.command}`);
  }
  evidence.push("installation_conversion:validation:ordered_commands_preserved");
  return evidence;
}

function renderInstallationLino(conversion) {
  const lines = ["installation_conversion_request"];
  lines.push(`  source_format ${conversion.sourceFormat}`);
  for (const target of conversion.targetFormats) lines.push(`  target_format ${target}`);
  lines.push(`  project ${linoString(conversion.project)}`);
  lines.push(`  validation ${linoString("ordered_commands_preserved")}`);
  lines.push(`  validation ${linoString("single_ir_renders_markdown_shell_powershell")}`);
  appendMetaAlgorithmConstructionLino(lines, "installation_conversion");
  for (const step of conversion.steps) {
    lines.push(`  step ${linoString(step.id)}`);
    lines.push(`  description ${linoString(step.description)}`);
    lines.push(`  command ${linoString(step.command)}`);
  }
  return lines.join("\n") + "\n";
}

function renderInstallationMetaAlgorithm() {
  const definition = metaAlgorithmDefinition();
  const lines = [answerFor("coding_meta_algorithm_heading", "en")];
  definition.stages.forEach((stage, index) => lines.push(renderMetaAlgorithmTemplate(
    answerFor("coding_meta_algorithm_stage", "en"),
    { index: index + 1, stage: stage.id, output: stage.output, verifier: stage.verifier },
  )));
  lines.push("", answerFor("coding_meta_algorithm_solutions_heading", "en"));
  for (const surface of definition.surfaces) {
    lines.push(renderMetaAlgorithmTemplate(
      answerFor("coding_meta_algorithm_surface", "en"),
      {
        surface: surface.slug,
        active: surface.slug === "installation_conversion" ? definition.activeMarker : "",
        projection: surface.projection,
      },
    ));
  }
  return lines.join("\n");
}

function activeMetaAlgorithmSurface(trace) {
  const hasCandidate = trace.some((entry) => entry.startsWith("rule_synthesis_candidate:"));
  const verified = trace.some(
    (entry) => entry.startsWith("rule_verification:") && entry.includes("status passed"),
  );
  return hasCandidate && verified ? "rule_synthesis" : "coding_catalog";
}

function renderInstallationMarkdownGuide(conversion) {
  const lines = ["README.md installation guide:", "", "## Installation", ""];
  conversion.steps.forEach((step, index) => {
    lines.push(`${index + 1}. ${step.description}.`);
    lines.push("");
    lines.push("   ```sh");
    lines.push(`   ${step.command}`);
    lines.push("   ```");
  });
  return lines.join("\n") + "\n";
}

function renderInstallationShellScript(conversion) {
  const lines = ["Bash script:", "```bash", "#!/usr/bin/env bash", "set -euo pipefail", ""];
  for (const step of conversion.steps) {
    lines.push(`# ${step.description}`);
    lines.push(step.command);
  }
  lines.push("```");
  return lines.join("\n") + "\n";
}

function renderInstallationPowerShellScript(conversion) {
  const lines = ["PowerShell script:", "```powershell", "$ErrorActionPreference = 'Stop'", ""];
  for (const step of conversion.steps) {
    lines.push(`# ${step.description}`);
    lines.push(step.command);
  }
  lines.push("```");
  return lines.join("\n") + "\n";
}

function renderInstallationConversion(conversion) {
  const lines = [
    `Converted installation instructions for ${conversion.project}.`,
    "",
    "Formalized meaning:",
    "```lino",
    renderInstallationLino(conversion).trimEnd(),
    "```",
    "",
    "Conversion algorithm:",
    "1. Detect the source surface and requested target surface(s).",
    "2. Extract command-like install/deploy steps in original order.",
    "3. Render every target from the same install-step IR.",
    "4. Preserve commands verbatim so the conversion can round-trip.",
    "",
    renderInstallationMetaAlgorithm(),
  ];
  for (const target of conversion.targetFormats) {
    lines.push("");
    if (target === INSTALL_FORMAT_MARKDOWN) {
      lines.push(renderInstallationMarkdownGuide(conversion).trimEnd());
    } else if (target === INSTALL_FORMAT_SHELL) {
      lines.push(renderInstallationShellScript(conversion).trimEnd());
    } else if (target === INSTALL_FORMAT_POWERSHELL) {
      lines.push(renderInstallationPowerShellScript(conversion).trimEnd());
    }
  }
  return lines.join("\n").trimEnd();
}

function tryInstallationConversion(prompt, normalized) {
  if (!isInstallationConversionRequest(normalized)) return null;
  const sourceFormat = detectInstallationSourceFormat(prompt, normalized);
  const targetFormats = detectInstallationTargetFormats(normalized, sourceFormat);
  const sourceText = extractInstallationSourceText(prompt, sourceFormat);
  let steps = extractInstallationSteps(sourceText, sourceFormat);
  if (steps.length === 0 && sourceFormat === INSTALL_FORMAT_MARKDOWN && sourceText !== String(prompt || "")) {
    steps = extractInstallationSteps(prompt, sourceFormat);
  }
  if (steps.length === 0) return null;
  const conversion = {
    sourceFormat,
    targetFormats,
    project: extractInstallationProject(prompt),
    steps,
  };
  return {
    intent: "installation_conversion",
    content: renderInstallationConversion(conversion),
    confidence: 0.84,
    evidence: installationEvidence(conversion),
  };
}

function trySoftwareProjectRequest(prompt, history = []) {
  const normalized = normalizePrompt(prompt);
  if (isSoftwareApprovalPrompt(normalized)) {
    const prior = priorSoftwareProjectMeaning(history);
    if (prior) {
      return {
        intent: "software_project_implementation",
        content: renderSoftwareProjectImplementation(prior),
        confidence: 0.82,
        evidence: softwareEvidence(prior, true),
      };
    }
  }

  const meaning = formalizeSoftwareProjectRequest(prompt);
  if (!meaning) return null;

  return {
    intent: "software_project_plan",
    content: renderSoftwareProjectPlan(meaning),
    confidence: 0.78,
    evidence: softwareEvidence(meaning, false),
  };
}

// Issue #918 (R918-2): the verb each follow-up kind renders is the
// software_project_followup_action table, the gates and the expected-output
// word limit the software_project_followup policy of data/seed/handler-rules.lino,
// and every sentence a seeded software_project_followup_* response. The words
// that *recognise* each kind are the software_followup_* roles (issue #386).
// Mirrors src/solver_handlers/software_project_followup.rs.
function softwareFollowUpGates() {
  return String(handlerRulesPolicy("software_project_followup", "gates") || "").split(" ").filter(Boolean);
}

function softwareFollowUpText(name, language, values = {}) {
  return handlerRulesFillOnce(answerFor(`software_project_followup_${name}`, language), values);
}

// Recover the active software-project dialogue from history regardless of
// whether the plan was approved. Mirrors `prior_software_project_dialogue`.
function priorSoftwareProjectDialogue(history) {
  const assistant = lastHistoryTurn(history, "assistant");
  if (!assistant || !assistant.includes("software_project_request")) {
    return null;
  }
  const approved = assistant.includes("approval_state approved");
  const user = lastHistoryTurn(history, "user");
  const meaning = user ? formalizeSoftwareProjectRequest(user) : null;
  return meaning ? { meaning, approved } : null;
}

// Pull the first domain-like token (e.g. `wikipedia.org`) out of the prompt.
function extractFollowUpTargetSite(prompt) {
  for (const raw of String(prompt || "").split(/\s+/)) {
    const token = raw.replace(/^[^A-Za-z0-9]+|[^A-Za-z0-9]+$/g, "");
    if (!token.includes(".")) continue;
    const lastDot = token.lastIndexOf(".");
    const host = token.slice(0, lastDot);
    const tld = token.slice(lastDot + 1);
    if (
      tld.length >= 2 &&
      /^[A-Za-z]+$/.test(tld) &&
      /[A-Za-z]/.test(host)
    ) {
      return token.toLowerCase();
    }
  }
  return null;
}

// Capture the clause after an output_display_request prefix opener ("show me
// …", "покажи …"), in seed order and capped at the policy word limit, so the
// follow-up records what the user wants surfaced. Mirrors extract_expected_output.
function extractFollowUpExpectedOutput(prompt) {
  const source = String(prompt || "");
  const lower = source.toLowerCase();
  const limit = Number(handlerRulesPolicy("software_project_followup", "output_word_limit") || 0);
  for (const marker of prefixLiterals("output_display_request")) {
    const found = lower.indexOf(marker);
    if (found < 0) continue;
    const start = found + marker.length;
    const tail = source.slice(start);
    const stopMatch = tail.match(/[.?\n;]/);
    const stop = stopMatch ? stopMatch.index : tail.length;
    const clause = tail
      .slice(0, stop)
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, limit)
      .join(" ");
    if (clause) return clause;
  }
  return null;
}

// Recognise which follow-up a prompt evidences by *meaning*, not a hardcoded
// per-language marker table (issue #386). Each follow-up kind is a
// self-describing meaning in data/seed/meanings-software-project.lino; its
// surface words (every supported language) live there, while this code knows
// only the concepts and their precedence — verification outranks execution
// outranks demonstration, so a combined "test it and run it" records the
// stronger goal. Mirrors follow_up_kind in
// src/solver_handlers/software_project_followup.rs.
function detectSoftwareFollowUp(prompt, normalized) {
  let kind = null;
  let kindRole = null;
  for (const [role, candidate] of [
    [ROLE_SOFTWARE_FOLLOWUP_VERIFICATION, "verification"],
    [ROLE_SOFTWARE_FOLLOWUP_EXECUTION, "execution"],
    [ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION, "demonstration"],
  ]) {
    if (lexiconMentionsRoleSubstring(role, normalized)) {
      kind = candidate;
      kindRole = role;
      break;
    }
  }
  if (!kind) return null;
  return {
    kind,
    action: handlerRulesTableValue("software_project_followup_action", kindRole),
    targetSite: extractFollowUpTargetSite(prompt),
    expectedOutput: extractFollowUpExpectedOutput(prompt),
  };
}

function followUpMeaningId(meaning, followUp) {
  const key = [
    `parent=${stableSoftwareMeaningId(meaning)}`,
    `kind=${followUp.kind}`,
    `site=${followUp.targetSite || ""}`,
    `output=${followUp.expectedOutput || ""}`,
  ].join(";");
  return stableBehaviorRuleId("software_project_followup", key);
}

function followUpReasoningSteps(meaning, followUp, language) {
  const values = { action: followUp.action, kind: followUp.kind, artifact: meaning.artifact };
  const steps = [softwareFollowUpText("step_recognize", language, values)];
  if (followUp.targetSite) {
    steps.push(softwareFollowUpText("step_bind_site", language, { site: followUp.targetSite }));
  }
  if (followUp.expectedOutput) {
    steps.push(softwareFollowUpText("step_record_output", language, { output: followUp.expectedOutput }));
  }
  steps.push(softwareFollowUpText("step_fixture", language), softwareFollowUpText("step_gates", language));
  return steps;
}

function followUpPlanSteps(meaning, followUp, language) {
  const site = followUp.targetSite || softwareFollowUpText("default_target", language);
  const steps = [
    softwareFollowUpText("plan_generate", language, { artifact: meaning.artifact, site }),
    softwareFollowUpText("plan_assert", language),
  ];
  if (followUp.expectedOutput) {
    steps.push(softwareFollowUpText("plan_surface", language, { output: followUp.expectedOutput }));
  }
  steps.push(
    softwareFollowUpText("plan_run", language, { language: meaning.implementationLanguage }),
    softwareFollowUpText("plan_promote", language, { site }),
  );
  return steps;
}

function followUpEvidence(meaning, followUp, approved) {
  const evidence = [
    "formalization:text_to_links_notation",
    `meaning:${followUpMeaningId(meaning, followUp)}`,
    `software_project:parent:${stableSoftwareMeaningId(meaning)}`,
    `software_project:follow_up_kind:${followUp.kind}`,
  ];
  if (followUp.targetSite) {
    evidence.push(`software_project:target_site:${followUp.targetSite}`);
  }
  if (followUp.expectedOutput) {
    evidence.push(`software_project:expected_output:${followUp.expectedOutput}`);
  }
  evidence.push(`approval_state:${softwareApprovalLabel(approved)}`);
  for (const gate of softwareFollowUpGates()) {
    evidence.push(`approval_gate:${gate}`);
  }
  return evidence;
}

function renderSoftwareProjectFollowUp(meaning, followUp, approved, language) {
  const lines = [];
  lines.push(softwareFollowUpText("recorded", language, { kind: followUp.kind, artifact: meaning.artifact }));
  lines.push("");
  lines.push(softwareFollowUpText("heading_meaning", language));
  lines.push("```lino");
  lines.push("software_project_followup");
  lines.push(`  parent_request ${linoString(stableSoftwareMeaningId(meaning))}`);
  lines.push(`  parent_artifact ${linoString(meaning.artifact)}`);
  lines.push(`  action ${linoString(followUp.action)}`);
  lines.push(`  follow_up_kind ${followUp.kind}`);
  if (followUp.targetSite) {
    lines.push(`  target_site ${linoString(followUp.targetSite)}`);
  }
  if (followUp.expectedOutput) {
    lines.push(`  expected_output ${linoString(followUp.expectedOutput)}`);
  }
  lines.push(`  delivery_mode ${meaning.deliveryMode}`);
  lines.push(`  implementation_language ${linoString(meaning.implementationLanguage)}`);
  lines.push(`  approval_state ${softwareApprovalLabel(approved)}`);
  lines.push("  approval_required true");
  for (const gate of softwareFollowUpGates()) {
    lines.push(`  approval_gate ${linoString(gate)}`);
  }
  lines.push("```", "", softwareFollowUpText("heading_reasoning", language));
  followUpReasoningSteps(meaning, followUp, language).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("", softwareFollowUpText("heading_plan", language));
  followUpPlanSteps(meaning, followUp, language).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("", softwareFollowUpText(approved ? "approved" : "proposed", language));
  return lines.join("\n");
}

// Follow-up handler for an active software-project dialogue (issue #341). Runs
// before `tryConceptLookup` so a decomposed step like "test it by scraping
// wikipedia.org and show me the top 10 most frequent words" stays bound to the
// project instead of resolving the `wikipedia` concept or falling to the
// unknown opener. Mirrors `try_software_project_followup` in the Rust solver.
function trySoftwareProjectFollowup(prompt, history = []) {
  const normalized = normalizePrompt(prompt);
  // Approval prompts stay with the main request handler, which advances to the
  // implementation starter.
  if (isSoftwareApprovalPrompt(normalized)) return null;
  const dialogue = priorSoftwareProjectDialogue(history);
  if (!dialogue) return null;
  const followUp = detectSoftwareFollowUp(prompt, normalized);
  if (!followUp) return null;
  return {
    intent: "software_project_followup",
    content: renderSoftwareProjectFollowUp(dialogue.meaning, followUp, dialogue.approved, detectLanguage(prompt)),
    confidence: 0.74,
    evidence: followUpEvidence(dialogue.meaning, followUp, dialogue.approved),
  };
}

function tryJavaScriptExecution(prompt) {
  const program = extractJavaScriptProgram(prompt);
  if (program === null) return null;
  const logs = [];
  const captureConsole = {
    log: (...args) =>
      logs.push(
        args
          .map((value) =>
            typeof value === "string" ? value : JSON.stringify(value),
          )
          .join(" "),
      ),
  };
  let result;
  let error = null;
  try {
    const runner = new Function(
      "console",
      `"use strict"; return (function(){ ${program}\n })();`,
    );
    result = runner(captureConsole);
  } catch (err) {
    error = err;
  }
  const lines = [];
  lines.push("Execution status: ran in the demo's Web Worker sandbox.");
  lines.push("Source:");
  lines.push("```javascript");
  lines.push(program);
  lines.push("```");
  if (error) {
    lines.push("");
    lines.push(`Error: ${error.message || String(error)}`);
  } else {
    if (logs.length > 0) {
      lines.push("");
      lines.push("Output:");
      lines.push("```text");
      lines.push(logs.join("\n"));
      lines.push("```");
    }
    if (result !== undefined) {
      lines.push("");
      lines.push(`Returned: \`${String(result)}\``);
    }
    if (logs.length === 0 && result === undefined) {
      lines.push("");
      lines.push("Program completed without output or return value.");
    }
  }
  lines.push("");
  lines.push(
    "Note: the browser worker has no DOM or network access, so side effects are limited.",
  );
  return {
    intent: error ? "javascript_execution_error" : "javascript_execution",
    content: lines.join("\n"),
    confidence: error ? 0.5 : 0.95,
    evidence: [
      `execution_status:javascript:${error ? "error" : "ran"}`,
      "language:javascript",
    ],
  };
}

// `saveAs`, `setupHint`, `runCommand` and `checkCommand` mirror the same fields
// on `coding::catalog::ProgramLanguage` so the demo's novice "How to test it
// yourself" steps match the Rust engine exactly (issue #330). No entry carries
// its alias surfaces inline: the words a prompt must contain to resolve a
// language live in the `program_language_<slug>` meaning (role
// `program_language_alias`) and `programLanguageFromPrompt` reads them by slug
// (issue #386), matching the Rust catalog byte-for-byte through the shared seed.
const WRITE_PROGRAM_LANGUAGES = {
  rust: {
    name: "Rust",
    fence: "rust",
    saveAs: "main.rs",
    setupHint: "the Rust toolchain from https://rustup.rs",
    checkCommand: "rustc main.rs -o main",
    runCommand: "./main",
  },
  python: {
    name: "Python",
    fence: "python",
    saveAs: "main.py",
    setupHint: "Python 3 from https://www.python.org/downloads/",
    checkCommand: "python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile main.py",
    runCommand: "python3 main.py",
  },
  javascript: {
    name: "JavaScript",
    fence: "javascript",
    saveAs: "main.js",
    setupHint: "Node.js from https://nodejs.org/",
    checkCommand: "node --check main.js",
    runCommand: "node main.js",
  },
  typescript: {
    name: "TypeScript",
    fence: "typescript",
    saveAs: "hello.ts",
    setupHint:
      "Node.js from https://nodejs.org/ plus TypeScript via `npm install -g typescript`",
    checkCommand: "tsc hello.ts",
    runCommand: "node hello.js",
  },
  go: {
    name: "Go",
    fence: "go",
    saveAs: "main.go",
    setupHint: "Go from https://go.dev/dl/",
    checkCommand: null,
    runCommand: "go run main.go",
  },
  c: {
    name: "C",
    fence: "c",
    saveAs: "main.c",
    setupHint:
      "a C compiler such as GCC from https://gcc.gnu.org/ or your package manager",
    checkCommand: "gcc main.c -o main",
    runCommand: "./main",
  },
  cpp: {
    name: "C++",
    fence: "cpp",
    saveAs: "main.cpp",
    setupHint:
      "a C++ compiler such as g++ from https://gcc.gnu.org/ or your package manager",
    checkCommand: "g++ main.cpp -o main",
    runCommand: "./main",
  },
  java: {
    name: "Java",
    fence: "java",
    saveAs: "Main.java",
    setupHint: "a JDK from https://adoptium.net/",
    checkCommand: "javac Main.java",
    runCommand: "java Main",
  },
  csharp: {
    name: "C#",
    fence: "csharp",
    saveAs: "Program.cs",
    setupHint: "the .NET SDK from https://dotnet.microsoft.com/download",
    checkCommand: "dotnet build",
    runCommand: "dotnet run",
  },
  ruby: {
    name: "Ruby",
    fence: "ruby",
    saveAs: "main.rb",
    setupHint: "Ruby from https://www.ruby-lang.org/en/downloads/",
    checkCommand: "ruby -c main.rb",
    runCommand: "ruby main.rb",
  },
  scala: {
    name: "Scala",
    fence: "scala",
    saveAs: "Main.scala",
    setupHint:
      "Scala from https://www.scala-lang.org/download/ (a JDK is required as well)",
    checkCommand: "scalac Main.scala",
    runCommand: "scala Main",
  },
  kotlin: {
    name: "Kotlin",
    fence: "kotlin",
    saveAs: "Main.kt",
    setupHint:
      "the Kotlin compiler from https://kotlinlang.org/docs/command-line.html (a JDK is required as well)",
    checkCommand: "kotlinc Main.kt -include-runtime -d Main.jar",
    runCommand: "java -jar Main.jar",
  },
  php: {
    name: "PHP",
    fence: "php",
    saveAs: "main.php",
    setupHint: "PHP from https://www.php.net/downloads",
    checkCommand: "php -l main.php",
    runCommand: "php main.php",
  },
  // `frameworkOf` mirrors `ProgramLanguage::framework_of`: a catalog row is an
  // implementation target, and a target may be a framework of another target
  // (issue #723). Only the fields the request actually asked for are the
  // framework's own; the grammar and idioms belong to `frameworkOf`.
  laravel: {
    name: "Laravel",
    fence: "php",
    frameworkOf: "php",
    saveAs: "app/Console/Commands/HelloWorld.php",
    setupHint:
      "a Laravel application from https://laravel.com/docs/installation (`composer create-project laravel/laravel my-app`, which brings PHP and Composer with it)",
    checkCommand: "php -l app/Console/Commands/HelloWorld.php",
    runCommand: "php artisan hello:world",
  },
};

// The catalog's tasks and their programs (rust/src/coding/catalog/tasks.rs
// and templates_*.rs) are seed data, not worker tables: installSeedProgramTasks
// (formal_ai_worker_program_requests.js) fills both from
// data/seed/hello-world-programs.lino, the bundle the Rust catalog tests hold
// equal to the compiled tables (lino_parity.rs), in the Rust table order.
// A pair the seed retires to the documentation route stores no program here;
// writeProgramTemplate rediscovers it (issue #1165 R1165-4).
const WRITE_PROGRAM_TASKS = {};

const WRITE_PROGRAM_TEMPLATES = {};

// Issue #412 (R6/R8): the coding oracle treats public knowledge bases — Rosetta
// Code, Wikifunctions, the Hello World Collection, Stack Overflow — as cached
// external APIs even when they expose no machine API, and generalises the
// verified catalog above to languages it does not template (Kotlin, Swift, PHP,
// Bash, Lua, Haskell, …). This data, the lookup, and the answer renderer mirror
// `src/knowledge.rs` + `src/solver_handler_oracle.rs` byte-for-byte so the WASM
// worker and the native binary agree on every reasoning surface.
const KNOWLEDGE_SOURCES = {
  "rosetta-code": { displayName: "Rosetta Code", baseUrl: "https://rosettacode.org" },
  wikifunctions: { displayName: "Wikifunctions", baseUrl: "https://www.wikifunctions.org" },
  "hello-world-collection": {
    displayName: "Hello World Collection",
    baseUrl: "http://helloworldcollection.de",
  },
  "stack-overflow": { displayName: "Stack Overflow", baseUrl: "https://stackoverflow.com" },
  // Issue #1165: a page captured under data/seed/coding-documentation-captures.lino.
  "documentation-capture": { displayName: "Documentation capture", baseUrl: "" },
};

// The committed popular-case cache (mirrors ORACLE_SNAPSHOTS in src/knowledge.rs).
// Intentionally tiny — well under the cache cap for every source (R8) — and is
// the offline accelerator a gated live refresh would repopulate.
const CODING_ORACLE_SNAPSHOTS = [
  {
    taskSlug: "hello_world",
    languageSlug: "bash",
    languageLabel: "Bash",
    source: "hello-world-collection",
    sourceUrl: "http://helloworldcollection.de/#Bash",
    code: 'echo "Hello, World!"',
    expectedOutput: "Hello, World!",
  },
  {
    taskSlug: "hello_world",
    languageSlug: "haskell",
    languageLabel: "Haskell",
    source: "hello-world-collection",
    sourceUrl: "http://helloworldcollection.de/#Haskell",
    code: 'main :: IO ()\nmain = putStrLn "Hello, World!"',
    expectedOutput: "Hello, World!",
  },
  {
    taskSlug: "factorial",
    languageSlug: "kotlin",
    languageLabel: "Kotlin",
    source: "rosetta-code",
    sourceUrl: "https://rosettacode.org/wiki/Factorial#Kotlin",
    code: "fun factorial(n: Int): Long =\n    if (n <= 1) 1L else n * factorial(n - 1)\n\nfun main() {\n    println(factorial(5))\n}",
    expectedOutput: "120",
  },
];

// Resolve a (task, language) request to a cached snippet, matching the language
// by slug or case-insensitive display label (mirrors CodingOracle::lookup).
function codingOracleLookup(taskSlug, language) {
  if (!taskSlug || !language) return null;
  const needle = String(language).trim().toLowerCase();
  if (!needle) return null;
  return (
    CODING_ORACLE_SNAPSHOTS.find(
      (snippet) =>
        snippet.taskSlug === taskSlug &&
        (snippet.languageSlug === needle ||
          snippet.languageLabel.toLowerCase() === needle),
    ) || null
  );
}

function codingOracleKnowsLanguage(language) {
  const needle = String(language || "").trim().toLowerCase();
  if (!needle) return false;
  if (typeof documentationKnowsLanguage === "function" && documentationKnowsLanguage(needle)) return true;
  return CODING_ORACLE_SNAPSHOTS.some(
    (snippet) =>
      snippet.languageSlug === needle ||
      snippet.languageLabel.toLowerCase() === needle,
  );
}

// Render an otherwise-unsupported write_program request from the coding oracle's
// cached external snippets (mirrors try_write_program_from_oracle in
// src/solver_handler_oracle.rs, byte-for-byte on the content and evidence).
