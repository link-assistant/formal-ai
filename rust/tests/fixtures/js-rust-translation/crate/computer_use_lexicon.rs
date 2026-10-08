// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e92ff93f3773e19a19bf17d13ac9b6df568c00fa708fc2c77bdf329cc9ace054 bytes=5875
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=3

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript export_statement items=1 sha256=3dbc73de8045d4b4585eeaa99d396013fbc2fcb5e120acd3065b9ddbeb6a7204
// | /** `crate::seed::ROLE_COMPUTER_USE_OPERATION_CUE`. */
// | export const ROLE_COMPUTER_USE_OPERATION_CUE = 'computer_use_operation_cue';
pub const ROLE_COMPUTER_USE_OPERATION_CUE: &str = "computer_use_operation_cue";

// meta-language:translated JavaScript export_statement items=1 sha256=9f8b769c384e17c2d0bfbef8806080d4d57c786cb962d03b705c81c12923f759
// | /** `crate::seed::ROLE_COMPUTER_USE_RESOURCE_CUE`. */
// | export const ROLE_COMPUTER_USE_RESOURCE_CUE = 'computer_use_resource_cue';
pub const ROLE_COMPUTER_USE_RESOURCE_CUE: &str = "computer_use_resource_cue";

// meta-language:translated JavaScript export_statement items=1 sha256=284784be2072d294d5481935ebdd1a66520a02f89e1bb2777a8accbb87c95a65
// | /** `crate::seed::ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE`. */
// | export const ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE = 'computer_use_capability_gap_cue';
pub const ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE: &str = "computer_use_capability_gap_cue";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ba01ba93ad1387117643dcc1707094117d97c79f5ad7f44f2752afd3b59c4d94
// | const GAP_PREFIX = 'computer_use_gap_';
pub const GAP_PREFIX: &str = "computer_use_gap_";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8ec28d597b46aa364c6c48048ee3a63ab57f71732967283f9495f8cdc181d0d3
// | /** The inclusive code point ranges `fn is_combining_mark` accepts. */
// | const COMBINING_RANGES = [
// |   [0x0300, 0x036f], [0x0483, 0x0489], [0x0591, 0x05bd],
// |   [0x0610, 0x061a], [0x064b, 0x065f], [0x0670, 0x0670], [0x06d6, 0x06dc],
// |   [0x0900, 0x0903], [0x093a, 0x094f], [0x0951, 0x0957], [0x0962, 0x0963],
// |   [0x0981, 0x0983], [0x09bc, 0x09cd], [0x0a01, 0x0a03], [0x0a3c, 0x0a4d],
// |   [0x0b01, 0x0b4d], [0x0c00, 0x0c4d], [0x0d00, 0x0d4d],
// |   [0x0e31, 0x0e3a], [0x0e47, 0x0e4e],
// |   [0x1ab0, 0x1aff], [0x1dc0, 0x1dff], [0x20d0, 0x20f0], [0xfe20, 0xfe2f],
// | ];
pub static COMBINING_RANGES: std::sync::LazyLock<Vec<Vec<f64>>> = std::sync::LazyLock::new(|| vec![vec![768f64, 879f64], vec![1155f64, 1161f64], vec![1425f64, 1469f64], vec![1552f64, 1562f64], vec![1611f64, 1631f64], vec![1648f64, 1648f64], vec![1750f64, 1756f64], vec![2304f64, 2307f64], vec![2362f64, 2383f64], vec![2385f64, 2391f64], vec![2402f64, 2403f64], vec![2433f64, 2435f64], vec![2492f64, 2509f64], vec![2561f64, 2563f64], vec![2620f64, 2637f64], vec![2817f64, 2893f64], vec![3072f64, 3149f64], vec![3328f64, 3405f64], vec![3633f64, 3642f64], vec![3655f64, 3662f64], vec![6832f64, 6911f64], vec![7616f64, 7679f64], vec![8400f64, 8432f64], vec![65056f64, 65071f64]]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers COMBINING_RANGES.some | arrow callback of .some() | method call .codePointAt()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .join()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .sort() | call of a sibling function | call of an imported function | field access | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | call of an imported function | method call .filter() | method call .map() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | method call .indexOf() | null
