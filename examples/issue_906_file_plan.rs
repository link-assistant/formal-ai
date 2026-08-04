//! Probe the seed-driven file-write plan composer with the issue #906 prompts.
fn main() {
    for request in [
        "Create a file named hello.txt in the current directory whose entire content is the single line: Hello World.",
        "Create a file named hello.txt containing Hello World, in JavaScript.",
        "Write a program that prints hello world.",
    ] {
        println!("=== {request}");
        match formal_ai::agentic_coding::general_planner::compose_general_change_plan(request) {
            Some(plan) => println!("{}", plan.links_notation()),
            None => println!("(no plan)"),
        }
    }
}
