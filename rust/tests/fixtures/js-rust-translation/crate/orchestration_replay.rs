// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=94c01c5ea49d6bb01c88de91a13bbcd933fd7aaf082c5976b7babf9bf6689742 bytes=8927
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

/// The Math functions that Rust's f64 methods do not share with JavaScript.
pub mod ml_math {
    /// Math.round: the nearest integer, the one towards +Infinity on a tie,
    /// with the sign of a zero result from the argument.
    pub fn round(x: f64) -> f64 {
        if !x.is_finite() || x == 0.0 {
            return x;
        }
        if x > 0.0 && x < 0.5 {
            return 0.0;
        }
        if x < 0.0 && x >= -0.5 {
            return -0.0;
        }
        let floor = x.floor();
        if x - floor >= 0.5 {
            floor + 1.0
        } else {
            floor
        }
    }

    /// Math.sign, which keeps -0, 0 and NaN, where f64::signum does not.
    pub fn sign(x: f64) -> f64 {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            x
        }
    }

    /// Math.max of two Numbers: NaN when either is, and 0 above -0.
    pub fn max(a: f64, b: f64) -> f64 {
        if a > b {
            a
        } else if b > a {
            b
        } else if a == b {
            if a.is_sign_negative() {
                b
            } else {
                a
            }
        } else {
            f64::NAN
        }
    }

    /// Math.min of two Numbers: NaN when either is, and -0 below 0.
    pub fn min(a: f64, b: f64) -> f64 {
        if a < b {
            a
        } else if b < a {
            b
        } else if a == b {
            if a.is_sign_negative() {
                a
            } else {
                b
            }
        } else {
            f64::NAN
        }
    }

    pub fn max_of(values: &[f64]) -> f64 {
        values.iter().fold(f64::NEG_INFINITY, |a, &b| max(a, b))
    }

    pub fn min_of(values: &[f64]) -> f64 {
        values.iter().fold(f64::INFINITY, |a, &b| min(a, b))
    }

    pub fn is_integer(x: f64) -> bool {
        x.is_finite() && x.trunc() == x
    }

    pub fn is_safe_integer(x: f64) -> bool {
        is_integer(x) && x.abs() <= 9007199254740991.0
    }
}
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=7bbefa9855f1d92b680d94a70cc38c1c474bd2431ba0fbb595bb7ee5f9cb3fd4
// | export const SESSION_SCHEMA = 'formal-ai-agent-session-v1';
pub const SESSION_SCHEMA: &str = "formal-ai-agent-session-v1";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b9bef72d50baed8d481709325f6570f7f64cfb03107b87913e58844038d739f9
// | const SESSION_FIELDS = ['schema', 'cli', 'target', 'task', 'model', 'base_url', 'workspace', 'program', 'args', 'status', 'exit_code', 'wall_time_ms', 'stdout', 'stderr', 'changes', 'verification', 'events'];
// formal-ai:presentation:v1 rawSHA256=b9bef72d50baed8d481709325f6570f7f64cfb03107b87913e58844038d739f9 rawBytes=518 renderedSHA256=5db1c680b2dbbf5f6b86b3e1b15ab1bbad65e7520c9fde583f1db8062f42a583
// formal-ai:presentation-raw cHViIHN0YXRpYyBTRVNTSU9OX0ZJRUxEUzogc3RkOjpzeW5jOjpMYXp5TG9jazxWZWM8U3RyaW5nPj4gPSBzdGQ6OnN5bmM6
// formal-ai:presentation-raw OkxhenlMb2NrOjpuZXcofHwgdmVjIVtTdHJpbmc6OmZyb20oInNjaGVtYSIpLCBTdHJpbmc6OmZyb20oImNsaSIpLCBTdHJp
// formal-ai:presentation-raw bmc6OmZyb20oInRhcmdldCIpLCBTdHJpbmc6OmZyb20oInRhc2siKSwgU3RyaW5nOjpmcm9tKCJtb2RlbCIpLCBTdHJpbmc6
// formal-ai:presentation-raw OmZyb20oImJhc2VfdXJsIiksIFN0cmluZzo6ZnJvbSgid29ya3NwYWNlIiksIFN0cmluZzo6ZnJvbSgicHJvZ3JhbSIpLCBT
// formal-ai:presentation-raw dHJpbmc6OmZyb20oImFyZ3MiKSwgU3RyaW5nOjpmcm9tKCJzdGF0dXMiKSwgU3RyaW5nOjpmcm9tKCJleGl0X2NvZGUiKSwg
// formal-ai:presentation-raw U3RyaW5nOjpmcm9tKCJ3YWxsX3RpbWVfbXMiKSwgU3RyaW5nOjpmcm9tKCJzdGRvdXQiKSwgU3RyaW5nOjpmcm9tKCJzdGRl
// formal-ai:presentation-raw cnIiKSwgU3RyaW5nOjpmcm9tKCJjaGFuZ2VzIiksIFN0cmluZzo6ZnJvbSgidmVyaWZpY2F0aW9uIiksIFN0cmluZzo6ZnJv
// formal-ai:presentation-raw bSgiZXZlbnRzIildKTs=
// formal-ai:presentation-raw end
pub static SESSION_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("schema"),
    String::from("cli"),
    String::from("target"),
    String::from("task"),
    String::from("model"),
    String::from("base_url"),
    String::from("workspace"),
    String::from("program"),
    String::from("args"),
    String::from("status"),
    String::from("exit_code"),
    String::from("wall_time_ms"),
    String::from("stdout"),
    String::from("stderr"),
    String::from("changes"),
    String::from("verification"),
    String::from("events")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=145b21d25b65560d44971da1f577d9e2bd4c2f31283ce82ecb4c699336e3e857
