// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=decf7d24ec67954d34cafa8c04fc0a6ff71821610c0a7e8ff8d254dc040cdb5a bytes=12917
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=5

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
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | call of an imported function | method call .codePointAt() | method call .filter() | method call .indexOf() | method call .push() | method call .slice() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow callback of .some() | arrow function | call of an imported function | method call .slice() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers method call .codePointAt()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of an imported function | method call .slice() | method call .some() | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | assignment of a field or element | call of a sibling function | call of an imported function | method call .push() | method call .slice() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.from | call of a sibling function | call of an imported function | method call .slice() | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow function | call of a sibling function | field access | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of an imported function | method call .encode() | method call .pop() | method call .slice() | null | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .find() | null | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .indexOf() | method call .push() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers field access | method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers field access | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of an imported function | field access | method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | call of a sibling function | call of an imported function | field access | method call .join() | method call .reduce() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | global call String() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | call of a sibling function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | field access | method call .filter()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function | field access

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function
