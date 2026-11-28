// The issue #707 generalization reference: for one held-out case, check the
// locally synthesized and executed outcome against the explicit plan the
// external client is shown, and derive the `expected.jsonl` entry the verifier
// holds that client to.
//
// Kept out of run_issue_707_generalization.sh's inline `node -e` so a pull
// request test can replay it (issue #1639). 66169b8fa compared the explicit
// plan's dotted names ("fs.write") with the serde plan's variant names
// ("FsWrite"), so every case failed only once main ran the E2E.
import { createHash } from "node:crypto";
import fs from "node:fs";
import { fileURLToPath } from "node:url";

/** Enum variant names as the plan serializes them, to the advertised tool names. */
export const PRIMITIVES = Object.freeze({
  FsRead: "fs.read",
  FsWrite: "fs.write",
  FsList: "fs.list",
  FsMove: "fs.move",
  ShellRun: "shell.run",
  HttpFetch: "http.fetch",
  HttpPost: "http.post",
  DomQuery: "dom.query",
  DomExtract: "dom.extract",
  ArchivePack: "archive.pack",
  ArchiveUnpack: "archive.unpack",
  ProcessStatus: "process.status",
});

/**
 * @param {{outcome: object, caseId: string, prompt: string, explicitText: string}} input
 *   `outcome` is the `computer-use --agent-mode` result's `outcome`; `explicitText`
 *   is the `computer-use --plan` output, whose second-to-last line is the plan.
 * @returns {object} the `expected.jsonl` entry
 */
export function referenceEntry({ outcome, caseId, prompt, explicitText }) {
  if (!outcome.verified) throw new Error(caseId + " did not verify locally");
  if (!outcome.plan.id.startsWith("synthesized-")) {
    throw new Error(caseId + " was recalled, not synthesized: " + outcome.plan.id);
  }
  const explicit = explicitText.trimEnd().split("\n").at(-2);
  // The explicit plan and each executed step name primitives by their dotted
  // wire name; the serde plan carries the enum variant the verifier maps back.
  const dotted = JSON.parse(explicit).map((step) => step.primitive);
  const executed = outcome.steps.map((step) => step.primitive);
  if (JSON.stringify(dotted) !== JSON.stringify(executed)) {
    throw new Error(caseId + " explicit plan differs from local synthesis");
  }
  const steps = outcome.plan.steps.map((step) => step.primitive);
  const planned = steps.map((variant) => PRIMITIVES[variant] ?? "unknown variant " + variant);
  if (JSON.stringify(planned) !== JSON.stringify(dotted)) {
    throw new Error(caseId + " local plan and executed steps differ: " + JSON.stringify(steps));
  }
  const digest = createHash("sha256").update(explicit).digest("hex");
  return {
    case_id: caseId,
    prompt,
    plan_id: "computer-use-" + digest.slice(0, 24),
    learned_plan_id: outcome.plan.id,
    steps,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [file, caseId, prompt, explicitPath] = process.argv.slice(2);
  const outcome = JSON.parse(fs.readFileSync(file, "utf8")).outcome;
  const explicitText = fs.readFileSync(explicitPath, "utf8");
  process.stdout.write(JSON.stringify(referenceEntry({ outcome, caseId, prompt, explicitText })) + "\n");
}
