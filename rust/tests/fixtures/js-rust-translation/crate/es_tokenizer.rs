// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=5e1119d443dd83e1ae64114d39c44c4be531cc1fc571edaea266c47e2b381881 bytes=12017
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
// formal-ai:presentation:v1 rawSHA256=36e417d81c90fce382d2770abb23ed63e23949ea949154a6a5c45038cde5cd3f rawBytes=420 renderedSHA256=d0dd12b9cce52591af625ff9171fb2416baaa4d88d5fc6e2ba61cb8e34479109
// formal-ai:presentation-raw cHViIHN0YXRpYyBSRUdFWF9DT05URVhUX0tFWVdPUkRTOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxTdHJpbmc+PiA9IHN0
// formal-ai:presentation-raw ZDo6c3luYzo6TGF6eUxvY2s6Om5ldyh8fCB2ZWMhW1N0cmluZzo6ZnJvbSgiYXdhaXQiKSwgU3RyaW5nOjpmcm9tKCJjYXNl
// formal-ai:presentation-raw IiksIFN0cmluZzo6ZnJvbSgiZGVsZXRlIiksIFN0cmluZzo6ZnJvbSgiZG8iKSwgU3RyaW5nOjpmcm9tKCJlbHNlIiksIFN0
// formal-ai:presentation-raw cmluZzo6ZnJvbSgiaW4iKSwgU3RyaW5nOjpmcm9tKCJpbnN0YW5jZW9mIiksIFN0cmluZzo6ZnJvbSgibmV3IiksIFN0cmlu
// formal-ai:presentation-raw Zzo6ZnJvbSgib2YiKSwgU3RyaW5nOjpmcm9tKCJyZXR1cm4iKSwgU3RyaW5nOjpmcm9tKCJ0aHJvdyIpLCBTdHJpbmc6OmZy
// formal-ai:presentation-raw b20oInR5cGVvZiIpLCBTdHJpbmc6OmZyb20oInZvaWQiKSwgU3RyaW5nOjpmcm9tKCJ5aWVsZCIpXSk7
// formal-ai:presentation-raw end
pub static REGEX_CONTEXT_KEYWORDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("await"),
    String::from("case"),
    String::from("delete"),
    String::from("do"),
    String::from("else"),
    String::from("in"),
    String::from("instanceof"),
    String::from("new"),
    String::from("of"),
    String::from("return"),
    String::from("throw"),
    String::from("typeof"),
    String::from("void"),
    String::from("yield")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=6308b4c88400fa31faf9d9d2c144145c38283f0ddf62396fee9331474391da31
