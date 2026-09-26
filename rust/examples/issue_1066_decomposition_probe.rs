//! Dump the decomposition Formal AI builds for a task (issue #1066 probe).

fn main() {
    let task = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let decomposition = formal_ai::task_decomposition::decompose_task(&task, 4);
    println!("task: {:?}", decomposition.task);
    println!("atomic: {}", decomposition.is_atomic());
    println!("reason: {:?}", decomposition.root.reason);
    for (path, text, criterion, bounded) in decomposition.rows() {
        println!("{path} | {text:?} | {criterion} | bounded={bounded}");
    }
    println!(
        "split_once_checkable: {:?}",
        formal_ai::task_decomposition::split_once_checkable(&task)
    );
}
