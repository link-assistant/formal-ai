# Issue #1162: widen the own-status patterns to the older Hive notice formats
# found on the Scala test pull request (37 of 117 viewer notices survived).
import sys
OLD_LOG = r'🔄 Auto-restart(?:-until-mergeable)? (?:Log )?[0-9]+/[0-9]+(?: Log)?)'
NEW_LOG = r'🔄 Auto-restart(?:-until-mergeable)? (?:Log )?[0-9]+/[0-9]+(?: Log)?|🔄 Auto-restart-until-mergeable Log \(iteration [0-9]+\))'
OLD_RESTART = r'^## 🔄 Auto-restart [0-9]+/[0-9]+\n\n\*\*Reason:'
NEW_RESTART = r'^## 🔄 Auto-restart (?:[0-9]+/[0-9]+|triggered \(iteration [0-9]+\))\n\n\*\*Reason:'
ANCHOR = '  pull-request-automation-stopped-pattern '
ADDED = [
    r'  pull-request-auto-restart-evidence-pattern "^## 🔄 Auto-restart [0-9]+/[0-9]+\n\n\*\*Reason:\*\* [^\n]+\n\n\*\*Evidence:\*\*\n(?:- [^\n]+\n)+\nStarting a new session to address it\.$"',
    r'  pull-request-auto-restart-uncommitted-pattern "^## 🔄 Auto-restart [0-9]+/[0-9]+\n\nDetected uncommitted changes from previous run\. Starting new session to review and commit or discard them\.\n\n\*\*Uncommitted files:\*\*\n```\n[\s\S]*\n```\n\n---\n\*Auto-restart will stop after changes are committed or discarded, or after [0-9]+ more iterations?\. Please wait until working session will end and give your feedback\.\*$"',
    r'  pull-request-restart-safety-limit-pattern "^## ⚠️ Auto-restart limit reached\n\nHive Mind stopped auto-restart-until-mergeable after [0-9]+ restart iterations\.\n[\s\S]+\n\n---\n\*Auto-restart-until-mergeable stopped by the safety limit\.\*$"',
    r'  pull-request-session-started-draft-pattern "^🤖 \*\*AI Work Session Started\*\*\n\nStarting automated work session at [0-9TZ:.+-]+\n\nThe PR has been converted to draft mode while work is in progress\.\n\n_This comment marks the beginning of an AI work session\. Please wait for the session to finish, and provide your feedback\._$"',
]
for path in sys.argv[1:]:
    text = open(path, encoding='utf-8').read()
    assert text.count(OLD_LOG) == 1 and text.count(OLD_RESTART) == 1, path
    text = text.replace(OLD_LOG, NEW_LOG).replace(OLD_RESTART, NEW_RESTART)
    lines = text.split('\n')
    at = next(i for i, line in enumerate(lines) if line.startswith(ANCHOR))
    lines[at + 1:at + 1] = ADDED
    open(path, 'w', encoding='utf-8').write('\n'.join(lines))

# Second pass: session-start notices carry extra runtime fields (task image,
# Formal AI version) and the draft-era notice may end with that runtime line.
OLD_RUNTIME = r'_Runtime: solve `v[^`\n]+` · tool `[^`\n]+` · model `[^`\n]+`_$'
RUNTIME = r'_Runtime: solve `v[^`\n]+`(?: · [^`\n·]+ `[^`\n]+`)+_'
OLD_DRAFT_END = r'Please wait for the session to finish, and provide your feedback\._$'
for path in sys.argv[1:]:
    text = open(path, encoding='utf-8').read()
    if OLD_RUNTIME in text:
        text = text.replace(OLD_RUNTIME, RUNTIME + '$')
        text = text.replace(OLD_DRAFT_END, OLD_DRAFT_END[:-1] + r'(?:\n\n' + RUNTIME + ')?$')
        open(path, 'w', encoding='utf-8').write(text)
