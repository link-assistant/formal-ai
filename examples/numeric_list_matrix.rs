// Issue #395 — machine-readable dump of the numeric-list answer matrix.
//
// For every (operation, language, value class) cell the solver claims, this
// prints one JSON line `{"canonical","slug","class","prompt","answer"}` with
// the engine's full answer. The companion harness
// `experiments/issue-395-cross-runtime-codegen-parity.mjs` replays the same
// prompts through the browser worker mirror and byte-compares the answers, so
// any drift between the Rust composer and the JS composer (both driven by
// `data/seed/coding-idioms.lino`) is caught across the entire matrix instead
// of a sampled subset.
//
// Run with: cargo run --example numeric_list_matrix

use formal_ai::FormalAiEngine;

/// Distinct integer inputs make a wrong ordering or reduction observable.
const INTEGERS: &str = "3, 1, 4, 5, 9";
/// Float inputs exercise the float value class and its formatting parity.
const FLOATS: &str = "3.5, 1.25, 4.5, 0.5, 9.75";
/// Quoted text inputs exercise the string value class (transformations only —
/// reductions are numeric by definition).
const STRINGS: &str = "\"pear\", \"apple\", \"banana\"";

/// Operations paired with a prompt template; `<items>` is replaced with the
/// value-class-specific list. Reductions only accept numeric lists, so the
/// string class is restricted to transformations.
const OPERATIONS: &[(&str, &str, bool)] = &[
    ("sort", "Sort the <noun> <items>", true),
    (
        "reverse_sort",
        "Sort the <noun> <items> in descending order",
        true,
    ),
    ("reverse", "Reverse the <noun> <items>", true),
    ("sum", "Sum the <noun> <items>", false),
    ("product", "Multiply the <noun> <items>", false),
    ("minimum", "Find the minimum of <items>", false),
    ("maximum", "Find the maximum of <items>", false),
];

/// Catalog slug plus the surface word the prompt uses (`csharp`/`cpp` because
/// prompt normalization folds `#` and `+` to whitespace).
const LANGS: &[(&str, &str)] = &[
    ("javascript", "JavaScript"),
    ("typescript", "TypeScript"),
    ("python", "Python"),
    ("rust", "Rust"),
    ("go", "Go"),
    ("ruby", "Ruby"),
    ("java", "Java"),
    ("csharp", "csharp"),
    ("cpp", "cpp"),
    ("c", "C"),
];

const CLASSES: &[(&str, &str, &str)] = &[
    ("integer", "numbers", INTEGERS),
    ("float", "numbers", FLOATS),
    ("string", "strings", STRINGS),
];

fn main() {
    let mut emitted = 0_u32;
    for (canonical, template, transforms_only) in OPERATIONS {
        for (class, noun, items) in CLASSES {
            if *class == "string" && !transforms_only {
                continue;
            }
            for (slug, word) in LANGS {
                let phrase = template.replace("<noun>", noun).replace("<items>", items);
                let prompt = format!("{phrase} in {word}, give me the code and the result");
                let response = FormalAiEngine.answer(&prompt);
                assert_eq!(
                    response.intent, "write_program",
                    "[{canonical}/{slug}/{class}] expected write_program, got {} for: {prompt}",
                    response.intent
                );
                let record = serde_json::json!({
                    "canonical": canonical,
                    "slug": slug,
                    "class": class,
                    "prompt": prompt,
                    "answer": response.answer,
                });
                println!("{record}");
                emitted += 1;
            }
        }
    }
    eprintln!("emitted {emitted} matrix records");
}
