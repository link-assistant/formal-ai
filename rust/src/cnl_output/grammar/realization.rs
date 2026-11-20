use super::{CompiledGrammar, Graph, SLOT, js_space};
use regex::Regex;
use std::sync::LazyLock;

static LETTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\p{L}").expect("fixed CNL letter grammar"));

fn capitalized(text: &str) -> String {
    match LETTER.find(text) {
        Some(letter) => {
            let upper: String = letter.as_str().to_uppercase();
            format!(
                "{}{upper}{}",
                &text[..letter.start()],
                &text[letter.end()..]
            )
        }
        None => text.to_owned(),
    }
}

/// Declared variants of a frame/style/mood/polarity, preferred first.
fn variants_of<'a>(graph: &Graph, style: &str, grammar: &'a CompiledGrammar) -> Vec<&'a String> {
    grammar
        .data
        .templates
        .iter()
        .filter(|item| {
            item.frame == graph.frame
                && item.mood == graph.mood
                && item.styles.iter().any(|known| known == style)
        })
        .flat_map(|item| match graph.polarity.as_str() {
            "positive" => item.positive.iter(),
            "negative" => item.negative.iter(),
            _ => [].iter(),
        })
        .collect()
}

// Punctuation that already closes an inner clause after a slot.
const CLOSING: &[char] = &[
    '.', ',', ';', ':', '?', '!', '…', '»', ')', '"', '。', '，', '？', '！', '、',
];

/// One declared variant (the preferred one by default) in one register, or `None`.
#[must_use]
pub fn realize_graph(
    graph: &Graph,
    style: &str,
    register: &str,
    grammar: &CompiledGrammar,
    variant: usize,
) -> Option<String> {
    let source = *variants_of(graph, style, grammar).get(variant)?;
    let mut missing = false;
    let mut text = String::new();
    let mut last = 0;
    for found in SLOT.captures_iter(source) {
        let whole = found.get(0).expect("whole match");
        text.push_str(&source[last..whole.start()]);
        last = whole.end();
        let Some(value) = graph.slots.get(&found[1]) else {
            missing = true;
            continue;
        };
        let key = &found[2];
        if key == "number" {
            text.push_str(value);
            continue;
        }
        let forms = grammar
            .term(value)
            .and_then(|term| term.register(register).or_else(|| term.register("forms")));
        let position = grammar.data.keys.iter().position(|item| item == key);
        let Some(form) = forms.zip(position).and_then(|(forms, at)| forms.get(at)) else {
            missing = true;
            continue;
        };
        // A form with an inner clause (for example Russian "..., которая может
        // двигаться") is closed by the language's declared mark unless punctuation
        // or the end of the sentence already follows it. Parsing ignores the mark.
        let rest = source[whole.end()..].trim_start_matches(js_space);
        text.push_str(form);
        if !grammar.data.clause_close.is_empty()
            && form.contains(',')
            && rest
                .chars()
                .next()
                .is_some_and(|next| !CLOSING.contains(&next))
        {
            text.push_str(&grammar.data.clause_close);
        }
    }
    text.push_str(&source[last..]);
    (!missing).then(|| capitalized(&text))
}

/// Every distinct realization of every variant, in the order of the style/register pairs.
#[must_use]
pub fn realizations(
    graph: &Graph,
    pairs: &[(String, String)],
    grammar: &CompiledGrammar,
) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for (style, register) in pairs {
        for variant in 0..variants_of(graph, style, grammar).len() {
            if let Some(text) = realize_graph(graph, style, register, grammar, variant) {
                if !texts.contains(&text) {
                    texts.push(text);
                }
            }
        }
    }
    texts
}

#[must_use]
pub fn all_pairs(grammar: &CompiledGrammar) -> Vec<(String, String)> {
    grammar
        .output_styles
        .iter()
        .flat_map(|style| {
            grammar
                .shared
                .registers
                .iter()
                .map(move |register| (style.clone(), register.clone()))
        })
        .collect()
}
