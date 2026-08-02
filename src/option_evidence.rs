//! Reading constraints and candidates out of fetched evidence — issue #781.
//!
//! [`crate::option_network`] can rank options once it knows what each candidate
//! supplies. This module is how it finds out: it turns a page of fetched text
//! into [`Supply`] values and an [`Offer`].
//!
//! ## Why this needs no vocabulary
//!
//! Extracting "the voltage" from arbitrary prose sounds like it needs to know
//! the word *voltage* in every language the web is written in. It does not,
//! because the network already carries the question. A constraint declares that
//! it wants `output_voltage` measured in `V`; the page is then searched for a
//! **number followed by `V`**. The prose around that number can be in any
//! language without changing the search, so a Russian spec sheet and an Indian
//! listing are read by the same code. Nothing here matches words, which is what
//! keeps it inside the project's no-hardcoded-natural-language rule and what
//! makes it general across domains the engine was never told about.
//!
//! The cost of that choice is real and worth stating plainly. A page mentioning
//! several quantities in the same unit yields the first. A page stating its
//! numbers only in prose ("nineteen and a half volts") yields nothing. And a
//! page that localises the *symbol* itself — `19.5 В` in Cyrillic rather than
//! `19.5 V` — also yields nothing, because a symbol match is exactly what this
//! relies on. In every one of those cases the attribute is simply left
//! unsupplied, which the research loop already treats as an open question worth
//! another round. Abstaining is the designed outcome; guessing is not.

use crate::option_network::{Candidate, Constraint, Demand, Offer, Supply, Tier, SCALE};

/// The largest number of fractional digits kept when parsing.
///
/// [`SCALE`] is thousandths, so a fourth digit cannot be represented. Extra
/// digits are truncated rather than rounded: truncation cannot push a value
/// across a constraint boundary it did not already cross.
const FRACTION_DIGITS: usize = 3;

/// Parse the first quantity stated in `unit` anywhere in `text`.
///
/// The unit must appear immediately after the number, optionally separated by
/// spaces, and must not run into a longer word — `2 A` is amperes, `2 Apr` is
/// not. Matching is case-sensitive, because unit symbols are: `m` and `M`, `V`
/// and `v` are different in general, and a case-insensitive match would read
/// `mm` as metres.
#[must_use]
pub fn quantity_in(text: &str, unit: &str) -> Option<i64> {
    if unit.is_empty() {
        return None;
    }
    let bytes: Vec<char> = text.chars().collect();
    let unit_chars: Vec<char> = unit.chars().collect();
    let mut index = 0;
    while index < bytes.len() {
        let Some((value, after)) = number_at(&bytes, index) else {
            index += 1;
            continue;
        };
        let mut cursor = after;
        while bytes.get(cursor).is_some_and(|c| *c == ' ') {
            cursor += 1;
        }
        let matches_unit = bytes
            .get(cursor..cursor + unit_chars.len())
            .is_some_and(|slice| slice == unit_chars.as_slice());
        // A unit symbol must end where it ends. Without this, `mm` matches the
        // `m` in a millimetre figure and reports metres.
        let ends_cleanly = matches_unit
            && bytes
                .get(cursor + unit_chars.len())
                .is_none_or(|next| !next.is_alphanumeric());
        if ends_cleanly {
            return Some(value);
        }
        index = after.max(index + 1);
    }
    None
}

