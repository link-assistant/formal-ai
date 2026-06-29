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
    "algorithm_construction:meta_algorithm:problem_class_to_shared_ir_to_renderers_to_verification",
    `installation_conversion:source_format:${conversion.sourceFormat}`,
    `installation_conversion:project:${conversion.project}`,
  ];
  for (const stage of INSTALL_ALGORITHM_CONSTRUCTION_STAGES) {
    evidence.push(`algorithm_construction:stage:${stage.id}:output=${stage.output}:verifier=${stage.verifier}`);
  }
  for (const surface of INSTALL_CODING_SURFACE_PROJECTIONS) {
    evidence.push(`algorithm_construction:coding_surface:${surface.slug}:projection=${surface.projection}`);
  }
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
  lines.push(
    `  meta_algorithm ${linoString("problem_class_to_shared_ir_to_renderers_to_verification")}`,
  );
  for (const stage of INSTALL_ALGORITHM_CONSTRUCTION_STAGES) {
    lines.push(`  construction_stage ${linoString(stage.id)}`);
    lines.push(`  stage_output ${linoString(stage.output)}`);
    lines.push(`  stage_verifier ${linoString(stage.verifier)}`);
  }
  for (const surface of INSTALL_CODING_SURFACE_PROJECTIONS) {
    lines.push(`  coding_surface ${linoString(surface.slug)}`);
    lines.push(`  surface_projection ${linoString(surface.projection)}`);
  }
  for (const step of conversion.steps) {
    lines.push(`  step ${linoString(step.id)}`);
    lines.push(`  description ${linoString(step.description)}`);
    lines.push(`  command ${linoString(step.command)}`);
  }
  return lines.join("\n") + "\n";
}

