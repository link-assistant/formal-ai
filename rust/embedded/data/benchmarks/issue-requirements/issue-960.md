**Problem statement.**
Source: R222-1 (#222 PR comment, https://github.com/link-assistant/formal-ai/pull/222#issuecomment-4513844358), R234-2 (#234 PR comment, https://github.com/link-assistant/formal-ai/pull/234#issuecomment-4528554549), R234-4 (same thread). Three separate konard requirements each landed as a one-time practice but never got the CI/CONTRIBUTING.md enforcement he explicitly asked for:
1. "we should cache not more than 128 the most frequently used words ... each .lino file cannot be larger than 1500 lines." .lino <= 1500 IS enforced (scripts/check-file-size.rs in release.yml:390 + tests/unit/data_files.rs), but check-file-size.rs:57 excludes data/cache/wikidata/ from the gate, and MAX_SEED_RECORDS_PER_BUCKET=128 (src/translation/cache.rs:70) is a documented constant with no active enforcement — data/cache/wikidata/entity holds 394 entities, over 3x the cap.
2. "I need more detailed examples so tests are like docs ... we need a test or CI/CD rule that will guarantee it." The style is practiced (tests/unit/assistant_name.rs) but no test/CI rule enforces it repo-wide.
3. "Word Addresses is not recognized by GitHub as explicit link to the issue... will cause it to automatically close on pull request merge." Applied once (docs/case-studies/pull-request-234/ exists) but codified nowhere — CONTRIBUTING.md and .github/pull_request_template.md contain no linking guidance.

**What to do.**
1. Include data/cache/wikidata/ in the .lino line-count gate (or state explicitly, with reason, why it's exempted if there's a real constraint).
2. Add active enforcement of MAX_SEED_RECORDS_PER_BUCKET=128 — a CI check or test that fails if any cache bucket exceeds 128 records (currently data/cache/wikidata/entity fails this at 394).
3. Write a CI script that checks conversational/behavioral test files for the "exact example answer" style (not merely contains/not-contains assertions) — flag or fail on tests using only loose assertions where an exact-answer style is expected.
4. Add a CONTRIBUTING.md / .github/pull_request_template.md section codifying: PR descriptions must use GitHub's recognized "Fixes #N" / "Fixes <url>" syntax (never "Addresses"), and PR case studies go to docs/case-studies/pull-request-{id}.

**How to test.**
- Automated: (a) a test/CI failure demonstrating data/cache/wikidata/entity's 394-entity violation is caught, then fixed to <=128 (or bucketed); (b) the new tests-as-docs CI rule fails on a deliberately-loose test fixture and passes on an exact-answer one; (c) a CI check (or PR-template lint) confirming a PR body contains "Fixes #N"/"Fixes <url>", not "Addresses".
- Manual: open a test PR with an "Addresses #1" body and confirm the new check flags it.
- Standing clauses: docs/case-studies/issue-{id}; single PR.

**Source refs:** #222 (R222-1), #234 (R234-2, R234-4). **Dedup:** merged three related enforcement-debt requirements into one issue per the grouping instructions.

