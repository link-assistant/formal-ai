//! SI dimension algebra for unit conversion (issue #700, E58).
//!
//! Issue #1176's engine (`rust/src/unit_conversion.rs`) converts pairs that
//! have an explicit `conversion` record in data/seed/meanings-units.lino
//! and inverts linear ones. Every pair the seed does not state — miles to
//! kilometers, horsepower to watts, psi to pascals — fell through, and
//! users got research suggestions instead of arithmetic.
//!
//! This module adds the missing half: every unit is placed in the
//! seven-dimensional SI quantity system (length L, mass M, time T,
//! electric current I, thermodynamic temperature K, amount of substance
//! N, luminous intensity J) with its exact factor to the coherent SI unit
//! of its dimension. Two units of the same dimension then convert through
//! SI — `value × factor(from) / factor(to)` — with no per-pair record at
//! all, and two units of different dimensions fail honestly as an
//! incompatibility instead of a wrong number.
//!
//! All data lives in `data/seed/si-unit-dimensions.lino` (mirrored at
//! rust/embedded/data/seed/): `base_dimension` records define the system,
//! `si_unit` records state one unit's `dimension` (an exponent string
//! like `L2MT-3`, the watt's kg·m²/s³) and its decimal `si_factor`, and
//! plain `surface` rows carry the recognition words in the languages the
//! seed covers. Arithmetic is exact rational arithmetic on `i128`; a
//! computation that cannot fit fails as [`SiConversion::Overflow`] rather
//! than silently losing precision.
//!
//! The user-facing handler wiring (dispatch entry, and the fallback call
//! from the #1176 engine when no stated pair matches) is left for the
//! maintainer's integration; `ENGINE_NAME` is the evidence-link token to
//! attach when it lands.

use std::fmt::Write as _;

use crate::seed::parser::parse_lino;

const SEED_PATH: &str = "data/seed/si-unit-dimensions.lino";

/// The evidence-link token naming this engine (issue #700 asks that
/// answers carry evidence naming the calculation engine used).
pub const ENGINE_NAME: &str = "si-dimension-algebra";

/// The seven SI base dimension symbols, in vector order.
pub const BASE_SYMBOLS: [char; 7] = ['L', 'M', 'T', 'I', 'K', 'N', 'J'];

/// A quantity dimension: exponents over the seven SI base dimensions.
///
/// `L2MT-3` (power) parses to `[2, 1, -3, 0, 0, 0, 0]`; `1` is
/// dimensionless (radians, counts, bits).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dimension {
    exponents: [i8; 7],
}

impl Dimension {
    /// The dimensionless dimension `1`.
    #[must_use]
    pub const fn dimensionless() -> Self {
        Self { exponents: [0; 7] }
    }

    /// Parse an exponent string such as `L`, `LT-1`, `L2MT-3`, or `1`.
    ///
    /// Returns `None` for unknown symbols or exponents that do not fit
    /// in an `i8`.
    #[must_use]
    pub fn parse(expr: &str) -> Option<Self> {
        let trimmed = expr.trim();
        if trimmed == "1" {
            return Some(Self::dimensionless());
        }
        let chars: Vec<char> = trimmed.chars().collect();
        let mut exponents = [0i8; 7];
        let mut index = 0;
        while index < chars.len() {
            let symbol = chars[index];
            let slot = BASE_SYMBOLS.iter().position(|base| *base == symbol)?;
            index += 1;
            let mut sign = 1i8;
            if index < chars.len() && chars[index] == '-' {
                sign = -1;
                index += 1;
            }
            let mut magnitude = 0i8;
            let mut digits = false;
            while index < chars.len() && chars[index].is_ascii_digit() {
                magnitude = magnitude
                    .checked_mul(10)?
                    .checked_add(i8::try_from(chars[index] as u8 - b'0').ok()?)?;
                digits = true;
                index += 1;
            }
            let exponent = if digits { sign * magnitude } else { sign };
            exponents[slot] = exponents[slot].checked_add(exponent)?;
        }
        Some(Self { exponents })
    }

    /// Whether this is the dimensionless dimension.
    #[must_use]
    pub fn is_dimensionless(&self) -> bool {
        self.exponents == [0; 7]
    }

    /// The exponent of one base dimension symbol, if it is one of the seven.
    #[must_use]
    pub fn exponent(&self, symbol: char) -> Option<i8> {
        BASE_SYMBOLS
            .iter()
            .position(|base| *base == symbol)
            .map(|slot| self.exponents[slot])
    }

    /// Render back to the seed's exponent-string notation.
    #[must_use]
    pub fn as_string(&self) -> String {
        if self.is_dimensionless() {
            return "1".to_owned();
        }
        let mut out = String::new();
        for (slot, symbol) in BASE_SYMBOLS.iter().enumerate() {
            let exponent = self.exponents[slot];
            match exponent {
                0 => {}
                1 => out.push(*symbol),
                n => {
                    let _ = write!(out, "{symbol}{n}");
                }
            }
        }
        out
    }

