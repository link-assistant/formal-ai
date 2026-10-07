//! Statistics over number lists stated in the prompt, and the small
//! arithmetic word problems that share their vocabulary.
//!
//! Issue #1176: "what are the mean and the median of 4, 8, 15, 16, 23, 42?"
//! used to fall through to the generic fallback. The answer is pure
//! computation over stated values, so this handler derives it exactly — the
//! mean is 108 / 6 = 18, the median (15 + 16) / 2 = 15.5 — and shows the
//! arithmetic it performed. Every operation runs on integers; only the final
//! rendering writes a decimal point.
//!
//! Recognition vocabulary (operation names, word-problem markers) lives in
//! `data/seed/meanings-statistics.lino`; answer prose lives in
//! `data/seed/multilingual-responses-quantities.lino` (issue #386's
//! seed-first convention). Code here owns only the arithmetic.
//!
//! Numbers are read from a lowercased copy of the raw prompt, not the dispatch
//! `normalized` form: normalization turns every punctuation mark into a space,
//! which would split a stated "26.2" into two numbers and drops the question
//! mark these gates read.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{lexicon, localized_response};
use crate::solver_handlers::finalize_simple;
use std::cmp::Ordering;
use super::calendar::contains_term;
use super::numeric_list::parse_numbers;

mod primality;
mod word_relations;

/// Seed meaning slugs carrying the word-problem markers of issue #1176.
const MARKER_UNIT_PRICE: &str = "word_problem_unit_price";
const MARKER_PAYMENT: &str = "word_problem_payment";
const MARKER_CHANGE: &str = "word_problem_change";
const MARKER_TOTAL: &str = "word_problem_total";

/// Largest accepted mantissa magnitude (10¹²). Together with the scale and
/// count bounds below this keeps every intermediate product inside `i128`
/// (each step is checked anyway; a prompt beyond the bound is refused rather
/// than computed wrongly).
const MAX_MANTISSA: i128 = 1_000_000_000_000;
/// Largest accepted number of fractional digits in a stated value.
const MAX_SCALE: u32 = 9;
/// Largest accepted list length.
const MAX_VALUES: usize = 100;

/// An exact decimal: `mantissa × 10⁻ˢᶜᵃˡᵉ`.
///
/// Statistics and unit-conversion answers must be exact wherever exactness is
/// expressible (issue #1176): the mean of 4, 8, 15, 16, 23, 42 is exactly 18
/// and 26.2 miles is exactly 42.1648128 kilometres, so both are computed on
/// integer mantissas. Shared with `unit_conversion.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Decimal {
    mantissa: i128,
    scale: u32,
}

impl Decimal {
    /// Parse the exact decimal a prompt stated ("4", "26.2", "-3.5").
    /// Returns `None` for anything that is not a plain decimal literal or
    /// that exceeds the exact-arithmetic bounds above.
    pub(super) fn parse(text: &str) -> Option<Self> {
        let (negative, digits) = match text.as_bytes().first() {
            Some(b'-') => (true, &text[1..]),
            Some(b'+') => (false, &text[1..]),
            _ => (false, text),
        };
        let mut mantissa: i128 = 0;
        let mut scale: u32 = 0;
        let mut seen_digit = false;
        let mut seen_point = false;
        for character in digits.chars() {
            match character {
                '0'..='9' => {
                    seen_digit = true;
                    if seen_point {
                        scale += 1;
                        if scale > MAX_SCALE {
                            return None;
                        }
                    }
                    mantissa = mantissa
                        .checked_mul(10)?
                        .checked_add(i128::from(character as u8 - b'0'))?;
                }
                '.' if !seen_point => seen_point = true,
                _ => return None,
            }
        }
        if !seen_digit || mantissa > MAX_MANTISSA {
            return None;
        }
        Some(Self {
            mantissa: if negative { -mantissa } else { mantissa },
            scale,
        })
    }

