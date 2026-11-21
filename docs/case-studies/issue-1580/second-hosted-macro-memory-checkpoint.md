# Second hosted macro memory checkpoint

The original reports from [Security run 44785512004](https://github.com/link-assistant/formal-ai/actions/runs/44785512004) bind to published `f4588ce40023a1cc722cbed4ed7f977581fd4530`. All 124 producer files are retained with exact byte and archive identities. Rust and Actions SARIF each contain zero findings and zero warning/error notifications. The Rust report still contains 631 informational macro notifications; individual semantic/dataflow qualification remains open.

The optional query actually compiles and executes, retaining 122,534 rows across 2,216 paths, with the same 42,427 outside-expanded, 79,476 inside-expanded and 631 unexpanded partition. All 631 original notification start positions match unexpanded contexts. This agreement does not prove every included child maps to its expanded semantic entity.

The 3 GiB request leaves a measured 1,698 MiB Java heap and still emits `Not caching stages during query-loading`. The previous 2 GiB request left 914 MiB and the same warning. The next source adjustment requests 4 GiB based on those observations. It preserves two threads, a 512 MiB disk cache, the four-minute step boundary, strict compiler warnings and the original SARIF acceptance gate. Actual hosted warning removal remains unverified; no local query or native compilation runs.

The updated adapter contract fails against the 3 GiB source. After the bounded adjustment, all 19 security groups and 18 production-profile controls pass. Complete original producer reports, local negative controls, passing transcripts and supplied scripts are retained in the companion evidence archives. A reference-reading command that the Formal AI driver did not execute verbatim receives no verification credit. Every supplied script, conclusion and source replacement receives zero independent authorship credit.

This is an intermediate checkpoint. Macro semantics, unchanged original-native acceptance, complete eight-template review, applicable runtime adoption, production publication/served identity, warning cleanup and whole-PR review remain unfinished. The PR stays draft and unmerged.
