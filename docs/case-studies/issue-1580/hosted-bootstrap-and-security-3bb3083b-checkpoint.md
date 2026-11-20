# Hosted bootstrap and security checkpoint on 3bb3083b

All 27 workflows on published head `3bb3083ba9d91dbb3b45386083875a1adb9a92f2`
are terminal: 26 succeed and one is explicitly skipped. Every one of the 296
collected command streams matches its original/compressed hash, byte count and
line count. The earlier stale Layered profile assertion no longer fails.

[CI/CD Pipeline 44764893448](https://github.com/link-assistant/formal-ai/actions/runs/44764893448)
passes 528 actual Rust cases across nine native jobs, with no Rust failures.
The release registry job also passes eight actual Python-driven CLI cases
(`job-136387247048.stdout.gz`, lines 757–759). This does not qualify changed
original fixtures as unchanged historical contracts or prove all three moved
script test modules execute in this focused job. Complete summary line numbers
and original job logs are retained in `native-families-3bb3083b/`.

[Workflows 44764893157](https://github.com/link-assistant/formal-ai/actions/runs/44764893157)
verifies two actual BuildKit daemon starts and mirror recovery through the real
official setup action. The wrapper/probe/workflow bytes match the published
head and tested merge; both containers bind to the same pinned image ID.
See [the detailed bootstrap checkpoint](buildkit-bootstrap-identity-checkpoint.md).
Its one error annotation is the deliberately injected canonical pull failure,
not an unexamined failure: full log lines 657–661 show the exact pin, injection
and expected canonical failure before the verified mirror start. No annotations
are removed or hidden. Application image publication remains unverified.

[Manual Security 44765053281](https://github.com/link-assistant/formal-ai/actions/runs/44765053281)
retains original CodeQL 2.27.5 Rust and Actions SARIF with zero findings and
zero warning/error notifications, including omitted-level warning semantics.
All 2,576 expected Rust paths are extracted; exact-message crosswalk verifies
all 105 formerly context-free paths. The 631 informational unresolved macro
records still need semantic qualification. The new local default-off observer
has not yet compiled or run on a hosted database. Original reports and identities
are in `codeql-producer-3bb3083b/`.

The fresh full workflow logs still emit 119 source-size warnings and one
crates.io token-certainty warning. Preserve these as unfinished requirements.
Earlier 122 warnings precede the three structural script splits; ceilings are
unchanged. Dependencies, original/budget contracts, Pages publication and served
identity, eight full template comparisons/practices, delivery/provenance and
complete pull-request review remain open. Strict original-byte replay evidence
stays 31/53 and independent accepted reasoning stays 0/20.

The next local repair batch is separately verified: three runtime regressions
fail on original bytes and pass after repair; four maintenance, 16 retry,
three actual HTTP and 19 security adapter groups pass. Five touched gates and
whitespace pass. See [the local repair checkpoint](macro-observation-and-runtime-budget-checkpoint.md).
The entire issue remains unfinished and [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
stays draft and unmerged.