    /// Re-express at a higher scale (`mantissa × 10ᵏ`, `scale + k`) — the same
    /// exact value. Lowering a scale is only exact when the dropped digits are
    /// zeros, and no operation here needs it.
    fn rescaled(self, scale: u32) -> Option<Self> {
        if scale < self.scale {
            return None;
        }
        let factor = 10_i128.checked_pow(scale - self.scale)?;
        Some(Self {
            mantissa: self.mantissa.checked_mul(factor)?,
            scale,
        })
    }

    pub(super) fn add(self, other: Self) -> Option<Self> {
        let scale = self.scale.max(other.scale);
        let left = self.rescaled(scale)?;
        let right = other.rescaled(scale)?;
        Some(Self {
            mantissa: left.mantissa.checked_add(right.mantissa)?,
            scale,
        })
    }

    pub(super) fn sub(self, other: Self) -> Option<Self> {
        let scale = self.scale.max(other.scale);
        let left = self.rescaled(scale)?;
        let right = other.rescaled(scale)?;
        Some(Self {
            mantissa: left.mantissa.checked_sub(right.mantissa)?,
            scale,
        })
    }

    /// Multiply exactly: both mantissas are integers, so the product never
    /// rounds (26.2 × 1.609344 = 42.1648128, not a float echo).
    pub(super) fn mul(self, other: Self) -> Option<Self> {
        Some(Self {
            mantissa: self.mantissa.checked_mul(other.mantissa)?,
            scale: self.scale + other.scale,
        })
    }

    /// Divide by `other`, carrying the quotient to `precision` fractional
    /// digits. The flag says whether the quotient terminated there; callers
    /// render an inexact quotient with a leading `≈` (issue #1176).
    pub(super) fn div(self, other: Self, precision: u32) -> Option<(Self, bool)> {
        if other.mantissa == 0 {
            return None;
        }
        // a/b = (am·10^bs) / (bm·10^as) as a plain integer fraction.
        let numerator = self.mantissa.checked_mul(10_i128.checked_pow(other.scale)?)?;
        let denominator = other.mantissa.checked_mul(10_i128.checked_pow(self.scale)?)?;
        Self::from_quotient(numerator, denominator, precision)
    }

    /// Multiply by the exact rational `numerator / denominator`, carrying the
    /// product to `self.scale + precision` fractional digits. The temperature
    /// formulas scale by 9/5 and 5/9, which no finite decimal factor states.
    pub(super) fn mul_div(
        self,
        numerator: i128,
        denominator: i128,
        precision: u32,
    ) -> Option<(Self, bool)> {
        if denominator == 0 {
            return None;
        }
        let scaled_numerator = self.mantissa.checked_mul(numerator)?;
        let scaled_denominator = 10_i128
            .checked_pow(self.scale)?
            .checked_mul(denominator)?;
        let scale = self.scale + precision;
        Self::from_quotient(scaled_numerator, scaled_denominator, scale)
    }

    /// The decimal `numerator / denominator` (plain integers) at `scale`
    /// fractional digits, rounded half away from zero on the last kept digit.
    fn from_quotient(numerator: i128, denominator: i128, scale: u32) -> Option<(Self, bool)> {
        if denominator == 0 {
            return None;
        }
        let scaled = numerator.checked_mul(10_i128.checked_pow(scale)?)?;
        let sign = if (scaled < 0) == (denominator < 0) {
            1_i128
        } else {
            -1_i128
        };
        let magnitude = scaled.unsigned_abs();
        let divisor = denominator.unsigned_abs();
        let mut mantissa = i128::try_from(magnitude / divisor).ok()?;
        let exact = magnitude % divisor == 0;
        if !exact && (magnitude % divisor) * 2 >= divisor {
            mantissa += 1;
        }
        Some((Self {
            mantissa: sign * mantissa,
            scale,
        }, exact))
    }

    /// Order by value. Scales align first; within the parse bounds above the
    /// aligned mantissa always fits `i128`, so the alignment cannot fail.
    fn compare(self, other: Self) -> Ordering {
        let scale = self.scale.max(other.scale);
        let left = self.rescaled(scale).expect("bounded decimals align");
        let right = other.rescaled(scale).expect("bounded decimals align");
        left.mantissa.cmp(&right.mantissa)
    }

