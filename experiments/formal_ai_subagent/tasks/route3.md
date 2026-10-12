TASK (tag ROUTE3): finish R1173-3 (docs/requirements/issue-1173-no-canned-search-answer.md). The routing-probe misroutes, from ROUTE2's report:

1. **p352, p355 (both runtimes).** "a program that prints <text>" matches no catalogued task.
   - Add a parameterised catalog task: the text to print is an operand, not a memorized program.
   - Read R1165-4/R1165-8 first. The no-memorization gate forbids verbatim per-language Hello World strings in data/, so the program must be composed from each language's grammar or print procedure, with the text substituted.
   - Keep `lino_parity.rs` (catalog tables in both runtimes) consistent.
2. **p223–p225 (native only).** The browser answers from the committed recurrence cache `js/source-cache/wikifunctions-recurrences.lino` through `trySourceRecurrenceSynthesis`; no Rust reads that cache.
   - Write the Rust reader as a twin of the JS one.
   - Feed it to `discover_and_compose` as a source candidate.
   - Rust's `verify` needs python3: check how CI provides it.
3. **p133 (native).** "Can I see who visited my GitHub repository?" goes to web_search in Rust and to github_repository_traffic in the browser. Compare the two dispatch orders: the browser runs `claimRouteRun("tryGithubRepositoryTraffic")` early in `formal_ai_worker_20.js`, while Rust checks it in `meta_method_dispatch.rs` around line 444. Make native match the browser, generally.
4. When CI reports the measured native values (the test prints corrected `rust_misroute` lines), LEAD records them. You state your expected values in your report.

Update R1173-3 honestly. Ledger rows T230–T239. Never run cargo; the routing probe Rust tests are confirmed by CI only.
