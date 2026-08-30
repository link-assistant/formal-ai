//! Print, for each argument, whether Formal AI judges it independently
//! checkable (issue #1066 probe).

fn main() {
    for segment in std::env::args().skip(1) {
        println!(
            "{:5} | {segment}",
            formal_ai::task_decomposition::is_checkable(&segment)
        );
    }
}