    pub(super) const fn is_negative(self) -> bool {
        self.mantissa < 0
    }

    pub(super) const fn negate(self) -> Self {
        Self {
            mantissa: -self.mantissa,
            scale: self.scale,
        }
    }

    /// Exact rational form for the seed-driven SI dimension interpreter.
    pub(super) fn ratio(self) -> Option<(i128, i128)> {
        Some((self.mantissa, 10_i128.checked_pow(self.scale)?))
    }

    /// Render the minimal decimal form: no trailing fraction zeros, no `-0`.
    pub(super) fn render(self) -> String {
        let negative = self.mantissa < 0;
        let digits = self.mantissa.unsigned_abs().to_string();
        let scale = self.scale as usize;
        let mut text = if scale == 0 {
            digits
        } else if digits.len() <= scale {
            format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
        } else {
            let split = digits.len() - scale;
            format!("{}.{}", &digits[..split], &digits[split..])
        };
        if text.contains('.') {
            while text.ends_with('0') {
                text.pop();
            }
            if text.ends_with('.') {
                text.pop();
            }
        }
        if negative && text != "0" {
            format!("-{text}")
        } else {
            text
        }
    }
}

/// The statistics operations the seed vocabulary can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Mean,
    Median,
    Mode,
    Variance,
    StdDev,
    Percentile,
    Range,
}

/// Canonical answer order: a multi-operation question lists its results in
/// this order regardless of the order the words appeared in the prompt.
const OP_ORDER: [Op; 7] = [
    Op::Mean,
    Op::Median,
    Op::Mode,
    Op::Variance,
    Op::StdDev,
    Op::Percentile,
    Op::Range,
];

/// Most characters an ordinal suffix glued to a percentile rank may carry
/// ("90th", "90-й", "90वाँ").
const MAX_RANK_SUFFIX_CHARS: usize = 4;
/// Most whitespace characters between a percentile surface and a rank stated
/// after it ("percentil 90").
const MAX_RANK_LEAD_CHARS: usize = 3;

impl Op {
    /// The seed meaning slug carrying this operation's multilingual surfaces
    /// (`data/seed/meanings-statistics.lino`).
    const fn meaning_slug(self) -> &'static str {
        match self {
            Self::Mean => "statistics_mean",
            Self::Median => "statistics_median",
            Self::Mode => "statistics_mode",
            Self::Variance => "statistics_variance",
            Self::StdDev => "statistics_standard_deviation",
            Self::Percentile => "statistics_percentile",
            Self::Range => "statistics_range",
        }
    }

    /// Evidence-log kind for the computed line.
    const fn evidence_kind(self) -> &'static str {
        match self {
            Self::Mean => "statistics:mean",
            Self::Median => "statistics:median",
            Self::Mode => "statistics:mode",
            Self::Variance => "statistics:variance",
            Self::StdDev => "statistics:standard_deviation",
            Self::Percentile => "statistics:percentile",
            Self::Range => "statistics:range",
        }
    }
}

/// One computed operation: the rendered value and, when it clarifies, the
/// arithmetic that produced it.
struct OpResult {
    value: String,
    derivation: Option<String>,
}

/// Does the prompt mention any surface of the marker meaning `slug`? The
/// marker vocabulary (each, pays with, change, total, and every translation)
/// lives in `data/seed/meanings-statistics.lino` (issue #1176).
fn mentions_marker(normalized: &str, slug: &str) -> bool {
    lexicon().meaning(slug).is_some_and(|meaning| {
        meaning
            .words()
            .any(|word| contains_term(normalized, word))
    })
}

/// The statistics operations the prompt names, in canonical order. Operation
/// surfaces come from the seed, so a new language or synonym is a data edit.
fn detect_ops(normalized: &str) -> Vec<Op> {
    let lex = lexicon();
    let asked = |op: Op| {
        lex.meaning(op.meaning_slug()).is_some_and(|meaning| {
            meaning
                .words()
                .any(|word| contains_term(normalized, word))
        })
    };
    OP_ORDER.into_iter().filter(|op| asked(*op)).collect()
}

