// Agent-CLI execution of arbitrary natural-language procedures (issue #674;
// rust/src/agentic_coding/procedure.rs).
//
// The symbolic solver and the agent surface call the same compiler. Agent mode
// materializes the complete artifact in its workspace, reads it back through a
// shell tool, and only then returns the inspectable restatement.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { detect } from './crate/language.mjs';
import { trim } from './crate/rust_str.mjs';
import { localizedResponse } from './crate/seed.mjs';
import {
  PROCEDURE_CONFORMANCE_TRIGGER, compileProcedure, conformanceLinksNotation, restateSteps,
} from './crate/skill_procedure.mjs';
import { artifactLinksNotation, extractCompiledProcedureArtifact, proceduresEqual } from './crate/skill_procedure_artifact.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { Progress } from './progress.mjs';

/** Mirrors `COMPILED_PROCEDURE_PATH` in rust/src/agentic_coding/procedure.rs. */
export const COMPILED_PROCEDURE_PATH = 'compiled-procedure.lino';

/**
 * Mirrors `fn compile_task` in rust/src/agentic_coding/procedure.rs: the
 * compiled procedure, or null when the task is not one.
 * @param {string} task
 * @returns {object|null}
 */
export function compileTask(task) {
  return compileProcedure(task);
}

/**
 * Mirrors `fn plan_step` in rust/src/agentic_coding/procedure.rs: write the
 * artifact, read it back, run the conformance walk, then restate the steps.
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {object} procedure a `CompiledProcedure`
 * @returns {object} an `AgenticPlan`
 */
export function planStep(messages, toolNames, procedure) {
  const document = artifactLinksNotation(procedure);
  const progress = Progress.scan(messages);
  const writeTool = toolFor(toolNames, Capability.Write);
  if (writeTool !== null && !progress.done(Capability.Write)) {
    return planOne(writeTool, writeArguments(COMPILED_PROCEDURE_PATH, document));
  }
  if (writeTool === null) {
    return finalAnswer(renderResponse('agent_procedure_write_unavailable', procedure, document, ''));
  }
  const runTool = toolFor(toolNames, Capability.Run);
  if (runTool !== null && progress.run_outputs.length === 0) {
    return planOne(runTool, jsonText({ command: ['cat', COMPILED_PROCEDURE_PATH].join(' ') }));
  }
  if (runTool === null) {
    return finalAnswer(renderResponse('agent_procedure_readback_unavailable', procedure, document, ''));
  }
  const restored = extractCompiledProcedureArtifact(progress.run_outputs[0]);
  if (restored === null || !proceduresEqual(restored, procedure)) {
    return finalAnswer(renderResponse('agent_procedure_verification_failed', procedure, document, ''));
  }
  const expectedExecution = conformanceLinksNotation(procedure, PROCEDURE_CONFORMANCE_TRIGGER);
  if (progress.run_outputs.length === 1) {
    const command = ['formal-ai', 'procedure', 'conformance', '--artifact', COMPILED_PROCEDURE_PATH,
      '--trigger', PROCEDURE_CONFORMANCE_TRIGGER].join(' ');
    return planOne(runTool, jsonText({ command }));
  }
  if (trim(progress.run_outputs[1]) !== trim(expectedExecution)) {
    return finalAnswer(renderResponse('agent_procedure_execution_failed', procedure, document, ''));
  }
  return finalAnswer(renderResponse('agent_procedure_executed', procedure, document, expectedExecution));
}

/** Mirrors `fn render_response` in rust/src/agentic_coding/procedure.rs (sequential replaces, in order). */
function renderResponse(intent, procedure, document, execution) {
  const template = localizedResponse(intent, detect(procedure.source_description)) ?? '';
  return [
    ['{path}', COMPILED_PROCEDURE_PATH],
    ['{procedure_id}', procedure.id],
    ['{artifact}', document],
    ['{execution}', execution],
    ['{steps}', restateSteps(procedure)],
  ].reduce((text, [placeholder, value]) => text.split(placeholder).join(value), template);
}
