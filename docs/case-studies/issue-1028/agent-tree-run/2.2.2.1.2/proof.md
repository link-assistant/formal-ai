node_path=2.2.2.1.2

The `grep` command completed. Output:

```text
Found 100 matches
/tmp/tmp.VmbloAJ0sk/CHANGELOG.md:
  Line 654:   `issue_914_case_study_and_planning_docs_are_traceable` (#914)
  Line 3189: - A traceability test (`issue_468_agentic_coding_case_study_is_traceable` in

/tmp/tmp.VmbloAJ0sk/scripts/check-file-size.rs:
  Line 669:     fn check_directory_does_not_measure_quoted_case_study_workflows() {
  Line 671:         let case_study = repo.join("docs/case-studies/issue-561/template-comparison/js");
  Line 672:         fs::create_dir_all(&case_study).unwrap();
  Line 674:             &case_study.join("release.yml"),

/tmp/tmp.VmbloAJ0sk/tests/issue_973_solve_flags.rs:
  Line 240: fn the_case_study_records_the_unrecoverable_failure_and_the_fix() {
  Line 241:     let case_study = read("docs/case-studies/issue-973/README.md");
  Line 254:             case_study.contains(needle),

/tmp/tmp.VmbloAJ0sk/tests/issue_885_docs.rs:
  Line 171:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 174:         &case_study,
  Line 194:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 198:         case_study.contains(agent_leaf.trim()),
  Line 242: fn issue_case_study_preserves_requirements_research_and_solution_artifacts() {
  Line 243:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 246:         &case_study,
  Line 293:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 294:     assert!(case_study.contains("repository-audit-summary.md"));

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_563.rs:
  Line 62:     let case_study = read(root.join("docs/case-studies/issue-563/README.md"));
  Line 65:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_657.rs:
  Line 18:     let case_study = read("docs/case-studies/issue-657/README.md");
  Line 29:             case_study.contains(expected),

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_1021.rs:
  Line 87: fn the_case_study_records_the_data_the_analysis_rests_on() {
  Line 89:     let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
  Line 92:         &case_study,
  Line 143:     let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
  Line 250:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_531.rs:
  Line 5: fn issue_531_pattern_inference_case_study_is_traceable() {

/tmp/tmp.VmbloAJ0sk/dev/log/issues/1014/pulls/1015/raw-data/related/merged-ci-cd-prs.json:
  Line 1: [omitted columns 1..131370 of line 1] ... -fresh-merge.sh`, with a reproduction, a workaround and a suggested fix. (The rust template's copy quotes correctly; python does not ship the script.)\n- link-foundation/python-ai-driven-development-pipeline-template#33 and #35 — already open upstream, so recorded rather than re-filed.\n\n## Verification\n\n- `cargo test --test unit` → **1953 passed, 0 failed** (6 new regression tests)\n- `actionlint -shellcheck` over every workflow → exit 0 (type-checking confirmed live by injecting a bogus `needs.secrets-scanX` into a copy and observing the error)\n- `shellcheck --severity=warning` over the 17 shipped scripts → exit 0\n- `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `rust-script scripts/check-file-size.rs` → exit 0\n\nNew tests: `releases_do_not_publish_past_a_failing_secrets_scan_or_e2e_suite`, `lint_job_gates_on_workflow_shell_and_clippy_findings`, `check_directory_measures_github_workflows`, `check_directory_does_not_measure_quoted_case_study_workflows`, plus the ratchet exclusion and differential-gate tests.\n\n## Debug output (default off, per the issue)\n\n- `FORMAL_AI_MACOS_SIGN_DEBUG=1` — per-path `[adhoc-sign-mac]` sign decisions; its *absence* from the failing log is what proved the healing-build diagnosis.\n- `experiments/self_hosting_ratchet_replay/replay.py` — per-commit attribution, reason and subject for any range, without `rust-script`.\n\n## Second pass: defects found by running the fixed pipeline\n\nRun [29767811026](https://github.com/link-assistant/formal-ai/actions/runs/29767811026) was the first full run of the repaired pipeline, and it found two more — one introduced by this PR's own first commit.\n\n**`Test (ubuntu-latest)` reported failure with 1953 tests passed and 0 failed.** The suite finished at 18:43:50; `timeout-minutes: 15` killed the job 1.1 seconds later, during teardown. Run 29749095334 on `main` had already done exactly this and was never diagnosed, because GitHub renders a timeout as  ... [omitted columns 133371..371420 of line 1]

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_922.rs:
  Line 5: fn issue_922_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_686.rs:
  Line 5: fn issue_686_associative_persistence_case_study_documents_are_present_and_traceable() {
  Line 62:     let case_study = read(root.join("docs/case-studies/issue-686/README.md"));
  Line 65:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_712.rs:
  Line 5: fn issue_712_case_study_and_semantic_routing_contract_are_traceable() {
  Line 11:     let case_study = read("docs/case-studies/issue-712/README.md");
  Line 20:         assert!(case_study.contains(expected), "missing {expected}");

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_890.rs:
  Line 8: fn issue_890_case_study_and_release_metadata_are_traceable() {
  Line 49:     let case_study = read(root.join("docs/case-studies/issue-890/README.md"));
  Line 52:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/issue_933_self_authoring.rs:
  Line 166:     let case_study = read("docs/case-studies/issue-933/README.md");
  Line 170:         assert!(case_study.contains(&id), "case study is missing {id}");

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_659.rs:
  Line 13:     let case_study = read("docs/case-studies/issue-659/README.md");
  Line 24:             case_study.contains(expected),

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_858.rs:
  Line 71: fn the_case_study_preserves_the_original_and_live_before_after_evidence() {

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_451.rs:
  Line 65:     let case_study = read(root.join("docs/case-studies/issue-451/README.md"));
  Line 68:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_526.rs:
  Line 73:     let case_study = read(root.join("docs/case-studies/issue-526/README.md"));
  Line 76:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/integration/issue_703_orchestration_followup.rs:
  Line 39:     let case_study = include_str!("../../docs/case-studies/issue-703/README.md");
  Line 49:     assert!(case_study.contains("ses_04e25ba4cffeibfMekv188DNLX"));

/tmp/tmp.VmbloAJ0sk/REQUIREMENTS.md:
  Line 1702: | R890-6 | Issue, PR, related-work, online-research, requirement, plan, and release evidence must remain traceable in the repository. | `issue_890_case_study_and_release_metadata_are_traceable` guards `docs/case-studies/issue-890`, this matrix, architecture, roadmap, and the minor changelog fragment. |
  Line 1825: | R914-2 | First update documentation to fully track implementation progress of all requirements. | Implemented by the ninth-pass audit in `ROADMAP.md` (2026-08-03) and this table; guarded by `issue_914_case_study_and_planning_docs_are_traceable`. |
  Line 1854: | R917-6 | Issue, PR, related-work, research, requirements, plan, architecture, roadmap, and release evidence must remain traceable. | `issue_917_case_study_and_release_metadata_are_traceable` guards `docs/case-studies/issue-917`, the root documents, raw snapshots, and the minor changelog fragment. |

/tmp/tmp.VmbloAJ0sk/docs/requirements-traceability.md:
  Line 759: | R973-3 | 1856 | PR #974 (issue #973) | tests/issue_973_solve_flags.rs::every_published_solve_invocation_carries_both_evidence_flags; tests/issue_973_solve_flags.rs::the_case_study_records_the_unrecoverable_failure_and_the_fix | not yet confirmed beyond the two falsification runs above |

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_540.rs:
  Line 95:     let case_study = read(root.join("docs/case-studies/issue-540/README.md"));
  Line 98:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_909.rs:
  Line 131:     let case_study = read(root.join("docs/case-studies/issue-909/README.md"));
  Line 134:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_918.rs:
  Line 7: fn issue_918_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_914.rs:
  Line 5: fn issue_914_case_study_and_planning_docs_are_traceable() {
  Line 51:             "issue_914_case_study_and_planning_docs_are_traceable",

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_923.rs:
  Line 8: fn issue_923_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_917.rs:
  Line 7: fn issue_917_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_660.rs:
  Line 13:     let case_study = read("docs/case-studies/issue-660/README.md");
  Line 24:             case_study.contains(expected),

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_558.rs:
  Line 5: fn issue_558_auto_learning_case_study_is_traceable() {

/tmp/tmp.VmbloAJ0sk/tests/unit/issue_661_repository_audit.rs:
  Line 405: fn committed_case_study_is_a_byte_replay_of_the_generalized_core() {

/tmp/tmp.VmbloAJ0sk/tests/unit/issue_848_coding_ladder.rs:
  Line 616: fn case_study_and_release_trace_every_issue_848_acceptance_boundary() {
  Line 621:     let case_study = read("docs/case-studies/issue-848/README.md");
  Line 633:         assert!(case_study.contains(evidence), "missing {evidence}");

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_709.rs:
  Line 75: fn case_study_release_and_agent_authorship_evidence_are_committed() {

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_468.rs:
  Line 5: fn issue_468_agentic_coding_case_study_is_traceable() {
  Line 34:     let case_study = read(root.join("docs/case-studies/issue-468/README.md"));
  Line 37:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements.rs:
  Line 51:     let case_study = read(root.join("docs/case-studies/issue-12/README.md"));
  Line 54:         &case_study,
  Line 104:     let case_study = read(root.join("docs/case-studies/issue-16/README.md"));
  Line 107:         &case_study,
  Line 172:     let case_study = read(root.join("docs/case-studies/issue-103/README.md"));
  Line 175:         &case_study,
  Line 208:     let case_study = read(root.join("docs/case-studies/issue-117/README.md"));
  Line 211:         &case_study,
  Line 286:     let case_study = read(root.join("docs/case-studies/issue-115/README.md"));
  Line 289:         &case_study,
  Line 446:     let case_study = read(root.join("docs/case-studies/issue-207/README.md"));
  Line 449:         &case_study,
  Line 509:     let case_study = read(root.join("docs/case-studies/issue-195/README.md"));
  Line 512:         &case_study,
  Line 613:     let case_study = read(root.join("docs/case-studies/issue-438/README.md"));
  Line 616:         &case_study,

/tmp/tmp.VmbloAJ0sk/tests/unit/docs_requirements_issue_844.rs:
  Line 34: fn the_case_study_explains_the_merge_its_production_boundaries_and_the_defects_it_uncovered() {

/tmp/tmp.VmbloAJ0sk/dev/log/issues/1014/pulls/1015/ci-logs/pushed-head-c5fae9d4/Coverage-31897604157.log:
  Line 4231: Code Coverage	Generate code coverage	2026-08-15T17:20:21.8093937Z test issue_case_study_preserves_requirements_research_and_solution_artifacts ... ok
  Line 4283: Code Coverage	Generate code coverage	2026-08-15T17:20:31.8523426Z test the_case_study_records_the_unrecoverable_failure_and_the_fix ... ok
  Line 4894: Code Coverage	Generate code coverage	2026-08-15T17:20:54.6318524Z test ci_cd::check_file_size::tests::check_directory_does_not_measure_quoted_case_study_workflows ... ok
  Line 5005: Code Coverage	Generate code coverage	2026-08-15T17:20:55.9267083Z test ci_cd::issue_846::case_study_preserves_the_incident_and_complete_template_audit ... ok
  Line 5152: Code Coverage	Generate code coverage	2026-08-15T17:21:01.5269541Z test docs_requirements_issue_468::issue_468_agentic_coding_case_study_is_traceable ... ok
  Line 5159: Code Coverage	Generate code coverage	2026-08-15T17:21:01.6199777Z test docs_requirements_issue_531::issue_531_pattern_inference_case_study_is_traceable ... ok
  Line 5161: Code Coverage	Generate code coverage	2026-08-15T17:21:01.6907297Z test docs_requirements_issue_558::issue_558_auto_learning_case_study_is_traceable ... ok
  Line 5185: Code Coverage	Generate code coverage	2026-08-15T17:21:01.8549652Z test docs_requirements_issue_649::issue_649_world_model_case_study_documents_are_present_and_traceable ... ok
  Line 5190: Code Coverage	Generate code coverage	2026-08-15T17:21:01.9149755Z test docs_requirements_issue_686::issue_686_associative_persistence_case_study_documents_are_present_and_traceable ... ok

(Results are truncated. Consider using a more specific path or pattern.)
```