/// The numbers the prompt states, as exact decimals, with each surface's
/// byte offset in the lowercased prompt (used to role them in word problems).
fn stated_numbers(lowered: &str) -> Option<(Vec<Decimal>, Vec<usize>)> {
    let items = parse_numbers(lowered);
    let mut values = Vec::with_capacity(items.len());
    let mut positions = Vec::with_capacity(items.len());
    for item in items {
        values.push(Decimal::parse(&item.text)?);
        positions.push(lowered.find(&item.text)?);
        if values.len() > MAX_VALUES {
            return None;
        }
    }
    Some((values, positions))
}

/// Index of the position nearest before `limit` (the largest one still under
/// it), if any — used to find the price stated next to "each".
fn nearest_before(positions: &[usize], limit: usize) -> Option<usize> {
    positions
        .iter()
        .enumerate()
        .filter(|(_, position)| **position < limit)
        .max_by_key(|(_, position)| **position)
        .map(|(index, _)| index)
}

/// The mean of the stated values: the sum divided by the count, exact when
/// the quotient terminates (108 / 6 = 18 for the issue's example).
fn mean_of(values: &[Decimal]) -> Option<OpResult> {
    let count = Decimal {
        mantissa: i128::try_from(values.len()).ok()?,
        scale: 0,
    };
    let mut sum = values[0];
    for value in &values[1..] {
        sum = sum.add(*value)?;
    }
    let (mean, exact) = sum.div(count, 7)?;
    let joiner = if exact { "=" } else { "≈" };
    Some(OpResult {
        value: render_approx(mean, exact),
        derivation: Some(format!(
            "{} / {} {} {}",
            sum.render(),
            values.len(),
            joiner,
            mean.render()
        )),
    })
}

/// The median: the middle value once sorted, or the exact average of the two
/// middle values when the count is even ((15 + 16) / 2 = 15.5).
fn median_of(values: &[Decimal]) -> Option<OpResult> {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.compare(*right));
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        return Some(OpResult {
            value: sorted[middle].render(),
            derivation: None,
        });
    }
    let total = sorted[middle - 1].add(sorted[middle])?;
    let (median, exact) = total.div(
        Decimal {
            mantissa: 2,
            scale: 0,
        },
        7,
    )?;
    let joiner = if exact { "=" } else { "≈" };
    Some(OpResult {
        value: render_approx(median, exact),
        // The "/ 2" is stated: "(15 + 16) = 15.5" would display a false sum.
        derivation: Some(format!(
            "({} + {}) / 2 {} {}",
            sorted[middle - 1].render(),
            sorted[middle].render(),
            joiner,
            median.render()
        )),
    })
}

/// The mode: the most frequent value. When every value is equally frequent
/// every value is a mode, and all are listed (2, 3, 4, 5 has no repeat to
/// point at, so hiding the line would be the dishonest option).
fn mode_of(values: &[Decimal]) -> Option<OpResult> {
    let mut counts: Vec<(Decimal, usize)> = Vec::new();
    for value in values {
        match counts
            .iter_mut()
            .find(|(seen, _)| seen.compare(*value) == Ordering::Equal)
        {
            Some(entry) => entry.1 += 1,
            None => counts.push((*value, 1)),
        }
    }
    let best = counts.iter().map(|(_, count)| *count).max()?;
    let modes = counts
        .iter()
        .filter(|(_, count)| *count == best)
        .map(|(value, _)| value.render())
        .collect::<Vec<_>>();
    Some(OpResult {
        value: modes.join(", "),
        derivation: None,
    })
}

