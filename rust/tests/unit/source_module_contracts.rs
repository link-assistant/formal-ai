//! Unit coverage that used to sit inline in `src/` as `#[cfg(test)]` modules.
//!
//! Production code under `src/` carries no inline tests
//! (`ci_cd::source_test_placement`, issue #398 rule 7), so each module's own
//! checks live here, one nested module per source file, exercising only the
//! crate's public surface.

mod small_model_fallback {
    use formal_ai::event_log::EventLog;
    use formal_ai::small_model_fallback::{
        FormalizationCandidate, SmallModelOptions, catalog, confirm_proposal, eligible,
        propose_best_match, recommend,
    };

    #[test]
    fn default_options_show_and_load_nothing() {
        assert!(eligible(&SmallModelOptions::default()).is_empty());
        assert!(recommend(&SmallModelOptions::default()).is_none());
    }

    #[test]
    fn nothing_in_the_catalog_is_packaged() {
        for model in catalog() {
            assert!(
                !model.packaged,
                "{} must not ship in the package",
                model.model_id
            );
        }
    }

    #[test]
    fn hardware_fit_hides_oversized_models_and_sorts_by_rating() {
        let small_machine = SmallModelOptions::enabled_for(1024);
        let shown = eligible(&small_machine);
        assert!(
            !shown.is_empty(),
            "a 1 GB machine is offered the small tier"
        );
        assert!(shown.iter().all(|model| model.ram_required_mb <= 1024));
        assert!(
            !shown.iter().any(|model| model.model_id == "phi_3_5_mini"),
            "a 6 GB model must not be offered on a 1 GB machine"
        );
        let ratings: Vec<u8> = shown.iter().map(|model| model.rating).collect();
        let mut sorted = ratings.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(ratings, sorted, "sorted by public rating, best first");
    }

    #[test]
    fn the_proposal_is_advisory_and_the_rules_hold_the_pen() {
        let options = SmallModelOptions::enabled_for(1024);
        let candidates = vec![
            FormalizationCandidate {
                qid: "Q218593".into(),
                label: "inch".into(),
                description: "unit of length".into(),
            },
            FormalizationCandidate {
                qid: "Q11573".into(),
                label: "metre".into(),
                description: "SI unit of length".into(),
            },
        ];
        let mut log = EventLog::new();
        let proposal = propose_best_match(&options, "метр-класс: metre", &candidates, &mut log)
            .expect("a fitting model proposes");
        assert!(proposal.is_advisory());
        assert!(
            !confirm_proposal(&proposal, false),
            "rules disagree: discarded"
        );
        assert!(confirm_proposal(&proposal, true), "rules agree: confirmed");
        assert!(log.first_of("small_model:proposal").is_some());
    }

    #[test]
    fn a_disabled_feature_proposes_nothing() {
        let candidates = vec![FormalizationCandidate {
            qid: "Q11573".into(),
            label: "metre".into(),
            description: "SI unit of length".into(),
        }];
        let mut log = EventLog::new();
        assert!(
            propose_best_match(
                &SmallModelOptions::default(),
                "metre",
                &candidates,
                &mut log
            )
            .is_none()
        );
    }

    #[test]
    fn zero_affinity_stays_silent_instead_of_guessing() {
        let options = SmallModelOptions::enabled_for(1024);
        let candidates = vec![FormalizationCandidate {
            qid: "Q42".into(),
            label: "Douglas Adams".into(),
            description: "English writer".into(),
        }];
        let mut log = EventLog::new();
        assert!(
            propose_best_match(&options, "zzzz qqqq", &candidates, &mut log).is_none(),
            "no overlap means no proposal, not a random one"
        );
    }
}

mod triz_solver {
    use formal_ai::triz_solver::{relevant_tasks, triz_benchmark_tasks, triz_cues, triz_families};