// | const SESSION_OPTIONAL = ['native_session', 'continuation'];
pub static SESSION_OPTIONAL: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("native_session"), String::from("continuation")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=50b78fe0a43cb66156ca70da0bc81a279272d9e21b9f4a6cff35795e0266a08c
// | const EVENT_FIELDS = ['sequence', 'kind', 'detail', 'previous_sha256', 'sha256'];
// formal-ai:presentation:v1 rawSHA256=50b78fe0a43cb66156ca70da0bc81a279272d9e21b9f4a6cff35795e0266a08c rawBytes=223 renderedSHA256=ea25093a976ae2ece3ad1bcb34e4df2c35066182ade733951d4ea27bb9782108
// formal-ai:presentation-raw cHViIHN0YXRpYyBFVkVOVF9GSUVMRFM6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5jOjpM
// formal-ai:presentation-raw YXp5TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJzZXF1ZW5jZSIpLCBTdHJpbmc6OmZyb20oImtpbmQiKSwgU3Ry
// formal-ai:presentation-raw aW5nOjpmcm9tKCJkZXRhaWwiKSwgU3RyaW5nOjpmcm9tKCJwcmV2aW91c19zaGEyNTYiKSwgU3RyaW5nOjpmcm9tKCJzaGEy
// formal-ai:presentation-raw NTYiKV0pOw==
// formal-ai:presentation-raw end
pub static EVENT_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("sequence"),
    String::from("kind"),
    String::from("detail"),
    String::from("previous_sha256"),
    String::from("sha256")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=bf6d031af084b0f71585c2e64fad03b776a2e9f08362c366c7adbd603975eb4f
// | const CHANGE_FIELDS = ['path', 'kind', 'before_sha256', 'after_sha256', 'bytes_changed'];
// formal-ai:presentation:v1 rawSHA256=bf6d031af084b0f71585c2e64fad03b776a2e9f08362c366c7adbd603975eb4f rawBytes=231 renderedSHA256=2f7a3060c46ee91a1e6a9769c04238100ecd0518918a5b2b34c3e0a2b95f3183
// formal-ai:presentation-raw cHViIHN0YXRpYyBDSEFOR0VfRklFTERTOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxTdHJpbmc+PiA9IHN0ZDo6c3luYzo6
// formal-ai:presentation-raw TGF6eUxvY2s6Om5ldyh8fCB2ZWMhW1N0cmluZzo6ZnJvbSgicGF0aCIpLCBTdHJpbmc6OmZyb20oImtpbmQiKSwgU3RyaW5n
// formal-ai:presentation-raw Ojpmcm9tKCJiZWZvcmVfc2hhMjU2IiksIFN0cmluZzo6ZnJvbSgiYWZ0ZXJfc2hhMjU2IiksIFN0cmluZzo6ZnJvbSgiYnl0
// formal-ai:presentation-raw ZXNfY2hhbmdlZCIpXSk7
// formal-ai:presentation-raw end
pub static CHANGE_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("path"),
    String::from("kind"),
    String::from("before_sha256"),
    String::from("after_sha256"),
    String::from("bytes_changed")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9f0ff6fb144a8298a6f74a4a03516e339237eee68b50904a26708f721fbaf05a
