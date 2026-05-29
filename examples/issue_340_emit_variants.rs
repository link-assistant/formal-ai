// Issue #340: emit both the documented and the de-commented (composed) form of
// each blueprint program — exactly as a user receives them — to
// `target/issue-340-variants/`, so an external experiment can compile-check them
// with the real toolchains. This proves the `comments` capability genuinely
// *composes* the program (the stripped form is a different, still-valid program)
// rather than the blueprint serving a single frozen string.
//
// Run with: cargo run --example issue_340_emit_variants
use std::fs;
use std::path::Path;

use formal_ai::FormalAiEngine;

/// Pull the first fenced code block out of a rendered answer.
fn extract_code(answer: &str) -> &str {
    let after_open = answer
        .split_once("```")
        .map(|(_, rest)| rest)
        .expect("answer has an opening fence");
    // Skip the fence's language tag line.
    let body = after_open
        .split_once('\n')
        .map(|(_, rest)| rest)
        .expect("fence has a body");
    body.split("```")
        .next()
        .expect("answer has a closing fence")
}

fn main() {
    let dir = Path::new("target/issue-340-variants");
    fs::create_dir_all(dir).expect("create output dir");

    // (label, language word, file extension, with-comments prompt, no-comments prompt)
    let langs = [
        ("rust", "Rust", "rs"),
        ("python", "Python", "py"),
        ("javascript", "JavaScript", "js"),
    ];
    for (slug, word, extension) in langs {
        let documented_prompt = format!(
            "Write a {word} program that makes an HTTP GET request to a URL, parses the \
             JSON response, calculates statistics mean and median, outputs the results, \
             with error handling and comments."
        );
        let stripped_prompt = format!(
            "Write a {word} program that makes an HTTP GET request to a URL, parses the \
             JSON response, calculates statistics mean and median, and outputs the results."
        );
        let documented = FormalAiEngine.answer(&documented_prompt);
        let stripped = FormalAiEngine.answer(&stripped_prompt);
        fs::write(
            dir.join(format!("{slug}_documented.{extension}")),
            extract_code(&documented.answer),
        )
        .expect("write documented");
        fs::write(
            dir.join(format!("{slug}_stripped.{extension}")),
            extract_code(&stripped.answer),
        )
        .expect("write stripped");
        println!("wrote {slug} documented + stripped ({extension})");
    }
    println!("output dir: {}", dir.display());
}
