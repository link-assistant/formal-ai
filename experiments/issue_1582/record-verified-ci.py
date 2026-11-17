"""Record only completed, successful repair-head CI with its complete archive."""
from pathlib import Path
import json, subprocess
sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
root = Path('docs/case-studies/issue-1582')
runs = json.loads((root / f'raw-data/ci/repair-runs-{sha[:12]}.json').read_text())
assert len(runs) == 22
assert all(run['head_sha'] == sha and run['status'] == 'completed' and run['conclusion'] == 'success' for run in runs)
for run in runs:
    receipt = json.loads((root / f"raw-data/ci/{run['id']}.log-status.json").read_text())
    assert receipt['exit'] == 0 and receipt['sha'] == sha, receipt
artifacts = json.loads((root / f'raw-data/ci/artifacts-{sha[:12]}.json').read_text())
for run in artifacts:
    for artifact in run['artifacts']:
        assert (root / f"raw-data/ci/artifact-{artifact['id']}.zip").is_file()
case = root / 'README.md'
text = case.read_text()
assert '## Verified repair CI' not in text
urls = {run['name']: run['html_url'] for run in runs}
pr_snapshot = json.loads((root / f'raw-data/pr-{sha[:12]}.json').read_text())
files = pr_snapshot['changedFiles']
main = subprocess.check_output(['git', 'rev-parse', 'origin/main'], text=True).strip()
subprocess.run(['git', 'merge-base', '--is-ancestor', 'origin/main', 'HEAD'], check=True)
text += f'''
## Verified repair CI

All 22 workflows for final source head `{sha}`
completed successfully, including [Security]({urls['Security']}),
[Layered CI]({urls['Layered CI']})
and the [CI/CD pipeline]({urls['CI/CD Pipeline']}).
Every run log and attached artifact is retained. The paginated PR files API
independently matched all {files:,} remote blob identities to the local branch.
Current main (`{main}`) is already an ancestor.

The raw CI archive has this explicit source-head cutoff. A later commit records
these results and evidence; its own checks are verified through the PR checks
view without recursively adding commits merely to archive their own runs.
No application source changes after the verified repair head are included in
that evidence commit.

The PR remains draft and unmerged. Gemini input context is still blocked by
the upstream interface, and the four native integration regressions have not
run. The repository's PR build policy permits the production native type/lint
check but excludes native test builds; no local Rust build was substituted.
These acceptance/validation limits are unfinished, even though hosted CI is green.
'''
case.write_text(text)
plan = root / 'work-plan.md'
text = plan.read_text().replace('- [ ] Review the full PR diff, merge current main, commit, check the push gate, push once and resolve current-head CI.', '- [x] Review the full PR diff (paginated API when the combined diff exceeds GitHub limits), confirm current main is already included, commit, check the push gate and resolve all current repair-head CI: 22/22 workflows pass.')
text = text.replace('- [ ] Update PR title/description with reproduction and test evidence, retain `Closes #1582`, mark ready, verify clean status.', '- [x] Update PR title/description with reproduction and test evidence; retain `Closes #1582`.\n- [ ] Mark ready only after the Gemini criterion and native regression execution are complete. Keep draft/unmerged while blocked. Verify the evidence-commit checks and clean status before reporting.')
plan.write_text(text)
pr = Path('experiments/issue_1582/pull-request.md')
text = pr.read_text().replace('The final source head still requires hosted CI verification.', f"The final source head, `{sha}`, completes **all 22 workflows successfully**, including [Security]({urls['Security']}), [Layered CI]({urls['Layered CI']}) and [CI/CD]({urls['CI/CD Pipeline']}). All its logs and artifacts are archived. The later evidence commit records this explicit source-head cutoff; its checks are verified separately in the PR checks view.")
pr.write_text(text)
subprocess.run(['git', 'diff', '--check'], check=True)
