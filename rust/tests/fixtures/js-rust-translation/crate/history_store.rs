// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a618022402f8306a371a9d4a9fed5826e2ea8cdd6cd3b35d28e325d4139147db bytes=13585
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=5

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=e45012721f42b7bb1f09173424b44fdfe7770c7c9431df406913db57851f3b90
// | /** Mirrors `const CURSOR_ROOT`. */
// | export const CURSOR_ROOT = 'repository_history_cursor';
pub const CURSOR_ROOT: &str = "repository_history_cursor";

// meta-language:translated JavaScript export_statement items=1 sha256=a360cf473dde3df730ea7318cf52443d3235e5a58c3d84cb21ba2867df286efe
// | /** Mirrors `const LOG_FORMAT`. */
// | export const LOG_FORMAT = '\u001e%H\u001f%an\u001f%cI\u001f%s\u001f%b\u001f';
pub const LOG_FORMAT: &str = "\u{1e}%H\u{1f}%an\u{1f}%cI\u{1f}%s\u{1f}%b\u{1f}";

// meta-language:translated JavaScript export_statement items=1 sha256=b0aadd8b9e9c261eb0ebbaa07101c35ed891f051e1d1c42121295bd4fa80e0e9
// | /** Mirrors the fallback slug of `repository_slug`. */
// | export const LOCAL_REPOSITORY = 'local-repository';
pub const LOCAL_REPOSITORY: &str = "local-repository";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal destructured parameter
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow function | bitwise operator | field access | method call .dirname() | method call .execFileSync() | method call .existsSync() | method call .filter() | method call .isDirectory() | method call .join() | method call .map() | method call .mkdirSync() | method call .readFileSync() | method call .readdirSync() | method call .writeFileSync() | null | object spread | object without a $ tag | typeof operator

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal rest parameter
// formal-ai:blockers arrow function | field access | method call .join()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .dirname()
// formal-ai:blockers arrow function | field access | method call .dirname() | method call .lastIndexOf() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers method call .runGit() | null | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | method call .filter() | method call .map() | method call .split() | null

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | field access | method call .some() | null | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers field access | method call .astCensus() | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | destructuring | method call .filter() | method call .get() | method call .keys() | method call .map() | method call .set() | method call .sort() | new Map | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers call of an imported function | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .flatMap() | method call .map() | method call .push() | method call .runGit() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of an imported function | field access | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .createDirAll() | method call .events() | method call .exportLinksNotation() | method call .has() | method call .map() | method call .push() | method call .readText() | method call .replaceFromLinksNotation() | method call .writeText() | new MemoryStore | new Set | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | arrow function | call of an imported function | field access | method call .find() | method call .readText() | method call .test() | null | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | field access | global call String() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .createDirAll()
// formal-ai:blockers call of a sibling function | field access | method call .createDirAll() | method call .writeText()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .find() | arrow function | assignment of a field or element | field access | method call .find() | method call .slice() | method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | method call .filter() | method call .slice() | method call .split() | nullish coalescing | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .listFiles() | method call .push() | null | undefined
