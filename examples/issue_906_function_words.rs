//! Print the seed-driven closed-class word list the implementation-language
//! extractor uses to reject determiners (issue #906).
fn main() {
    let words = formal_ai::summarization::vocabulary::function_words();
    println!("{} function words", words.len());
    println!("{}", words.join(" "));
}
