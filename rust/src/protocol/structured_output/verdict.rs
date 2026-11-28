//! Evidence rules for one criterion and its labeled source text.

use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

const NUMBER: &str = r"([0-9]+(?:[.,][0-9]+)?)";
const DATE: &str = r"([0-9]{4}(?:-[0-9]{2}-[0-9]{2})?)";

fn pattern(rule: &str, replacements: &[(&str, &str)]) -> Regex {
    let source = crate::seed::agentic_message(&format!("structured-verdict-{rule}"), replacements);
    Regex::new(&format!("(?i){source}")).expect("structured verdict pattern")
}

static SECTIONS: LazyLock<Regex> = LazyLock::new(|| pattern("sections", &[]));
static MINIMUM_LEADING: LazyLock<Regex> =
    LazyLock::new(|| pattern("minimum-leading", &[("number", NUMBER)]));
static MINIMUM_TRAILING: LazyLock<Regex> =
    LazyLock::new(|| pattern("minimum-trailing", &[("number", NUMBER)]));
static PERIOD: LazyLock<Regex> = LazyLock::new(|| pattern("period", &[("date", DATE)]));
static CLAIMS: LazyLock<Regex> = LazyLock::new(|| pattern("claims", &[]));
static LIST: LazyLock<Regex> = LazyLock::new(|| pattern("list", &[]));
static USE: LazyLock<Regex> = LazyLock::new(|| pattern("use", &[]));
static ONLY: LazyLock<Regex> = LazyLock::new(|| pattern("only", &[]));
static ROLE: LazyLock<Regex> = LazyLock::new(|| pattern("role", &[]));
static ROLE_PREFIX: LazyLock<Regex> = LazyLock::new(|| pattern("role-prefix", &[]));
static ITEM_TAIL: LazyLock<Regex> = LazyLock::new(|| pattern("item-tail", &[]));
static RUSSIAN_ENDING: LazyLock<Regex> = LazyLock::new(|| pattern("russian-ending", &[]));
static EXPERIENCE_TAIL: LazyLock<Regex> = LazyLock::new(|| pattern("experience-tail", &[]));
static TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\p{L}\p{N}_]+(?:[.-][\p{L}\p{N}_]+)*(?:\+\+|#)?").expect("skill token")
});
static CYRILLIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\p{Cyrillic}+$").expect("Cyrillic word"));
static EXACT_DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^{DATE}$")).expect("verdict date"));

fn trim_span(text: &str) -> &str {
    text.trim()
        .trim_end_matches(['.', '!', '?', ';'])
        .trim_end()
}

fn sections(source: &str) -> [Vec<&str>; 3] {
    let mut markers = SECTIONS.captures_iter(source).peekable();
    let mut sections = [Vec::new(), Vec::new(), Vec::new()];
    while let Some(marker) = markers.next() {
        let kind = if marker.get(1).is_some() {
            0
        } else if marker.get(2).is_some() {
            1
        } else {
            2
        };
        let start = marker.get(0).expect("whole section marker").end();
        let end = markers
            .peek()
            .and_then(|next| next.get(0))
            .map_or(source.len(), |next| next.start());
        sections[kind].push(trim_span(&source[start..end]));
    }
    sections
}

fn word_matches(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    if !CYRILLIC.is_match(left) || !CYRILLIC.is_match(right) {
        return false;
    }
    let stem = RUSSIAN_ENDING.replace(left, "");
    stem.chars().count() >= 4 && (stem == right || stem == RUSSIAN_ENDING.replace(right, ""))
}

fn subject_spans(text: &str, subject: &[&str]) -> Vec<(usize, usize)> {
    let tokens: Vec<_> = TOKEN.find_iter(text).collect();
    tokens
        .windows(subject.len())
        .filter(|terms| {
            subject
                .iter()
                .zip(terms.iter())
                .all(|(word, term)| word_matches(word, &term.as_str().to_lowercase()))
        })
        .map(|terms| (terms[0].start(), terms[terms.len() - 1].end()))
        .collect()
}

fn item_matches(item: &str, subject: &[&str]) -> bool {
    let phrase = item.trim().trim_start_matches(['"', '\'', '“', '«']);
    subject_spans(phrase, subject)
        .into_iter()
        .any(|(start, end)| {
            let tail = phrase[end..]
                .trim()
                .trim_start_matches(['"', '\'', '”', '»'])
                .trim_start();
            start == 0 && (tail.is_empty() || ITEM_TAIL.is_match(tail))
        })
}

#[derive(Default)]
struct Classification {
    positive: bool,
    negative: bool,
    excluded: bool,
}

