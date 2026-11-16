// Probe: which issue-#1166 obligation roles each #1450 clause (or a given body) evidences.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { workItemRequirements } from '../../js/agentic/work_item_requirements.mjs';
import { mentionsRole, meaningEvidencedIn } from '../../js/agentic/write_lexicon.mjs';
import { normalizePrompt } from '../../js/agentic/crate/engine.mjs';
await installNodeHost(new WorkerHost());
const body = readFileSync(process.argv[2] ?? new URL('./issue-1450-body.md', import.meta.url), 'utf8');
const roles = ['program_file_obligation', 'code_comment_obligation', 'run_instruction_obligation', 'judgment_obligation', 'workflow_trigger_push', 'workflow_trigger_pull_request', 'workflow_status_obligation', 'workflow_context_noun', 'ci_workflow_request', 'optional_obligation_marker', 'applicability_marker', 'illustrative_marker', 'condition_marker', 'negated_condition_marker', 'otherwise_marker', 'execution_circumstance', 'software_followup_execution', 'file_naming_obligation', 'ci_badge_obligation', 'output_obligation_verb', 'coding_request_verb', 'statement_negation_cue'];
for (const [clause, span] of workItemRequirements(body)) {
  const n = normalizePrompt(clause).toLowerCase();
  const hit = roles.filter((role) => mentionsRole(role, n));
  if (meaningEvidencedIn('print_stdout', n)) hit.push('print_stdout');
  console.log(span.join(':'), JSON.stringify(clause.slice(0, 60)), hit.join(','));
}