    /// The seeded principle identifiers (forty inventive + four separation),
    /// read from the seed's `principle_id` fields.
    fn principle_ids() -> Vec<String> {
        let seed = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../data/seed/triz-principles.lino"
        ))
        .expect("the TRIZ seed is readable");
        seed.lines()
            .filter_map(|line| line.trim().strip_prefix("principle_id "))
            .map(|value| value.trim().trim_matches('"').to_owned())
            .filter(|id| !id.is_empty())
            .collect()
    }

    #[test]
    fn families_and_benchmark_load_from_the_seed() {
        let families = triz_families();
        assert!(
            families.len() >= 12,
            "twelve general families, got {}",
            families.len()
        );
        assert!(
            families
                .iter()
                .any(|family| family.method_id == "family_range_selection")
        );
        let tasks = triz_benchmark_tasks();
        assert_eq!(tasks.len(), 20, "the top-20 corpus");
        for task in &tasks {
            assert!(
                !task.methods.is_empty(),
                "{} must name its methods",
                task.task_id
            );
        }
        // Every method a task names must exist as a family or a seeded
        // principle identifier -- the corpus cannot cite ghosts.
        let known: Vec<String> = families
            .iter()
            .map(|family| family.method_id.clone())
            .chain(principle_ids())
            .collect();
        for task in &tasks {
            for method in &task.methods {
                assert!(
                    known.iter().any(|id| id == method),
                    "{} cites unknown method {method}",
                    task.task_id
                );
            }
        }
    }

    // The prompt is the corpus row's own statement, read from the seed: a
    // statement shares every one of its content words with itself, so no
    // other row can outrank it. The prose-prompt version of this check is
    // the handler test in rust/tests/unit/issue_901_triz_solver.rs.
    #[test]
    fn umbrella_prompt_finds_the_umbrella_precedent_first() {
        let tasks = triz_benchmark_tasks();
        let umbrella = tasks
            .iter()
            .find(|task| task.task_id == "umbrella_crowd")
            .expect("the corpus carries the umbrella task");
        let relevant = relevant_tasks(&umbrella.statement, &tasks);
        assert_eq!(
            relevant.first().map(|task| task.task_id.as_str()),
            Some("umbrella_crowd")
        );
    }

    #[test]
    fn stems_match_russian_and_english_cues() {
        let cues = triz_cues();
        assert!(cues.iter().any(|cue| cue == "противоречи"));
        assert!(cues.iter().any(|cue| cue == "contradiction"));
    }
}

mod si_units {
    use formal_ai::si_units::{Dimension, parse_factor};

    #[test]
    fn dimension_strings_round_trip() {
        assert_eq!(Dimension::parse("L").unwrap().as_string(), "L");
        assert_eq!(Dimension::parse("LT-1").unwrap().as_string(), "LT-1");
        assert_eq!(Dimension::parse("L2MT-3").unwrap().as_string(), "L2MT-3");
        assert_eq!(Dimension::parse("L-1MT-2").unwrap().as_string(), "L-1MT-2");
        assert_eq!(Dimension::parse("1").unwrap().as_string(), "1");
        assert!(Dimension::parse("LX").is_none());
    }

    #[test]
    fn dimension_algebra_multiplies_and_inverts() {
        let speed = Dimension::parse("LT-1").unwrap();
        let time = Dimension::parse("T").unwrap();
        assert_eq!(
            speed.multiply(&time).unwrap(),
            Dimension::parse("L").unwrap()
        );
        assert_eq!(time.inverse().unwrap(), Dimension::parse("T-1").unwrap());
        assert!(
            speed
                .inverse()
                .unwrap()
                .multiply(&speed)
                .unwrap()
                .is_dimensionless()
        );
    }

    #[test]
    fn factors_parse_exactly() {
        assert_eq!(parse_factor("1000"), Some((1000, 1)));
        assert_eq!(parse_factor("0.0254"), Some((127, 5000)));
        assert_eq!(parse_factor("254/10000"), Some((127, 5000)));
        assert_eq!(parse_factor("-1"), None);
        assert_eq!(parse_factor("abc"), None);
    }
}

mod least_action {
    use formal_ai::least_action::{
        ActionCost, SolutionCandidate, Subtask, least_action_solution, plan, rank_by_least_action,
    };

    fn subtask(title: &str, atomic: bool, steps: u32) -> Subtask {
        Subtask {
            title: title.to_owned(),
            atomic,
            solution_steps: steps,
        }
    }

    #[test]
    fn the_ladder_is_always_one_two_four_eight() {
        let eight: Vec<Subtask> = (0..8).map(|i| subtask(&format!("t{i}"), true, 1)).collect();
        assert_eq!(plan(&eight).level_counts, vec![1, 2, 4, 8]);
        let one = vec![subtask("root", true, 1)];
        assert_eq!(plan(&one).level_counts, vec![1]);
        let five: Vec<Subtask> = (0..5).map(|i| subtask(&format!("t{i}"), true, 1)).collect();
        assert_eq!(plan(&five).level_counts, vec![1, 2, 4, 5]);
    }

