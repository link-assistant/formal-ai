node_path=2.2.2.1.1

The `grep` command completed. Output:

```text
Found 100 matches
/tmp/tmp.nRipMVQxIr/CHANGELOG.md:
  Line 654:   `issue_914_case_study_and_planning_docs_are_traceable` (#914)
  Line 3189: - A traceability test (`issue_468_agentic_coding_case_study_is_traceable` in

/tmp/tmp.nRipMVQxIr/docs/requirements-traceability.md:
  Line 759: | R973-3 | 1856 | PR #974 (issue #973) | tests/issue_973_solve_flags.rs::every_published_solve_invocation_carries_both_evidence_flags; tests/issue_973_solve_flags.rs::the_case_study_records_the_unrecoverable_failure_and_the_fix | not yet confirmed beyond the two falsification runs above |

/tmp/tmp.nRipMVQxIr/tests/issue_973_solve_flags.rs:
  Line 240: fn the_case_study_records_the_unrecoverable_failure_and_the_fix() {
  Line 241:     let case_study = read("docs/case-studies/issue-973/README.md");
  Line 254:             case_study.contains(needle),

/tmp/tmp.nRipMVQxIr/docs/requirements/issue-0890-formal-proof-program-translation.md:
  Line 15: | R890-6 | Issue, PR, related-work, online-research, requirement, plan, and release evidence must remain traceable in the repository. | `issue_890_case_study_and_release_metadata_are_traceable` guards `docs/case-studies/issue-890`, this matrix, architecture, roadmap, and the minor changelog fragment. |

/tmp/tmp.nRipMVQxIr/docs/requirements/issue-0917-general-natural-formal-translation.md:
  Line 15: | R917-6 | Issue, PR, related-work, research, requirements, plan, architecture, roadmap, and release evidence must remain traceable. | `issue_917_case_study_and_release_metadata_are_traceable` guards `docs/case-studies/issue-917`, the root documents, raw snapshots, and the minor changelog fragment. |

/tmp/tmp.nRipMVQxIr/docs/requirements/issue-0914-vision-implementation-planning-coding-first.md:
  Line 16: | R914-2 | First update documentation to fully track implementation progress of all requirements. | Implemented by the ninth-pass audit in `ROADMAP.md` (2026-08-03) and this table; guarded by `issue_914_case_study_and_planning_docs_are_traceable`. |

/tmp/tmp.nRipMVQxIr/tests/issue_885_docs.rs:
  Line 171:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 174:         &case_study,
  Line 194:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 198:         case_study.contains(agent_leaf.trim()),
  Line 242: fn issue_case_study_preserves_requirements_research_and_solution_artifacts() {
  Line 243:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 246:         &case_study,
  Line 293:     let case_study = read("docs/case-studies/issue-885/README.md");
  Line 294:     assert!(case_study.contains("repository-audit-summary.md"));

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_563.rs:
  Line 62:     let case_study = read(root.join("docs/case-studies/issue-563/README.md"));
  Line 65:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_657.rs:
  Line 18:     let case_study = read("docs/case-studies/issue-657/README.md");
  Line 29:             case_study.contains(expected),

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_1021.rs:
  Line 87: fn the_case_study_records_the_data_the_analysis_rests_on() {
  Line 89:     let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
  Line 92:         &case_study,
  Line 143:     let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
  Line 250:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_531.rs:
  Line 5: fn issue_531_pattern_inference_case_study_is_traceable() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_922.rs:
  Line 5: fn issue_922_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_712.rs:
  Line 5: fn issue_712_case_study_and_semantic_routing_contract_are_traceable() {
  Line 11:     let case_study = read("docs/case-studies/issue-712/README.md");
  Line 20:         assert!(case_study.contains(expected), "missing {expected}");

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_890.rs:
  Line 8: fn issue_890_case_study_and_release_metadata_are_traceable() {
  Line 49:     let case_study = read(root.join("docs/case-studies/issue-890/README.md"));
  Line 52:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/issue_933_self_authoring.rs:
  Line 166:     let case_study = read("docs/case-studies/issue-933/README.md");
  Line 170:         assert!(case_study.contains(&id), "case study is missing {id}");

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_659.rs:
  Line 13:     let case_study = read("docs/case-studies/issue-659/README.md");
  Line 24:             case_study.contains(expected),

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_858.rs:
  Line 71: fn the_case_study_preserves_the_original_and_live_before_after_evidence() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_451.rs:
  Line 65:     let case_study = read(root.join("docs/case-studies/issue-451/README.md"));
  Line 68:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_526.rs:
  Line 73:     let case_study = read(root.join("docs/case-studies/issue-526/README.md"));
  Line 76:         &case_study,

/tmp/tmp.nRipMVQxIr/scripts/check-file-size.rs:
  Line 669:     fn check_directory_does_not_measure_quoted_case_study_workflows() {
  Line 671:         let case_study = repo.join("docs/case-studies/issue-561/template-comparison/js");
  Line 672:         fs::create_dir_all(&case_study).unwrap();
  Line 674:             &case_study.join("release.yml"),

/tmp/tmp.nRipMVQxIr/docs/case-studies/issue-834/test-logs/full-suite.log:
  Line 1311: test ci_cd::check_file_size::tests::check_directory_does_not_measure_quoted_case_study_workflows ... ok
  Line 1474: test docs_requirements_issue_468::issue_468_agentic_coding_case_study_is_traceable ... ok
  Line 1482: test docs_requirements_issue_558::issue_558_auto_learning_case_study_is_traceable ... ok
  Line 1507: test docs_requirements_issue_649::issue_649_world_model_case_study_documents_are_present_and_traceable ... ok
  Line 1512: test docs_requirements_issue_712::issue_712_case_study_and_semantic_routing_contract_are_traceable ... ok
  Line 1527: test docs_requirements_issue_686::issue_686_associative_persistence_case_study_documents_are_present_and_traceable ... ok
  Line 1818: test issue_661_repository_audit::committed_case_study_is_a_byte_replay_of_the_generalized_core ... ok

/tmp/tmp.nRipMVQxIr/docs/case-studies/issue-834/test-logs/check-file-size.log:
  Line 2: [omitted columns 1..21036 of line 2] ... hub_workflows() {\n        let repo = temp_dir(\"github-workflows\");\n        let workflows = repo.join(\".github/workflows\");\n        fs::create_dir_all(&workflows).unwrap();\n        write_file_with_lines(\n            &workflows.join(\"release.yml\"),\n            WORKFLOW_YAML_LIMIT.max_lines + 1,\n        );\n\n        let result = check_directory(&repo);\n\n        assert_eq!(\n            result.violations,\n            vec![Finding {\n                file: \".github/workflows/release.yml\".to_string(),\n                lines: WORKFLOW_YAML_LIMIT.max_lines + 1,\n                max_lines: WORKFLOW_YAML_LIMIT.max_lines,\n                warn_lines: WORKFLOW_YAML_LIMIT.warn_lines,\n                label: WORKFLOW_YAML_LIMIT.label,\n            }]\n        );\n    }\n\n    /// Case studies quote other projects' pipelines verbatim as evidence;\n    /// trimming them to our ceiling would destroy what they document.\n    #[test]\n    fn check_directory_does_not_measure_quoted_case_study_workflows() {\n        let repo = temp_dir(\"case-study-workflows\");\n        let case_study = repo.join(\"docs/case-studies/issue-561/template-comparison/js\");\n        fs::create_dir_all(&case_study).unwrap();\n        write_file_with_lines(\n            &case_study.join(\"release.yml\"),\n            WORKFLOW_YAML_LIMIT.max_lines + 1,\n        );\n\n        let result = check_directory(&repo);\n\n        assert_eq!(result.violations, Vec::new());\n        assert_eq!(result.warnings, Vec::new());\n    }\n\n    #[test]\n    fn check_directory_skips_generated_wikidata_cache() {\n        let repo = temp_dir(\"wikidata-cache\");\n        let cache_dir = repo.join(\"data/cache/wikidata\");\n        fs::create_dir_all(&cache_dir).unwrap();\n        let lino_limit = FILE_LIMITS[1];\n        write_lino_file_with_lines(&cache_dir.join(\"Q1860.lino\"), lino_limit.max_lines + 1);\n\n        let result = check_directory(&repo);\n\n        assert_eq!(result.violations, Vec::new());\n      ... [omitted columns 23037..24496 of line 2]

/tmp/tmp.nRipMVQxIr/docs/case-studies/issue-844/test-logs/unit-issue-844.txt:
  Line 3: test docs_requirements_issue_844::the_case_study_explains_the_merge_its_production_boundaries_and_the_defects_it_uncovered ... ok

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_540.rs:
  Line 95:     let case_study = read(root.join("docs/case-studies/issue-540/README.md"));
  Line 98:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_909.rs:
  Line 131:     let case_study = read(root.join("docs/case-studies/issue-909/README.md"));
  Line 134:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_918.rs:
  Line 7: fn issue_918_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_914.rs:
  Line 5: fn issue_914_case_study_and_planning_docs_are_traceable() {
  Line 51:             "issue_914_case_study_and_planning_docs_are_traceable",

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_923.rs:
  Line 8: fn issue_923_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_917.rs:
  Line 7: fn issue_917_case_study_and_release_metadata_are_traceable() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_660.rs:
  Line 13:     let case_study = read("docs/case-studies/issue-660/README.md");
  Line 24:             case_study.contains(expected),

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_558.rs:
  Line 5: fn issue_558_auto_learning_case_study_is_traceable() {

/tmp/tmp.nRipMVQxIr/tests/unit/issue_661_repository_audit.rs:
  Line 405: fn committed_case_study_is_a_byte_replay_of_the_generalized_core() {

/tmp/tmp.nRipMVQxIr/tests/unit/issue_848_coding_ladder.rs:
  Line 616: fn case_study_and_release_trace_every_issue_848_acceptance_boundary() {
  Line 621:     let case_study = read("docs/case-studies/issue-848/README.md");
  Line 633:         assert!(case_study.contains(evidence), "missing {evidence}");

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_709.rs:
  Line 75: fn case_study_release_and_agent_authorship_evidence_are_committed() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_468.rs:
  Line 5: fn issue_468_agentic_coding_case_study_is_traceable() {
  Line 34:     let case_study = read(root.join("docs/case-studies/issue-468/README.md"));
  Line 37:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements.rs:
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

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_844.rs:
  Line 34: fn the_case_study_explains_the_merge_its_production_boundaries_and_the_defects_it_uncovered() {

/tmp/tmp.nRipMVQxIr/tests/unit/ci-cd/issue_846.rs:
  Line 102: fn case_study_preserves_the_incident_and_complete_template_audit() {

/tmp/tmp.nRipMVQxIr/tests/unit/ci-cd/issue_1021.rs:
  Line 166: fn no_case_study_evidence_is_hidden_from_the_repository_by_gitignore() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements_issue_698.rs:
  Line 8: fn issue_698_case_study_and_external_benchmark_contract_are_traceable() {

/tmp/tmp.nRipMVQxIr/tests/unit/docs_requirements/benchmarks.rs:
  Line 60:     let case_study = super::read(root.join("docs/case-studies/issue-408/README.md"));
  Line 63:         &case_study,

/tmp/tmp.nRipMVQxIr/tests/unit/issue_905.rs:
  Line 389: fn issue_905_case_study_and_self_authorship_are_preserved() {

(Results are truncated. Consider using a more specific path or pattern.)
```
