"""Record the verified application-source cutoff and hosted run results."""
import json,shutil
from pathlib import Path
root=Path('/tmp/issue-1582-review')
head='fe8d31d6445f2c37ce0dc141488a183eef50a227'
runs=[row for row in json.loads((root/'runs-source-final.json').read_text()) if row['headSha']==head]
assert len(runs)==22 and all(row['status']=='completed' and row['conclusion']=='success' for row in runs)
text='''

### Hosted verification of the continuation repair

Application-source cutoff: `fe8d31d6445f2c37ce0dc141488a183eef50a227`.
All 22 workflows for that head completed successfully. The
[CI/CD run](https://github.com/link-assistant/formal-ai/actions/runs/44150504632)
first failed during checkout, before tests, with an RPC low-speed timeout,
`bad pack header` and a missing promisor blob (attempt-one log lines 123-125).
One failed-job retry passed. Both attempts are preserved under
`raw-data/local/continuation/ci-logs/`.
The [Layered CI run](https://github.com/link-assistant/formal-ai/actions/runs/44150503985)
passed both JavaScript shards. Its log explicitly records all 35 permission
cases at lines 4154-4189, including every coding target, and the defaults tests
at lines 6747-6758. The native production type/lint check passed. This does not
execute or compile the five drafted native regressions.

The first support matrix inspected only direct mode children and omitted
context entries in four global companion files. The corrected census walks
nested companions, retains their destination paths, and asserts context
metadata for every non-MCP mode. It now records 48 declarations across all
25 modes, including global Gemini output, Qwen, Grok and Aider settings.
No application change was needed for this evidence correction. A paginated
PR-file review verified all 1,681 remote file/blob identities against the
application-source cutoff; GitHub's aggregate diff endpoint rejects this
archive's file count. The latest test and source diffs were also read locally.

The local dependency audit remains a real failing result even though hosted
PR CI passes: that audit is outside the fast PR profile. The active advisory
has no published patched release. Gemini input context and native runtime
validation remain unfinished; successful workflow wrappers do not complete
those criteria. The PR therefore stays draft and unmerged. The following
commit preserves the hosted results and corrects review evidence, with no
application-source changes; its checks are verified without recursively
committing their own logs.

Additional execution exceptions were remote metadata/API inspection and the
bounded failed-job retry. These have zero independent authorship credit. A
commit recipe first used the nonexistent `data/meta/requirement-status` path;
Git rejected it without committing, and the corrected ledger path succeeded.
Both operational attempts are retained. No local Rust build was run.
'''
p=Path('docs/case-studies/issue-1582/README.md');p.write_text(p.read_text()+text)
p=Path('docs/case-studies/pull-request-1583/README.md')
s=p.read_text().replace('Current-head hosted runs will be recorded after the complete repair batch is\npushed.', 'All 22 hosted workflows pass for application-source head\n`'+head+'`. [CI/CD](https://github.com/link-assistant/formal-ai/actions/runs/44150504632)\npassed after one failed-job retry for an initial checkout transport failure;\n[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/44150503985)\npasses both JavaScript shards and records the new permission cases. Native\nproduction type/lint passes; native regressions remain unexecuted. The local\ndependency audit is outside the fast PR profile and still fails. The corrected\nsupport matrix includes 48 context declarations across all 25 modes, including\nnested companion files. The final evidence commit changes no application source.')
p.write_text(s)
p=Path('experiments/issue_1582/continuation-plan.md')
s=p.read_text().replace('- Units 19-20: pending the single push gate, current-head hosted checks and\n  remote review metadata. Readiness cannot be claimed while acceptance is open.', '- Units 19-20: complete repair batch pushed after the single push gate; all\n  22 application-source workflows pass after one checkout-failure retry. PR\n  metadata is updated, main is an ancestor, and paginated remote blob review\n  matches 1,681 files. The final evidence commit and its checks remain to verify.\n  Readiness cannot be claimed while acceptance is open.')
p.write_text(s)
p=root/'pull-request-body.md'
s=p.read_text().replace('Current-head hosted checks will be linked after the push.', 'All **22 hosted workflows pass** for application-source head `fe8d31d64`: [CI/CD](https://github.com/link-assistant/formal-ai/actions/runs/44150504632) passed after one retry of an initial checkout transport failure; [Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/44150503985) records the new permission cases. The local dependency audit is outside the fast PR profile and still fails. The final evidence commit changes no application source and is checked separately.')
s=s.replace('Source/configuration inspection is distinguished', 'The recursive census includes **48 context declarations**, including global companion files. Source/configuration inspection is distinguished')
p.write_text(s)
for source,name in [('/tmp/issue-1582-commit-evidence-final.log','commit-evidence-failed.log'),('/tmp/issue-1582-commit-evidence-retry.log','commit-evidence-succeeded.log'),('/tmp/issue-1582-stage-evidence-final.log','stage-evidence-final.log'),('/tmp/issue-1582-push.sh','push-recipe.sh'),('/tmp/issue-1582-review-pr.sh','remote-review-recipe.sh')]:
 shutil.copyfile(source,root/name)
print(json.dumps({'applicationSourceHead':head,'successfulWorkflows':len(runs),'nativeRegressionsExecuted':False}))
