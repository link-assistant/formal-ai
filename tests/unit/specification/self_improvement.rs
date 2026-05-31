//! Issue #364: learn inspectable seed-rule proposals from accumulated unknown
//! traces, gated by the issue #362 coding-modification benchmark.

use formal_ai::{
    learn_rules_from_unknown_traces, BenchmarkGateReport, EventLog, LearnedRuleAdoption,
    SolverConfig, SubstitutionRuleSet, UniversalSolver, UnknownTrace,
};
use lino_objects_codec::format::parse_indented;

fn verified_reverse_sort_unknown_trace() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "selected_rule",
        "initial unknown reason no_seed_route next try_rule_synthesis",
    );
    log.append(
        "rule_synthesis_candidate",
        "rule_synthesis_candidate\n  id reverse_sort_list_files\n  source constructed_from_operation_vocabulary\n  base_task list_files\n  modifier reverse_sort\n  operation sort\n  operation_modifier descending\n  target program:last.output_order\n  resolved_task list_files_reverse_sort",
    );
    log.append(
        "rule_verification",
        "rule_verification\n  candidate reverse_sort_list_files\n  fixture list_files_output_order\n  input a.txt,b.txt,c.txt\n  expected_order c.txt,b.txt,a.txt\n  lowering_check passed\n  render_check passed\n  status passed",
    );
    log.append(
        "write_program_plan",
        "program_plan\n  base_task list_files\n  resolved_task list_files_reverse_sort\n  modifier reverse_sort",
    );

    UnknownTrace::from_event_log("Sort the results in reverse order", "write_program", &log)
        .expect("selected_rule initial unknown should be accumulated")
}

#[test]
fn self_improvement_proposes_human_readable_seed_rule_from_verified_unknown_trace() {
    let trace = verified_reverse_sort_unknown_trace();
    let gate = BenchmarkGateReport::issue_362_from_counts(4, 0);
    let run = learn_rules_from_unknown_traces(&[trace], gate);

    assert_eq!(run.proposals.len(), 1);
    assert!(run.rejections.is_empty());

    let proposal = &run.proposals[0];
    assert_eq!(proposal.rule_id, "reverse_sort_list_files");
    assert_eq!(proposal.base_task, "list_files");
    assert_eq!(proposal.modifier, "reverse_sort");
    assert_eq!(proposal.resolved_task, "list_files_reverse_sort");
    assert_eq!(proposal.adoption, LearnedRuleAdoption::Adoptable);
    assert!(proposal.summary.contains("benchmark"));
    assert!(proposal.seed_rule_lino.contains("event \"learned\""));

    let rules = SubstitutionRuleSet::from_links_notation(&proposal.seed_rule_lino)
        .expect("learned seed rule should parse");
    let modifiers = vec![String::from("reverse_sort")];
    let plan = formal_ai::program_plan::lower_with_rules(&rules, "list_files", &modifiers);
    assert_eq!(plan.resolved_task, "list_files_reverse_sort");

    let run_lino = run.links_notation();
    parse_indented(&run_lino).expect("learning run should render valid Links Notation");
    assert!(run_lino.contains("self_improvement_run"));
    assert!(run_lino.contains("adoption \"adoptable\""));
    assert!(run_lino.contains("seed_rule"));
}

#[test]
fn benchmark_gate_blocks_adoption_when_ratchet_would_regress() {
    let trace = verified_reverse_sort_unknown_trace();
    let gate = BenchmarkGateReport::issue_362_from_counts(3, 1);
    let run = learn_rules_from_unknown_traces(&[trace], gate);

    assert_eq!(run.proposals.len(), 1);
    assert_eq!(
        run.proposals[0].adoption,
        LearnedRuleAdoption::BlockedByBenchmark
    );
    assert!(run.adoptable_rules().is_empty());
    assert!(run.links_notation().contains("blocked_by_benchmark"));
}

#[test]
fn accumulated_unknown_answers_without_candidates_are_rejected_not_adopted() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let prompt = "Quxblort fnordwarble plimsy gabble what?";
    let answer = solver.solve(prompt);
    assert_eq!(answer.intent, "unknown");

    let trace = UnknownTrace::from_symbolic_answer(prompt, &answer)
        .expect("unknown answer should be accumulated");
    let trace_lino = trace.links_notation();
    parse_indented(&trace_lino).expect("unknown trace should render valid Links Notation");

    let gate = BenchmarkGateReport::issue_362_from_counts(4, 0);
    let run = learn_rules_from_unknown_traces(&[trace], gate);

    assert!(run.proposals.is_empty());
    assert_eq!(run.rejections.len(), 1);
    assert!(run.rejections[0]
        .reason
        .contains("no rule_synthesis_candidate"));
}