    #[test]
    fn a_shorter_solution_that_fails_inputs_is_not_least_action() {
        let candidates = vec![
            SolutionCandidate {
                id: "short_but_wrong".into(),
                solves_entire_range: false,
                cost: ActionCost {
                    steps: 1,
                    ..ActionCost::ZERO
                },
            },
            SolutionCandidate {
                id: "correct".into(),
                solves_entire_range: true,
                cost: ActionCost {
                    steps: 6,
                    ..ActionCost::ZERO
                },
            },
        ];
        let best = least_action_solution(&candidates).unwrap();
        assert_eq!(best.id, "correct");
        assert_eq!(rank_by_least_action(&candidates).len(), 1);
    }
}

mod lino_adapter_links_notation {
    use formal_ai::lino_adapters::links_notation::{format_lino, parse_lino};

    #[test]
    fn indentation_and_quoted_values_survive() {
        let doc = "# seed comment\nstrategy\n  value \"quoted value\"\n  step\n    name first\n";
        let parsed = parse_lino(doc).unwrap();
        assert_eq!(parsed.children[0].name, "strategy");
        assert_eq!(parsed.children[0].find_child_value("value"), "quoted value");
        assert_eq!(parsed.children[0].children[1].children[0].id, "first");
        let reparsed = parse_lino(&format_lino(&parsed)).unwrap();
        assert_eq!(
            reparsed.children[0].find_child_value("value"),
            "quoted value"
        );
    }

    #[test]
    fn malformed_document_is_rejected() {
        assert!(parse_lino("value: (\n").is_err());
    }
}

mod seed_report_slots {
    use formal_ai::seed::fill_slots;

    #[test]
    fn slots_fill_in_one_pass_and_unknown_braces_survive() {
        let template = concat!("{", "a}+{", "b}={", "c}");
        let braced_b = concat!("{", "b}");
        assert_eq!(
            fill_slots(template, &[("a", braced_b), ("b", "2")]),
            concat!("{", "b}+2={", "c}")
        );
    }
}

mod restart_feedback {
    use formal_ai::agentic_coding::restart_feedback::{changed_paths, feedback_needs_changes};

    #[test]
    fn only_the_marked_fenced_listing_is_consumed() {
        assert_eq!(
            changed_paths(
                "⚠️ UNCOMMITTED CHANGES DETECTED\n```text\n?? Main.kt\n M README.md\n```"
            ),
            Some(vec!["Main.kt".to_owned(), "README.md".to_owned()])
        );
        assert_eq!(
            changed_paths("UNCOMMITTED CHANGES DETECTED\n```\n?? ../outside\n```"),
            None
        );
        assert_eq!(changed_paths("?? Main.kt"), None);
    }

    #[test]
    fn unreadable_or_requested_feedback_blocks_readiness() {
        assert!(!feedback_needs_changes(r#"{"comments":[],"reviews":[]}"#));
        assert!(feedback_needs_changes(
            r#"{"comments":[{"body":"Fix the output"}],"reviews":[]}"#
        ));
        assert!(feedback_needs_changes(
            r#"{"comments":[],"reviews":[{"state":"CHANGES_REQUESTED"}]}"#
        ));
        assert!(feedback_needs_changes("{}"));
        assert!(feedback_needs_changes("authentication failed"));
    }
}

mod program_contract_stdout {
    use formal_ai::lino_adapters::links_notation::parse_lino;
    use formal_ai::program_contract::explicit_stdout;

    /// The request/expectation pairs live beside the other test fixtures,
    /// one `case` per behaviour: repeated mentions of one literal are one
    /// output obligation, and distinct literals keep the request's order.
    const CASES: &str = include_str!("../fixtures/program-contract/explicit-stdout.lino");

    #[test]
    fn explicit_stdout_reads_every_fixture_case() {
        let root = parse_lino(CASES).expect("the fixture is Links Notation");
        let cases = &root
            .children
            .first()
            .expect("the fixture has a root")
            .children;
        assert_eq!(cases.len(), 2, "both fixture cases are read");
        for case in cases {
            // The fixture writes a line break as `\n` inside the quoted value.
            let expected = case.find_child_value("expected").replace("\\n", "\n");
            assert_eq!(
                explicit_stdout(case.find_child_value("prompt")),
                Some(expected),
                "fixture case {}",
                case.id
            );
        }
    }
}
