//! Structural currency-expression operands shared by admission and evaluation.

use crate::rule_interpreter::{handler_table_rows, handler_table_value};
use crate::seed;

const CURRENCY_SYMBOLS: &[char] = &['$', '€', '¥', '₹', '₽'];

pub(crate) fn currency_code(word: &str) -> Option<String> {
    let symbol_code = match word {
        "$" => "USD",
        "€" => "EUR",
        "¥" => "JPY",
        "₹" => "INR",
        "₽" => "RUB",
        _ => "",
    };
    if !symbol_code.is_empty() {
        return Some(symbol_code.to_owned());
    }
    if word.len() == 3
        && word.bytes().all(|byte| byte.is_ascii_uppercase())
        && seed::parser::split_pipe_list(&handler_table_value(
            "currency-quote-provider",
            "iso-codes",
        )?)
        .iter()
        .any(|code| code == word)
    {
        return Some(word.to_owned());
    }
    let word = word.to_lowercase();
    for (role, code) in [
        (seed::ROLE_CURRENCY_USD_REFERENCE, "USD"),
        (seed::ROLE_CURRENCY_EUR_REFERENCE, "EUR"),
        (seed::ROLE_CURRENCY_RUB_REFERENCE, "RUB"),
    ] {
        if seed::lexicon().words_for_role(role).iter().any(|surface| {
            let cyrillic = surface
                .chars()
                .next()
                .is_some_and(|c| ('\u{0400}'..='\u{04ff}').contains(&c));
            if cyrillic {
                word.starts_with(surface.as_str())
            } else {
                word == *surface
            }
        }) {
            return Some(code.to_owned());
        }
    }
    None
}

/// Currency tokens keep their position for reading an adjacent amount.
pub(crate) fn currency_mentions(text: &str) -> impl Iterator<Item = (usize, &str, String)> {
    let mut in_word = false;
    text.char_indices().filter_map(move |(at, character)| {
        let starts_word = character.is_alphabetic() && !in_word;
        in_word = character.is_alphabetic();
        let token = if starts_word {
            let tail = &text[at..];
            &tail[..tail
                .find(|ch: char| !ch.is_alphabetic())
                .unwrap_or(tail.len())]
        } else if CURRENCY_SYMBOLS.contains(&character) {
            &text[at..at + character.len_utf8()]
        } else {
            return None;
        };
        Some((at, token, currency_code(token)?))
    })
}

fn date_token(word: &str) -> Option<&str> {
    let word = word.trim_matches(|character: char| !character.is_ascii_digit() && character != '-');
    let bytes = word.as_bytes();
    (bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().filter(|&&byte| byte.is_ascii_digit()).count() == 8)
        .then_some(word)
}

pub(crate) fn stated_quote_date(prompt: &str) -> Option<&str> {
    prompt.split_whitespace().find_map(date_token)
}

pub(crate) fn decimal(text: &str, fraction_limit: usize) -> Option<(i128, i128)> {
    let text = text.replace(',', ".");
    let negative = text.starts_with('-');
    let unsigned = text.trim_start_matches(['+', '-']);
    if text.len() > unsigned.len() + 1 {
        return None;
    }
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || whole.len() > 12
        || fraction.len() > fraction_limit
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let mantissa = format!("{whole}{fraction}").parse::<i128>().ok()?;
    let divisor = 10_i128.pow(u32::try_from(fraction.len()).ok()?);
    Some((if negative { -mantissa } else { mantissa }, divisor))
}

pub(crate) struct CurrencyRequest<'a> {
    pub amount: String,
    pub from: String,
    pub to: String,
    pub date: Option<&'a str>,
}

pub(crate) fn conversion_operands(expression: &str) -> Option<CurrencyRequest<'_>> {
    // A date belongs to the outer expression even when an amount glyph is expanded.
    let date = stated_quote_date(expression);
    let mut expression = expression.trim();
    let lowered = expression.to_lowercase();
    for cue in seed::lexicon().words_for_role("quantity_conversion_cue") {
        if lowered
            .strip_prefix(cue.as_str())
            .is_some_and(|tail| tail.starts_with(char::is_whitespace))
        {
            expression = expression[cue.len()..].trim_start();
            break;
        }
    }
    let expanded;
    if let Some(symbol) = expression
        .chars()
        .next()
        .filter(|symbol| CURRENCY_SYMBOLS.contains(symbol))
    {
        let rest = expression[symbol.len_utf8()..].trim_start();
        let (amount, tail) = rest.split_once(char::is_whitespace)?;
        expanded = format!("{amount} {symbol} {tail}");
        expression = &expanded;
    }
    let tokens = expression.split_whitespace().collect::<Vec<_>>();
    let tokens = match tokens.split_last() {
        Some((word, preceding)) if date_token(word).is_some() => {
            let preceding = preceding.strip_suffix(&["on"]).unwrap_or(preceding);
            preceding
        }
        _ => tokens.as_slice(),
    };
    if tokens.len() < 4 {
        return None;
    }
    let connector = tokens[2..tokens.len() - 1].join(" ").to_lowercase();
    if !handler_table_rows("currency-conversion-connector")
        .iter()
        .any(|(_, words)| words.split('|').any(|word| word == connector))
    {
        return None;
    }
    let from = currency_code(tokens[1])?;
    let to = currency_code(tokens[tokens.len() - 1])?;
    decimal(tokens[0], 6)?;
    Some(CurrencyRequest {
        amount: tokens[0].replace(',', "."),
        from,
        to,
        date,
    })
}
