// Honest browser projection of the generic verifiable-task route (#1138 B8).
//
// This module deliberately does not implement a second solver in JavaScript.
// It projects the shared meaning lexicon into a task/ProgramIr-shaped record,
// then stops at the browser execution boundary.  Native Formal AI owns
// discovery, composition and verification; until that executor is available in
// the browser, no derived value is emitted as an answer.

function verifiableExpectationRoles(meaning) {
  return (meaning.roles || []).filter((role) =>
    role.startsWith("verifiable_expectation_") && role !== "verifiable_expectation",
  );
}

function captureVerifiableForm(prompt, surface) {
  const original = String(prompt || "");
  const lowered = original.toLowerCase();
  const form = makeWordForm(String(surface || "").toLowerCase(), "", "");
  if (form.slot === "bare") {
    return lowered.includes(form.text) ? "" : null;
  }
  const start = form.before ? lowered.indexOf(form.before) : 0;
  if (start < 0) return null;
  const begin = start + form.before.length;
  const relativeEnd = form.after ? lowered.slice(begin).indexOf(form.after) : -1;
  if (form.after && relativeEnd < 0) return null;
  const end = form.after ? begin + relativeEnd : original.length;
  return original.slice(begin, end).trim();
}

function recogniseBrowserVerifiableTask(prompt) {
  // Match native `recognise_verifiable`: callable/stdout coding specs are
  // owned by the established coding route and must not be reclassified merely
  // because their prose also contains a pattern word such as "sequence".
  if (writeProgramParameters(prompt)) return null;
  const detected = detectLanguage(prompt);
  const availableLanguages = [];
  for (const meaning of meaningsWithRole("verifiable_expectation")) {
    for (const lexeme of meaning.lexemes || []) {
      if (lexeme.language && !availableLanguages.includes(lexeme.language)) {
        availableLanguages.push(lexeme.language);
      }
    }
  }
  const languages = [detected, ...availableLanguages.filter((item) => item !== detected)];
  const candidates = [];
  for (const language of languages) {
    for (const meaning of meaningsWithRole("verifiable_expectation")) {
      const roles = verifiableExpectationRoles(meaning);
      if (roles.length === 0) continue;
      for (const lexeme of meaning.lexemes || []) {
        if (lexeme.language !== language) continue;
        for (const surface of lexeme.words || []) {
          const capture = captureVerifiableForm(prompt, surface);
          if (capture === null) continue;
          candidates.push({
            expectation: roles[0].slice("verifiable_expectation_".length),
            capture,
            language,
            surface,
            languagePriority: languages.indexOf(language),
          });
        }
      }
    }
  }
  candidates.sort((left, right) =>
    left.languagePriority - right.languagePriority || right.capture.length - left.capture.length,
  );
  const matched = candidates[0];
  if (!matched) return null;
  const quantities = String(prompt || "").match(/\p{Number}+/gu) || [];
  if (matched.expectation === "unknown" &&
      (!String(prompt).includes("=") || quantities.length < 2)) {
    return null;
  }
  const requirements = String(prompt || "")
    .split(/[.!?。！？।]+/u)
    .map((sentence) => sentence.trim())
    .filter(Boolean);
  const identity = stableBehaviorRuleId(
    "verifiable_task",
    [matched.expectation, matched.language, matched.surface, requirements.length].join("\u0001"),
  );
  return {
    id: identity,
    expectation: matched.expectation,
    capture: matched.capture,
    language: matched.language,
    requirements,
    quantities,
  };
}

function browserVerifiableProgram(task) {
  const lines = [
    "program_ir",
    `  id ${linoString(task.id)}`,
    `  expectation ${linoString(task.expectation)}`,
    `  prose_language ${linoString(task.language)}`,
  ];
  for (const requirement of task.requirements) {
    lines.push(`  requirement ${linoString(requirement)}`);
  }
  for (const quantity of task.quantities) {
    lines.push(`  quantity ${linoString(quantity)}`);
  }
  lines.push("  stage recognise");
  lines.push("  stage discover");
  lines.push("  stage compose");
  lines.push("  stage execute");
  lines.push("  stage verify");
  return lines.join("\n");
}

function tryVerifiableTask(prompt) {
  const task = recogniseBrowserVerifiableTask(prompt);
  if (!task) return null;
  const program = browserVerifiableProgram(task);
  const content = String(answerFor("verifiable_task_browser_unverified", task.language))
    .replace("{expectation}", task.expectation);
  return {
    intent: "verifiable_task",
    content,
    confidence: 0.7,
    evidence: [
      `verifiable_task:recognised:${task.id}`,
      `verifiable_task:formalized:${task.expectation}:${task.language}`,
      `verifiable_task:derived:${stableBehaviorRuleId("program_ir", program)}`,
      "verifiable_task:unverified:browser_execution_unavailable",
    ],
    steps: [
      { step: "verifiable_task_recognise", detail: task.expectation },
      { step: "verifiable_task_formalize", detail: program },
      { step: "verifiable_task_derive", detail: "program_ir" },
      { step: "verifiable_task_render", detail: "unverified" },
    ],
    diagnostics: {
      verifiableTask: {
        id: task.id,
        expectation: task.expectation,
        language: task.language,
        requirements: task.requirements,
        program,
        status: "unverified",
      },
    },
    toolCalls: [],
  };
}
