// Source-bound observation goals before mutation (PR #1188).
// Native contracts: rust/src/repository_workspace/operation.rs and verify.rs.
import { childValue, childrenNamed, parseLino, readText } from "../agentic/host.mjs";
import { shellCommandForTask } from "../agentic/shell_command.mjs";
import { asksAboutTheWorkspace } from "../agentic/workspace_inspection.mjs";
import { mentionsRole } from "../agentic/crate/seed_meanings.mjs";
import {
  namedShellCommandInSentence,
  isProseWord,
} from "../agentic/shell_command_policy.mjs";
import { terminalCommandVocabulary } from "../agentic/crate/seed_terminal_commands.mjs";
import { normalizePrompt } from "../agentic/crate/engine.mjs";
import { sha256Hex } from "../agentic/crate/source_fetch.mjs";
import { quotedSegmentSpans } from "../agentic/crate/normal_markov.mjs";

/** Resolve existing seeded grammars; source dependencies are supplied separately. */
export function classifyRepositoryOperation(requirement) {
  let outside = requirement;
  for (const span of quotedSegmentSpans(requirement).slice().reverse()) {
    const bytes = new TextEncoder().encode(outside);
    outside =
      new TextDecoder().decode(bytes.slice(0, span.start)) +
      " ".repeat(span.end - span.start) +
      new TextDecoder().decode(bytes.slice(span.end));
  }
  const normalized = normalizePrompt(outside);
  const mutating = ["file_write_action_cue", "software_authoring_action"].some(
    (role) => mentionsRole(role, normalized),
  );
  let command = shellCommandForTask(requirement);
  if (
    command !== null &&
    !['"', "'", "`", "|", "&", ";", "<", ">", "$", "(", ")", "{", "}"].some(
      (character) => command.includes(character),
    ) &&
    command.split(/\s+/u).slice(1).some(isProseWord)
  ) {
    command =
      namedShellCommandInSentence(requirement, terminalCommandVocabulary()) ??
      command;
  }
  const inspection = asksAboutTheWorkspace(requirement);
  if (mutating)
    return {
      kind: command !== null || inspection ? "unsupported" : "mutation",
    };
  if (command !== null && inspection) return { kind: "unsupported" };
  if (command !== null) return { kind: "run", line: command, names: [] };
  if (inspection)
    return {
      kind: mentionsRole("coding_test_artifact_kind", normalized)
        ? "test-targets"
        : "unsupported",
    };
  return { kind: "mutation" };
}

/** Only these stages can satisfy the selected observation goal. */
export function selectsRepositoryStage(operation, stage) {
  if (operation.kind === "mutation") return true;
  if (operation.kind === "test-targets")
    return ["clone", "locate", "read"].includes(stage);
  if (operation.kind === "run") return ["clone", "verify"].includes(stage);
  return stage === "clone";
}

/** Select a unique shallow Cargo root from actual observed paths; ambiguity is a gap. */
export function selectCargoManifest(paths) {
  const candidates = paths
    .map((path) => path.replaceAll("\\", "/"))
    .filter((path) => path.split("/").at(-1) === "Cargo.toml");
  const depth = Math.min(
    ...candidates.map((path) => path.split("/").length - 1),
  );
  const nearest = candidates.filter(
    (path) => path.split("/").length - 1 === depth,
  );
  if (nearest.length !== 1)
    throw new Error("Cargo manifest is missing or ambiguous");
  return nearest[0];
}

/** Completion is defined by the exact observed process status, never output wording. */
export function observedCommandReport({
  line,
  argv,
  root,
  baseCommit,
  observedCommit,
  observation,
  evidence,
}) {
  if (
    !Array.isArray(argv) ||
    argv.length === 0 ||
    !root ||
    !/^[0-9a-f]{40}$/u.test(baseCommit)
  ) {
    throw new Error("command context is unbound");
  }
  if (
    evidence.command !== line ||
    evidence.exit_code !== observation.exit_code ||
    evidence.observed_byte_length !==
      new TextEncoder().encode(observation.partial_output).length ||
    evidence.observed_output_sha256 !==
      sha256Hex(new TextEncoder().encode(observation.partial_output)) ||
    JSON.stringify(evidence.argv) !== JSON.stringify(argv) ||
    !evidence.evidence_id
  ) {
    throw new Error("command evidence is unbound");
  }
  return {
    schema: "repository-command/v1",
    root,
    base_commit: baseCommit,
    observed_commit: observedCommit ?? null,
    source_ids: evidence.source_ids ?? [],
    command: line,
    argv,
    exit_code: observation.exit_code,
    timed_out: observation.timed_out,
    elapsed_seconds: observation.elapsed_seconds,
    deadline_seconds: observation.deadline_seconds,
    complete:
      !observation.timed_out &&
      observation.exit_code === 0 &&
      observedCommit === baseCommit,
    // The existing execution-box port provides a combined stream; do not invent its split.
    combined_output: observation.partial_output,
    evidence_id: evidence.evidence_id,
    observed_output_sha256: evidence.observed_output_sha256,
  };
}

// Native contract: rust/src/repository_workspace/operation.rs:command_context_excluded.
/** Reject actual operands using the canonical source-context policy; missing policy denies. */
export function commandChangesSourceContext(program, argumentsList) {
  const rule = childrenNamed(
    parseLino(readText("data/seed/repository-command-allowlist.lino")),
    "source-context-policy",
  ).find((node) => childValue(node, "program") === program);
  if (!rule) return true;
  return argumentsList.some((argument) =>
    (rule.children || []).some((field) =>
      field.name === "exact" ? argument === field.value
        : field.name === "prefix" && argument.startsWith(field.value),
    ),
  );
}
