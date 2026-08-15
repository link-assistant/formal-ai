# Issue #933 E81: CI check enforcing >=5 wording variations per language for every conversational test case (issues #103/#123/#134)

created: 2026-08-04T13:49:26Z

**Problem statement.**
Source: #123 comment (https://github.com/link-assistant/formal-ai/issues/123#issuecomment-4485896648). konard: "we should not stop until each test will have at least 5 variations per 4 languages, also I think we should have CI/CD checks to enforce that." PR #124 fixed the specific reported prompts, but the CI/CD enforcement was never delivered.
Current-code evidence: demo.spec.js:324 resolves example prompts through the real worker; check:language-parity and check:intent-coverage CI checks exist — but no check enforces "at least 5 wording variations per language" as a per-test-case floor.

**What to do.**
1. Define a machine-checkable convention for "wording variation" (e.g. a naming/tagging scheme in test fixtures, or a manifest listing prompt-variant groups per test case).
2. Write a CI script (in the style of check-language-parity) that walks the conversational test corpus and fails if any test case has fewer than 5 variations in any of en/ru/hi/zh.
3. Backfill variations for existing under-covered test cases until the new check passes.
4. Wire the check into release.yml.

**How to test.**
- Automated: the new CI script itself, plus a unit test on the script's counting logic using fixture data engineered to trip the floor.
- Manual: intentionally reduce one test case to 4 variations in one language and confirm the CI check fails locally.
- Multilingual: the check is inherently en/ru/hi/zh scoped; confirm coverage counts print per-language.
- Standing clauses: docs/case-studies/issue-{id}; single PR; verbose output listing exactly which test cases are under the floor.

**Source refs:** #123 (comment), follow-up #124. **Dedup:** none — check:language-test-coverage is a related but distinct existing check; confirm no overlap before implementing (note this in the case study).

