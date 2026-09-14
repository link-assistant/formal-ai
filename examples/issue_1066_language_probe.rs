//! Print the language Formal AI detects for a prompt (issue #1066 probe).

fn main() {
    let prompt = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    println!("language: {}", formal_ai::language::detect(&prompt).slug());
}