/// Parse the number starting at `start`, returning its fixed-point value at
/// [`SCALE`] and the index just past it. Digit-group separators inside the whole
/// part are skipped, so a listed price reads the same written `1,200` or `1200`.
fn number_at(text: &[char], start: usize) -> Option<(i64, usize)> {
    if !text.get(start)?.is_ascii_digit() {
        return None;
    }
    // A digit inside a word is not a measurement. Part numbers are the reason:
    // `A13-045N2A` ends in a digit-then-`A` that reads exactly like an ampere
    // figure, and taking it would attribute a current to a part whose current
    // the page never stated.
    if start > 0 && text[start - 1].is_alphanumeric() {
        return None;
    }
    let mut whole: i64 = 0;
    let mut index = start;
    while let Some(character) = text.get(index) {
        if character.is_ascii_digit() {
            whole = whole.checked_mul(10)?.checked_add(digit(*character))?;
            index += 1;
        } else if (*character == ',' || *character == '\u{202f}' || *character == ' ')
            && text.get(index + 1).is_some_and(char::is_ascii_digit)
        {
            index += 1;
        } else {
            break;
        }
    }
    let mut value = whole.checked_mul(SCALE)?;
    if text.get(index) == Some(&'.') && text.get(index + 1).is_some_and(char::is_ascii_digit) {
        index += 1;
        let mut place = SCALE / 10;
        let mut kept = 0;
        while let Some(character) = text.get(index) {
            if !character.is_ascii_digit() {
                break;
            }
            if kept < FRACTION_DIGITS {
                value = value.checked_add(digit(*character) * place)?;
                place /= 10;
                kept += 1;
            }
            index += 1;
        }
    }
    Some((value, index))
}

fn digit(character: char) -> i64 {
    i64::from(character as u32 - '0' as u32)
}

/// Parse the first price stated in `currency` anywhere in `text`.
///
/// Both orders are accepted, because both are written: a sign usually precedes
/// its amount and a code usually follows it.
#[must_use]
pub fn price_in(text: &str, currency: &str) -> Option<i64> {
    if currency.is_empty() {
        return None;
    }
    if let Some(value) = quantity_in(text, currency) {
        return Some(value);
    }
    let characters: Vec<char> = text.chars().collect();
    let symbol: Vec<char> = currency.chars().collect();
    let mut index = 0;
    while index + symbol.len() <= characters.len() {
        if characters[index..index + symbol.len()] == symbol[..] {
            let mut cursor = index + symbol.len();
            while characters.get(cursor).is_some_and(|c| *c == ' ') {
                cursor += 1;
            }
            if let Some((value, _)) = number_at(&characters, cursor) {
                return Some(value);
            }
        }
        index += 1;
    }
    None
}

/// Build a candidate from one fetched page, guided by what the constraints ask.
///
/// The constraints supply the units to look for, so no attribute name is ever
/// matched against the page's prose. A nominal constraint is satisfied only when
/// the page states its required value verbatim — nominal values here are part
/// identifiers and socket sizes, which are written the same everywhere, so a
/// literal search is the right test and a lenient one would invent fits.
///
/// Attributes the page does not state are simply absent from the result. That is
/// the point: an absent attribute keeps the question open instead of closing it
/// on a guess.
#[must_use]
pub fn candidate_from_page(
    id: impl Into<String>,
    tier: Tier,
    text: &str,
    constraints: &[Constraint],
) -> Candidate {
    let mut candidate = Candidate::new(id, tier);
    for constraint in constraints {
        match &constraint.demand {
            Demand::Quantity { unit, .. } => {
                if let Some(value) = quantity_in(text, unit) {
                    candidate = candidate
                        .supplying(&constraint.attribute, Supply::quantity(value, unit.clone()));
                }
            }
            Demand::Nominal(required) => {
                if text.to_lowercase().contains(&required.to_lowercase()) {
                    candidate = candidate
                        .supplying(&constraint.attribute, Supply::nominal(required.clone()));
                }
            }
        }
    }
    candidate
}

/// Attach a listing read from `text`, when a price in `currency` is stated.
///
/// A candidate with no readable price is still returned unchanged rather than
/// dropped — it remains a real option, and [`crate::option_network`] already
/// ranks unpriced plans after priced ones instead of discarding them.
#[must_use]
pub fn with_offer_from_page(
    candidate: Candidate,
    text: &str,
    currency: &str,
    seller: &str,
    url: &str,
) -> Candidate {
    match price_in(text, currency) {
        Some(price) => candidate.offered(Offer::new(price, currency, seller, url)),
        None => candidate,
    }
}