/// The exact population variance as the integer fraction
/// `(n·Σx² − (Σx)²) / n²` at the values' common scale. The subtraction form
/// keeps every intermediate an integer, so a mean that does not terminate
/// (4, 7, 11 → 22/3) never rounds mid-derivation (issue #1176).
fn variance_fraction(values: &[Decimal]) -> Option<(i128, i128, u32)> {
    let scale = values.iter().map(|value| value.scale).max()?;
    let mut sum = 0_i128;
    let mut sum_squares = 0_i128;
    for value in values {
        let mantissa = value.rescaled(scale)?.mantissa;
        sum = sum.checked_add(mantissa)?;
        sum_squares = sum_squares.checked_add(mantissa.checked_mul(mantissa)?)?;
    }
    let count = i128::try_from(values.len()).ok()?;
    // n·Σx² − (Σx)² ≥ 0 by Cauchy–Schwarz, so a negative here is overflow.
    let numerator = count
        .checked_mul(sum_squares)?
        .checked_sub(sum.checked_mul(sum)?)?;
    let denominator = count.checked_mul(count)?;
    Some((numerator, denominator, 2 * scale))
}

fn variance_of(values: &[Decimal]) -> Option<OpResult> {
    let (numerator, denominator, scale) = variance_fraction(values)?;
    let count = Decimal {
        mantissa: i128::try_from(values.len()).ok()?,
        scale: 0,
    };
    // The variance is `numerator / denominator` (n², not n): the fraction is
    // already (n·Σx² − (Σx)²) / n², so dividing by n again would square the
    // count away and report Σ(x − mean)² instead of its average (issue #1176).
    let (variance, exact) =
        Decimal {
            mantissa: numerator,
            scale,
        }
        .div(
            Decimal {
                mantissa: denominator,
                scale: 0,
            },
            7,
        )?;
    let mut sum = values[0];
    for value in &values[1..] {
        sum = sum.add(*value)?;
    }
    // The textbook Σ(x − mean)² / n reads better than the algebraic form, so
    // show it (910 / 6 for the issue's example) when the mean terminates;
    // otherwise the exact fraction stays in the evidence log.
    let mut derivation = None;
    if let Some((mean, true)) = sum.div(count, 7) {
        let mut squared_differences = Decimal {
            mantissa: 0,
            scale: 0,
        };
        let mut derivable = true;
        for value in values {
            match value.sub(mean).and_then(|diff| diff.mul(diff)) {
                Some(square) => squared_differences = squared_differences.add(square)?,
                None => derivable = false,
            }
        }
        if derivable && let Some((text, text_exact)) = squared_differences.div(count, 7) {
            let joiner = if text_exact { "=" } else { "≈" };
            derivation = Some(format!(
                "{} / {} {} {}",
                squared_differences.render(),
                values.len(),
                joiner,
                text.render()
            ));
        }
    }
    Some(OpResult {
        value: render_approx(variance, exact),
        derivation,
    })
}

/// The standard deviation: the square root of the variance. Irrational
/// outside perfect squares, it is the one statistics result rendered as a
/// marked approximation; the variance it derives from stays exact, and the
/// magnitude guard keeps the double-precision root honest about its seven
/// displayed fractional digits.
fn std_dev_of(values: &[Decimal]) -> Option<OpResult> {
    let (numerator, denominator, scale) = variance_fraction(values)?;
    // Decimal text parses to the nearest f64, the same rounding an `as`
    // cast performs, without a lossy-cast lint.
    let numerator: f64 = numerator.to_string().parse().ok()?;
    let denominator: f64 = denominator.to_string().parse().ok()?;
    let variance =
        numerator / denominator / 10_f64.powi(i32::try_from(scale).expect(
            "the scale bound keeps every list scale within i32",
        ));
    if !(0.0..=1_000_000_000.0).contains(&variance) {
        return None;
    }
    let root = variance.sqrt();
    // For finite values a difference of exactly zero is equality.
    let exact = root.mul_add(root, -variance) == 0.0;
    let mut text = format!("{root:.7}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    Some(OpResult {
        value: if exact {
            text
        } else {
            format!("≈{text}")
        },
        derivation: None,
    })
}

/// The range: the largest value minus the smallest (42 − 4 = 38).
fn range_of(values: &[Decimal]) -> Option<OpResult> {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.compare(*right));
    let smallest = sorted[0];
    let largest = sorted[sorted.len() - 1];
    let span = largest.sub(smallest)?;
    Some(OpResult {
        value: span.render(),
        derivation: Some(format!(
            "{} - {} = {}",
            largest.render(),
            smallest.render(),
            span.render()
        )),
    })
}

