// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=bffa47e9bd24c046a966ae8a4fadee6fed98c4fde2e50cf131b5fe0f43576a22 bytes=7405
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8ae9b7470652ebb303eb11b75c0809021d4ffb7cee3ae4ab124df3da48a1b304
// | const NUMBER_PLACEHOLDER = '%number%';
pub const NUMBER_PLACEHOLDER: &str = "%number%";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of an imported function | destructuring | method call .join() | method call .split() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers destructuring | global call String() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .test() | regular expression

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers call of a sibling function | field access | global call String() | method call .push() | method call .split() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers global call String() | method call .push() | method call .slice() | method call .split() | method call .test() | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers field access | global call String() | method call .join() | method call .push() | method call .reverse() | method call .slice() | method call .split() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .slice() | method call .test() | null | regular expression | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | method call .join() | method call .slice() | nullish coalescing | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | global call String() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers global call String() | method call .padStart() | method call .test() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers assignment of a field or element | destructuring | method call .join() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | call of a sibling function | call of an imported function | field access | global call String() | method call .find() | method call .reverse() | null
