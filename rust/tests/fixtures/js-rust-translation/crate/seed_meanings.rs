// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0ce3b636505c3bb4fe7c2a836087de861e96cbe7eceeb54607b5224e3944f8fe bytes=12355
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
/// Unbounded integers for the portable core's naturals and integers.
pub mod ml {
    use std::cmp::Ordering;
    use std::fmt;

    /// Sign and magnitude; the magnitude is little-endian base 2^32 without
    /// leading zero limbs, and zero is never negative.
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Big {
        negative: bool,
        magnitude: Vec<u32>,
    }

    fn trim(mut magnitude: Vec<u32>) -> Vec<u32> {
        while magnitude.last() == Some(&0) {
            magnitude.pop();
        }
        magnitude
    }

    fn compare_magnitude(a: &[u32], b: &[u32]) -> Ordering {
        a.len().cmp(&b.len()).then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }

    fn add_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for index in 0..a.len().max(b.len()) {
            let sum = u64::from(*a.get(index).unwrap_or(&0)) + u64::from(*b.get(index).unwrap_or(&0)) + carry;
            out.push(sum as u32);
            carry = sum >> 32;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
    }

    /// |a| - |b| where |a| >= |b|.
    fn sub_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for (index, limb) in a.iter().enumerate() {
            let mut difference = i64::from(*limb) - i64::from(*b.get(index).unwrap_or(&0)) - borrow;
            borrow = 0;
            if difference < 0 {
                difference += 1 << 32;
                borrow = 1;
            }
            out.push(difference as u32);
        }
        trim(out)
    }

    fn mul_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = vec![0u32; a.len() + b.len()];
        for (i, x) in a.iter().enumerate() {
            let mut carry = 0u64;
            for (j, y) in b.iter().enumerate() {
                let product = u64::from(out[i + j]) + u64::from(*x) * u64::from(*y) + carry;
                out[i + j] = product as u32;
                carry = product >> 32;
            }
            let mut index = i + b.len();
            while carry > 0 {
                let sum = u64::from(out[index]) + carry;
                out[index] = sum as u32;
                carry = sum >> 32;
                index += 1;
            }
        }
        trim(out)
    }

    /// Quotient and remainder of magnitudes; the divisor is not zero.
    fn divmod_magnitude(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        let mut quotient = vec![0u32; a.len()];
        if b.len() == 1 {
            let divisor = u64::from(b[0]);
            let mut remainder = 0u64;
            for index in (0..a.len()).rev() {
                let current = (remainder << 32) | u64::from(a[index]);
                quotient[index] = (current / divisor) as u32;
                remainder = current % divisor;
            }
            return (trim(quotient), trim(vec![remainder as u32]));
        }
        let mut remainder: Vec<u32> = Vec::new();
        for bit in (0..a.len() * 32).rev() {
            let mut carry = (a[bit / 32] >> (bit % 32)) & 1;
            for limb in remainder.iter_mut() {
                let next = *limb >> 31;
                *limb = (*limb << 1) | carry;
                carry = next;
            }
            if carry != 0 {
                remainder.push(carry);
            }
            if compare_magnitude(&remainder, b) != Ordering::Less {
                remainder = sub_magnitude(&remainder, b);
                quotient[bit / 32] |= 1 << (bit % 32);
            }
        }
        (trim(quotient), remainder)
    }

    impl Big {
        fn from_parts(negative: bool, magnitude: Vec<u32>) -> Self {
            let magnitude = trim(magnitude);
            Self { negative: negative && !magnitude.is_empty(), magnitude }
        }

        pub fn zero() -> Self {
            Self::from_parts(false, Vec::new())
        }

        pub fn from_i128(value: i128) -> Self {
            let mut big = Self::from_u128(value.unsigned_abs());
            big.negative = value < 0;
            big
        }

        pub fn from_u128(mut value: u128) -> Self {
            let mut magnitude = Vec::new();
            while value > 0 {
                magnitude.push(value as u32);
                value >>= 32;
            }
            Self::from_parts(false, magnitude)
        }

        /// A decimal literal with an optional leading minus sign.
        pub fn parse(text: &str) -> Self {
            let (negative, digits) = match text.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, text),
            };
            let ten = Self::from_i128(10);
            let mut value = Self::zero();
            for digit in digits.chars() {
                let digit = digit.to_digit(10).expect("decimal literal");
                value = value.mul(&ten).add(&Self::from_i128(i128::from(digit)));
            }
            if negative { value.neg() } else { value }
        }

        pub fn is_zero(&self) -> bool {
            self.magnitude.is_empty()
        }

        pub fn is_negative(&self) -> bool {
            self.negative
        }

        /// The array index this names, if it is one.
        pub fn to_index(&self) -> Option<usize> {
            if self.negative || self.magnitude.len() > 2 {
                return None;
            }
            let value = self.magnitude.iter().rev().fold(0u64, |acc, &digit| (acc << 32) | u64::from(digit));
            usize::try_from(value).ok()
        }

        pub fn neg(&self) -> Self {
            Self::from_parts(!self.negative, self.magnitude.clone())
        }

        pub fn add(&self, other: &Self) -> Self {
            if self.negative == other.negative {
                return Self::from_parts(self.negative, add_magnitude(&self.magnitude, &other.magnitude));
            }
            match compare_magnitude(&self.magnitude, &other.magnitude) {
                Ordering::Less => Self::from_parts(other.negative, sub_magnitude(&other.magnitude, &self.magnitude)),
                _ => Self::from_parts(self.negative, sub_magnitude(&self.magnitude, &other.magnitude)),
            }
        }

        pub fn sub(&self, other: &Self) -> Self {
            self.add(&other.neg())
        }

        pub fn mul(&self, other: &Self) -> Self {
            Self::from_parts(self.negative != other.negative, mul_magnitude(&self.magnitude, &other.magnitude))
        }

        /// Division rounding toward zero; the divisor is not zero.
        pub fn divmod_trunc(&self, other: &Self) -> (Self, Self) {
            let (quotient, remainder) = divmod_magnitude(&self.magnitude, &other.magnitude);
            (
                Self::from_parts(self.negative != other.negative, quotient),
                Self::from_parts(self.negative, remainder),
            )
        }
    }

    impl Ord for Big {
        fn cmp(&self, other: &Self) -> Ordering {
            match (self.negative, other.negative) {
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (false, false) => compare_magnitude(&self.magnitude, &other.magnitude),
                (true, true) => compare_magnitude(&other.magnitude, &self.magnitude),
            }
        }
    }

    impl PartialOrd for Big {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl fmt::Display for Big {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.is_zero() {
                return formatter.write_str("0");
            }
            let billion = [1_000_000_000u32];
            let mut chunks = Vec::new();
            let mut rest = self.magnitude.clone();
            while !rest.is_empty() {
                let (quotient, remainder) = divmod_magnitude(&rest, &billion);
                chunks.push(remainder.first().copied().unwrap_or(0));
                rest = quotient;
            }
            let mut text = String::new();
            if self.negative {
                text.push('-');
            }
            text.push_str(&chunks.pop().unwrap_or(0).to_string());
            for chunk in chunks.iter().rev() {
                text.push_str(&format!("{chunk:09}"));
            }
            formatter.write_str(&text)
        }
    }

    /// Natural subtraction: truncated at zero.
    pub fn nat_sub(a: &Big, b: &Big) -> Big {
        if a > b { a.sub(b) } else { Big::zero() }
    }

    pub fn pred(a: &Big) -> Big {
        a.sub(&Big::from_i128(1))
    }

    /// Integer division with the source's rounding ("trunc", "floor" or
    /// "euclid"); by zero it aborts with the source's message or is total
    /// (x / 0 = 0, x % 0 = x).
    pub fn divide(a: &Big, b: &Big, rounding: &str, by_zero: Option<&str>, remainder: bool) -> Big {
        if b.is_zero() {
            if let Some(message) = by_zero {
                panic!("{message}");
            }
            return if remainder { a.clone() } else { Big::zero() };
        }
        let one = Big::from_i128(1);
        let (mut quotient, rest) = a.divmod_trunc(b);
        if !rest.is_zero() {
            if rounding == "floor" && rest.is_negative() != b.is_negative() {
                quotient = quotient.sub(&one);
            }
            if rounding == "euclid" && rest.is_negative() {
                quotient = if b.is_negative() { quotient.add(&one) } else { quotient.sub(&one) };
            }
        }
        if remainder { a.sub(&quotient.mul(b)) } else { quotient }
    }

    pub fn to_nat_checked(value: Big, message: &str) -> Big {
        if value.is_negative() {
            panic!("{message}");
        }
        value
    }

    pub fn clamp_nat(value: Big) -> Big {
        if value.is_negative() { Big::zero() } else { value }
    }

    /// Bounded domains for executable theorem checks.
    pub fn range(low: i128, high: i128) -> Vec<Big> {
        (low..=high).map(Big::from_i128).collect()
    }
}

