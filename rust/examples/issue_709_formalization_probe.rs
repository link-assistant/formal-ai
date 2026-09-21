//! Probe the existing formalizer as a cross-language statement join for issue #709.

use formal_ai::translation::formalize_prompt;

fn main() {
    for (language, statement) in [
        ("en", "Apple is a fruit"),
        ("ru", "Яблоко это фрукт"),
        ("hi", "सेब एक फल है"),
        ("zh", "苹果是一种水果"),
    ] {
        let candidate = formalize_prompt(statement, language);
        println!("{language}\t{statement}\t{}", candidate.compact_summary());
        println!("{}", candidate.to_links_notation());
    }
}