function renderInstallationMetaAlgorithm() {
  const lines = ["Meta algorithm for constructing conversion algorithms:"];
  INSTALL_ALGORITHM_CONSTRUCTION_STAGES.forEach((stage, index) => {
    lines.push(
      `${index + 1}. ${stage.id} -> ${stage.output}; verification fixture: ${stage.verifier}.`,
    );
  });
  lines.push("");
  lines.push("Existing coding solutions producible by the same meta algorithm:");
  for (const surface of INSTALL_CODING_SURFACE_PROJECTIONS) {
    lines.push(`- ${surface.slug}: ${surface.projection}.`);
  }
  return lines.join("\n");
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

// Maps a follow-up kind to the imperative verb used when rendering it. Mirrors
// `FollowUpKind::action` in src/solver_handlers/software_project_followup.rs.
// The surface words that *recognise* each kind no longer live here — they are
// self-describing meanings in data/seed/meanings-software-project.lino, queried
// by detectSoftwareFollowUp via the software_followup_* roles (issue #386).
const SOFTWARE_FOLLOW_UP_ACTIONS = {
  verification: "test",
  execution: "run",
  demonstration: "show",
};

const SOFTWARE_FOLLOW_UP_GATES = ["generated_code", "test_execution", "network_access"];

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

// Capture the clause after "show me"/"show"/"print"/"display" (capped at 12
// words) so the follow-up records what the user wants surfaced.
function extractFollowUpExpectedOutput(prompt) {
  const source = String(prompt || "");
  const lower = source.toLowerCase();
  for (const marker of ["show me ", "show ", "print ", "display "]) {
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
      .slice(0, 12)
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
  for (const [role, candidate] of [
    [ROLE_SOFTWARE_FOLLOWUP_VERIFICATION, "verification"],
    [ROLE_SOFTWARE_FOLLOWUP_EXECUTION, "execution"],
    [ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION, "demonstration"],
  ]) {
    if (lexiconMentionsRoleSubstring(role, normalized)) {
      kind = candidate;
      break;
    }
  }
  if (!kind) return null;
  return {
    kind,
    action: SOFTWARE_FOLLOW_UP_ACTIONS[kind],
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

function followUpReasoningSteps(meaning, followUp) {
  const steps = [
    `Recognize "${followUp.action}" as a ${followUp.kind} request that exercises the ${meaning.artifact} from the active plan, not a fact lookup.`,
  ];
  if (followUp.targetSite) {
    steps.push(
      `Bind the test target to ${followUp.targetSite} and keep live fetches behind the network_access gate.`,
    );
  }
  if (followUp.expectedOutput) {
    steps.push(
      `Record the expected output as "${followUp.expectedOutput}" so the test harness can assert it.`,
    );
  }
  steps.push(
    "Drive the artifact through a deterministic fixture before any host API or network call.",
  );
  steps.push(
    "Keep code execution behind approval gates because the sandbox cannot run untrusted code.",
  );
  return steps;
}

function followUpPlanSteps(meaning, followUp) {
  const site = followUp.targetSite || "the requested target";
  const steps = [
    `Generate the ${meaning.artifact} core plus a deterministic test harness with a captured ${site} fixture.`,
    "Assert each requirement (parsing, extraction, counting, summary) against the fixture.",
  ];
  if (followUp.expectedOutput) {
    steps.push(`Surface ${followUp.expectedOutput} from the fixture run.`);
  }
  steps.push(
    `Run the ${meaning.implementationLanguage} test command once the generated_code gate is approved.`,
  );
  steps.push(
    `Promote the run to live ${site} only after the test_execution and network_access gates pass.`,
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
  for (const gate of SOFTWARE_FOLLOW_UP_GATES) {
    evidence.push(`approval_gate:${gate}`);
  }
  return evidence;
}

function renderSoftwareProjectFollowUp(meaning, followUp, approved) {
  const lines = [];
  lines.push(
    `Recorded a ${followUp.kind} follow-up for the ${meaning.artifact} from the active plan.`,
  );
  lines.push("");
  lines.push("Formalized meaning:");
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
  for (const gate of SOFTWARE_FOLLOW_UP_GATES) {
    lines.push(`  approval_gate ${linoString(gate)}`);
  }
  lines.push("```");
  lines.push("");
  lines.push("Reasoning steps:");
  followUpReasoningSteps(meaning, followUp).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  lines.push("Verification plan:");
  followUpPlanSteps(meaning, followUp).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  if (approved) {
    lines.push(
      "The plan is approved, so the generated starter already includes this test harness. " +
        "Running it live needs the test_execution and network_access gates.",
    );
  } else {
    lines.push(
      "Reply `approve plan` to generate the artifact plus this test harness. Running it live " +
        "against the target needs the test_execution and network_access gates.",
    );
  }
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
    content: renderSoftwareProjectFollowUp(dialogue.meaning, followUp, dialogue.approved),
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
    checkCommand: "python3 -m py_compile main.py",
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
};

const WRITE_PROGRAM_TASKS = {
  hello_world: {
    label: "hello world",
    output: "Hello, world!",
  },
  count_to_three: {
    label: "count to three",
    output: "1\n2\n3",
  },
  list_files: {
    label: "list files in the current directory",
    // Fallback output for the Rust-flavoured sample. Rendered answers resolve
    // list-file samples from each language's `saveAs` name (issue #440).
    output: "Cargo.toml\nREADME.md\nmain.rs",
  },
  list_files_arg: {
    label: "list files in the directory given as a path argument",
    // Issue #324 follow-up: "Сделай так, чтобы программа принимала путь как
    // аргумент" (make the program accept a path as an argument). This is the
    // path-argument variant of `list_files`; conversation context maps a bare
    // "accept a path argument" modification onto it through the program-plan
    // rules. Mirrors the Rust `list_files_arg` task.
    output: "Cargo.toml\nREADME.md\nmain.rs",
  },
  list_files_reverse_sort: {
    label: "list files in the current directory in reverse-sorted order",
    output: "main.rs\nREADME.md\nCargo.toml",
  },
  list_files_arg_reverse_sort: {
    label: "list files from a path argument in reverse-sorted order",
    output: "main.rs\nREADME.md\nCargo.toml",
  },
  // Issue #330: classic branching/looping exercise over 1..=15. Mirrors the Rust
  // `fizzbuzz` task; fixed range so the output is deterministic and verifiable.
  fizzbuzz: {
    label: "FizzBuzz",
    output: "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz",
  },
  // Issue #330: fixed to 5! = 120 so the verified output is unambiguous (the
  // aliases require the number 5). Mirrors the Rust `factorial` task.
  factorial: {
    label: "factorial of 5",
    output: "120",
  },
  // Issue #330: reverses the literal string "hello" -> "olleh". Mirrors the Rust
  // `reverse_string` task; fixed input keeps the output verifiable.
  reverse_string: {
    label: "string reversal",
    output: "olleh",
  },
  // Issue #330: sums 1..=10 -> 55. Mirrors the Rust `sum_to_ten` task.
  sum_to_ten: {
    label: "sum from 1 to 10",
    output: "55",
  },
  // Issue #334: a recursive `fibonacci` function evaluated at the 10th term
  // (F(1)=F(2)=1 -> F(10)=55). Mirrors the Rust `fibonacci` task; fixed index so
  // the output is verifiable.
  fibonacci: {
    label: "recursive Fibonacci",
    output: "55",
  },
};

const WRITE_PROGRAM_TEMPLATES = {
  hello_world: {
    rust: 'fn main() {\n    println!("Hello, world!");\n}',
    python: 'print("Hello, world!")',
    javascript: 'console.log("Hello, world!");',
    typescript: 'console.log("Hello, world!");',
    go: 'package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello, world!")\n}',
    c: '#include <stdio.h>\n\nint main(void) {\n    puts("Hello, world!");\n    return 0;\n}',
    cpp: '#include <iostream>\n\nint main() {\n    std::cout << "Hello, world!" << std::endl;\n    return 0;\n}',
    java: 'public class Main {\n    public static void main(String[] args) {\n        System.out.println("Hello, world!");\n    }\n}',
    csharp:
      'using System;\n\nclass Program {\n    static void Main() {\n        Console.WriteLine("Hello, world!");\n    }\n}',
    ruby: 'puts "Hello, world!"',
  },
  count_to_three: {
    rust:
      'fn main() {\n    for number in 1..=3 {\n        println!("{number}");\n    }\n}',
    python: "for number in range(1, 4):\n    print(number)",
    javascript:
      "for (let number = 1; number <= 3; number += 1) {\n    console.log(number);\n}",
    typescript:
      "for (let number = 1; number <= 3; number += 1) {\n    console.log(number);\n}",
    go:
      'package main\n\nimport "fmt"\n\nfunc main() {\n    for number := 1; number <= 3; number++ {\n        fmt.Println(number)\n    }\n}',
    c:
      '#include <stdio.h>\n\nint main(void) {\n    for (int number = 1; number <= 3; number++) {\n        printf("%d\\n", number);\n    }\n    return 0;\n}',
  },
  list_files: {
    rust:
      'use std::fs;\n\nfn main() -> std::io::Result<()> {\n    let mut names: Vec<String> = fs::read_dir(".")?\n        .filter_map(Result::ok)\n        .filter(|entry| entry.path().is_file())\n        .map(|entry| entry.file_name().to_string_lossy().into_owned())\n        .collect();\n    names.sort();\n    for name in names {\n        println!("{name}");\n    }\n    Ok(())\n}',
    python:
      'import os\n\nnames = sorted(name for name in os.listdir(".") if os.path.isfile(name))\nfor name in names:\n    print(name)',
    javascript:
      'const fs = require("fs");\n\nconst names = fs\n  .readdirSync(".")\n  .filter((name) => fs.statSync(name).isFile())\n  .sort();\n\nfor (const name of names) {\n  console.log(name);\n}',
    typescript:
      'import * as fs from "fs";\n\nconst names: string[] = fs\n  .readdirSync(".")\n  .filter((name) => fs.statSync(name).isFile())\n  .sort();\n\nfor (const name of names) {\n  console.log(name);\n}',
    go:
      'package main\n\nimport (\n    "fmt"\n    "os"\n    "sort"\n)\n\nfunc main() {\n    entries, err := os.ReadDir(".")\n    if err != nil {\n        panic(err)\n    }\n    var names []string\n    for _, entry := range entries {\n        if !entry.IsDir() {\n            names = append(names, entry.Name())\n        }\n    }\n    sort.Strings(names)\n    for _, name := range names {\n        fmt.Println(name)\n    }\n}',
    c:
      '#include <dirent.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <sys/stat.h>\n\nstatic int compare(const void *a, const void *b) {\n    return strcmp(*(const char *const *)a, *(const char *const *)b);\n}\n\nint main(void) {\n    DIR *dir = opendir(".");\n    if (dir == NULL) {\n        return 1;\n    }\n    char *names[1024];\n    size_t count = 0;\n    struct dirent *entry;\n    while ((entry = readdir(dir)) != NULL && count < 1024) {\n        struct stat info;\n        if (stat(entry->d_name, &info) == 0 && S_ISREG(info.st_mode)) {\n            names[count++] = strdup(entry->d_name);\n        }\n    }\n    closedir(dir);\n    qsort(names, count, sizeof(char *), compare);\n    for (size_t i = 0; i < count; i++) {\n        printf("%s\\n", names[i]);\n        free(names[i]);\n    }\n    return 0;\n}',
    cpp:
      "#include <algorithm>\n#include <filesystem>\n#include <iostream>\n#include <string>\n#include <vector>\n\nint main() {\n    namespace fs = std::filesystem;\n    std::vector<std::string> names;\n    for (const auto &entry : fs::directory_iterator(\".\")) {\n        if (entry.is_regular_file()) {\n            names.push_back(entry.path().filename().string());\n        }\n    }\n    std::sort(names.begin(), names.end());\n    for (const auto &name : names) {\n        std::cout << name << '\\n';\n    }\n}",
    java:
      'import java.io.File;\nimport java.util.Arrays;\n\npublic class Main {\n    public static void main(String[] args) {\n        File[] entries = new File(".").listFiles();\n        if (entries == null) {\n            return;\n        }\n        String[] names = Arrays.stream(entries)\n            .filter(File::isFile)\n            .map(File::getName)\n            .sorted()\n            .toArray(String[]::new);\n        for (String name : names) {\n            System.out.println(name);\n        }\n    }\n}',
    csharp:
      'using System;\nusing System.IO;\nusing System.Linq;\n\nclass Program {\n    static void Main() {\n        var names = Directory.GetFiles(".")\n            .Select(Path.GetFileName)\n            .OrderBy(name => name, StringComparer.Ordinal);\n        foreach (var name in names) {\n            Console.WriteLine(name);\n        }\n    }\n}',
    ruby:
      'names = Dir.entries(".").select { |name| File.file?(name) }.sort\nnames.each { |name| puts name }',
  },
  // Issue #324 follow-up: list files in the directory passed as the first
  // command-line argument, defaulting to "." when none is supplied. Mirrors the
  // Rust `list_files_arg` templates.
  list_files_arg: {
    rust:
      'use std::env;\nuse std::fs;\n\nfn main() {\n    let path = env::args().nth(1).unwrap_or_else(|| String::from("."));\n    let mut names: Vec<String> = fs::read_dir(&path)\n        .expect("failed to read directory")\n        .filter_map(|entry| entry.ok())\n        .filter(|entry| entry.path().is_file())\n        .map(|entry| entry.file_name().to_string_lossy().into_owned())\n        .collect();\n    names.sort();\n    for name in names {\n        println!("{name}");\n    }\n}',
    python:
      'import os\nimport sys\n\npath = sys.argv[1] if len(sys.argv) > 1 else "."\nnames = sorted(\n    name for name in os.listdir(path) if os.path.isfile(os.path.join(path, name))\n)\nfor name in names:\n    print(name)',
    javascript:
      'const fs = require("fs");\nconst path = require("path");\n\nconst dir = process.argv[2] || ".";\nconst names = fs\n  .readdirSync(dir)\n  .filter((name) => fs.statSync(path.join(dir, name)).isFile())\n  .sort();\n\nfor (const name of names) {\n  console.log(name);\n}',
    typescript:
      'import * as fs from "fs";\nimport * as path from "path";\n\nconst dir: string = process.argv[2] ?? ".";\nconst names: string[] = fs\n  .readdirSync(dir)\n  .filter((name) => fs.statSync(path.join(dir, name)).isFile())\n  .sort();\n\nfor (const name of names) {\n  console.log(name);\n}',
    go:
      'package main\n\nimport (\n    "fmt"\n    "os"\n    "sort"\n)\n\nfunc main() {\n    dir := "."\n    if len(os.Args) > 1 {\n        dir = os.Args[1]\n    }\n    entries, err := os.ReadDir(dir)\n    if err != nil {\n        panic(err)\n    }\n    var names []string\n    for _, entry := range entries {\n        if !entry.IsDir() {\n            names = append(names, entry.Name())\n        }\n    }\n    sort.Strings(names)\n    for _, name := range names {\n        fmt.Println(name)\n    }\n}',
    c:
      '#include <dirent.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <sys/stat.h>\n\nstatic int compare(const void *a, const void *b) {\n    return strcmp(*(const char *const *)a, *(const char *const *)b);\n}\n\nint main(int argc, char *argv[]) {\n    const char *path = argc > 1 ? argv[1] : ".";\n    DIR *dir = opendir(path);\n    if (dir == NULL) {\n        return 1;\n    }\n    char *names[1024];\n    size_t count = 0;\n    struct dirent *entry;\n    while ((entry = readdir(dir)) != NULL && count < 1024) {\n        char full[4096];\n        snprintf(full, sizeof(full), "%s/%s", path, entry->d_name);\n        struct stat info;\n        if (stat(full, &info) == 0 && S_ISREG(info.st_mode)) {\n            names[count++] = strdup(entry->d_name);\n        }\n    }\n    closedir(dir);\n    qsort(names, count, sizeof(char *), compare);\n    for (size_t i = 0; i < count; i++) {\n        printf("%s\\n", names[i]);\n        free(names[i]);\n    }\n    return 0;\n}',
    cpp:
      "#include <algorithm>\n#include <filesystem>\n#include <iostream>\n#include <string>\n#include <vector>\n\nint main(int argc, char *argv[]) {\n    namespace fs = std::filesystem;\n    std::string path = argc > 1 ? argv[1] : \".\";\n    std::vector<std::string> names;\n    for (const auto &entry : fs::directory_iterator(path)) {\n        if (entry.is_regular_file()) {\n            names.push_back(entry.path().filename().string());\n        }\n    }\n    std::sort(names.begin(), names.end());\n    for (const auto &name : names) {\n        std::cout << name << '\\n';\n    }\n}",
    java:
      'import java.io.File;\nimport java.util.Arrays;\n\npublic class Main {\n    public static void main(String[] args) {\n        String path = args.length > 0 ? args[0] : ".";\n        File[] entries = new File(path).listFiles();\n        if (entries == null) {\n            return;\n        }\n        String[] names = Arrays.stream(entries)\n            .filter(File::isFile)\n            .map(File::getName)\n            .sorted()\n            .toArray(String[]::new);\n        for (String name : names) {\n            System.out.println(name);\n        }\n    }\n}',
    csharp:
      'using System;\nusing System.IO;\nusing System.Linq;\n\nclass Program {\n    static void Main(string[] args) {\n        var path = args.Length > 0 ? args[0] : ".";\n        var names = Directory.GetFiles(path)\n            .Select(Path.GetFileName)\n            .OrderBy(name => name, StringComparer.Ordinal);\n        foreach (var name in names) {\n            Console.WriteLine(name);\n        }\n    }\n}',
    ruby:
      'path = ARGV[0] || "."\nnames = Dir.entries(path).select { |name| File.file?(File.join(path, name)) }.sort\nnames.each { |name| puts name }',
  },
  list_files_reverse_sort: {
    rust:
      'use std::fs;\n\nfn main() -> std::io::Result<()> {\n    let mut names: Vec<String> = fs::read_dir(".")?\n        .filter_map(Result::ok)\n        .filter(|entry| entry.path().is_file())\n        .map(|entry| entry.file_name().to_string_lossy().into_owned())\n        .collect();\n    names.sort_by(|a, b| b.cmp(a));\n    for name in names {\n        println!("{name}");\n    }\n    Ok(())\n}',
    python:
      'import os\n\nnames = sorted(\n    (name for name in os.listdir(".") if os.path.isfile(name)),\n    reverse=True,\n)\nfor name in names:\n    print(name)',
    javascript:
      'const fs = require("fs");\n\nconst names = fs\n  .readdirSync(".")\n  .filter((name) => fs.statSync(name).isFile())\n  .sort()\n  .reverse();\n\nfor (const name of names) {\n  console.log(name);\n}',
    typescript:
      'import * as fs from "fs";\n\nconst names: string[] = fs\n  .readdirSync(".")\n  .filter((name) => fs.statSync(name).isFile())\n  .sort()\n  .reverse();\n\nfor (const name of names) {\n  console.log(name);\n}',
    go:
      'package main\n\nimport (\n    "fmt"\n    "os"\n    "sort"\n)\n\nfunc main() {\n    entries, err := os.ReadDir(".")\n    if err != nil {\n        panic(err)\n    }\n    var names []string\n    for _, entry := range entries {\n        if !entry.IsDir() {\n            names = append(names, entry.Name())\n        }\n    }\n    sort.Sort(sort.Reverse(sort.StringSlice(names)))\n    for _, name := range names {\n        fmt.Println(name)\n    }\n}',
    c:
      '#include <dirent.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <sys/stat.h>\n\nstatic int compare_desc(const void *a, const void *b) {\n    return strcmp(*(const char *const *)b, *(const char *const *)a);\n}\n\nint main(void) {\n    DIR *dir = opendir(".");\n    if (dir == NULL) {\n        return 1;\n    }\n    char *names[1024];\n    size_t count = 0;\n    struct dirent *entry;\n    while ((entry = readdir(dir)) != NULL && count < 1024) {\n        struct stat info;\n        if (stat(entry->d_name, &info) == 0 && S_ISREG(info.st_mode)) {\n            names[count++] = strdup(entry->d_name);\n        }\n    }\n    closedir(dir);\n    qsort(names, count, sizeof(char *), compare_desc);\n    for (size_t i = 0; i < count; i++) {\n        printf("%s\\n", names[i]);\n        free(names[i]);\n    }\n    return 0;\n}',
    cpp:
      "#include <algorithm>\n#include <filesystem>\n#include <iostream>\n#include <string>\n#include <vector>\n\nint main() {\n    namespace fs = std::filesystem;\n    std::vector<std::string> names;\n    for (const auto &entry : fs::directory_iterator(\".\")) {\n        if (entry.is_regular_file()) {\n            names.push_back(entry.path().filename().string());\n        }\n    }\n    std::sort(names.rbegin(), names.rend());\n    for (const auto &name : names) {\n        std::cout << name << '\\n';\n    }\n}",
    java:
      'import java.io.File;\nimport java.util.Arrays;\nimport java.util.Comparator;\n\npublic class Main {\n    public static void main(String[] args) {\n        File[] entries = new File(".").listFiles();\n        if (entries == null) {\n            return;\n        }\n        String[] names = Arrays.stream(entries)\n            .filter(File::isFile)\n            .map(File::getName)\n            .sorted(Comparator.reverseOrder())\n            .toArray(String[]::new);\n        for (String name : names) {\n            System.out.println(name);\n        }\n    }\n}',
    csharp:
      'using System;\nusing System.IO;\nusing System.Linq;\n\nclass Program {\n    static void Main() {\n        var names = Directory.GetFiles(".")\n            .Select(Path.GetFileName)\n            .OrderByDescending(name => name, StringComparer.Ordinal);\n        foreach (var name in names) {\n            Console.WriteLine(name);\n        }\n    }\n}',
    ruby:
      'names = Dir.entries(".").select { |name| File.file?(name) }.sort.reverse\nnames.each { |name| puts name }',
  },
  list_files_arg_reverse_sort: {
    rust:
      'use std::env;\nuse std::fs;\n\nfn main() {\n    let path = env::args().nth(1).unwrap_or_else(|| String::from("."));\n    let mut names: Vec<String> = fs::read_dir(&path)\n        .expect("failed to read directory")\n        .filter_map(|entry| entry.ok())\n        .filter(|entry| entry.path().is_file())\n        .map(|entry| entry.file_name().to_string_lossy().into_owned())\n        .collect();\n    names.sort_by(|a, b| b.cmp(a));\n    for name in names {\n        println!("{name}");\n    }\n}',
    python:
      'import os\nimport sys\n\npath = sys.argv[1] if len(sys.argv) > 1 else "."\nnames = sorted(\n    (\n        name\n        for name in os.listdir(path)\n        if os.path.isfile(os.path.join(path, name))\n    ),\n    reverse=True,\n)\nfor name in names:\n    print(name)',
    javascript:
      'const fs = require("fs");\nconst path = require("path");\n\nconst dir = process.argv[2] || ".";\nconst names = fs\n  .readdirSync(dir)\n  .filter((name) => fs.statSync(path.join(dir, name)).isFile())\n  .sort()\n  .reverse();\n\nfor (const name of names) {\n  console.log(name);\n}',
    typescript:
      'import * as fs from "fs";\nimport * as path from "path";\n\nconst dir: string = process.argv[2] ?? ".";\nconst names: string[] = fs\n  .readdirSync(dir)\n  .filter((name) => fs.statSync(path.join(dir, name)).isFile())\n  .sort()\n  .reverse();\n\nfor (const name of names) {\n  console.log(name);\n}',
    go:
      'package main\n\nimport (\n    "fmt"\n    "os"\n    "sort"\n)\n\nfunc main() {\n    dir := "."\n    if len(os.Args) > 1 {\n        dir = os.Args[1]\n    }\n    entries, err := os.ReadDir(dir)\n    if err != nil {\n        panic(err)\n    }\n    var names []string\n    for _, entry := range entries {\n        if !entry.IsDir() {\n            names = append(names, entry.Name())\n        }\n    }\n    sort.Sort(sort.Reverse(sort.StringSlice(names)))\n    for _, name := range names {\n        fmt.Println(name)\n    }\n}',
    c:
      '#include <dirent.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <sys/stat.h>\n\nstatic int compare_desc(const void *a, const void *b) {\n    return strcmp(*(const char *const *)b, *(const char *const *)a);\n}\n\nint main(int argc, char *argv[]) {\n    const char *path = argc > 1 ? argv[1] : ".";\n    DIR *dir = opendir(path);\n    if (dir == NULL) {\n        return 1;\n    }\n    char *names[1024];\n    size_t count = 0;\n    struct dirent *entry;\n    while ((entry = readdir(dir)) != NULL && count < 1024) {\n        char full[4096];\n        snprintf(full, sizeof(full), "%s/%s", path, entry->d_name);\n        struct stat info;\n        if (stat(full, &info) == 0 && S_ISREG(info.st_mode)) {\n            names[count++] = strdup(entry->d_name);\n        }\n    }\n    closedir(dir);\n    qsort(names, count, sizeof(char *), compare_desc);\n    for (size_t i = 0; i < count; i++) {\n        printf("%s\\n", names[i]);\n        free(names[i]);\n    }\n    return 0;\n}',
    cpp:
      "#include <algorithm>\n#include <filesystem>\n#include <iostream>\n#include <string>\n#include <vector>\n\nint main(int argc, char *argv[]) {\n    namespace fs = std::filesystem;\n    std::string path = argc > 1 ? argv[1] : \".\";\n    std::vector<std::string> names;\n    for (const auto &entry : fs::directory_iterator(path)) {\n        if (entry.is_regular_file()) {\n            names.push_back(entry.path().filename().string());\n        }\n    }\n    std::sort(names.rbegin(), names.rend());\n    for (const auto &name : names) {\n        std::cout << name << '\\n';\n    }\n}",
    java:
      'import java.io.File;\nimport java.util.Arrays;\nimport java.util.Comparator;\n\npublic class Main {\n    public static void main(String[] args) {\n        String path = args.length > 0 ? args[0] : ".";\n        File[] entries = new File(path).listFiles();\n        if (entries == null) {\n            return;\n        }\n        String[] names = Arrays.stream(entries)\n            .filter(File::isFile)\n            .map(File::getName)\n            .sorted(Comparator.reverseOrder())\n            .toArray(String[]::new);\n        for (String name : names) {\n            System.out.println(name);\n        }\n    }\n}',
    csharp:
      'using System;\nusing System.IO;\nusing System.Linq;\n\nclass Program {\n    static void Main(string[] args) {\n        var path = args.Length > 0 ? args[0] : ".";\n        var names = Directory.GetFiles(path)\n            .Select(Path.GetFileName)\n            .OrderByDescending(name => name, StringComparer.Ordinal);\n        foreach (var name in names) {\n            Console.WriteLine(name);\n        }\n    }\n}',
    ruby:
      'path = ARGV[0] || "."\nnames = Dir.entries(path).select { |name| File.file?(File.join(path, name)) }.sort.reverse\nnames.each { |name| puts name }',
  },
  fizzbuzz: {
    rust:
      "fn main() {\n    for number in 1..=15 {\n        if number % 15 == 0 {\n            println!(\"FizzBuzz\");\n        } else if number % 3 == 0 {\n            println!(\"Fizz\");\n        } else if number % 5 == 0 {\n            println!(\"Buzz\");\n        } else {\n            println!(\"{number}\");\n        }\n    }\n}",
    python:
      "for number in range(1, 16):\n    if number % 15 == 0:\n        print(\"FizzBuzz\")\n    elif number % 3 == 0:\n        print(\"Fizz\")\n    elif number % 5 == 0:\n        print(\"Buzz\")\n    else:\n        print(number)",
    javascript:
      "for (let number = 1; number <= 15; number += 1) {\n  if (number % 15 === 0) {\n    console.log(\"FizzBuzz\");\n  } else if (number % 3 === 0) {\n    console.log(\"Fizz\");\n  } else if (number % 5 === 0) {\n    console.log(\"Buzz\");\n  } else {\n    console.log(number);\n  }\n}",
    typescript:
      "for (let number = 1; number <= 15; number += 1) {\n  if (number % 15 === 0) {\n    console.log(\"FizzBuzz\");\n  } else if (number % 3 === 0) {\n    console.log(\"Fizz\");\n  } else if (number % 5 === 0) {\n    console.log(\"Buzz\");\n  } else {\n    console.log(number);\n  }\n}",
    go:
      "package main\n\nimport \"fmt\"\n\nfunc main() {\n    for number := 1; number <= 15; number++ {\n        switch {\n        case number%15 == 0:\n            fmt.Println(\"FizzBuzz\")\n        case number%3 == 0:\n            fmt.Println(\"Fizz\")\n        case number%5 == 0:\n            fmt.Println(\"Buzz\")\n        default:\n            fmt.Println(number)\n        }\n    }\n}",
    c:
      "#include <stdio.h>\n\nint main(void) {\n    for (int number = 1; number <= 15; number++) {\n        if (number % 15 == 0) {\n            puts(\"FizzBuzz\");\n        } else if (number % 3 == 0) {\n            puts(\"Fizz\");\n        } else if (number % 5 == 0) {\n            puts(\"Buzz\");\n        } else {\n            printf(\"%d\\n\", number);\n        }\n    }\n    return 0;\n}",
    cpp:
      "#include <iostream>\n\nint main() {\n    for (int number = 1; number <= 15; number++) {\n        if (number % 15 == 0) {\n            std::cout << \"FizzBuzz\\n\";\n        } else if (number % 3 == 0) {\n            std::cout << \"Fizz\\n\";\n        } else if (number % 5 == 0) {\n            std::cout << \"Buzz\\n\";\n        } else {\n            std::cout << number << '\\n';\n        }\n    }\n}",
    java:
      "public class Main {\n    public static void main(String[] args) {\n        for (int number = 1; number <= 15; number++) {\n            if (number % 15 == 0) {\n                System.out.println(\"FizzBuzz\");\n            } else if (number % 3 == 0) {\n                System.out.println(\"Fizz\");\n            } else if (number % 5 == 0) {\n                System.out.println(\"Buzz\");\n            } else {\n                System.out.println(number);\n            }\n        }\n    }\n}",
    csharp:
      "using System;\n\nclass Program {\n    static void Main() {\n        for (int number = 1; number <= 15; number++) {\n            if (number % 15 == 0) {\n                Console.WriteLine(\"FizzBuzz\");\n            } else if (number % 3 == 0) {\n                Console.WriteLine(\"Fizz\");\n            } else if (number % 5 == 0) {\n                Console.WriteLine(\"Buzz\");\n            } else {\n                Console.WriteLine(number);\n            }\n        }\n    }\n}",
    ruby:
      "(1..15).each do |number|\n  if (number % 15).zero?\n    puts \"FizzBuzz\"\n  elsif (number % 3).zero?\n    puts \"Fizz\"\n  elsif (number % 5).zero?\n    puts \"Buzz\"\n  else\n    puts number\n  end\nend",
  },
  factorial: {
    rust:
      "fn main() {\n    let mut result: u64 = 1;\n    for number in 1..=5 {\n        result *= number;\n    }\n    println!(\"{result}\");\n}",
    python:
      "result = 1\nfor number in range(1, 6):\n    result *= number\nprint(result)",
    javascript:
      "let result = 1;\nfor (let number = 1; number <= 5; number += 1) {\n  result *= number;\n}\nconsole.log(result);",
    typescript:
      "let result = 1;\nfor (let number = 1; number <= 5; number += 1) {\n  result *= number;\n}\nconsole.log(result);",
    go:
      "package main\n\nimport \"fmt\"\n\nfunc main() {\n    result := 1\n    for number := 1; number <= 5; number++ {\n        result *= number\n    }\n    fmt.Println(result)\n}",
    c:
      "#include <stdio.h>\n\nint main(void) {\n    unsigned long long result = 1;\n    for (int number = 1; number <= 5; number++) {\n        result *= number;\n    }\n    printf(\"%llu\\n\", result);\n    return 0;\n}",
    cpp:
      "#include <iostream>\n\nint main() {\n    unsigned long long result = 1;\n    for (int number = 1; number <= 5; number++) {\n        result *= number;\n    }\n    std::cout << result << '\\n';\n}",
    java:
      "public class Main {\n    public static void main(String[] args) {\n        long result = 1;\n        for (int number = 1; number <= 5; number++) {\n            result *= number;\n        }\n        System.out.println(result);\n    }\n}",
    csharp:
      "using System;\n\nclass Program {\n    static void Main() {\n        long result = 1;\n        for (int number = 1; number <= 5; number++) {\n            result *= number;\n        }\n        Console.WriteLine(result);\n    }\n}",
    ruby:
      "result = (1..5).reduce(1, :*)\nputs result",
  },
  reverse_string: {
    rust:
      "fn main() {\n    let text = \"hello\";\n    let reversed: String = text.chars().rev().collect();\n    println!(\"{reversed}\");\n}",
    python:
      "text = \"hello\"\nprint(text[::-1])",
    javascript:
      "const text = \"hello\";\nconsole.log(text.split(\"\").reverse().join(\"\"));",
    typescript:
      "const text: string = \"hello\";\nconsole.log(text.split(\"\").reverse().join(\"\"));",
    go:
      "package main\n\nimport \"fmt\"\n\nfunc main() {\n    text := \"hello\"\n    runes := []rune(text)\n    for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {\n        runes[i], runes[j] = runes[j], runes[i]\n    }\n    fmt.Println(string(runes))\n}",
    c:
      "#include <stdio.h>\n#include <string.h>\n\nint main(void) {\n    char text[] = \"hello\";\n    size_t length = strlen(text);\n    for (size_t i = 0; i < length / 2; i++) {\n        char temp = text[i];\n        text[i] = text[length - 1 - i];\n        text[length - 1 - i] = temp;\n    }\n    puts(text);\n    return 0;\n}",
    cpp:
      "#include <algorithm>\n#include <iostream>\n#include <string>\n\nint main() {\n    std::string text = \"hello\";\n    std::reverse(text.begin(), text.end());\n    std::cout << text << '\\n';\n}",
    java:
      "public class Main {\n    public static void main(String[] args) {\n        String text = \"hello\";\n        System.out.println(new StringBuilder(text).reverse().toString());\n    }\n}",
    csharp:
      "using System;\n\nclass Program {\n    static void Main() {\n        var text = \"hello\".ToCharArray();\n        Array.Reverse(text);\n        Console.WriteLine(new string(text));\n    }\n}",
    ruby:
      "text = \"hello\"\nputs text.reverse",
  },
  sum_to_ten: {
    rust:
      "fn main() {\n    let total: u32 = (1..=10).sum();\n    println!(\"{total}\");\n}",
    python:
      "total = sum(range(1, 11))\nprint(total)",
    javascript:
      "let total = 0;\nfor (let number = 1; number <= 10; number += 1) {\n  total += number;\n}\nconsole.log(total);",
    typescript:
      "let total = 0;\nfor (let number = 1; number <= 10; number += 1) {\n  total += number;\n}\nconsole.log(total);",
    go:
      "package main\n\nimport \"fmt\"\n\nfunc main() {\n    total := 0\n    for number := 1; number <= 10; number++ {\n        total += number\n    }\n    fmt.Println(total)\n}",
    c:
      "#include <stdio.h>\n\nint main(void) {\n    int total = 0;\n    for (int number = 1; number <= 10; number++) {\n        total += number;\n    }\n    printf(\"%d\\n\", total);\n    return 0;\n}",
    cpp:
      "#include <iostream>\n\nint main() {\n    int total = 0;\n    for (int number = 1; number <= 10; number++) {\n        total += number;\n    }\n    std::cout << total << '\\n';\n}",
    java:
      "public class Main {\n    public static void main(String[] args) {\n        int total = 0;\n        for (int number = 1; number <= 10; number++) {\n            total += number;\n        }\n        System.out.println(total);\n    }\n}",
    csharp:
      "using System;\n\nclass Program {\n    static void Main() {\n        int total = 0;\n        for (int number = 1; number <= 10; number++) {\n            total += number;\n        }\n        Console.WriteLine(total);\n    }\n}",
    ruby:
      "total = (1..10).sum\nputs total",
  },
  // Issue #334: recursive `fibonacci` function evaluated at the 10th term (55).
  fibonacci: {
    rust:
      "fn fibonacci(n: u64) -> u64 {\n    if n <= 2 {\n        1\n    } else {\n        fibonacci(n - 1) + fibonacci(n - 2)\n    }\n}\n\nfn main() {\n    println!(\"{}\", fibonacci(10));\n}",
    python:
      "def fibonacci(n):\n    if n <= 2:\n        return 1\n    return fibonacci(n - 1) + fibonacci(n - 2)\n\n\nprint(fibonacci(10))",
    javascript:
      "function fibonacci(n) {\n  if (n <= 2) {\n    return 1;\n  }\n  return fibonacci(n - 1) + fibonacci(n - 2);\n}\n\nconsole.log(fibonacci(10));",
    typescript:
      "function fibonacci(n: number): number {\n  if (n <= 2) {\n    return 1;\n  }\n  return fibonacci(n - 1) + fibonacci(n - 2);\n}\n\nconsole.log(fibonacci(10));",
    go:
      "package main\n\nimport \"fmt\"\n\nfunc fibonacci(n int) int {\n    if n <= 2 {\n        return 1\n    }\n    return fibonacci(n-1) + fibonacci(n-2)\n}\n\nfunc main() {\n    fmt.Println(fibonacci(10))\n}",
    c:
      "#include <stdio.h>\n\nunsigned long long fibonacci(int n) {\n    if (n <= 2) {\n        return 1;\n    }\n    return fibonacci(n - 1) + fibonacci(n - 2);\n}\n\nint main(void) {\n    printf(\"%llu\\n\", fibonacci(10));\n    return 0;\n}",
    cpp:
      "#include <iostream>\n\nunsigned long long fibonacci(int n) {\n    if (n <= 2) {\n        return 1;\n    }\n    return fibonacci(n - 1) + fibonacci(n - 2);\n}\n\nint main() {\n    std::cout << fibonacci(10) << '\\n';\n}",
    java:
      "public class Main {\n    static long fibonacci(int n) {\n        if (n <= 2) {\n            return 1;\n        }\n        return fibonacci(n - 1) + fibonacci(n - 2);\n    }\n\n    public static void main(String[] args) {\n        System.out.println(fibonacci(10));\n    }\n}",
    csharp:
      "using System;\n\nclass Program {\n    static long Fibonacci(int n) {\n        if (n <= 2) {\n            return 1;\n        }\n        return Fibonacci(n - 1) + Fibonacci(n - 2);\n    }\n\n    static void Main() {\n        Console.WriteLine(Fibonacci(10));\n    }\n}",
    ruby:
      "def fibonacci(n)\n  return 1 if n <= 2\n\n  fibonacci(n - 1) + fibonacci(n - 2)\nend\n\nputs fibonacci(10)",
  },
};

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
};

// The committed popular-case cache (mirrors ORACLE_SNAPSHOTS in src/knowledge.rs).
// Intentionally tiny — well under the cache cap for every source (R8) — and is
// the offline accelerator a gated live refresh would repopulate.
const CODING_ORACLE_SNAPSHOTS = [
  {
    taskSlug: "hello_world",
    languageSlug: "kotlin",
    languageLabel: "Kotlin",
    source: "hello-world-collection",
    sourceUrl: "http://helloworldcollection.de/#Kotlin",
    code: 'fun main() {\n    println("Hello, World!")\n}',
    expectedOutput: "Hello, World!",
  },
  {
    taskSlug: "hello_world",
    languageSlug: "swift",
    languageLabel: "Swift",
    source: "hello-world-collection",
    sourceUrl: "http://helloworldcollection.de/#Swift",
    code: 'print("Hello, World!")',
    expectedOutput: "Hello, World!",
  },
  {
    taskSlug: "hello_world",
    languageSlug: "php",
    languageLabel: "PHP",
    source: "hello-world-collection",
    sourceUrl: "http://helloworldcollection.de/#PHP",
    code: '<?php\necho "Hello, World!\\n";',
    expectedOutput: "Hello, World!",
  },
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
    languageSlug: "lua",
    languageLabel: "Lua",
    source: "hello-world-collection",
    sourceUrl: "http://helloworldcollection.de/#Lua",
    code: 'print("Hello, World!")',
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
  return CODING_ORACLE_SNAPSHOTS.some(
    (snippet) =>
      snippet.languageSlug === needle ||
      snippet.languageLabel.toLowerCase() === needle,
  );
}

// Render an otherwise-unsupported write_program request from the coding oracle's
// cached external snippets (mirrors try_write_program_from_oracle in
// src/solver_handler_oracle.rs, byte-for-byte on the content and evidence).
