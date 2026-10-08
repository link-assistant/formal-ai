// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=2713b2ca073e82505ee238fc5e8b3bbfd19469a29244b95bce0f20f7da15e2d6 bytes=8569
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=6c226461e1ce0d13d6c9ab0de81a80710f2157067ccca987e51cd17d0e6c6c1a
// | /** The directory build.rs embeds, relative to the crate manifest (`rust/`). */
// | const CRATE_DIR = 'rust';
pub const CRATE_DIR: &str = "rust";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=63e06497e66b2986631f5e3c396c49c5ef16a136cc6c7670abc67dc81f338549
// | const SOURCE_DIR = 'src';
pub const SOURCE_DIR: &str = "src";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=0b3966278c57c18e3103b06c71c78310cbbc56d61095d344b68a7f272a79e7c1
// | const FNV_PRIME_LOW = 0x1b3;
pub const FNV_PRIME_LOW: f64 = 435f64;

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal ^= assignment
// formal-ai:blockers arrow function | bitwise operator | method call .encode() | method call .padStart() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of an imported function | field access | method call .push()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers method call .encode() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | method call .map() | method call .sort() | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | call of an imported function | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | field access | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow callback of .reduce() | assignment of a field or element | call of a sibling function | class | export … | field access | method call .coveragePermille() | method call .faithfulCount() | method call .filter() | method call .isFullyFaithful() | method call .join() | method call .map() | method call .moduleCount() | method call .push() | method call .reduce() | method call .totalLinkCount() | method call .totalNamedNodeCount() | new SourceLinks

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | method call .map()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | call of a sibling function | field access | method call .reduce()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | method call .join() | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replaceAll()
// formal-ai:blockers method call .replaceAll()