/// The percentile by linear interpolation between closest ranks (the C = 1
/// variant the seed meaning cites): the rank 1 + (n − 1) · p / 100 = k + f
/// over the sorted values gives x(k) + f · (x(k + 1) − x(k)). The rank is the
/// integer fraction (n − 1) · p / (100 · 10ˢᶜᵃˡᵉ), whose denominator has only
/// the factors 2 and 5, so the fraction, the rank and the interpolated value
/// all terminate within `scale + 2` digits and the answer is exact.
fn percentile_of(values: &[Decimal], rank: Decimal, language: &str) -> Option<OpResult> {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.compare(*right));
    let gaps = i128::try_from(sorted.len()).ok()? - 1;
    let numerator = gaps.checked_mul(rank.mantissa)?;
    let denominator = 100_i128.checked_mul(10_i128.checked_pow(rank.scale)?)?;
    let lower_index = usize::try_from(numerator / denominator).ok()?;
    let remainder = numerator % denominator;
    let lower = *sorted.get(lower_index)?;
    let upper = sorted.get(lower_index + 1).copied().unwrap_or(lower);
    let precision = rank.scale + 2;
    let (fraction, _) = Decimal::from_quotient(remainder, denominator, precision)?;
    let (offset, _) = Decimal::from_quotient(numerator, denominator, precision)?;
    let position = offset.add(Decimal {
        mantissa: 1,
        scale: 0,
    })?;
    let (step, _) = upper
        .sub(lower)?
        .mul_div(remainder, denominator, precision)?;
    let value = lower.add(step)?;
    let derivation =
        localized_response("statistics_percentile_derivation", language).map(|template| {
            template
                .replace(concat!("{", "p}"), &rank.render())
                .replace(concat!("{", "count}"), &sorted.len().to_string())
                .replace(concat!("{", "rank}"), &position.render())
                .replace(concat!("{", "lower}"), &lower.render())
                .replace(concat!("{", "fraction}"), &fraction.render())
                .replace(concat!("{", "upper}"), &upper.render())
                .replace(concat!("{", "value}"), &value.render())
        });
    Some(OpResult {
        value: value.render(),
        derivation,
    })
}

/// Byte span of the earliest surface of the meaning `slug` in `lowered` (the
/// longest one on a tie), if any.
fn surface_span(lowered: &str, slug: &str) -> Option<(usize, usize)> {
    lexicon().meaning(slug).and_then(|meaning| {
        meaning
            .words()
            .filter_map(|word| {
                super::calendar::term_position(lowered, word)
                    .map(|start| (start, start + word.len()))
            })
            .min_by_key(|(start, end)| (*start, std::cmp::Reverse(*end)))
    })
}

/// Index, among the stated numbers, of the percentile rank `p` — the number
/// glued to the percentile surface: before it with at most a short ordinal
/// suffix ("the 90th percentile", "90-й перцентиль", "第90百分位数"), or right
/// after it and not opening the list ("el percentil 90 de 1, 2, 3"). The
/// spans are found in order, so a rank that repeats a listed value is still
/// told apart from it.
fn percentile_rank_index(lowered: &str) -> Option<usize> {
    let (surface_start, surface_end) = surface_span(lowered, Op::Percentile.meaning_slug())?;
    let mut spans = Vec::new();
    let mut cursor = 0;
    for item in parse_numbers(lowered) {
        let start = cursor + lowered[cursor..].find(&item.text)?;
        cursor = start + item.text.len();
        spans.push((start, cursor));
    }
    let before = spans
        .iter()
        .rposition(|(_, end)| *end <= surface_start)
        .filter(|index| {
            let suffix = lowered[spans[*index].1..surface_start].trim_end();
            suffix.chars().count() <= MAX_RANK_SUFFIX_CHARS
                && suffix
                    .chars()
                    .all(|character| character.is_alphabetic() || character == '-')
        });
    before.or_else(|| {
        let index = spans.iter().position(|(start, _)| *start >= surface_end)?;
        let (start, end) = spans[index];
        let lead = &lowered[surface_end..start];
        let opens_list = lowered[end..]
            .chars()
            .next()
            .is_some_and(|character| matches!(character, ',' | '，' | '、' | ';'));
        (lead.chars().count() <= MAX_RANK_LEAD_CHARS
            && lead.chars().all(char::is_whitespace)
            && !opens_list)
            .then_some(index)
    })
}

