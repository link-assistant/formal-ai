# Compact inputs and useful coding work

Formal AI should usually deliver more useful coding work than the external
instructions needed to request it. This applies to ordinary coding and coding
Formal AI itself. Correct behavior, focused scope and maintainable changes
come first. Deletion, refactoring and small fixes can be valuable with zero
net growth. Mathematics and other short-answer tasks have no growth floor.

This is R1188-U31 through R1188-U35 in
[the requirement shard](../../requirements/issue-1188-coding-amplification.md).

## What to measure

Keep the original natural-language task unchanged in the evidence. Record
all attempts, retry instructions, supplied source, reviewed patches, tool
receipts and actual before/after files. Record actual model token usage,
priced usage and Formal AI runtime/resources when those are available.
Unknown usage and costs remain unknown; byte counts are not token counts.

`experiments/formal_ai_subagent/coding-amplification.mjs` accepts exact source
snapshots and an independent verifier. The verifier must run acceptance
checks against the final source and return actual check exits and matching
before/after SHA-256 bindings. A solver's final answer is not verification.
The initial code proxy supports JavaScript only and rejects other languages
rather than inventing comparable numbers.

Report both autonomous validated net code bytes / original task bytes and
that same numerator / all observed input bytes. The latter includes retries,
repository reads, supplied patches and tool receipts. Preserve the separate
categories so a short request followed by a long externally authored patch
cannot be presented as cheap autonomous coding. A reviewed patch or generator
has zero autonomous numerator, even if its resulting code passes all checks.

The byte proxy sums non-comment lexical code, subtracts the initial code and
clamps negative growth at zero. Literal payload length, comments and formatting
do not count. Generated mirrors, captured evidence and documentation are
excluded. Report removed code separately. This deliberately undercounts some
valuable work and cannot prove that added code is necessary or well designed.
Reviewers must reject redundant statements, unused functions, repeated
wrappers and weak tests; a favorable ratio never overrides a failed check.

## CI and development practice

The existing JavaScript CI suite discovers
`rust/tests/web/coding-amplification.test.mjs`. Its assertions bind verification
to exact sources, reject failed checks and false autonomous attribution,
exclude common padding, preserve retry costs and count failed self-coding
attempts in the cohort denominator. These checks verify instrumentation;
they do not certify a representative autonomous coding success rate.

Keep a fixed cohort of real ordinary coding and self-coding tasks with
independent behavioral acceptance, plus held-out changes. Bind every declared cohort run to a unique stable identity, retain thrown
verifiers as measured failures, reject duplicates or missing runs, and include every
attempt and report success rate and the share with amplification above one.
"Usually" means a majority of the declared cohort, reported separately for
self-coding. Establish real baselines before introducing a reviewed ratchet.
Do not expand generated output to make that baseline pass, cherry-pick
successful tasks, discard failed retries or require every repair to grow.

Give Formal AI the desired behavior and let it inspect and derive the work.
If it fails, preserve the original request, repair the general capability and
retry unchanged. Reviewed repair programs remain separately attributed.
Cache learned contracts and reusable knowledge with valid source identities.
Amortized benefit must state its repeated-task cohort and account for initial
learning and repair costs; tool-call counts alone do not establish savings.

## Current observation

The existing broad self-coding suite has three passing fixture controls and zero of seven requested feature-composition checks. No feature was synthesized by that attempt. In T2639 the new
compact-input requirement itself was attempted through the real JavaScript
planner: two reads, no writes, and failure on the absent target module. The
reviewed instrumentation integration does not close that autonomous task.
No representative amplification baseline or monetary saving is claimed.

A separate predeclared twenty-task JavaScript calculation cohort retains all original prompts, source identities, full replies and independent verifier failures in [the finite followup packet](../../../experiments/formal_ai_subagent/evidence/coordinator-1188/ci-020be4-finite-followup/README.md). All twenty raw replies are shorter than their requests, yet none follows the requested numeric-only response format: strict acceptance is0/20. No reply was trimmed or replaced. This limited worker-surface observation establishes neither representative task performance nor coding amplification; source-declaration identity, supplied harness costs and unknown model usage are retained separately. Size relation cannot replace task acceptance.

## Expected relation by task category

Classify the task before executing it and save the original task/category declaration
with the attempt receipt. Input size alone never establishes output quality.
A large repository-aware change can follow a concise request; an excellent repair
may delete code, reuse a function or change one character. A calculation's numeric
result compresses the problem, whereas a requested proof expands its explanation.
The classification follows the requested deliverable, rather than its broad subject.

| Task category | Expected useful output versus original task | Conditions and exceptions |
| --- | --- | --- |
| Feature implementation; independent test construction | Larger | New behavior and meaningful tests; exclude supplied patches and generated mirrors from autonomous net code. A feature completed mainly by reuse may be a valuable recorded exception. |
| Detailed explanation; mathematical proof | Larger | Only independently accepted content counts. A short proof or familiar explanation can be a valid exception. |
| Yes/no decision; numeric calculation | Smaller | Check the actual decision or value; a requested explanation/proof belongs to its separately declared category. An exact large integer (for example2^128) can legitimately exceed the request size. |
| Extraction; summarization | Smaller | Preserve required source facts and scope. Tiny inputs or requested detail can be valid exceptions. |
| Repair; refactoring; deletion | Unrestricted | Check resulting behavior and removed code without requiring positive net growth. |
| Translation | Unrestricted | Source/target languages and preservation constraints determine size; no universal direction. |

`experiments/formal_ai_subagent/task-relations.mjs` binds declarations to the
original task, stable identity, category and expected relation. A complete saved
cohort rejects changed categories, duplicate runs and omitted failures. For code,
it retains the independently validated autonomous net lexical proxy. For other
tasks, it requires independent checks bound to the exact UTF-8 response SHA-256.
Response length still needs content review; expanding prose does not prove usefulness.
The original-task ratio and ratio against every observed input are both reported.
These ratios cannot be compared across code proxies and response bytes as one metric.

“Almost always” is an improvement objective of at least95% accepted matching tasks
over at least20 declared attempts in each eligible category. Every failed attempt
counts against this share; unrestricted categories have no size-performance target.
CI currently checks this accounting and independent acceptance with fixtures. It does
not enforce an unobserved performance baseline. Record actual representative Formal AI
runs and exceptions before adopting a reviewed performance ratchet. This objective
never makes a correct small answer or focused repair unacceptable.
