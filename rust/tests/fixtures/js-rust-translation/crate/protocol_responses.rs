// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e0981d77f0dc717ca123e173237ad2e2f3d79ddb9e4b2aea0f8244970f1d0b87 bytes=9704
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers export … | imported value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers Array.isArray | arrow function | global call Boolean() | typeof operator

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Object.prototype.hasOwnProperty.call
// formal-ai:blockers Object.prototype | arrow function | field access | method call .call()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Object.keys
// formal-ai:blockers Object.keys | arrow function | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers call of an imported function | field access | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of an imported function | field access | method call .pop() | method call .split() | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.isArray | JSDoc type {…} | JSON.parse | Object.keys | arrow callback of .some() | assignment of a field or element | call of a sibling function | call of an imported function | field access | let without a value | method call .some() | null | object without a $ tag | try statement | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers call of a sibling function | field access | undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=27de87701f704d8b6b17c4da1dc771256a745224312d6dd0a0e518e92c8bca4d
// | const ALIASES = [
// |   [['path', 'filePath', 'file_path', 'absolute_path'], ['path', 'filePath', 'file_path', 'absolute_path']],
// |   [['command', 'cmd'], ['command', 'cmd']],
// |   [['query', 'pattern'], ['query', 'pattern']],
// |   [['paths', 'file_paths', 'files'], ['paths', 'file_paths', 'files']],
// |   [['old', 'oldString', 'old_string', 'old_str'], ['old', 'oldString', 'old_string', 'old_str']],
// |   [['new', 'newString', 'new_string', 'new_str'], ['new', 'newString', 'new_string', 'new_str']],
// | ];
// formal-ai:presentation:v1 rawSHA256=27de87701f704d8b6b17c4da1dc771256a745224312d6dd0a0e518e92c8bca4d rawBytes=1159 renderedSHA256=433dad851ee3d9cf5dda8a50e9fd8df1d799ddd19199519a5f2d422b1fd29ef3
// formal-ai:presentation-raw cHViIHN0YXRpYyBBTElBU0VTOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxWZWM8VmVjPFN0cmluZz4+Pj4gPSBzdGQ6OnN5
// formal-ai:presentation-raw bmM6OkxhenlMb2NrOjpuZXcofHwgdmVjIVt2ZWMhW3ZlYyFbU3RyaW5nOjpmcm9tKCJwYXRoIiksIFN0cmluZzo6ZnJvbSgi
// formal-ai:presentation-raw ZmlsZVBhdGgiKSwgU3RyaW5nOjpmcm9tKCJmaWxlX3BhdGgiKSwgU3RyaW5nOjpmcm9tKCJhYnNvbHV0ZV9wYXRoIildLCB2
// formal-ai:presentation-raw ZWMhW1N0cmluZzo6ZnJvbSgicGF0aCIpLCBTdHJpbmc6OmZyb20oImZpbGVQYXRoIiksIFN0cmluZzo6ZnJvbSgiZmlsZV9w
// formal-ai:presentation-raw YXRoIiksIFN0cmluZzo6ZnJvbSgiYWJzb2x1dGVfcGF0aCIpXV0sIHZlYyFbdmVjIVtTdHJpbmc6OmZyb20oImNvbW1hbmQi
// formal-ai:presentation-raw KSwgU3RyaW5nOjpmcm9tKCJjbWQiKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCJjb21tYW5kIiksIFN0cmluZzo6ZnJvbSgiY21k
// formal-ai:presentation-raw IildXSwgdmVjIVt2ZWMhW1N0cmluZzo6ZnJvbSgicXVlcnkiKSwgU3RyaW5nOjpmcm9tKCJwYXR0ZXJuIildLCB2ZWMhW1N0
// formal-ai:presentation-raw cmluZzo6ZnJvbSgicXVlcnkiKSwgU3RyaW5nOjpmcm9tKCJwYXR0ZXJuIildXSwgdmVjIVt2ZWMhW1N0cmluZzo6ZnJvbSgi
// formal-ai:presentation-raw cGF0aHMiKSwgU3RyaW5nOjpmcm9tKCJmaWxlX3BhdGhzIiksIFN0cmluZzo6ZnJvbSgiZmlsZXMiKV0sIHZlYyFbU3RyaW5n
// formal-ai:presentation-raw Ojpmcm9tKCJwYXRocyIpLCBTdHJpbmc6OmZyb20oImZpbGVfcGF0aHMiKSwgU3RyaW5nOjpmcm9tKCJmaWxlcyIpXV0sIHZl
// formal-ai:presentation-raw YyFbdmVjIVtTdHJpbmc6OmZyb20oIm9sZCIpLCBTdHJpbmc6OmZyb20oIm9sZFN0cmluZyIpLCBTdHJpbmc6OmZyb20oIm9s
// formal-ai:presentation-raw ZF9zdHJpbmciKSwgU3RyaW5nOjpmcm9tKCJvbGRfc3RyIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgib2xkIiksIFN0cmluZzo6
// formal-ai:presentation-raw ZnJvbSgib2xkU3RyaW5nIiksIFN0cmluZzo6ZnJvbSgib2xkX3N0cmluZyIpLCBTdHJpbmc6OmZyb20oIm9sZF9zdHIiKV1d
// formal-ai:presentation-raw LCB2ZWMhW3ZlYyFbU3RyaW5nOjpmcm9tKCJuZXciKSwgU3RyaW5nOjpmcm9tKCJuZXdTdHJpbmciKSwgU3RyaW5nOjpmcm9t
// formal-ai:presentation-raw KCJuZXdfc3RyaW5nIiksIFN0cmluZzo6ZnJvbSgibmV3X3N0ciIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIm5ldyIpLCBTdHJp
// formal-ai:presentation-raw bmc6OmZyb20oIm5ld1N0cmluZyIpLCBTdHJpbmc6OmZyb20oIm5ld19zdHJpbmciKSwgU3RyaW5nOjpmcm9tKCJuZXdfc3Ry
// formal-ai:presentation-raw IildXV0pOw==
// formal-ai:presentation-raw end
pub static ALIASES: std::sync::LazyLock<Vec<Vec<Vec<String>>>> = std::sync::LazyLock::new(||
    vec![vec![vec![String::from("path"),
    String::from("filePath"),
    String::from("file_path"),
    String::from("absolute_path")],
    vec![String::from("path"),
    String::from("filePath"),
    String::from("file_path"),
    String::from("absolute_path")]],
    vec![vec![String::from("command"),
    String::from("cmd")],
    vec![String::from("command"),
    String::from("cmd")]],
    vec![vec![String::from("query"),
    String::from("pattern")],
    vec![String::from("query"),
    String::from("pattern")]],
    vec![vec![String::from("paths"),
    String::from("file_paths"),
    String::from("files")],
    vec![String::from("paths"),
    String::from("file_paths"),
    String::from("files")]],
    vec![vec![String::from("old"),
    String::from("oldString"),
    String::from("old_string"),
    String::from("old_str")],
    vec![String::from("old"),
    String::from("oldString"),
    String::from("old_string"),
    String::from("old_str")]],
    vec![vec![String::from("new"),
    String::from("newString"),
    String::from("new_string"),
    String::from("new_str")],
    vec![String::from("new"),
    String::from("newString"),
    String::from("new_string"),
    String::from("new_str")]]]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers ALIASES.find | arrow callback of .find() | call of a sibling function | method call .find() | undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=943e885b0c6108c2eb5a2f5d9f866ce2814b3346ab5b308df5f7bad4aac9725c
// | const PATH_PROPERTIES = ['path', 'filePath', 'file_path', 'absolute_path', 'dir_path', 'directory', 'notebook_path'];
// formal-ai:presentation:v1 rawSHA256=943e885b0c6108c2eb5a2f5d9f866ce2814b3346ab5b308df5f7bad4aac9725c rawBytes=287 renderedSHA256=d004741cc96026703bd22364e4caed23c4448a1c7556327ed9228b0a9afe4055
// formal-ai:presentation-raw cHViIHN0YXRpYyBQQVRIX1BST1BFUlRJRVM6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5j
// formal-ai:presentation-raw OjpMYXp5TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJwYXRoIiksIFN0cmluZzo6ZnJvbSgiZmlsZVBhdGgiKSwg
// formal-ai:presentation-raw U3RyaW5nOjpmcm9tKCJmaWxlX3BhdGgiKSwgU3RyaW5nOjpmcm9tKCJhYnNvbHV0ZV9wYXRoIiksIFN0cmluZzo6ZnJvbSgi
// formal-ai:presentation-raw ZGlyX3BhdGgiKSwgU3RyaW5nOjpmcm9tKCJkaXJlY3RvcnkiKSwgU3RyaW5nOjpmcm9tKCJub3RlYm9va19wYXRoIildKTs=
// formal-ai:presentation-raw end
pub static PATH_PROPERTIES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("path"),
    String::from("filePath"),
    String::from("file_path"),
    String::from("absolute_path"),
    String::from("dir_path"),
    String::from("directory"),
    String::from("notebook_path")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PATH_PROPERTIES.includes | arrow function | call of a sibling function | field access | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers call of a sibling function | field access | undefined

// meta-language:translated JavaScript function_declaration items=1 sha256=4e14c7d0c77ec8398ed3d2d0814883d4c151487531fca0109570798b34861638
// | /**
// |  * `Path::join` for a relative `path` under `root`.
// |  * @param {string} root
// |  * @param {string} path
// |  * @returns {string}
// |  */
// | function joinPath(root, path) {
// |   return root.endsWith('/') ? `${root}${path}` : `${root}/${path}`;
// | }
pub fn join_path(root: String, path: String) -> String {
    if root.ends_with("/") {
        format!("{}{}", root, path)
    } else {
        format!("{}{}", (format!("{}{}", root, (String::from("/")))), path)
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | call of an imported function | method call .filter() | method call .join() | method call .split() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9dcc71624ce0249786ccb61b47b37c0d1b7bef49f7a631cf2801931bfc943c54
// | const CONTENT_PROPERTIES = ['content', 'contents', 'file_text', 'text', 'new_string'];
// formal-ai:presentation:v1 rawSHA256=9dcc71624ce0249786ccb61b47b37c0d1b7bef49f7a631cf2801931bfc943c54 rawBytes=228 renderedSHA256=8919b4923ce70e6b8f8ee4b2c6ac35597a76b1444c6b8888bf6d473d6f4efcb8
// formal-ai:presentation-raw cHViIHN0YXRpYyBDT05URU5UX1BST1BFUlRJRVM6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpz
// formal-ai:presentation-raw eW5jOjpMYXp5TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJjb250ZW50IiksIFN0cmluZzo6ZnJvbSgiY29udGVu
// formal-ai:presentation-raw dHMiKSwgU3RyaW5nOjpmcm9tKCJmaWxlX3RleHQiKSwgU3RyaW5nOjpmcm9tKCJ0ZXh0IiksIFN0cmluZzo6ZnJvbSgibmV3
// formal-ai:presentation-raw X3N0cmluZyIpXSk7
// formal-ai:presentation-raw end
pub static CONTENT_PROPERTIES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("content"),
    String::from("contents"),
    String::from("file_text"),
    String::from("text"),
    String::from("new_string")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers CONTENT_PROPERTIES.some | arrow callback of .some() | call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers Array.isArray | call of a sibling function | call of an imported function | field access | nullish coalescing | undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=abf836d03d47e1dc3f522b2bca213dd7262d3760c70b2e8f7073d0409df16887
// | const FREE_TEXT = ['prompt', 'instruction', 'message', 'commit_message', 'commitMessage'];
// formal-ai:presentation:v1 rawSHA256=abf836d03d47e1dc3f522b2bca213dd7262d3760c70b2e8f7073d0409df16887 rawBytes=232 renderedSHA256=bea30cd78af924923dcda5d8110f10b146ebf5ccf517c3113ee0c2b59125675c
// formal-ai:presentation-raw cHViIHN0YXRpYyBGUkVFX1RFWFQ6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5jOjpMYXp5
// formal-ai:presentation-raw TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJwcm9tcHQiKSwgU3RyaW5nOjpmcm9tKCJpbnN0cnVjdGlvbiIpLCBT
// formal-ai:presentation-raw dHJpbmc6OmZyb20oIm1lc3NhZ2UiKSwgU3RyaW5nOjpmcm9tKCJjb21taXRfbWVzc2FnZSIpLCBTdHJpbmc6OmZyb20oImNv
// formal-ai:presentation-raw bW1pdE1lc3NhZ2UiKV0pOw==
// formal-ai:presentation-raw end
pub static FREE_TEXT: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("prompt"),
    String::from("instruction"),
    String::from("message"),
    String::from("commit_message"),
    String::from("commitMessage")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.isArray
// formal-ai:blockers Array.isArray | FREE_TEXT.includes | call of a sibling function | field access | null | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.isArray
// formal-ai:blockers Array.isArray | arrow callback of .map() | arrow callback of .some() | assignment of a field or element | call of a sibling function | field access | method call .map() | method call .push() | method call .some() | null | object without a $ tag | typeof operator | undefined