/// Render a decimal, marking inexact quotients so the answer never claims
/// more precision than was computed (issue #1176).
fn render_approx(value: Decimal, exact: bool) -> String {
    if exact {
        value.render()
    } else {
        format!("≈{}", value.render())
    }
}

/// Statistics entry point: answer questions that name a statistics operation
/// over a stated list of at least two numbers.
pub fn handle_statistics(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // R1017: a number property of one stated number ("is 97 prime?") shares
    // the stated-values surface; it needs no list and no question mark.
    if let Some(answer) = primality::handle_primality(prompt, log) {
        return Some(answer);
    }
    // Question gate: statistics prompts ask ("what is the mean of …?"). The
    // mark also keeps a stray "range" in a coding prompt from reading as the
    // statistics range; ASCII and fullwidth forms both count.
    let lowered = prompt.to_lowercase();
    if !lowered.contains(['?', '？']) {
        return None;
    }
    let ops = detect_ops(normalized);
    if ops.is_empty() {
        return None;
    }
    let (mut values, _) = stated_numbers(&lowered)?;
    // The percentile rank is a parameter, not a data point: it leaves the
    // list before any operation reads it, and an unstated or out-of-range
    // rank declines the question rather than guessing one.
    let rank = if ops.contains(&Op::Percentile) {
        let index = percentile_rank_index(&lowered)?;
        let stated = *values.get(index)?;
        values.remove(index);
        let hundred = Decimal {
            mantissa: 100,
            scale: 0,
        };
        if stated.is_negative() || stated.compare(hundred) == Ordering::Greater {
            return None;
        }
        Some(stated)
    } else {
        None
    };
    if values.len() < 2 {
        return None;
    }
    let language = detect_language(prompt).slug();
    let values_text = values
        .iter()
        .map(|value| value.render())
        .collect::<Vec<_>>()
        .join(", ");
    log.append("statistics:values", values_text.clone());
    log.append("statistics:count", values.len().to_string());

    let lex = lexicon();
    let mut lines = Vec::new();
    for op in ops {
        let result = match op {
            Op::Mean => mean_of(&values)?,
            Op::Median => median_of(&values)?,
            Op::Mode => mode_of(&values)?,
            Op::Variance => variance_of(&values)?,
            Op::StdDev => std_dev_of(&values)?,
            Op::Percentile => percentile_of(&values, rank?, language)?,
            Op::Range => range_of(&values)?,
        };
        // The operation's name in the answer language comes from the seed
        // lexeme, so labelling results in a new language is a data edit.
        let label = lex
            .meaning(op.meaning_slug())
            .and_then(|meaning| meaning.word_in(language).or_else(|| meaning.word_in("en")))
            .unwrap_or_else(|| op.meaning_slug())
            .to_owned();
        let line = match &result.derivation {
            Some(derivation) => format!("{}: {} ({})", label, result.value, derivation),
            None => format!("{}: {}", label, result.value),
        };
        log.append(op.evidence_kind(), line.clone());
        lines.push(line);
    }
    let results = lines.join("\n");
    let body = localized_response("statistics", language)
        .map(|template| {
            template
                .replace(concat!("{", "values}"), &values_text)
                .replace("{count}", &values.len().to_string())
                .replace(concat!("{", "results}"), &results)
        })
        .unwrap_or(results);
    Some(finalize_simple(
        prompt,
        log,
        "statistics",
        "response:statistics",
        &body,
        1.0,
    ))
}