    /// The product of two dimensions (exponents add): speed `LT-1` times
    /// time `T` is length `L`.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Option<Self> {
        let mut exponents = [0i8; 7];
        for (slot, exponent) in exponents.iter_mut().enumerate() {
            *exponent = self.exponents[slot].checked_add(other.exponents[slot])?;
        }
        Some(Self { exponents })
    }

    /// The reciprocal dimension (exponents negate): time `T` inverted is
    /// frequency `T-1`.
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let mut exponents = [0i8; 7];
        for (slot, exponent) in exponents.iter_mut().enumerate() {
            *exponent = self.exponents[slot].checked_neg()?;
        }
        Some(Self { exponents })
    }
}

/// One `si_unit` record: canonical name, dimension, exact factor to the
/// coherent SI unit of that dimension, and recognition surfaces.
#[derive(Clone, Debug)]
pub struct SiUnitEntry {
    pub unit: String,
    pub dimension: Dimension,
    /// 1 unit = `factor_num / factor_den` SI units (e.g. 1 inch = 254/10000 m).
    pub factor_num: i128,
    pub factor_den: i128,
    pub surfaces: Vec<String>,
}

/// One `base_dimension` record of the seed.
#[derive(Clone, Debug)]
pub struct BaseDimension {
    pub symbol: char,
    pub quantity: String,
    pub unit: String,
}

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Parse a factor written as a terminating decimal (`0.0254`) or a
/// fraction (`254/10000`) into an exact reduced rational. Negative
/// factors are rejected: a unit's magnitude to SI is positive.
#[must_use]
pub fn parse_factor(text: &str) -> Option<(i128, i128)> {
    let trimmed = text.trim();
    if let Some((num, den)) = trimmed.split_once('/') {
        let num: i128 = num.trim().parse().ok()?;
        let den: i128 = den.trim().parse().ok()?;
        if num <= 0 || den <= 0 {
            return None;
        }
        return Some(reduce(num, den));
    }
    let (whole, fractional) = match trimmed.split_once('.') {
        Some((whole, fractional)) => (whole, fractional),
        None => (trimmed, ""),
    };
    if whole.is_empty() && fractional.is_empty() {
        return None;
    }
    if !whole.chars().all(|c| c.is_ascii_digit()) || !fractional.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let mut num: i128 = whole.parse().ok()?;
    let mut den: i128 = 1;
    for digit in fractional.chars() {
        num = num
            .checked_mul(10)?
            .checked_add(i128::from(digit as u8 - b'0'))?;
        den = den.checked_mul(10)?;
    }
    if num <= 0 {
        return None;
    }
    Some(reduce(num, den))
}

/// Greatest common divisor of two positive integers.
const fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.abs()
}

/// Reduce a rational to lowest terms.
const fn reduce(num: i128, den: i128) -> (i128, i128) {
    let common = gcd(num, den);
    if common <= 1 {
        return (num, den);
    }
    (num / common, den / common)
}

/// Every `base_dimension` record in the seed.
#[must_use]
pub fn base_dimensions() -> Vec<BaseDimension> {
    let mut out = Vec::new();
    let Some(text) = seed_text(SEED_PATH) else {
        return out;
    };
    let tree = parse_lino(text);
    let Some(root) = tree.children.iter().find(|child| child.name == "si_units") else {
        return out;
    };
    for record in &root.children {
        if record.name != "base_dimension" {
            continue;
        }
        let symbol = record.find_child_value("symbol").trim().to_owned();
        let Some(symbol) = symbol.chars().next() else {
            continue;
        };
        if !BASE_SYMBOLS.contains(&symbol) {
            continue;
        }
        out.push(BaseDimension {
            symbol,
            quantity: record.find_child_value("quantity").to_string(),
            unit: record.find_child_value("unit").to_string(),
        });
    }
    out
}

/// Every `si_unit` record in the seed (dimensions parsed, factors exact).
#[must_use]
pub fn si_units() -> Vec<SiUnitEntry> {
    let mut out = Vec::new();
    let Some(text) = seed_text(SEED_PATH) else {
        return out;
    };
    let tree = parse_lino(text);
    let Some(root) = tree.children.iter().find(|child| child.name == "si_units") else {
        return out;
    };
    for record in &root.children {
        if record.name != "si_unit" {
            continue;
        }
        let unit = record.find_child_value("unit").trim().to_owned();
        if unit.is_empty() {
            continue;
        }
        let Some(dimension) = Dimension::parse(record.find_child_value("dimension")) else {
            continue;
        };
        let Some((factor_num, factor_den)) = parse_factor(record.find_child_value("si_factor"))
        else {
            continue;
        };
        let surfaces = record
            .children
            .iter()
            .filter(|child| child.name == "surface")
            .filter_map(|child| {
                let surface = child.id.trim().to_lowercase();
                (!surface.is_empty()).then_some(surface)
            })
            .collect();
        out.push(SiUnitEntry {
            unit,
            dimension,
            factor_num,
            factor_den,
            surfaces,
        });
    }
    out
}

