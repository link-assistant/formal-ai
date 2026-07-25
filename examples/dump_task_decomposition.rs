//! Dump the decomposition of a task so the splitter can be inspected by hand.
//!
//! Usage: `cargo run --example dump_task_decomposition -- "<task>" [max_depth]`

use formal_ai::task_decomposition::{decompose_task, is_checkable};

fn main() {
    let mut args = std::env::args().skip(1);
    let task = args
        .next()
        .unwrap_or_else(|| "Add a flag to release.yml and update the changelog.".to_owned());
    let max_depth: u8 = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(4);

    let decomposition = decompose_task(&task, max_depth);
    println!("task: {}", decomposition.task);
    println!("atomic: {}", decomposition.is_atomic());
    println!(
        "depth_bound_reached: {}",
        decomposition.depth_bound_reached()
    );
    for line in decomposition.numbered_lines("[depth bound]") {
        println!("{line}");
    }
    for leaf in decomposition.leaves() {
        println!(
            "leaf {} reason={} checkable={} :: {}",
            leaf.path,
            leaf.reason.slug(),
            is_checkable(&leaf.text),
            leaf.text
        );
    }
}
