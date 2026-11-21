from pathlib import Path
p=Path('experiments/issue_1582/continuation-matrix.mjs')
s=p.read_text().replace("await installDefaultNodeSourceHost();", "import {WorkerHost} from '../../js/server/worker-host.mjs';\nawait installDefaultNodeSourceHost(new WorkerHost());")
p.write_text(s)
p=Path('rust/src/solver_handler_how.rs');p.write_text(p.read_text().replace('if crate::internet_access_help_requested(prompt, normalized)', 'if internet_access_help_requested(prompt, normalized)'))
p=Path('docs/requirements/issue-1582-client-read-access-and-context.md')
s=p.read_text().replace('guidance in five languages; `rust/tests/web/issue-1582-defaults.test.mjs`', 'guidance with automatic detection in all five registered languages, including quoted-input exclusions; `rust/tests/web/issue-1582-permissions.test.mjs`')
s=s.replace('and final current-head CI evidence remains to collect.', 'review-continuation evidence distinguishes all 12 registered tools and JavaScript, TypeScript and Rust verification; final current-head CI evidence remains to collect.')
p.write_text(s)
p=Path('docs/requirements-traceability.md');s=p.read_text().replace('implemented — five-language harness permission guidance | rust/tests/web/issue-1582-defaults.test.mjs', 'implemented — automatic five-language permission guidance and quoted-input exclusions | rust/tests/web/issue-1582-permissions.test.mjs');p.write_text(s)
p=Path('changelog.d/20261117_1582_client_access_diagnostics.md');s=p.read_text().replace('controls in five languages (#1582).', 'controls in five languages. Recognize direct permission paraphrases without taking over quoted translation or programming tasks (#1582).');p.write_text(s)
p=Path('docs/case-studies/issue-1582/README.md');s=p.read_text();marker='## Review continuation: complete support census'
if marker not in s:
 s+='''

## Review continuation: complete support census

The [latest review](https://github.com/link-assistant/formal-ai/pull/1583#issuecomment-6955718572)
asks for all supported tools and natural/programming languages. The retained
`raw-data/local/continuation-support-matrix.json` enumerates all 12 registered
adapters, every declared global/ephemeral mode, all five natural languages and
all three source roots. Cursor is an MCP tool adapter: its host model's context
is outside this adapter. A source/configuration census does not demonstrate an
installed vendor CLI. Gemini input context remains an unmet requirement.

This audit found a new reproducible permission-routing gap. Several natural
paraphrases missed the seed role, while a permission phrase embedded in a
translation or programming request could wrongly take ownership. The initial
18 shared cases passed six checks and failed twelve. Added seed aliases and a
direct-request prefix guard repair this reusable cause in JavaScript,
generated TypeScript and Rust. Quoted leading text stays with its enclosing
task. The final 19 checks pass: twelve direct requests use actual automatic
language detection across English, Russian, Hindi, Chinese and Spanish, six
negative cases exclude permission routing, and the generated TypeScript
recognizer executes the same fixture. The shared fixture is
`data/parity/issue-1582-permissions.json`; the executable test is
`rust/tests/web/issue-1582-permissions.test.mjs`. The corresponding Rust unit
regression is `rust/tests/unit/issue_1582_permissions.rs`; it is drafted and
unexecuted, in addition to the four unexecuted native integration regressions.
PR type/lint validation cannot replace native runtime evidence.

Current main was merged at `512ce428029a92687628b4180309ee5ab660fe08`.
Both sides' boundary inventories and obligations were retained. Main's governed
witness was refreshed only after checking all 221 before/after observations
were identical; the receipt records its two offline fixture call-site changes.
The merged issue-1588 source suite passes all 62 checks. New helper lines are
measured in the exact boundary and worker budgets without changing dispositions.

Fresh upstream Gemini source was inspected at the actual token-limit consumers
in `core/client.ts` and `context/chatCompressionService.ts`, together with model
configuration types. Unknown `formal-ai` still uses 1,048,576 input tokens;
model metadata does not supply an override to those consumers. The source probe
is bounded and is not an installed-client test. The existing upstream report
[google-gemini/gemini-cli#30333](https://github.com/google-gemini/gemini-cli/issues/30333)
remains open. Its input-context criterion is unfinished. Native runtime checks
also remain unfinished under the repository's pre-merge build policy; no local
Rust build was substituted in this memory-limited environment.

The frozen continuation plan and supplied repair recipes are under
`experiments/issue_1582/continuation-*`. Their execution through the JavaScript
Formal AI driver receives zero independent authorship credit. Initial read-only
inspection and the explicit parser/bootstrap recovery were external exceptions:
stale worker identity prevented the driver from loading, and prerequisite
regeneration needed the missing Babel parser tools. The bounded JavaScript-only
bootstrap and all failed attempts are retained. The independent requirement-gap
query returned existing partial markers; it did not independently author a fix.
The continuation's 20-unit denominator stays fixed; no success percentage is
inferred from supplied-command execution.
'''
p.write_text(s)
