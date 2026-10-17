// Pure shared serde-compatible numeric markers and rendering; one private FLOAT identity.
const FLOAT = Symbol('rustFloat');

/**
 * Wrap a number that Rust serializes as an `f64`.
 * @param {number} value
 * @returns {object}
 */
export function f64(value) {
  return { [FLOAT]: true, value: Number(value), bits: 64 };
}

/**
 * Wrap a number that Rust serializes as an `f32` (rounded to single precision).
 * @param {number} value
 * @returns {object}
 */
export function f32(value) {
  return { [FLOAT]: true, value: Math.fround(Number(value)), bits: 32 };
}

/** Whether `value` is a float marker. @param {unknown} value @returns {boolean} */
export function isFloat(value) {
  return Boolean(value && typeof value === 'object' && value[FLOAT] === true);
}

// `zmij::FloatTraits::FIXED_DEC_EXP`: the decimal exponents printed without
// exponent notation.
const FIXED_EXPONENTS = { 64: [-5, 15], 32: [-6, 12] };

/** The shortest round-tripping digits and decimal exponent of a finite, non-zero `value`. */
function shortestDigits(value, bits) {
  let text = value.toExponential();
  if (bits === 32) {
    for (let precision = 1; precision <= 9; precision += 1) {
      const candidate = value.toExponential(precision - 1);
      if (Math.fround(Number(candidate)) === value) {
        text = candidate;
        break;
      }
    }
  }
  const [mantissa, exponent] = text.split('e');
  const digits = mantissa.replace('-', '').replace('.', '').replace(/0+$/u, '') || '0';
  return { digits, exponent: Number(exponent) };
}

/**
 * Mirrors the `zmij` crate's `Buffer::format_finite` (serde_json's
 * `write_f64` / `write_f32` and `Number`'s `Display`).
 * @param {number} value
 * @param {number} [bits]
 * @returns {string}
 */
export function renderFloat(value, bits = 64) {
  if (!Number.isFinite(value)) return 'null';
  const sign = value < 0 || Object.is(value, -0) ? '-' : '';
  if (value === 0) return `${sign}0.0`;
  const { digits, exponent } = shortestDigits(Math.abs(value), bits);
  const [low, high] = FIXED_EXPONENTS[bits] || FIXED_EXPONENTS[64];
  if (exponent >= low && exponent <= high) {
    if (digits.length - 1 <= exponent) return `${sign}${digits}${'0'.repeat(exponent + 1 - digits.length)}.0`;
    if (exponent >= 0) return `${sign}${digits.slice(0, exponent + 1)}.${digits.slice(exponent + 1)}`;
    return `${sign}0.${'0'.repeat(-exponent - 1)}${digits}`;
  }
  const mantissa = digits.length > 1 ? `${digits[0]}.${digits.slice(1)}` : digits;
  return `${sign}${mantissa}e${exponent < 0 ? '-' : '+'}${Math.abs(exponent)}`;
}
