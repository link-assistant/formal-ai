# Issue #1162, third pass: notice shapes found on the Kotlin/claude and
# Rust/codex test pull requests (ready-to-merge without footer, reworded
# restart notices, intermediate log, failure, no-change and session-kill
# reports). Replaces whole policy lines by key and appends new keys.
import sys
REPLACE = {
    'pull-request-ready-to-merge-pattern': r'^## ✅ Ready to merge\n\nThis pull request is now ready to be merged:(?:\n- [^\n]+)+(?:\n\n---\n\*Monitored by hive-mind with --auto-restart-until-mergeable flag\*)?$',
    'pull-request-auto-restart-evidence-pattern': r'^## 🔄 Auto-restart [0-9]+/[0-9]+\n\n\*\*Reason:\*\* [^\n]+\n\n(?:\*\*Evidence:\*\*\n(?:- [^\n]+\n)+\n)?Starting a new session to address it\.$',
    'pull-request-auto-restart-uncommitted-pattern': r'^## 🔄 Auto-restart [0-9]+/[0-9]+\n\n(?:Detected uncommitted changes from previous run\. Starting new session to review and commit or discard them\.|The previous session left uncommitted changes\. Starting a new session to commit or discard them\.)\n\n\*\*Uncommitted files:\*\*\n```\n[^`]*\n```(?:\n\n---\n\*Auto-restart will stop after changes are committed or discarded, or after [0-9]+ more iterations?\. Please wait until working session will end and give your feedback\.\*)?$',
}
ADDED = {
    'pull-request-solution-failed-pattern': r'^## 🚨 Solution Draft Failed\n[\s\S]+\n\nLogs were not attached because `--attach-logs` was not enabled, so this comment is the only surviving record of the failure\. Rerun with `--attach-logs --verbose` to publish the full session log with the raw tool error records\.$',
    'pull-request-no-changes-pattern': r'^⚠️ \*\*No changes were produced by this session\*\*\n\nThe pull request (?:still )?has an empty diff against its base branch \(only the solver placeholder file is present\), so it (?:stays a draft instead of being|was not) marked ready for review\.(?:\n\nNothing was implemented, committed or pushed during this working session\. Review the session log above for why, then re-run the solver or continue manually\.| Check the session log for why, then rerun the solver or continue manually\.)$',
    'pull-request-session-killed-pattern': r'^<!-- hive-mind:session-kill-notice -->\n## ❌ Working session was force-killed\n[\s\S]+\n\n<sub>Reported by Hive Mind</sub>$',
}
OLD_LOG = r'|🔄 Auto-restart-until-mergeable Log \(iteration [0-9]+\))'
NEW_LOG = r'|🔄 Auto-restart-until-mergeable Log \(iteration [0-9]+\)|📎 Intermediate working-session log \(killed session\))'
for path in sys.argv[1:]:
    text = open(path, encoding='utf-8').read()
    assert text.count(OLD_LOG) == 1, path
    lines = text.replace(OLD_LOG, NEW_LOG).split('\n')
    for key, pattern in REPLACE.items():
        at = next(i for i, line in enumerate(lines) if line.startswith(f'  {key} '))
        lines[at] = f'  {key} "{pattern}"'
    last = max(i for i, line in enumerate(lines) if line.startswith('  pull-request-') and line.split(' ')[2].endswith('-pattern'))
    lines[last + 1:last + 1] = [f'  {key} "{pattern}"' for key, pattern in ADDED.items()]
    open(path, 'w', encoding='utf-8').write('\n'.join(lines))

# Fourth pass: the limit notice without a footer (Rust/codex test pull request).
OLD_LIMIT = r'^## ❌ Auto-restart [0-9]+/[0-9]+ - limit reached\n[\s\S]+\n\n---\n\*This run is reported as failed because the auto-restart limit was reached\.\*$'
NEW_LIMIT = r'^## ❌ Auto-restart [0-9]+/[0-9]+ - limit reached\n[\s\S]+\n\n(?:---\n\*This run is reported as failed because the auto-restart limit was reached\.\*|No more sessions will start automatically\. Resolve the blocker, or rerun with a higher `--auto-restart-max-iterations`\.)$'
for path in sys.argv[1:]:
    text = open(path, encoding='utf-8').read()
    if OLD_LIMIT in text:
        open(path, 'w', encoding='utf-8').write(text.replace(OLD_LIMIT, NEW_LIMIT))
