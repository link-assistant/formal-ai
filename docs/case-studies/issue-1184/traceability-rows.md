# Issue #1184 — Traceability Rows

Requirement shard: `docs/requirements/issue-1184-white-box-derivation.md`.
The traceability columns for the generated ledger (ID | Shard | Delivered |
Automated test | Manual confirmation) are, per row:

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R1184-1 | issue-1184-white-box-derivation | id scheme: yes (`derivation::answer_derivation_id`); `SymbolicAnswer` field is the wiring pass's one-line engine site | `every_symbolic_answer_carries_a_stable_derivation_id` | run `formal-ai chat` after wiring and confirm the answer surface shows the id |
| R1184-2 | issue-1184-white-box-derivation | yes (`Derivation::record_for`, stage kinds as public constants) | `derivation_carries_search_queries_fetches_and_hashes_for_an_online_answer` | — |
| R1184-3 | issue-1184-white-box-derivation | yes (`persist`/`load` under `data/cache/derivations/`) | `durable_record_round_trips_by_answer_id` | inspect one persisted `.lino` record after an online answer |
| R1184-4 | issue-1184-white-box-derivation | yes (`Command::Explain` + `cli_explain.rs`) | covered via R1184-3/R1184-5 through `explain_answer` | run `formal-ai explain <id>` on a returned answer |
| R1184-5 | issue-1184-white-box-derivation | yes | `explain_reports_not_recorded_for_stages_a_route_did_not_populate` | — |
| R1184-6 | issue-1184-white-box-derivation | yes (one `EventLog`, two projections) | `explain_and_thinking_trace_agree_on_the_same_event_log` | — |
| R1184-7 | issue-1184-white-box-derivation | yes | `explain_command_does_not_collide_with_the_self_explanation_recipe` | ask "explain how Formal AI works" in chat and confirm the recipe still routes |
| R1184-8 | issue-1184-white-box-derivation | with R1184-1's field (struct derives `Serialize`) | via R1184-1 | `formal-ai serve` + one chat completion shows `derivation_id` in the JSON |
| R1184-9 | issue-1184-white-box-derivation | open follow-up (parity pass) | — | — |
