// Browser twin of solver_handlers/procedure_rules.rs. Prose comes from the seed.
function tryCompiledProcedure(prompt, language) {
  const compiler = crateModule("crate/skill_procedure.mjs");
  const result = compiler.compileProcedureResult(prompt);
  const records = [];
  const record = (kind, payload) => records.push(solverEvent(kind, payload));
  let intent;
  let values;
  if (result.procedure) {
    const procedure = result.procedure;
    const program = crateModule("crate/skill_procedure_artifact.mjs").artifactLinksNotation(procedure);
    record("skill_compile:procedure", procedure.id);
    record("skill_compile:procedure_artifact", program);
    for (const step of procedure.steps) record("skill_compile:procedure_step", `${step.index} ${step.kind} ${step.id}`);
    intent = "compiled_procedure";
    values = { program, steps: compiler.restateSteps(procedure) };
  } else if (result.error.kind === "uncompilable_step") {
    const { step, span, gap } = result.error;
    // ProcedureLearningProposal::from_compile_error and links_notation.
    const identifier = crateModule("crate/engine_stable_identifier.mjs").stableId(
      "procedure_learning_proposal", `${step.toLowerCase()}:${span[0]}..${span[1]}:${gap}`);
    let proposal = "";
    const append = (depth, name, value) => {
      proposal = crateModule("crate/links_format.mjs").pushLinoNode(proposal, depth, name, String(value));
    };
    append(0, "procedure_learning_proposal", identifier);
    append(2, "status", "human_review_required");
    append(2, "missing_step", step);
    append(2, "span_start", span[0]);
    append(2, "span_end", span[1]);
    append(2, "gap", gap);
    record("skill_gap", gap);
    record("skill_learning_proposal", identifier);
    record("skill_learning_proposal:artifact", proposal);
    intent = "skill_gap";
    values = { step, gap, proposal };
  } else {
    return null;
  }
  return { intent, content: handlerRulesFillOnce(answerFor(intent, language), values), confidence: 1,
    solverEvents: records, evidence: [`response:${intent}`] };
}