// | const PUNCTUATORS = [
// |   '>>>=', '...', '===', '!==', '**=', '<<=', '>>=', '>>>', '&&=', '||=', '??=', '=>', '==', '!=',
// |   '<=', '>=', '&&', '||', '??', '?.', '++', '--', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=',
// |   '<<', '>>', '**', '{', '}', '(', ')', '[', ']', ';', ',', '<', '>', '+', '-', '*', '/', '%',
// |   '&', '|', '^', '!', '~', '?', ':', '=', '.', '@', '#',
// | ];
// formal-ai:presentation:v1 rawSHA256=6308b4c88400fa31faf9d9d2c144145c38283f0ddf62396fee9331474391da31 rawBytes=1259 renderedSHA256=d45c74e06cca94acb5ac0b88189b6f84cbb3f7e51e2b4e50f51df28f23c1617c
// formal-ai:presentation-raw cHViIHN0YXRpYyBQVU5DVFVBVE9SUzogc3RkOjpzeW5jOjpMYXp5TG9jazxWZWM8U3RyaW5nPj4gPSBzdGQ6OnN5bmM6Okxh
// formal-ai:presentation-raw enlMb2NrOjpuZXcofHwgdmVjIVtTdHJpbmc6OmZyb20oIj4+Pj0iKSwgU3RyaW5nOjpmcm9tKCIuLi4iKSwgU3RyaW5nOjpm
// formal-ai:presentation-raw cm9tKCI9PT0iKSwgU3RyaW5nOjpmcm9tKCIhPT0iKSwgU3RyaW5nOjpmcm9tKCIqKj0iKSwgU3RyaW5nOjpmcm9tKCI8PD0i
// formal-ai:presentation-raw KSwgU3RyaW5nOjpmcm9tKCI+Pj0iKSwgU3RyaW5nOjpmcm9tKCI+Pj4iKSwgU3RyaW5nOjpmcm9tKCImJj0iKSwgU3RyaW5n
// formal-ai:presentation-raw Ojpmcm9tKCJ8fD0iKSwgU3RyaW5nOjpmcm9tKCI/Pz0iKSwgU3RyaW5nOjpmcm9tKCI9PiIpLCBTdHJpbmc6OmZyb20oIj09
// formal-ai:presentation-raw IiksIFN0cmluZzo6ZnJvbSgiIT0iKSwgU3RyaW5nOjpmcm9tKCI8PSIpLCBTdHJpbmc6OmZyb20oIj49IiksIFN0cmluZzo6
// formal-ai:presentation-raw ZnJvbSgiJiYiKSwgU3RyaW5nOjpmcm9tKCJ8fCIpLCBTdHJpbmc6OmZyb20oIj8/IiksIFN0cmluZzo6ZnJvbSgiPy4iKSwg
// formal-ai:presentation-raw U3RyaW5nOjpmcm9tKCIrKyIpLCBTdHJpbmc6OmZyb20oIi0tIiksIFN0cmluZzo6ZnJvbSgiKz0iKSwgU3RyaW5nOjpmcm9t
// formal-ai:presentation-raw KCItPSIpLCBTdHJpbmc6OmZyb20oIio9IiksIFN0cmluZzo6ZnJvbSgiLz0iKSwgU3RyaW5nOjpmcm9tKCIlPSIpLCBTdHJp
// formal-ai:presentation-raw bmc6OmZyb20oIiY9IiksIFN0cmluZzo6ZnJvbSgifD0iKSwgU3RyaW5nOjpmcm9tKCJePSIpLCBTdHJpbmc6OmZyb20oIjw8
// formal-ai:presentation-raw IiksIFN0cmluZzo6ZnJvbSgiPj4iKSwgU3RyaW5nOjpmcm9tKCIqKiIpLCBTdHJpbmc6OmZyb20oInsiKSwgU3RyaW5nOjpm
// formal-ai:presentation-raw cm9tKCJ9IiksIFN0cmluZzo6ZnJvbSgiKCIpLCBTdHJpbmc6OmZyb20oIikiKSwgU3RyaW5nOjpmcm9tKCJbIiksIFN0cmlu
// formal-ai:presentation-raw Zzo6ZnJvbSgiXSIpLCBTdHJpbmc6OmZyb20oIjsiKSwgU3RyaW5nOjpmcm9tKCIsIiksIFN0cmluZzo6ZnJvbSgiPCIpLCBT
// formal-ai:presentation-raw dHJpbmc6OmZyb20oIj4iKSwgU3RyaW5nOjpmcm9tKCIrIiksIFN0cmluZzo6ZnJvbSgiLSIpLCBTdHJpbmc6OmZyb20oIioi
// formal-ai:presentation-raw KSwgU3RyaW5nOjpmcm9tKCIvIiksIFN0cmluZzo6ZnJvbSgiJSIpLCBTdHJpbmc6OmZyb20oIiYiKSwgU3RyaW5nOjpmcm9t
// formal-ai:presentation-raw KCJ8IiksIFN0cmluZzo6ZnJvbSgiXiIpLCBTdHJpbmc6OmZyb20oIiEiKSwgU3RyaW5nOjpmcm9tKCJ+IiksIFN0cmluZzo6
// formal-ai:presentation-raw ZnJvbSgiPyIpLCBTdHJpbmc6OmZyb20oIjoiKSwgU3RyaW5nOjpmcm9tKCI9IiksIFN0cmluZzo6ZnJvbSgiLiIpLCBTdHJp
// formal-ai:presentation-raw bmc6OmZyb20oIkAiKSwgU3RyaW5nOjpmcm9tKCIjIildKTs=
// formal-ai:presentation-raw end
pub static PUNCTUATORS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from(">>>="),
    String::from("..."),
    String::from("==="),
    String::from("!=="),
    String::from("**="),
    String::from("<<="),
    String::from(">>="),
    String::from(">>>"),
    String::from("&&="),
    String::from("||="),
    String::from("??="),
    String::from("=>"),
    String::from("=="),
    String::from("!="),
    String::from("<="),
    String::from(">="),
    String::from("&&"),
    String::from("||"),
    String::from("??"),
    String::from("?."),
    String::from("++"),
    String::from("--"),
    String::from("+="),
    String::from("-="),
    String::from("*="),
    String::from("/="),
    String::from("%="),
    String::from("&="),
    String::from("|="),
    String::from("^="),
    String::from("<<"),
    String::from(">>"),
    String::from("**"),
    String::from("{"),
    String::from("}"),
    String::from("("),
    String::from(")"),
    String::from("["),
    String::from("]"),
    String::from(";"),
    String::from(","),
    String::from("<"),
    String::from(">"),
    String::from("+"),
    String::from("-"),
    String::from("*"),
    String::from("/"),
    String::from("%"),
    String::from("&"),
    String::from("|"),
    String::from("^"),
    String::from("!"),
    String::from("~"),
    String::from("?"),
    String::from(":"),
    String::from("="),
    String::from("."),
    String::from("@"),
    String::from("#")]);

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
// formal-ai:blockers JSDoc type {…} | REGEX_CONTEXT_KEYWORDS.includes | arrow callback of .find() | arrow function | assignment of a field or element | call of a sibling function | field access | method call .decode() | method call .encode() | method call .find() | method call .pop() | method call .push() | method call .reverse() | method call .slice() | method call .subarray() | new TextDecoder | new TextEncoder | object spread | object without a $ tag | throw of a non-error value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers method call .has() | sibling value
