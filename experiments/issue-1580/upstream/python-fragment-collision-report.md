`scripts/create_manual_changeset.py --no-scriv` silently replaces a pending changelog entry when the same Git user and branch create another entry during the same second. Both invocations return success and print the same created filename, but the earlier category and description are lost.

Affected source: [`c532c173801dfb12e681b0730c4abbf93aefd0ca`](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/blob/c532c173801dfb12e681b0730c4abbf93aefd0ca/scripts/create_manual_changeset.py). Lines 130–139 construct a filename from second-resolution time, user and branch; line 184 uses `Path.write_text`, which truncates an existing file. Current main's helper was checked on 2026-11-17 and is byte-identical (Git blob `24f4c2b023e08ebb9397c497f2a12f95dd88a93e`).

For example, run these within one second on one branch:

```sh
python scripts/create_manual_changeset.py minor --no-scriv --description "Earlier feature"
python scripts/create_manual_changeset.py patch --no-scriv --description "Later patch"
```

The single resulting file contains only `### Fixed` and `Later patch`; the earlier `### Added` entry is gone. Expected: preserve the existing entry, either rejecting the collision clearly or creating a distinct file atomically.

The [four-case executable reproduction](https://github.com/link-assistant/formal-ai/blob/9c018344df2e6b09a0f2496897b14a002dad62f4/experiments/issue-1580/template-python-fragment-integrity.test.mjs) executes the complete unchanged entrypoint, preserving its source/Unlicense identities. It uses actual temporary Git repositories and a frozen clock, so no timing race or long wait is required. Python 3.15.0 children are bounded to 128 MiB, three CPU seconds and a five-second wall limit. No Scriv execution, build, publication or external write is involved.

A disposable candidate replaces the truncating write with exclusive creation:

```python
with fragment_path.open("x") as fragment:
    fragment.write(content)
```

The collision then fails with `FileExistsError` while preserving the earlier exact bytes. The preservation assertion fails on the original: three pass, one fails. With the candidate, all four cases pass, including distinct-timestamp success and nonmutating invalid-argument controls. The maintained Formal AI entrypoint passes 29 cases without skips. [Complete logs and identities](https://github.com/link-assistant/formal-ai/blob/9c018344df2e6b09a0f2496897b14a002dad62f4/experiments/issue-1580/evidence/checks/python-fragment-integrity.json) retain the initial reporter-format harness correction as well.

Run `node --test --test-concurrency=1 experiments/issue-1580/template-python-fragment-integrity.test.mjs` from that evidence checkout. A unique suffix may reduce collision frequency, but exclusive creation should enforce preservation. Choosing/retrying a fresh filename can retain successful repeated creation if desired.

All 156 current issues/PRs were searched. Validator replacement handling #100/#101, release recovery #148 and dependency locking #152 address different boundaries; none changes this creator. The candidate is tested only in disposable copies. This report does not claim an upstream repair or hosted release verification. Related work: https://github.com/link-assistant/formal-ai/pull/1581.
