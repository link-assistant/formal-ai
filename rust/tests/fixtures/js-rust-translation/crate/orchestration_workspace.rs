// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a76f28ec8cdee8929fe9d546bf43954566b06a441dd2f6c435124e2bbf0c264b bytes=10717
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from '…'

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from '…'

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers JSON.stringify | assignment of a field or element | class | export … | field access

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow function | new IoError

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers JSON.stringify | arrow function | global call String() | new IoError

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | JSON.stringify | call of a sibling function | call of an imported function | field access | global call String() | imported value | instanceof operator | new IoError | nullish coalescing | optional chaining | sibling value

// meta-language:translated JavaScript lexical_declaration items=1 sha256=41a8d6040f8c5911a460f6b84381ff424eb32814ace41b1d478f93bdab56a280
// | const IGNORED_ROOTS = ['.git', 'target', '.formal-ai', '.formal-ai-orchestration'];
pub static IGNORED_ROOTS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from(".git"), String::from("target"), String::from(".formal-ai"), String::from(".formal-ai-orchestration")]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .filter() | arrow function | field access | imported value | method call .filter() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | call of a sibling function | method call .every()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers IGNORED_ROOTS.includes | call of a sibling function | imported value | method call .relative()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .update()
// formal-ai:blockers arrow function | call of an imported function | method call .digest() | method call .update()

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers Buffer.compare | Buffer.from | arrow function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow function | imported value | method call .join() | method call .relative() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | imported value | let without a value | method call .isDirectory() | method call .isFile() | method call .isSymbolicLink() | method call .join() | method call .lstatSync() | method call .push() | method call .readdirSync() | method call .sort() | object without a $ tag | throw of a non-error value | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow function | call of a sibling function | field access | imported value | let without a value | method call .readFileSync() | method call .set() | new Map | object without a $ tag | throw of a non-error value | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access | method call .get() | method call .keys() | method call .push() | method call .sort() | new Set | null | nullish coalescing | object without a $ tag | optional chaining | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal throw of a non-error value
// formal-ai:blockers arrow function | call of a sibling function | field access | imported value | method call .copyFileSync() | method call .dirname() | method call .existsSync() | method call .join() | method call .mkdirSync() | method call .relative() | object without a $ tag | throw of a non-error value | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of a sibling function | field access | method call .isAbsolute() | method call .split() | throw of a non-error value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null | throw of a non-error value | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | imported value | method call .isFile() | method call .lstatSync() | method call .readFileSync() | null | nullish coalescing | throw of a non-error value | try statement

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access | imported value | method call .join()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers call of a sibling function | field access | imported value | method call .copyFileSync() | method call .dirname() | method call .existsSync() | method call .join() | method call .mkdirSync() | method call .rmSync() | object without a $ tag | throw of a non-error value | try statement
