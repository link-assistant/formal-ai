// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=3965fcdc370593335989e44e2621dbf7ec7aa7304f6ded943e4d9e99ff8c578d bytes=11001
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers JSDoc type {…} | Object.assign | new Error | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=36e417d81c90fce382d2770abb23ed63e23949ea949154a6a5c45038cde5cd3f
// | const REGEX_CONTEXT_KEYWORDS = [
// |   'await', 'case', 'delete', 'do', 'else', 'in', 'instanceof', 'new', 'of', 'return', 'throw',
// |   'typeof', 'void', 'yield',
// | ];
pub static REGEX_CONTEXT_KEYWORDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("await"), String::from("case"), String::from("delete"), String::from("do"), String::from("else"), String::from("in"), String::from("instanceof"), String::from("new"), String::from("of"), String::from("return"), String::from("throw"), String::from("typeof"), String::from("void"), String::from("yield")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=6308b4c88400fa31faf9d9d2c144145c38283f0ddf62396fee9331474391da31
// | const PUNCTUATORS = [
// |   '>>>=', '...', '===', '!==', '**=', '<<=', '>>=', '>>>', '&&=', '||=', '??=', '=>', '==', '!=',
// |   '<=', '>=', '&&', '||', '??', '?.', '++', '--', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=',
// |   '<<', '>>', '**', '{', '}', '(', ')', '[', ']', ';', ',', '<', '>', '+', '-', '*', '/', '%',
// |   '&', '|', '^', '!', '~', '?', ':', '=', '.', '@', '#',
// | ];
pub static PUNCTUATORS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from(">>>="), String::from("..."), String::from("==="), String::from("!=="), String::from("**="), String::from("<<="), String::from(">>="), String::from(">>>"), String::from("&&="), String::from("||="), String::from("??="), String::from("=>"), String::from("=="), String::from("!="), String::from("<="), String::from(">="), String::from("&&"), String::from("||"), String::from("??"), String::from("?."), String::from("++"), String::from("--"), String::from("+="), String::from("-="), String::from("*="), String::from("/="), String::from("%="), String::from("&="), String::from("|="), String::from("^="), String::from("<<"), String::from(">>"), String::from("**"), String::from("{"), String::from("}"), String::from("("), String::from(")"), String::from("["), String::from("]"), String::from(";"), String::from(","), String::from("<"), String::from(">"), String::from("+"), String::from("-"), String::from("*"), String::from("/"), String::from("%"), String::from("&"), String::from("|"), String::from("^"), String::from("!"), String::from("~"), String::from("?"), String::from(":"), String::from("="), String::from("."), String::from("@"), String::from("#")]);

// meta-language:translated JavaScript function_declaration items=1 sha256=e408bad3068c382e51b83cc7577baba71d89d072f58e23141111501757202a4f
// | /** @param {number} b @returns {boolean} */
// | function isWhitespace(b) {
// |   // Rust's u8::is_ascii_whitespace: space, \t, \n, \x0C, \r (not \x0B).
// |   return b === 0x20 || b === 0x09 || b === 0x0a || b === 0x0c || b === 0x0d;
// | }
pub fn is_whitespace(b: f64) -> bool {
    (((((b == (32f64)) || (b == (9f64))) || (b == (10f64))) || (b == (12f64))) || (b == (13f64)))
}

// meta-language:translated JavaScript function_declaration items=1 sha256=eb0fb3dd538ac7673e35527149bf17d0675176f7630ad85c02d4bd3dd1496f32
// | /** @param {number} b @returns {boolean} */
// | function isDigit(b) {
// |   return b >= 0x30 && b <= 0x39;
// | }
pub fn is_digit(b: f64) -> bool {
    ((b >= (48f64)) && (b <= (57f64)))
}

// meta-language:translated JavaScript function_declaration items=1 sha256=4af52c5c59b35b5bb5779fbabb8437712c6e90ad04aa3cd42354b55c807c914b
// | /** @param {number} b @returns {boolean} */
// | function isAlpha(b) {
// |   return (b >= 0x41 && b <= 0x5a) || (b >= 0x61 && b <= 0x7a);
// | }
pub fn is_alpha(b: f64) -> bool {
    (((b >= (65f64)) && (b <= (90f64))) || ((b >= (97f64)) && (b <= (122f64))))
}

// meta-language:translated JavaScript function_declaration items=1 sha256=799cf70fe5e756b9dd3ab1a877d20e91aa21777e4800f613195ee5240597d68f
// | /** @param {number} b @returns {boolean} */
// | function isIdentByte(b) {
// |   return isAlpha(b) || isDigit(b) || b === 0x5f || b === 0x24 || b >= 0x80;
// | }
pub fn is_ident_byte(b: f64) -> bool {
    ((((is_alpha(b) || is_digit(b)) || (b == (95f64))) || (b == (36f64))) || (b >= (128f64)))
}

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as a Uint8Array and as an array; declare the types of the function with JSDoc
// formal-ai:blockers JSDoc type {…}

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as a Uint8Array and as an array; declare the types of the function with JSDoc
// formal-ai:blockers JSDoc type {…}

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as a Uint8Array and as an array; declare the types of the function with JSDoc
// formal-ai:blockers JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | object without a $ tag

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as a Uint8Array and as an array; declare the types of the function with JSDoc
// formal-ai:blockers JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | object without a $ tag | throw of a non-error value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers JSDoc type {…} | method call .charCodeAt()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | REGEX_CONTEXT_KEYWORDS.includes | arrow callback of .find() | arrow function | assignment of a field or element | call of a sibling function | field access | method call .decode() | method call .encode() | method call .find() | method call .pop() | method call .push() | method call .reverse() | method call .slice() | method call .subarray() | new TextDecoder | new TextEncoder | throw of a non-error value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access
