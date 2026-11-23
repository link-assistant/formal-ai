# Link-check verdicts and report boundaries

The deployed workflow accepted HTTP 429 and every 5xx response, and accepted
timeouts. Its archive helper also treated missing reports and failed local links
as fully archived. Recovery could overlook a local failure beside a recovered
HTTP request, or a checker configuration failure. These outcomes could clear a
failed link check without a successful observation.

Reproduce with `node --test --test-concurrency=1
experiments/issue-1580/link-observation-boundaries.test.mjs`. The same final 33
controls run against an isolated copy of the original production files: 23 fail
and 10 pass. The repaired source passes all 33, plus 38 adjacent tests. Five
finite local HTTP cases exercise 408, 429, 500, 503 and 599, including recovery,
exhaustion and permanent 404/410 responses. Actual CLI controls verify missing,
empty, incomplete and malformed reports, checker exit codes, local links, mixed
HTTP/local failures, confirmed archives, missing archives and unavailable archive
observations. External services are replaced only at the transport boundary;
these controls do not establish current external availability.

The workflow now accepts only 200 through 204, retries transient responses within
the existing three-attempt/deadline limits, and accepts recovery only after a
successful response. Both helpers share report completeness and non-HTTP failure
validation. Only lychee exit 2 enters link recovery; other failed exits remain
workflow failures. Confirmed archive absence fails. An unavailable archive lookup
retains the existing warning policy and emits `all_archived=false`, rather than
claiming an archive exists. Mixed failures retain valid archive suggestions.

Comparison uses the current Rust template commit
`260ae6f826d6bff0cbfe2280098c287a4c09b3f4`: the two helper files and deployed
workflow retain their previously reviewed blob identities. Its report validation,
whole-report failure handling and exit-code guards are applicable and adopted.
Its unavailable-archive failure policy differs from Formal AI's existing warning
policy; the evidence value remains false here. The template's prior issues #136,
#220, #228 and #236 already address these boundaries, so no duplicate upstream
report is opened. This comparison qualifies this practice only, not all template
files or the complete issue.

The 71 focused tests and four touched JavaScript gates pass. Complete original,
candidate and pinned-template evidence is hash-checked and round-trip verified in
`experiments/issue-1580/evidence/checks/link-observation-boundaries/`. Hosted
verification remains pending the next combined push. Supplied controls and edits
receive zero independent authorship credit.
