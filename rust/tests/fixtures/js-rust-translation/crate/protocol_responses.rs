// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=c0d8751e60a81ae5c680643e924c2062e948186e8cef71b32a60c358afa6c0d5 bytes=7957
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
// formal-ai:blockers JSDoc type {…} | JSON.parse | Object.keys | call of a sibling function | call of an imported function | field access | let without a value | null | try statement | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.isArray
// formal-ai:blockers Object.create | Object.keys | arrow callback of .map() | arrow callback of .some() | assignment of a field or element | call of a sibling function | field access | method call .isArray() | method call .map() | method call .some() | null | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers call of a sibling function | field access | undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b85977188f5d0ef9ae5c1c8d4bf1f7fe325e5e48aa95b2ceba8d6b691ee45cae
// | const ALIASES = [
// |   [['path', 'filePath', 'file_path', 'absolute_path'], ['path', 'filePath', 'file_path', 'absolute_path']],
// |   [['command', 'cmd'], ['command', 'cmd']],
// |   [['query', 'pattern', 'input'], ['query', 'pattern', 'input']],
// |   [['paths', 'file_paths', 'files'], ['paths', 'file_paths', 'files']],
// |   [['old', 'oldString', 'old_string', 'old_str'], ['old', 'oldString', 'old_string', 'old_str']],
// |   [['new', 'newString', 'new_string', 'new_str'], ['new', 'newString', 'new_string', 'new_str']],
// | ];
// formal-ai:presentation:v1 rawSHA256=b85977188f5d0ef9ae5c1c8d4bf1f7fe325e5e48aa95b2ceba8d6b691ee45cae rawBytes=1205 renderedSHA256=ef237f935fc5131563ddfc68f06e86932c1239ca73199705406cf54043e6ff0a
// formal-ai:presentation-raw cHViIHN0YXRpYyBBTElBU0VTOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxWZWM8VmVjPFN0cmluZz4+Pj4gPSBzdGQ6OnN5
// formal-ai:presentation-raw bmM6OkxhenlMb2NrOjpuZXcofHwgdmVjIVt2ZWMhW3ZlYyFbU3RyaW5nOjpmcm9tKCJwYXRoIiksIFN0cmluZzo6ZnJvbSgi
// formal-ai:presentation-raw ZmlsZVBhdGgiKSwgU3RyaW5nOjpmcm9tKCJmaWxlX3BhdGgiKSwgU3RyaW5nOjpmcm9tKCJhYnNvbHV0ZV9wYXRoIildLCB2
// formal-ai:presentation-raw ZWMhW1N0cmluZzo6ZnJvbSgicGF0aCIpLCBTdHJpbmc6OmZyb20oImZpbGVQYXRoIiksIFN0cmluZzo6ZnJvbSgiZmlsZV9w
// formal-ai:presentation-raw YXRoIiksIFN0cmluZzo6ZnJvbSgiYWJzb2x1dGVfcGF0aCIpXV0sIHZlYyFbdmVjIVtTdHJpbmc6OmZyb20oImNvbW1hbmQi
// formal-ai:presentation-raw KSwgU3RyaW5nOjpmcm9tKCJjbWQiKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCJjb21tYW5kIiksIFN0cmluZzo6ZnJvbSgiY21k
// formal-ai:presentation-raw IildXSwgdmVjIVt2ZWMhW1N0cmluZzo6ZnJvbSgicXVlcnkiKSwgU3RyaW5nOjpmcm9tKCJwYXR0ZXJuIiksIFN0cmluZzo6
// formal-ai:presentation-raw ZnJvbSgiaW5wdXQiKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCJxdWVyeSIpLCBTdHJpbmc6OmZyb20oInBhdHRlcm4iKSwgU3Ry
// formal-ai:presentation-raw aW5nOjpmcm9tKCJpbnB1dCIpXV0sIHZlYyFbdmVjIVtTdHJpbmc6OmZyb20oInBhdGhzIiksIFN0cmluZzo6ZnJvbSgiZmls
// formal-ai:presentation-raw ZV9wYXRocyIpLCBTdHJpbmc6OmZyb20oImZpbGVzIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgicGF0aHMiKSwgU3RyaW5nOjpm
// formal-ai:presentation-raw cm9tKCJmaWxlX3BhdGhzIiksIFN0cmluZzo6ZnJvbSgiZmlsZXMiKV1dLCB2ZWMhW3ZlYyFbU3RyaW5nOjpmcm9tKCJvbGQi
// formal-ai:presentation-raw KSwgU3RyaW5nOjpmcm9tKCJvbGRTdHJpbmciKSwgU3RyaW5nOjpmcm9tKCJvbGRfc3RyaW5nIiksIFN0cmluZzo6ZnJvbSgi
// formal-ai:presentation-raw b2xkX3N0ciIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIm9sZCIpLCBTdHJpbmc6OmZyb20oIm9sZFN0cmluZyIpLCBTdHJpbmc6
// formal-ai:presentation-raw OmZyb20oIm9sZF9zdHJpbmciKSwgU3RyaW5nOjpmcm9tKCJvbGRfc3RyIildXSwgdmVjIVt2ZWMhW1N0cmluZzo6ZnJvbSgi
// formal-ai:presentation-raw bmV3IiksIFN0cmluZzo6ZnJvbSgibmV3U3RyaW5nIiksIFN0cmluZzo6ZnJvbSgibmV3X3N0cmluZyIpLCBTdHJpbmc6OmZy
// formal-ai:presentation-raw b20oIm5ld19zdHIiKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCJuZXciKSwgU3RyaW5nOjpmcm9tKCJuZXdTdHJpbmciKSwgU3Ry
// formal-ai:presentation-raw aW5nOjpmcm9tKCJuZXdfc3RyaW5nIiksIFN0cmluZzo6ZnJvbSgibmV3X3N0ciIpXV1dKTs=
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
    String::from("pattern"),
    String::from("input")],
    vec![String::from("query"),
    String::from("pattern"),
    String::from("input")]],
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

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5c3468f8eeed10ba042a267c856f04155f175f1df69f93be1d393f44738ed2d5
// | const FREE_TEXT = ['prompt', 'instruction', 'message', 'description', 'commit_message', 'commitMessage'];
// formal-ai:presentation:v1 rawSHA256=5c3468f8eeed10ba042a267c856f04155f175f1df69f93be1d393f44738ed2d5 rawBytes=261 renderedSHA256=008c300a39664f8219366458f4f5a3abc738ae715abfdca93bb22b1099c0ba3b
// formal-ai:presentation-raw cHViIHN0YXRpYyBGUkVFX1RFWFQ6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5jOjpMYXp5
// formal-ai:presentation-raw TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJwcm9tcHQiKSwgU3RyaW5nOjpmcm9tKCJpbnN0cnVjdGlvbiIpLCBT
// formal-ai:presentation-raw dHJpbmc6OmZyb20oIm1lc3NhZ2UiKSwgU3RyaW5nOjpmcm9tKCJkZXNjcmlwdGlvbiIpLCBTdHJpbmc6OmZyb20oImNvbW1p
// formal-ai:presentation-raw dF9tZXNzYWdlIiksIFN0cmluZzo6ZnJvbSgiY29tbWl0TWVzc2FnZSIpXSk7
// formal-ai:presentation-raw end
pub static FREE_TEXT: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("prompt"),
    String::from("instruction"),
    String::from("message"),
    String::from("description"),
    String::from("commit_message"),
    String::from("commitMessage")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers Array.isArray | FREE_TEXT.includes | call of a sibling function | call of an imported function | field access | nullish coalescing | undefined
