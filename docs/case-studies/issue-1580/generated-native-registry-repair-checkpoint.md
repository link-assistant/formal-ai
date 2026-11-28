# Generated native source witness repair

At published head `27e895c1103926b4f73862c0eb15fa77fb505089`,
[Layered CI 44860043349](https://github.com/link-assistant/formal-ai/actions/runs/44860043349)
fails its existing generated-program currency assertion. Full-log lines
7015–7023 and 8073–8102 identify the generator check, rather than a native
compiler failure. The original 12,774-line log remains retained; only targeted
contexts have been literally reviewed.

The previous current-source QA repair changed `rust/src/history_context.rs`.
Two generated source witnesses still referenced its earlier digest. The
maintained generator's `--check` reproduces the failure before repair. Its
normal generation changes exactly those two hashes in
`scripts/lib/generated-native-programs.mjs`, with no program or public-root
change. An exact before/after substitution proof and original registry are
retained. The existing generated-program suite then passes all five cases,
including currency, substitutions, literal refusal and semantic contracts.
The focused run takes 16.272 seconds, with a 256 MiB JavaScript heap.

An initial search incorrectly required a match; another helper read its
dependency before the earlier command completed. A provisional public-root
hypothesis was rejected by the actual diff. The first archive helper also
expected an absent summary key before reading the retained record structure.
These failed helper attempts are
retained separately. They provide no product-failure or independent-reasoning
credit. The local repair commit is
`858943eb676fe2e68dc51b7cccefa157169c21a0`; it has not been pushed while the
remaining workflows on the published head are running. Hosted repair
qualification, full template practice comparison and the wider issue remain
unfinished. No native build or execution was performed locally.

Evidence: `experiments/issue-1580/evidence/checks/current-native-registry-repair/`.
