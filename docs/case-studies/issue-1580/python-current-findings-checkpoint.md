# Current Python review and bounded template findings

The Python template is pinned at 62cc393707ddfe7ceeddbc580d43cc35334636f1: 338 current blobs and 35
changed files. All 32 changed plain source, workflow, configuration,
documentation and test files have been read completely, along with all 426
generated license lines. The 2.2 MB generated action bundle and 3421-line
dependency lock still need complete comparison. The 338-file, historical and
practice-adoption requirements remain open; retained bytes do not establish
review.

Current release and validation changes include budgeted npm operations,
Secretlint 14, exclusive creation in the manual no-Scriv branch, fail-closed
Git observation, literal-preserving release formatting, same-run Pages
artifact selection and dependency-maintenance checks. Current hosted Python
run 44857673625 fails at full-log lines 153 and 158–160 with PyPI
422 invalid-publisher. Its terminal status at lines 2393–2394 names
release-preflight; the status input reports successful lint, test and build.
The log specifies the required trusted-publisher claims and project setting.
This owner configuration is unresolved.

The Scriv branch still selects a fragment by newest modification time across
the entire directory. The actual Python function overwrites an unrelated
future-dated entry while leaving the new draft untouched. Four maintained
tests use a finite mocked Scriv producer and a disposable candidate that
selects a unique newly created path. The tests cover prior-file preservation,
no-created-file and concurrent-creation refusal, and ordinary success controls.
Python runs with 128 MiB, three CPU seconds and a five-second outer limit.
Scriv itself was not installed or executed. The upstream report is
https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/165 . No upstream repair has been applied.

The old actionlint 1.7.12 rejection of the root-dollar action reference is a
tool false positive. The official GitHub workflow-syntax documentation
explicitly defines the self repository prefix, resolves it at the running
commit, requires no checkout, and forbids a ref suffix. The controlled
Ubuntu 24.04 examples isolate the old parser rejection from runner-label
support. No template defect was reported for valid syntax. The fetched
primary page, exact relevant paragraphs and both parser controls are retained:
https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#example-using-an-action-in-the-same-repository-as-the-workflow-at-the-running-commit-recommended

The PHP Process defects are now reported at https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/33 .
The seven new PHP/Python observations are maintained executable tests, not
upstream repairs. Every supplied command, patch and conclusion receives zero
independent Formal AI authorship or reasoning credit.

The earlier PHP publication checks pass five valid repository gates and the
actual Secretlint 14 engine on all 28 changed files. The attempted gate name
check-tests-as-documentation is invalid and receives no pass credit.
The underlying node scripts/check-tests-as-docs.mjs checker passes separately;
the native twin parity wrapper was not run locally.

Evidence is retained under experiments/issue-1580/evidence/checks/current-template-findings . All raw and archive
SHA256 values are checked. A full issue conclusion is not implied.
PR: https://github.com/link-assistant/formal-ai/pull/1581
