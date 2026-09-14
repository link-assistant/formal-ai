//! Discover a reusable algorithm from three portable execution traces.
//!
//! Run with `cargo run --example issue_531_algorithm_discovery`. The first two
//! traces infer the schema and the third is held out. The resulting proposal is
//! materialized without effects; promotion and execution require separate
//! named review and green-gate values.

use formal_ai::MemoryStore;
use formal_ai::algorithm_discovery::{
    ArgumentPattern, discover_algorithms, traces_from_memory_events,
};

fn main() {
    let mut memory = MemoryStore::new();
    memory.replace_from_links_notation(include_str!(
        "../data/benchmarks/issue-531-algorithm-traces.lino"
    ));
    let run = discover_algorithms(&traces_from_memory_events(memory.events()));
    println!("{}", run.links_notation());

    let candidate = run
        .validated_candidates()
        .into_iter()
        .next()
        .expect("the benchmark contains two support and one held-out trace");
    let bindings = candidate
        .steps
        .iter()
        .flat_map(|step| step.arguments.values())
        .filter_map(|pattern| match pattern {
            ArgumentPattern::Parameter(name) => Some((name.clone(), String::from("delta"))),
            ArgumentPattern::Constant(_) => None,
        })
        .collect();
    println!(
        "{}",
        candidate
            .conformance_links_notation("example-trigger", &bindings)
            .expect("every inferred parameter is bound")
    );
}
