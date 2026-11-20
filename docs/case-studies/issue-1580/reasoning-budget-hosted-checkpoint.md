# Hosted reasoning and release registry replay

Published head `f4588ce40023a1cc722cbed4ed7f977581fd4530` finished all 27
observed workflows: 26 succeeded and one was explicitly skipped. The pipeline
[run 44785531588](https://github.com/link-assistant/formal-ai/actions/runs/44785531588)
passed its ten native jobs: 573 Rust cases and eight Python CLI cases passed.
The reasoning halves each exercised 74 cases, taking 8m18s and 11m02s. The
release-registry target exercised 3 library, 22 requirements, 19 ordered-list
and 4 self-hosting cases, then eight tests invoking the actual release scripts.
These include two previously imported module-path cases; counts alone do not
prove unchanged acceptance of every historical source. Strict original source
acceptance remains 31/53, with the other contract qualifications unfinished.

The exact-head check snapshot records 57 passing and 86 explicitly skipped
checks. Manual read-only security producer
[run 44785512004](https://github.com/link-assistant/formal-ai/actions/runs/44785512004)
also passed. All 290 original collection streams have compressed and raw hashes,
byte counts and line counts in `native-families-f4588ce4/records.json`. Individual
native summaries and original line numbers are in `assessment.json`. The
aggregate logs are preserved; they have not all been literally read.

Green jobs do not clear the warning and error requirements. The annotation
inventory retains 121 warnings and six errors. Of those, 119 warnings concern
source file sizes. The other warnings concern unverified crates.io publish-token
capability and checkout cleanup. The deliberate registry-outage negative control
emitted one expected daemon error. Hive-1's first partial checkout emitted a
20-second low-speed timeout and related Git errors at pipeline log lines
44885–44916; its existing second checkout recreated the incomplete repository
after the cleanup diagnostics at lines 44974–44978 and succeeded. This is
observed recovery, not a claim that an upstream network fault was repaired.

The macro observation still emitted its plain-text heap warning at 3 GiB even
though both original producer SARIF files contained zero findings and zero
warning/error notifications. The separate 4 GiB change requires another real
producer run. File-size remediation, publish capability, strict source coverage,
per-child macro semantics, actual publication and all eight full template
comparisons remain open. This checkpoint supplies no independent authorship or
reasoning credit and does not complete issue #1580.
