// Probe: Markdown clause enumeration and seed-role readings of the #1450 body.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { workItemRequirements } from '../../js/agentic/work_item_requirements.mjs';
import { mentionsRole, meaningEvidencedIn } from '../../js/agentic/write_lexicon.mjs';
import { normalizePrompt } from '../../js/agentic/crate/engine.mjs';
import { quotedSegmentSpans } from '../../js/agentic/crate/quoted_segments.mjs';
await installNodeHost(new WorkerHost());
const body = readFileSync(new URL(process.argv[2] ?? './issue-1450-body.md', import.meta.url), 'utf8');
const roles = ['program_request', 'coding_request_verb', 'ci_workflow_request', 'software_followup_execution', 'code_style_obligation', 'file_naming_obligation', 'ci_badge_obligation', 'output_obligation_verb', 'statement_negation_cue'];
for (const [clause, span] of workItemRequirements(body)) {
  const n = normalizePrompt(clause).toLowerCase();
  const hit = roles.filter((role) => mentionsRole(role, n));
  if (meaningEvidencedIn('print_stdout', n)) hit.push('print_stdout');
  console.log(span.join(':'), JSON.stringify(clause.slice(0, 80)), hit.join(','), quotedSegmentSpans(clause).map((s) => s.text).join('|'));
}
