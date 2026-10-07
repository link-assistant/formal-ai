// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e0981d77f0dc717ca123e173237ad2e2f3d79ddb9e4b2aea0f8244970f1d0b87 bytes=9704
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal typeof operator

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Object.prototype.hasOwnProperty.call

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Object.keys

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=27de87701f704d8b6b17c4da1dc771256a745224312d6dd0a0e518e92c8bca4d
// | const ALIASES = [
// |   [['path', 'filePath', 'file_path', 'absolute_path'], ['path', 'filePath', 'file_path', 'absolute_path']],
// |   [['command', 'cmd'], ['command', 'cmd']],
// |   [['query', 'pattern'], ['query', 'pattern']],
// |   [['paths', 'file_paths', 'files'], ['paths', 'file_paths', 'files']],
// |   [['old', 'oldString', 'old_string', 'old_str'], ['old', 'oldString', 'old_string', 'old_str']],
// |   [['new', 'newString', 'new_string', 'new_str'], ['new', 'newString', 'new_string', 'new_str']],
// | ];
pub static ALIASES: std::sync::LazyLock<Vec<Vec<Vec<String>>>> = std::sync::LazyLock::new(|| vec![vec![vec![String::from("path"), String::from("filePath"), String::from("file_path"), String::from("absolute_path")], vec![String::from("path"), String::from("filePath"), String::from("file_path"), String::from("absolute_path")]], vec![vec![String::from("command"), String::from("cmd")], vec![String::from("command"), String::from("cmd")]], vec![vec![String::from("query"), String::from("pattern")], vec![String::from("query"), String::from("pattern")]], vec![vec![String::from("paths"), String::from("file_paths"), String::from("files")], vec![String::from("paths"), String::from("file_paths"), String::from("files")]], vec![vec![String::from("old"), String::from("oldString"), String::from("old_string"), String::from("old_str")], vec![String::from("old"), String::from("oldString"), String::from("old_string"), String::from("old_str")]], vec![vec![String::from("new"), String::from("newString"), String::from("new_string"), String::from("new_str")], vec![String::from("new"), String::from("newString"), String::from("new_string"), String::from("new_str")]]]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:translated JavaScript lexical_declaration items=1 sha256=943e885b0c6108c2eb5a2f5d9f866ce2814b3346ab5b308df5f7bad4aac9725c
// | const PATH_PROPERTIES = ['path', 'filePath', 'file_path', 'absolute_path', 'dir_path', 'directory', 'notebook_path'];
pub static PATH_PROPERTIES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("path"), String::from("filePath"), String::from("file_path"), String::from("absolute_path"), String::from("dir_path"), String::from("directory"), String::from("notebook_path")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined

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

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9dcc71624ce0249786ccb61b47b37c0d1b7bef49f7a631cf2801931bfc943c54
// | const CONTENT_PROPERTIES = ['content', 'contents', 'file_text', 'text', 'new_string'];
pub static CONTENT_PROPERTIES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("content"), String::from("contents"), String::from("file_text"), String::from("text"), String::from("new_string")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing

// meta-language:translated JavaScript lexical_declaration items=1 sha256=abf836d03d47e1dc3f522b2bca213dd7262d3760c70b2e8f7073d0409df16887
// | const FREE_TEXT = ['prompt', 'instruction', 'message', 'commit_message', 'commitMessage'];
pub static FREE_TEXT: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("prompt"), String::from("instruction"), String::from("message"), String::from("commit_message"), String::from("commitMessage")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.isArray

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.isArray