/// Byte offset of the earliest surface of the marker meaning `slug` in the
/// lowercased prompt, if any — used to role the stated numbers.
fn marker_position(lowered: &str, slug: &str) -> Option<usize> {
    lexicon().meaning(slug).and_then(|meaning| {
        meaning
            .words()
            .filter_map(|word| super::calendar::term_position(lowered, word))
            .min()
    })
}

/// Word-problem entry point for the price×count pattern of issue #1176.
///
/// "buys 4 pens at 3 dollars each and pays with 20; what is the change?" → 20 −
/// 4 × 3 = 8 — and its total sibling ("what is the total?" → 4 × 3 = 12). The
/// markers are seed vocabulary; the arithmetic is shown in the answer.
pub fn handle_word_problem(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !mentions_marker(normalized, MARKER_UNIT_PRICE) {
        return word_relations::relation_word_problem(prompt, normalized, log);
    }
    let lowered = prompt.to_lowercase();
    let (values, positions) = stated_numbers(&lowered)?;
    let language = detect_language(prompt).slug();

    // Change: a unit price, a count, and a payment. The price is stated
    // nearest before the unit-price marker ("at 3 dollars each"), the payment
    // is the last stated number, and the count is the remaining one.
    if mentions_marker(normalized, MARKER_PAYMENT)
        && mentions_marker(normalized, MARKER_CHANGE)
        && values.len() == 3
    {
        let each_at = marker_position(&lowered, MARKER_UNIT_PRICE).unwrap_or(0);
        let price_index = nearest_before(&positions, each_at).unwrap_or(0);
        let payment_index = values.len() - 1;
        let count_index =
            (0..values.len()).find(|index| *index != price_index && *index != payment_index)?;
        let price = values[price_index];
        let count = values[count_index];
        let payment = values[payment_index];
        let cost = count.mul(price)?;
        let change = payment.sub(cost)?;
        if change.is_negative() {
            // A payment below the cost is not a change question this small
            // pattern can answer; decline instead of guessing.
            return None;
        }
        log.append("word_problem:unit_price", price.render());
        log.append("word_problem:count", count.render());
        log.append("word_problem:payment", payment.render());
        log.append("word_problem:change", change.render());
        let derivation = format!(
            "{} - {} × {} = {}",
            payment.render(),
            count.render(),
            price.render(),
            change.render()
        );
        log.append("word_problem:derivation", derivation.clone());
        let body = localized_response("word_problem_change", language)
            .map(|template| {
                template
                    .replace(concat!("{", "change}"), &change.render())
                    .replace(concat!("{", "derivation}"), &derivation)
            })
            .unwrap_or(derivation);
        return Some(finalize_simple(
            prompt,
            log,
            "word_problem_change",
            "response:word_problem_change",
            &body,
            1.0,
        ));
    }

    // Total: a unit price and a count ("4 pens at 3 dollars each; what is
    // the total?" → 4 × 3 = 12).
    if mentions_marker(normalized, MARKER_TOTAL) && values.len() == 2 {
        let each_at = marker_position(&lowered, MARKER_UNIT_PRICE).unwrap_or(0);
        let price_index = nearest_before(&positions, each_at).unwrap_or(0);
        let count_index = (0..values.len()).find(|index| *index != price_index)?;
        let price = values[price_index];
        let count = values[count_index];
        let total = count.mul(price)?;
        log.append("word_problem:unit_price", price.render());
        log.append("word_problem:count", count.render());
        log.append("word_problem:total", total.render());
        let derivation = format!(
            "{} × {} = {}",
            count.render(),
            price.render(),
            total.render()
        );
        log.append("word_problem:derivation", derivation.clone());
        let body = localized_response("word_problem_total", language)
            .map(|template| {
                template
                    .replace(concat!("{", "total}"), &total.render())
                    .replace(concat!("{", "derivation}"), &derivation)
            })
            .unwrap_or(derivation);
        return Some(finalize_simple(
            prompt,
            log,
            "word_problem_total",
            "response:word_problem_total",
            &body,
            1.0,
        ));
    }

    word_relations::relation_word_problem(prompt, normalized, log)
}
