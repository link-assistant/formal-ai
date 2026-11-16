"""Add the superseded-workflow retirement step to both work-item-steps copies (issue #1162)."""
import sys
NOTE = ('  superseded-workflow-source "Issue #1162: the Kotlin and Scala deliveries of 2026-11 ended with run.yml from an earlier session '
        'and Main.yml from the resumed one, both generated for the same program. Only a tracked, unmodified, non-symlink workflow in the '
        'same directory that carries this generator\'s exact header line and name line for the same program is retired; every other '
        'workflow stays untouched."\n')
STEP = ('  superseded-workflow-retire-command \'for formal_ai_workflow in {dir}/*.yml {dir}/*.yaml; do\\n'
        '  test "$formal_ai_workflow" != {workflow} && test -f "$formal_ai_workflow" && ! test -L "$formal_ai_workflow" &&\\n'
        '    git ls-files --error-unmatch -- "$formal_ai_workflow" >/dev/null 2>&1 &&\\n'
        '    git diff --quiet -- "$formal_ai_workflow" && git diff --cached --quiet -- "$formal_ai_workflow" &&\\n'
        '    grep -qxF -e {header} -- "$formal_ai_workflow" && grep -qxF -e {name} -- "$formal_ai_workflow" || continue\\n'
        '  rm -- "$formal_ai_workflow" || exit $?\\n'
        '  set -- "$@" "$formal_ai_workflow"\\n'
        'done\\n\'\n')
for path in sys.argv[1:]:
    text = open(path, encoding="utf-8").read()
    assert "superseded-workflow-retire-command" not in text
    old = "fi\\n{commit}\\n)'\n"
    assert text.count(old) == 1, path
    text = text.replace(old, "fi\\n{retire}{commit}\\n)'\n" + NOTE + STEP)
    open(path, "w", encoding="utf-8").write(text)
    print("updated", path)