/// The canonical unit name for a surface word or canonical name
/// (`"метров"` → `"meter"`, `"马力"` → `"horsepower_mechanical"`).
#[must_use]
pub fn unit_named_by(surface: &str) -> Option<String> {
    let needle = surface.trim().to_lowercase();
    let units = si_units();
    units
        .iter()
        .find(|entry| entry.unit == needle || entry.surfaces.contains(&needle))
        .map(|entry| entry.unit.clone())
}

/// The outcome of a conversion through SI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SiConversion {
    /// The exact converted value as a reduced rational.
    Converted { value_num: i128, value_den: i128 },
    /// The units exist but measure different dimensions (meters to
    /// seconds); converting would fabricate a number.
    Incompatible { from: Dimension, to: Dimension },
    /// A unit the seed does not carry; the gap is named, not papered over.
    UnknownUnit(String),
    /// The exact rational does not fit in i128 at this magnitude.
    Overflow,
}

impl SiConversion {
    /// The converted value as `f64` (for display and closeness checks).
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Converted {
                value_num,
                value_den,
            } => {
                // Decimal text parses to the nearest f64, the same rounding
                // an `as` cast performs, without a lossy-cast lint.
                let num: f64 = value_num.to_string().parse().ok()?;
                let den: f64 = value_den.to_string().parse().ok()?;
                Some(num / den)
            }
            _ => None,
        }
    }

    /// A human-readable form (the localized response template renders
    /// around it).
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Converted {
                value_num,
                value_den,
            } => format!("{value_num}/{value_den}"),
            Self::Incompatible { from, to } => crate::seed::report_text(
                "si_conversion_incompatible",
                &[("from", &from.as_string()), ("to", &to.as_string())],
            ),
            Self::UnknownUnit(unit) => {
                crate::seed::report_text("si_conversion_unknown_unit", &[("unit", unit)])
            }
            Self::Overflow => "value out of range for exact conversion".to_owned(),
        }
    }
}

/// Convert a value between two units of the same dimension, through the
/// coherent SI unit of that dimension. `from`/`to` accept canonical
/// names or surface words.
///
/// ```text
/// convert_through_si(3, 1, "horsepower", "watt") == 745.699872 × 3
/// convert_through_si(5, 1, "kilometer", "mile") ≈ 3.106856
/// convert_through_si(1, 1, "meter", "second")   == Incompatible { L, T }
/// convert_through_si(1, 1, "furlong", "meter")  == UnknownUnit("furlong")
/// ```
#[must_use]
pub fn convert_through_si(value_num: i128, value_den: i128, from: &str, to: &str) -> SiConversion {
    let units = si_units();
    let resolve = |name: &str| -> Option<&SiUnitEntry> {
        let needle = name.trim().to_lowercase();
        units
            .iter()
            .find(|entry| entry.unit == needle || entry.surfaces.contains(&needle))
    };
    let Some(from_entry) = resolve(from) else {
        return SiConversion::UnknownUnit(from.trim().to_owned());
    };
    let Some(to_entry) = resolve(to) else {
        return SiConversion::UnknownUnit(to.trim().to_owned());
    };
    if from_entry.dimension != to_entry.dimension {
        return SiConversion::Incompatible {
            from: from_entry.dimension,
            to: to_entry.dimension,
        };
    }
    // result = value × factor(from) / factor(to), kept exact:
    // num = value_num × from_num × to_den, den = value_den × from_den × to_num.
    let num = (|| {
        value_num
            .checked_mul(from_entry.factor_num)?
            .checked_mul(to_entry.factor_den)
    })();
    let den = (|| {
        value_den
            .checked_mul(from_entry.factor_den)?
            .checked_mul(to_entry.factor_num)
    })();
    // A zero or negative value converts fine; only the denominator must
    // stay positive.
    match (num, den) {
        (Some(num), Some(den)) if den > 0 => SiConversion::Converted {
            value_num: reduce(num, den).0,
            value_den: reduce(num, den).1,
        },
        _ => SiConversion::Overflow,
    }
}

/// Log an honest named gap when a unit is not in the seed, so the event
/// log says what is missing rather than dropping the request silently.
pub fn note_unknown_unit(log: &mut crate::event_log::EventLog, unit: &str) {
    log.append("si_units:unknown_unit", unit.trim().to_owned());
}
