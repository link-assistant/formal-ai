// Issue #340: emit every projection of each blueprint program — exactly as a
// user receives them — to `target/issue-340-variants/`, so an external
// experiment can compile-check them with the real toolchains. The blueprint
// composes along two *independent* capability axes:
//
//   * `comments`        — whole-line documentation is kept or stripped;
//   * `error_handling`  — `// region:error_handling` blocks (empty-input guards,
//                         HTTP status checks) are kept or dropped with their body.
//
// We emit the full cross-product (4 variants per language). That every variant
// is a *different, still-valid* program proves the synthesizer projects the
// decomposition rather than serving a single frozen string.
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

    // (label, language word, file extension)
    let langs = [
        ("rust", "Rust", "rs"),
        ("python", "Python", "py"),
        ("javascript", "JavaScript", "js"),
    ];
    // (variant suffix, asks for comments, asks for error handling). The two axes
    // compose independently, so the cross-product yields four distinct programs.
    let variants = [
        ("documented", true, true),
        ("comments_only", true, false),
        ("errors_only", false, true),
        ("stripped", false, false),
    ];
    let base = "makes an HTTP GET request to a URL, parses the JSON response, \
                calculates statistics mean and median, and outputs the results";
    for (slug, word, extension) in langs {
        for (suffix, wants_comments, wants_errors) in variants {
            let extras = match (wants_comments, wants_errors) {
                (true, true) => ", with error handling and comments",
                (true, false) => ", with comments",
                (false, true) => ", with error handling",
                (false, false) => "",
            };
            let prompt = format!("Write a {word} program that {base}{extras}.");
            let answer = FormalAiEngine.answer(&prompt);
            fs::write(
                dir.join(format!("{slug}_{suffix}.{extension}")),
                extract_code(&answer.answer),
            )
            .expect("write variant");
        }
        println!("wrote {slug} documented + comments_only + errors_only + stripped ({extension})");
    }
    println!("output dir: {}", dir.display());
}