// | const VERIFICATION_FIELDS = ['program', 'args', 'exit_code', 'stdout', 'stderr', 'timed_out', 'passed'];
// formal-ai:presentation:v1 rawSHA256=9f0ff6fb144a8298a6f74a4a03516e339237eee68b50904a26708f721fbaf05a rawBytes=274 renderedSHA256=3a2a49ceaea26a1d8936427e813ad42e365da2b0a3c73823d93d0dec4e516fab
// formal-ai:presentation-raw cHViIHN0YXRpYyBWRVJJRklDQVRJT05fRklFTERTOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxTdHJpbmc+PiA9IHN0ZDo6
// formal-ai:presentation-raw c3luYzo6TGF6eUxvY2s6Om5ldyh8fCB2ZWMhW1N0cmluZzo6ZnJvbSgicHJvZ3JhbSIpLCBTdHJpbmc6OmZyb20oImFyZ3Mi
// formal-ai:presentation-raw KSwgU3RyaW5nOjpmcm9tKCJleGl0X2NvZGUiKSwgU3RyaW5nOjpmcm9tKCJzdGRvdXQiKSwgU3RyaW5nOjpmcm9tKCJzdGRl
// formal-ai:presentation-raw cnIiKSwgU3RyaW5nOjpmcm9tKCJ0aW1lZF9vdXQiKSwgU3RyaW5nOjpmcm9tKCJwYXNzZWQiKV0pOw==
// formal-ai:presentation-raw end
pub static VERIFICATION_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("program"),
    String::from("args"),
    String::from("exit_code"),
    String::from("stdout"),
    String::from("stderr"),
    String::from("timed_out"),
    String::from("passed")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9224c42cde6defea78cb6fb8e0d8a9caa452e5f95ee17d7d73bae309dd4dd64b
// | const NATIVE_FIELDS = ['id', 'resume_command'];
pub static NATIVE_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("id"), String::from("resume_command")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a6e920fb2c0cc84a9e810ac2c39d5cc7d54b0772f9735eefde12703877a235ce
// | const CONTINUATION_FIELDS = ['parent_session_sha256', 'native_session_id', 'disproved_claim', 'evidence'];
// formal-ai:presentation:v1 rawSHA256=a6e920fb2c0cc84a9e810ac2c39d5cc7d54b0772f9735eefde12703877a235ce rawBytes=234 renderedSHA256=2072104aa15a40697790320758e6071177bb5bb1de82e227b66384d37f752fd1
// formal-ai:presentation-raw cHViIHN0YXRpYyBDT05USU5VQVRJT05fRklFTERTOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxTdHJpbmc+PiA9IHN0ZDo6
// formal-ai:presentation-raw c3luYzo6TGF6eUxvY2s6Om5ldyh8fCB2ZWMhW1N0cmluZzo6ZnJvbSgicGFyZW50X3Nlc3Npb25fc2hhMjU2IiksIFN0cmlu
// formal-ai:presentation-raw Zzo6ZnJvbSgibmF0aXZlX3Nlc3Npb25faWQiKSwgU3RyaW5nOjpmcm9tKCJkaXNwcm92ZWRfY2xhaW0iKSwgU3RyaW5nOjpm
// formal-ai:presentation-raw cm9tKCJldmlkZW5jZSIpXSk7
// formal-ai:presentation-raw end
pub static CONTINUATION_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("parent_session_sha256"),
    String::from("native_session_id"),
    String::from("disproved_claim"),
    String::from("evidence")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e7acbe53b43589fba8d0da05fd86a5ae356dbb2289d225e660196b0be0c1db07