/// Reads of JavaScript arrays, which the portable core never mutates.
pub mod ml_array {
    /// The element at an index; a read outside the array, undefined in
    /// JavaScript, aborts.
    pub fn at<T: Clone>(values: &[T], index: Option<usize>) -> T {
        match index.and_then(|index| values.get(index)) {
            Some(value) => value.clone(),
            None => panic!("array index out of range"),
        }
    }

    /// The index a Number names: a non-negative integer, -0 included.
    pub fn number_index(index: f64) -> Option<usize> {
        if index >= 0.0 && index.fract() == 0.0 && index < 9007199254740992.0 {
            Some(index as usize)
        } else {
            None
        }
    }

    pub fn append<T>(mut left: Vec<T>, right: Vec<T>) -> Vec<T> {
        left.extend(right);
        left
    }
}

pub fn wa_str_chars(text: String) -> Vec<String> {
    text.chars().map(String::from).collect() }

pub fn wa_str_code_point_at(text: String, index: f64) -> f64 {
    let units: Vec<u16> = text.encode_utf16().collect();
    let at = if index.is_nan() { 0.0 } else { index.trunc() };
    if at < 0.0 || at >= units.len() as f64 { return f64::NAN; }
    let first = units[at as usize];
    match units.get(at as usize + 1) {
        Some(&second) if (0xD800..0xDC00).contains(&first) && (0xDC00..0xE000).contains(&second) => {
            f64::from(((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00) + 0x10000)
        }
        _ => f64::from(first),
    } }
// formal-ai:workaround-prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript lexical_declaration items=1 sha256=2d9b8c1bb4b4437bcb320c603c2f802a000e40fa9092307c59f02af758705f8f
// | /** The one inventory of the seed files and the lexicons that read them. */
// | const SEED_REGISTRY = 'data/meta/seed-registry.lino';
pub const SEED_REGISTRY: &str = "data/meta/seed-registry.lino";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .some() | arrow function | call of an imported function | field access | method call .filter() | method call .map() | method call .some()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Map

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow function | field access | method call .find() | nullish coalescing | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .map() | global value Boolean | method call .filter() | method call .get() | method call .map() | method call .split() | nullish coalescing | regular expression | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers String.fromCodePoint | global call parseInt() | method call .filter() | method call .isFinite() | method call .slice() | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .filter() | assignment of a field or element | call of a sibling function | field access | method call .filter() | method call .map() | method call .push() | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | imported value | method call .join() | method call .map()

// formal-ai:workaround string-methods JavaScript export_statement items=3 sha256=cb0c64fa07c158a0f1b789acb897d3483928ff18ae34c1e243c4a1d2fb3e2db5
// | /** Mirrors `crate::coding::contains_cjk`. @param {string} text */
// | export function containsCjk(text) {
// |   for (const character of text) {
// |     const cp = character.codePointAt(0);
// |     if ((cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff)
// |       || (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3100 && cp <= 0x312f)) return true;
// |   }
// |   return false;
// | }
// ~ /** Mirrors `crate::coding::contains_cjk`. @param {string} text */
// ~ export function containsCjk(text) {
// ~   for (const character of waStrChars(text)) {
// ~     const cp = waStrCodePointAt(character, 0);
// ~     if ((cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff)
// ~       || (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3100 && cp <= 0x312f)) return true;
// ~   }
// ~   return false;
// ~ }
pub fn contains_cjk(text: String) -> bool {
    {
        let ml_values1 = crate::wa_str_chars(text.clone());
        {
            let ml_index1 = crate::ml::Big::from_i128(0);
            {
                let ml_s1 = crate::ml_contains_cjk_loop2(ml_values1.clone(), ml_index1.clone());
                match ml_s1.clone() {
                    crate::MlContainsCjkLoop2Result::MlContainsCjkLoop2Done => {
                        false
                    }
                    crate::MlContainsCjkLoop2Result::MlContainsCjkLoop2Return(ml_result2) => {
                        ml_result2
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MlContainsCjkLoop2Result {
    MlContainsCjkLoop2Done,
    MlContainsCjkLoop2Return(bool),
}

pub fn ml_contains_cjk_loop2(mut ml_values1: Vec<String>, mut ml_index1: crate::ml::Big) -> crate::MlContainsCjkLoop2Result {
    loop {
        return if (ml_index1 < (crate::ml::Big::from_u128(ml_values1.len() as u128))) {
            {
                let character = crate::ml_array::at(&ml_values1, ml_index1.to_index());
                {
                    let cp = crate::wa_str_code_point_at(character.clone(), 0f64);
                    if ((((((cp >= (13312f64)) && (cp <= (19903f64))) || ((cp >= (19968f64)) && (cp <= (40959f64)))) || ((cp >= (63744f64)) && (cp <= (64255f64)))) || ((cp >= (12352f64)) && (cp <= (12543f64)))) || ((cp >= (12544f64)) && (cp <= (12591f64)))) {
                        crate::MlContainsCjkLoop2Result::MlContainsCjkLoop2Return(true)
                    } else {
                        {
                            let ml_index1_2 = ml_index1.add(&crate::ml::Big::from_i128(1));
                            {
                                ml_index1 = ml_index1_2.clone();
                                continue;
                            }
                        }
                    }
                }
            }
        } else {
            crate::MlContainsCjkLoop2Result::MlContainsCjkLoop2Done
        };
    }
}

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: one value is used as a boolean and as a string; declare the types of the function with JSDoc

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers arrow function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | field access | method call .flatMap() | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | field access | method call .flatMap()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | null | nullish coalescing | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | field access | method call .filter() | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers field access | method call .indexOf()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers field access | method call .indexOf() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers field access | method call .indexOf() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers call of a sibling function | method call .flatMap() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | field access | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | method call .filter() | method call .some() | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .some()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=6e0c3d1fdc281b1934fb85c1db90e166dc9e6648a4f9ae4ca695389b1ad44c60
// | const PHRASAL_VERB_OBJECT_LIMIT = 6;
pub const PHRASAL_VERB_OBJECT_LIMIT: f64 = 6f64;

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .some() | assignment of a field or element | destructuring | global value Boolean | method call .filter() | method call .slice() | method call .some() | method call .split() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | field access | global value Boolean | method call .filter() | method call .flatMap() | method call .some() | method call .split() | regular expression | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .some() | call of a sibling function | field access | method call .filter() | method call .find() | method call .some() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | field access | method call .filter() | method call .some() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | method call .find() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | method call .find() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | nullish coalescing
