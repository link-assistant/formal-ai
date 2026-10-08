// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=249fcb4d9f23ed938265cead2e54bee88eef8bfcf9df804b14cc58f7bb3d18c8 bytes=12370
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

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
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow function | call of a sibling function | call of an imported function | field access | imported value | method call .filter() | method call .join() | method call .map() | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers method call .codePointAt()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: one value is used as a boolean and as a string; declare the types of the function with JSDoc
// formal-ai:blockers call of a sibling function

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
