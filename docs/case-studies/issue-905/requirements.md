# Issue 905 acceptance traceability

| Requirement | Implementation | Executable evidence |
| --- | --- | --- |
| Write exactly `Hello World`, without the control word `exactly:`. | `strip_clause_lead` drops clause separators and seed-defined `file_write_content_qualifier` adverbs from the recovered span, but only when a separator marks the adverb as introducing the payload. | `exact_modifier_is_not_written_as_part_of_the_payload`; twenty-case parsing matrix |
| Treat any explicit tool error as a failed step. | Preserve `ChatMessage::is_error`; normalize `is_error`, `isError`, failed status, nonzero exits, and raw exit markers. | explicit-error and nonzero-exit regressions |
| Preserve provider error signals across protocol surfaces. | Anthropic `tool_result.is_error` and Responses failure status project into the shared chat message. | `anthropic_and_responses_adapters_preserve_error_metadata` |
| Do not advance after a failed tool call. | `Progress` records observed attempts separately from successful step satisfaction. | failed-write regression; failed-verification regressions |
| Recover the reported read-before-write policy. | First rejected write per path emits a read; either existing bytes or missing-file result permits one retry. | `failed_write_is_followed_by_read_then_one_write_retry` |
| Report what the workspace answered, not only what the transport said. | When recovery is exhausted, the plan still runs the check it named once; the report then carries that command and its exit status. | `an_unrecoverable_write_still_asks_the_workspace_before_reporting`; write-effect ladder rung R916-01 |
| Bound recovery instead of looping. | Failure counts are keyed by concrete write path; the second failed write terminates with observed detail. | `a_second_failed_write_for_the_same_path_stops_retrying` |
| Never claim success after failed verification. | An active run failure renders a seeded failure report instead of reaching completion. | Qwen-style explicit error and Codex-style nonzero exit tests |
| Require evidence to match expected content. | Literal completion compares transport-normalized command output with the request-derived payload. | wrong-evidence and matching-evidence tests |
| Do not mistake requested words such as `failed` for transport failure. | Evidence comparison uses successful transport output without prose inference. | `requested_failure_vocabulary_is_valid_successful_evidence` |
| Keep failure/mismatch responses multilingual and seed-backed. | Mismatch and unverified response meanings exist for en/ru/hi/zh/es; generated total closure is refreshed. | total-closure audit; five-language response test; diff-aware language coverage check |
| Preserve raw issue, review, and reproduction evidence. | Case study retains authenticated GitHub snapshots, both provider traces, red/green logs, and self-hosting traces. | `issue_905_case_study_and_self_authorship_are_preserved` |
| Execute at least one smallest same-task leaf through Formal AI and Agent CLI. | Session `ses_034e9dafeffe7nxeTkFhmHLmZN` authored and verified the canonical invariant; reviewed decomposition records 1/5 (20%). | byte-equality assertion and raw client/server traces |