fn classify(claim: &str, subject: &[&str]) -> Classification {
    let mut cues = USE.captures_iter(claim).peekable();
    let mut result = Classification::default();
    if cues.peek().is_none() {
        let restricted = ONLY.replace(claim, "");
        if restricted != claim && !restricted.trim().is_empty() {
            result.positive = LIST
                .split(restricted.trim())
                .any(|item| item_matches(item, subject));
            result.excluded = !result.positive;
        }
        result.positive |= subject_spans(claim, subject)
            .into_iter()
            .any(|(start, end)| {
                ROLE_PREFIX.is_match(&claim[..start])
                    || (start == 0 && ROLE.is_match(claim[end..].trim_start()))
            });
    }
    while let Some(cue) = cues.next() {
        let start = cue.get(0).expect("whole usage cue").end();
        let end = cues
            .peek()
            .and_then(|next| next.get(0))
            .map_or(claim.len(), |next| next.start());
        let raw = claim[start..end].trim().trim_end_matches(',');
        let restricted = ONLY.replace(raw, "");
        let matched = LIST
            .split(restricted.trim())
            .any(|item| item_matches(item, subject));
        if cue.get(1).is_some() {
            result.negative |= matched;
        } else {
            result.positive |= matched;
            result.excluded |= !restricted.trim().is_empty() && restricted != raw && !matched;
        }
    }
    result
}

#[derive(Clone, Copy)]
struct Date {
    year: i32,
    month_day: Option<i32>,
}

fn date(text: &str) -> Option<Date> {
    if !EXACT_DATE.is_match(text) {
        return None;
    }
    let year = text[..4].parse::<i32>().ok().filter(|year| *year > 0)?;
    let month_day = if text.len() == 4 {
        None
    } else {
        let month: usize = text[5..7].parse().ok()?;
        let day: i32 = text[8..].parse().ok()?;
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let maximum = *days.get(month.checked_sub(1)?)?;
        if !(1..=maximum).contains(&day) {
            return None;
        }
        Some(i32::try_from(month).ok()? * 100 + day)
    };
    Some(Date { year, month_day })
}

fn duration_verdict(minimum: f64, start: Date, end: Date) -> Option<bool> {
    let years = end.year - start.year;
    if years < 0
        || (years == 0
            && start
                .month_day
                .zip(end.month_day)
                .is_some_and(|(a, b)| b < a))
    {
        return None;
    }
    if start.month_day.is_some() == end.month_day.is_some() {
        return Some(f64::from(years - i32::from(end.month_day < start.month_day)) >= minimum);
    }
    // With mixed precision, require the threshold to hold for every possible anniversary.
    let earliest_end = end.month_day.unwrap_or(101);
    let latest_start = start.month_day.unwrap_or(1231);
    let latest_end = end.month_day.unwrap_or(1231);
    let earliest_start = start.month_day.unwrap_or(101);
    if f64::from(years - i32::from(earliest_end < latest_start)) >= minimum {
        return Some(true);
    }
    (f64::from(years - i32::from(latest_end < earliest_start)) < minimum).then_some(false)
}

#[derive(Serialize)]
pub(super) struct Trace<'a> {
    rule: &'static str,
    evidence: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<&'a str>,
}

pub(super) struct Verdict<'a> {
    pub result: Option<bool>,
    pub trace: Trace<'a>,
}

fn outcome<'a>(
    result: Option<bool>,
    rule: &'static str,
    evidence: &'a str,
    reference: Option<&'a str>,
) -> Verdict<'a> {
    Verdict {
        result,
        trace: Trace {
            rule,
            evidence,
            reference,
        },
    }
}

fn duration_evidence<'a>(
    claim: &'a str,
    subject: &[&str],
    minimum: f64,
    reference: Option<&'a str>,
) -> Vec<Verdict<'a>> {
    let mut periods = PERIOD.captures_iter(claim).peekable();
    let mut previous_end = 0;
    let mut results = Vec::new();
    while let Some(period) = periods.next() {
        let span = period.get(0).expect("whole period");
        let before = claim[previous_end..span.start()].trim();
        previous_end = span.end();
        let next_start = periods
            .peek()
            .and_then(|next| next.get(0))
            .map_or(claim.len(), |next| next.start());
        let after = claim[span.end()..next_start].trim();
        let nearest = LIST.split(before).last().unwrap_or(before);
        let attached = if before.is_empty() {
            classify(after, subject).positive
        } else {
            classify(nearest, subject).positive || item_matches(nearest, subject)
        };
        if !attached {
            continue;
        }
        let since = period.get(4);
        if since.is_some() && reference.is_none() {
            results.push(outcome(
                None,
                "minimum_duration_missing_reference",
                claim,
                None,
            ));
            continue;
        }
        let start = period.get(1).or_else(|| period.get(2)).or(since);
        let end = period.get(3).map(|value| value.as_str()).or(reference);
        let result = start
            .and_then(|value| date(value.as_str()))
            .zip(end.and_then(date))
            .and_then(|(start, end)| duration_verdict(minimum, start, end));
        let rule = if result.is_some() {
            "minimum_duration"
        } else {
            "minimum_duration_incomplete"
        };
        results.push(outcome(
            result,
            rule,
            claim,
            reference.filter(|_| since.is_some()),
        ));
    }
    results
}