// | const STATUSES = ['succeeded', 'failed', 'timed_out'];
// formal-ai:presentation:v1 rawSHA256=e7acbe53b43589fba8d0da05fd86a5ae356dbb2289d225e660196b0be0c1db07 rawBytes=168 renderedSHA256=8e87eaa258a5d2df656ab00b5b16891d7e2df834f263b8d36b4e4de7d51003f2
// formal-ai:presentation-raw cHViIHN0YXRpYyBTVEFUVVNFUzogc3RkOjpzeW5jOjpMYXp5TG9jazxWZWM8U3RyaW5nPj4gPSBzdGQ6OnN5bmM6OkxhenlM
// formal-ai:presentation-raw b2NrOjpuZXcofHwgdmVjIVtTdHJpbmc6OmZyb20oInN1Y2NlZWRlZCIpLCBTdHJpbmc6OmZyb20oImZhaWxlZCIpLCBTdHJp
// formal-ai:presentation-raw bmc6OmZyb20oInRpbWVkX291dCIpXSk7
// formal-ai:presentation-raw end
pub static STATUSES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("succeeded"),
    String::from("failed"),
    String::from("timed_out")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=2c52931d91a22a388c4e72067b0349bf3acf1d69c7eff68941ee0fd0ce0ad525
// | const TARGETS = ['formal_ai', 'vendor'];
pub static TARGETS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("formal_ai"), String::from("vendor")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=0ebfdf8dfbae15e5ec52fdbdce541e29cf7e5bb18e75fae56a1dd483a6447c56
// | const CHANGE_KINDS = ['added', 'modified', 'removed'];
// formal-ai:presentation:v1 rawSHA256=0ebfdf8dfbae15e5ec52fdbdce541e29cf7e5bb18e75fae56a1dd483a6447c56 rawBytes=168 renderedSHA256=32adcecf523cc512677449fc67cda1d0b8ab90b1419622eb01f9b8dbee21b651
// formal-ai:presentation-raw cHViIHN0YXRpYyBDSEFOR0VfS0lORFM6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5jOjpM
// formal-ai:presentation-raw YXp5TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJhZGRlZCIpLCBTdHJpbmc6OmZyb20oIm1vZGlmaWVkIiksIFN0
// formal-ai:presentation-raw cmluZzo6ZnJvbSgicmVtb3ZlZCIpXSk7
// formal-ai:presentation-raw end
pub static CHANGE_KINDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("added"),
    String::from("modified"),
    String::from("removed")]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=866409fd0066d4f2dbf818adacc80256c8e7e793ff40d67964a0eccbf09225cd
// | const COMPOSITION_STARTED = 'composition_verification_started';
pub const COMPOSITION_STARTED: &str = "composition_verification_started";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a5d37e9c338d211955ee7d9e0092e9010449b474cb294f68a5e2581e0729fe01
// | const WORKSPACE_EFFECT = 'workspace_effect';
pub const WORKSPACE_EFFECT: &str = "workspace_effect";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9c5be5dcfc8705c0f1170ed49e7a5461cf11cf2c20a3c8b70e05d0ee507097ae
// | const NATIVE_SESSION_RESUMED = 'native_session_resumed';
pub const NATIVE_SESSION_RESUMED: &str = "native_session_resumed";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | class | export … | field access | null

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal throw new …
// formal-ai:blockers arrow function | new ReplayError | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.isArray | Object.keys | arrow callback of .find() | arrow callback of .some() | call of a sibling function | in operator | method call .find() | method call .has() | method call .indexOf() | method call .slice() | method call .some() | null | sibling value | typeof operator

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers arrow function | typeof operator

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a6b78a1c0462176ee4b28c136176425fba16d0a0ad0c33f082324824477957f9
// | const isCount = (value) => Number.isSafeInteger(value) && value >= 0;
pub fn is_count(value: f64) -> bool {
    (crate::ml_math::is_safe_integer(value) && (value >= (0f64)))
}

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers arrow function | null | undefined

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Array.isArray
// formal-ai:blockers Array.isArray | arrow function | method call .every() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers CHANGE_KINDS.includes | EVENT_FIELDS.slice | STATUSES.includes | TARGETS.includes | arrow callback of .every() | call of a sibling function | field access | loose equality | method call .every() | null | typeof operator

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Array.isArray
// formal-ai:blockers Array.isArray | arrow function | call of a sibling function

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unsupported template escape
// formal-ai:blockers call of a sibling function | call of an imported function | destructuring | field access | method call .entries() | method call .repeat()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .some() | call of a sibling function | field access | in operator | loose equality | method call .filter() | method call .map() | method call .some() | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers JSON.stringify | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | JSON.parse | call of a sibling function | field access | let without a value | try statement

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function | field access
