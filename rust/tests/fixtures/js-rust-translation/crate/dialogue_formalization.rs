// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=6c770c0675d07e8bbd1b3e09e4989ecb5152b5c8c50d8a2746584b38d5bac4f4 bytes=9927
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

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
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers export … | imported value

// meta-language:translated JavaScript lexical_declaration items=1 sha256=601c7f16f52095487eaaecad6e293c7b2ebf13fbdae15d88be0152a1156bad58
// | const questionKinds=[['where','location'],['how-many','quantity'],['why','text'],
// |   ['who','entity'],['when','time'],['which','text']];
// formal-ai:presentation:v1 rawSHA256=601c7f16f52095487eaaecad6e293c7b2ebf13fbdae15d88be0152a1156bad58 rawBytes=414 renderedSHA256=7302e6b7e5296124562ba724e5bdd9085d15c25d191bf55742dc0efb689ef498
// formal-ai:presentation-raw cHViIHN0YXRpYyBxdWVzdGlvbktpbmRzOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxWZWM8U3RyaW5nPj4+ID0gc3RkOjpz
// formal-ai:presentation-raw eW5jOjpMYXp5TG9jazo6bmV3KHx8IHZlYyFbdmVjIVtTdHJpbmc6OmZyb20oIndoZXJlIiksIFN0cmluZzo6ZnJvbSgibG9j
// formal-ai:presentation-raw YXRpb24iKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCJob3ctbWFueSIpLCBTdHJpbmc6OmZyb20oInF1YW50aXR5IildLCB2ZWMh
// formal-ai:presentation-raw W1N0cmluZzo6ZnJvbSgid2h5IiksIFN0cmluZzo6ZnJvbSgidGV4dCIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIndobyIpLCBT
// formal-ai:presentation-raw dHJpbmc6OmZyb20oImVudGl0eSIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIndoZW4iKSwgU3RyaW5nOjpmcm9tKCJ0aW1lIild
// formal-ai:presentation-raw LCB2ZWMhW1N0cmluZzo6ZnJvbSgid2hpY2giKSwgU3RyaW5nOjpmcm9tKCJ0ZXh0IildXSk7
// formal-ai:presentation-raw end
pub static questionKinds: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(||
    vec![vec![String::from("where"),
    String::from("location")],
    vec![String::from("how-many"),
    String::from("quantity")],
    vec![String::from("why"),
    String::from("text")],
    vec![String::from("who"),
    String::from("entity")],
    vec![String::from("when"),
    String::from("time")],
    vec![String::from("which"),
    String::from("text")]]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f3bd6b5ecf20704d1cfc79664e5ed1007756076a8c50accc8a31b47a84cd7447
// | const topics=['location','experience','compensation','contract','education','availability'];
// formal-ai:presentation:v1 rawSHA256=f3bd6b5ecf20704d1cfc79664e5ed1007756076a8c50accc8a31b47a84cd7447 rawBytes=255 renderedSHA256=f88a06df3c7879d21583941d55c33462be83edcd09b74cd173db527f27519cec
// formal-ai:presentation-raw cHViIHN0YXRpYyB0b3BpY3M6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5jOjpMYXp5TG9j
// formal-ai:presentation-raw azo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJsb2NhdGlvbiIpLCBTdHJpbmc6OmZyb20oImV4cGVyaWVuY2UiKSwgU3Ry
// formal-ai:presentation-raw aW5nOjpmcm9tKCJjb21wZW5zYXRpb24iKSwgU3RyaW5nOjpmcm9tKCJjb250cmFjdCIpLCBTdHJpbmc6OmZyb20oImVkdWNh
// formal-ai:presentation-raw dGlvbiIpLCBTdHJpbmc6OmZyb20oImF2YWlsYWJpbGl0eSIpXSk7
// formal-ai:presentation-raw end
pub static topics: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("location"),
    String::from("experience"),
    String::from("compensation"),
    String::from("contract"),
    String::from("education"),
    String::from("availability")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5f4283f0271bde4a6adf96edf9693de63dcd38441ce2fbe518eb3e926a596f34
// | const currencies=['RUB','USD','EUR'];
pub static currencies: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("RUB"), String::from("USD"), String::from("EUR")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=bd87ce5f9644172dca70c9d1e6845422d7977eaeaad7d6fb9e77ddcd12570155
// | const units=['year','month','hour'];
pub static units: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("year"), String::from("month"), String::from("hour")]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .normalize()
// formal-ai:blockers arrow function | global call String() | method call .normalize() | method call .replace() | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers arrow function | method call .test() | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | method call .map() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers Array.from | call of a sibling function | method call .indexOf() | method call .pop() | method call .slice()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of a sibling function | method call .some()

// meta-language:translated JavaScript function_declaration items=3 sha256=0dc5c95df6b0131e9f2f6c9be58ffcb6a81f1ec5224645ee08a3d3076e1f83aa
// | /**
// |  * Exact membership for normalized vocabulary surfaces.
// |  * @param {string} text
// |  * @param {string[]} values
// |  * @returns {boolean}
// |  */
// | function listedSurface(text,values) {
// |   for(const value of values)if(value===text)return true;
// |   return false;
// | }
pub fn listed_surface(text: String, values: Vec<String>) -> bool {
    {
        let ml_values1 = values.clone();
        {
            let ml_index1 = crate::ml::Big::from_i128(0);
            {
                let ml_s1 = crate::ml_listed_surface_loop2(text.clone(), ml_values1.clone(), ml_index1.clone());
                match ml_s1.clone() {
                    crate::MlListedSurfaceLoop2Result::MlListedSurfaceLoop2Done => {
                        false
                    }
                    crate::MlListedSurfaceLoop2Result::MlListedSurfaceLoop2Return(ml_result2) => {
                        ml_result2
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MlListedSurfaceLoop2Result {
    MlListedSurfaceLoop2Done,
    MlListedSurfaceLoop2Return(bool),
}

pub fn ml_listed_surface_loop2(mut text: String, mut ml_values1: Vec<String>, mut ml_index1: crate::ml::Big) -> crate::MlListedSurfaceLoop2Result {
    loop {
        return if (ml_index1 < (crate::ml::Big::from_u128(ml_values1.len() as u128))) {
            {
                let value = crate::ml_array::at(&ml_values1, ml_index1.to_index());
                if (value == text) {
                    crate::MlListedSurfaceLoop2Result::MlListedSurfaceLoop2Return(true)
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
        } else {
            crate::MlListedSurfaceLoop2Result::MlListedSurfaceLoop2Done
        };
    }
}

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of a sibling function | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Number
// formal-ai:blockers arrow callback of .every() | arrow callback of .some() | call of a sibling function | method call .at() | method call .every() | method call .join() | method call .replace() | method call .slice() | method call .some() | method call .split() | method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .find() | arrow callback of .from() | arrow callback of .some() | call of a sibling function | field access | method call .encode() | method call .filter() | method call .find() | method call .matchAll() | method call .pop() | method call .push() | method call .slice() | method call .some() | method call .test() | new TextEncoder | null | nullish coalescing | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal several declarators
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .find() | arrow callback of .from() | arrow callback of .some() | call of a sibling function | method call .filter() | method call .find() | method call .matchAll() | method call .replace() | method call .some() | method call .test() | null | nullish coalescing | object without a $ tag | optional chaining | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of an imported function | field access | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers ++ operator | arrow callback of .every() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | global call Boolean() | method call .concat() | method call .every() | method call .map() | method call .push() | method call .replace() | method call .slice() | method call .split() | null | object spread | object without a $ tag | optional chaining | regular expression