fn criterion_target(criterion: &str) -> Option<(&str, Option<f64>)> {
    if let Some(matched) = MINIMUM_LEADING.captures(criterion) {
        let minimum = matched.get(1).or_else(|| matched.get(2))?;
        return Some((matched.get(3)?.as_str(), Some(threshold(minimum.as_str())?)));
    }
    if let Some(matched) = MINIMUM_TRAILING.captures(criterion) {
        return Some((
            matched.get(1)?.as_str(),
            Some(threshold(matched.get(2)?.as_str())?),
        ));
    }
    let target = USE.captures(criterion).and_then(|captures| {
        captures.get(2)?;
        let cue = captures.get(0)?;
        (cue.start() == 0).then_some(&criterion[cue.end()..])
    });
    let target = target.unwrap_or_else(|| {
        EXPERIENCE_TAIL
            .find(criterion)
            .map_or(criterion, |tail| &criterion[..tail.start()])
    });
    Some((target, None))
}

fn threshold(text: &str) -> Option<f64> {
    text.replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

pub(super) fn criterion_verdict(source: &str) -> Option<Verdict<'_>> {
    let [criteria, texts, references] = sections(source);
    let ([criterion], [text]) = (criteria.as_slice(), texts.as_slice()) else {
        return None;
    };
    let reference = match references.as_slice() {
        [reference] if !reference.is_empty() => Some(*reference),
        _ => None,
    };
    assess_criterion(criterion, text, reference)
}

pub(super) fn criterion_reference(header: &str) -> Option<&str> {
    match sections(header)[2].as_slice() {
        [reference] if !reference.is_empty() => Some(*reference),
        _ => None,
    }
}

pub(super) fn mentions_subject(text: &str, target: &str) -> bool {
    let target = target.to_lowercase();
    let subject: Vec<_> = TOKEN.find_iter(&target).map(|word| word.as_str()).collect();
    !subject.is_empty() && !subject_spans(text, &subject).is_empty()
}

pub(super) fn assess_criterion<'a>(
    criterion: &str,
    text: &'a str,
    reference: Option<&'a str>,
) -> Option<Verdict<'a>> {
    let (target, minimum) = criterion_target(criterion)?;
    let target = target.to_lowercase();
    let subject: Vec<_> = TOKEN.find_iter(&target).map(|word| word.as_str()).collect();
    if subject.is_empty() {
        return None;
    }
    let claims: Vec<_> = CLAIMS
        .split(text)
        .map(trim_span)
        .filter(|claim| !claim.is_empty())
        .collect();
    let classifications: Vec<_> = claims
        .iter()
        .map(|&claim| (claim, classify(claim, &subject)))
        .collect();
    let negative = classifications
        .iter()
        .find_map(|(claim, item)| item.negative.then_some(*claim));
    let positive = classifications
        .iter()
        .find_map(|(claim, item)| item.positive.then_some(*claim));
    if negative.is_some() && positive.is_some() {
        return Some(outcome(None, "conflicting_criterion_evidence", text, None));
    }
    if let Some(claim) = negative {
        return Some(outcome(Some(false), "skill_negation", claim, None));
    }
    if let Some(minimum) = minimum {
        let mut durations: Vec<_> = claims
            .iter()
            .copied()
            .flat_map(|claim| duration_evidence(claim, &subject, minimum, reference))
            .collect();
        if let Some(index) = durations.iter().position(|item| item.result == Some(true)) {
            return Some(durations.swap_remove(index));
        }
        if durations.len() == 1 {
            return durations.pop();
        }
        if durations.len() > 1 {
            return Some(outcome(None, "minimum_duration_incomplete", text, None));
        }
    } else if let Some(claim) = positive {
        return Some(outcome(Some(true), "skill_use", claim, None));
    }
    if positive.is_none()
        && let Some(claim) = classifications
            .iter()
            .find_map(|(claim, item)| item.excluded.then_some(*claim))
    {
        return Some(outcome(Some(false), "skill_exclusion", claim, None));
    }
    Some(outcome(None, "no_criterion_evidence", text, None))
}
