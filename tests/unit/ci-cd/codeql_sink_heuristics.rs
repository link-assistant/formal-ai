//! Regression coverage for the `CodeQL` check that failed pull request #1027.
//!
//! The `CodeQL` check reported **99 new alerts, 98 of them critical**, and not
//! one of them was a defect. Both queries behind them identify their sinks by
//! *name*, so a naming choice — not a change in what the code does — decided
//! whether the security gate was red:
//!
//! * `rust/hard-coded-cryptographic-value` makes a sink of "any argument going
//!   to a parameter whose name matches a credential name", where the names are
//!   exactly `password`, `iv`, `nonce` and `salt`
//!   (`rust/ql/lib/codeql/rust/security/HardcodedCryptographicValueExtensions.qll`,
//!   `HeuristicSinks`). `translation::selection` seeded a deterministic draw
//!   through `fn sample_index(.., salt: &str)`, so every constant that reached
//!   it — `0.0`, `1.0` and each `SolverConfig` default — was reported as a
//!   hard-coded cryptographic salt, 98 alerts across 27 files. Nothing on that
//!   path is cryptography: `fnv1a64` is a non-cryptographic hash.
//! * `rust/cleartext-logging` takes its sources from
//!   `HeuristicNames::nameIndicatesSensitiveData`, whose account-information
//!   pattern is `.*(acc(ou)?nt|puid|user.?(name|id)|session.?(id|key)).*`
//!   (`shared/concepts/codeql/concepts/internal/SensitiveDataHeuristics.qll`).
//!   `formal-ai improve` printed an evidence line from a binding named
//!   `session_id`, so a content-addressed digest that this repository commits
//!   under `docs/case-studies/` was read as a session token written to a log.
//!
//! Renaming the two of them cleared the alerts. These tests keep the fix from
//! decaying: they hold the same two heuristics locally, over the same files
//! `CodeQL` analyses, so the next name that would turn the security gate red is
//! caught by `cargo test` at review time instead of by a scan after the push.
//!
//! Neither test lowers the bar. Both queries stay enabled with their default
//! settings; what is asserted here is that this repository — which performs no
//! cryptography and logs no credentials — never *names* anything in a way that
//! claims otherwise.
//!
//! Two things this deliberately does not do, both findings to report rather
//! than defects to fix:
//!
//! * `rust/cleartext-logging` also treats `assert!` as a write to a log. Two
//!   such alerts are open on `main` —
//!   `tests/unit/docs_requirements_issue_917.rs:192` and
//!   `tests/unit/docs_requirements_issue_918.rs:184` — reached from bindings
//!   named `session_id`. Those really are Agent CLI session ids, read from
//!   `session-id.txt` and asserted to start with `ses_`: the name is correct,
//!   so renaming them to satisfy a heuristic would make the tests lie. The
//!   guard below covers the log-writing macros, where a wrong name is simply a
//!   wrong name; the assertion sinks stay visible as accepted alerts.
//! * The upstream name patterns match anywhere inside an identifier, while
//!   [`reads_as_account_information`] anchors on word segments. Substring
//!   matching would flag `accounted_for` in `examples/issue_559_meta_core.rs`,
//!   which `CodeQL`'s own scan of this tree does not report; anchoring keeps the
//!   local guard aligned with the alerts that actually exist instead of
//!   inventing extra ones.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Parameter names `HeuristicSinks` treats as cryptographic sinks. `key` is
/// deliberately absent upstream ("matching `key` results in too many false
/// positives"), so it is absent here too: this test mirrors the query, it does
/// not invent a stricter rule of its own.
const CRYPTOGRAPHIC_SINK_PARAMETERS: &[&str] = &["password", "iv", "nonce", "salt"];

/// Macros whose arguments `CodeQL` models as a write to a log.
const LOGGING_MACROS: &[&str] = &["print", "println", "eprint", "eprintln", "write", "writeln"];

/// Word segments that make an identifier account information under
/// `HeuristicNames::maybeAccountInfo`. Each entry is a run of consecutive
/// segments: `["user", "id"]` matches `user_id` and `userId`, while the single
/// entry `["userid"]` matches the unseparated spelling, which together cover
/// the pattern's optional one-character separator.
const ACCOUNT_INFO_SEGMENTS: &[&[&str]] = &[
    &["acct"],
    &["accts"],
    &["account"],
    &["accounts"],
    &["puid"],
    &["username"],
    &["user", "name"],
    &["userid"],
    &["user", "id"],
    &["sessionid"],
    &["session", "id"],
    &["sessionkey"],
    &["session", "key"],
];

/// Fragments of `HeuristicNames::notSensitiveRegexp`: a name carrying one of
/// these says the value is already hashed, encoded or is a location, and the
/// upstream heuristic then treats it as non-sensitive. `digest` is *not* one of
/// them, but a name containing `digest` never matches the account-information
/// pattern in the first place.
const NOT_SENSITIVE_FRAGMENTS: &[&str] = &[
    "redact",
    "censor",
    "obfuscate",
    "hash",
    "md5",
    "sha",
    "random",
    "crypt",
    "encode",
    "path",
    "file",
    "url",
];

/// Directory prefixes `.github/codeql/codeql-config.yml` keeps out of the
/// extractor. A name there cannot produce an alert, so it is out of scope here
/// for exactly the same reason.
const PATHS_IGNORED_BY_CODEQL: &[&str] = &["docs/", "dev/", "experiments/"];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every tracked `.rs` file `CodeQL` extracts, read as `(relative path, source)`.
///
/// Tracked files rather than a directory walk: an untracked scratch file is
/// never pushed, so it is never scanned, and `target/` is skipped without a
/// filter of its own.
fn analysed_rust_sources() -> Vec<(String, String)> {
    let output = Command::new("git")
        .current_dir(repository_root())
        .args(["ls-files", "-z", "*.rs"])
        .output()
        .expect("list the tracked Rust files");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let listing = String::from_utf8(output.stdout).expect("git ls-files emits UTF-8 paths");
    let mut sources: Vec<(String, String)> = listing
        .split('\0')
        .filter(|path| !path.is_empty())
        .filter(|path| {
            !PATHS_IGNORED_BY_CODEQL
                .iter()
                .any(|ignored| path.starts_with(ignored))
        })
        .map(|path| {
            let source = fs::read_to_string(repository_root().join(path))
                .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
            (path.to_owned(), source.replace("\r\n", "\n"))
        })
        .collect();
    sources.sort();
    assert!(
        sources.len() > 100,
        "expected the whole Rust tree, got {} file(s)",
        sources.len()
    );
    sources
}

/// Blank out comments, string literals and character literals, keeping one
/// character per character and every newline so reported line numbers stay
/// true.
///
/// Without this a doc comment that *explains* the `salt` heuristic — this file,
/// for one — would read as a declaration of it.
///
/// Everything downstream indexes by character, never by byte: blanking replaces
/// a multi-byte character with a single space, so byte offsets taken from the
/// original source would stop lining up.
fn code_only(source: &str) -> Vec<char> {
    let characters: Vec<char> = source.chars().collect();
    let mut out = Vec::with_capacity(characters.len());
    let mut index = 0;
    while index < characters.len() {
        let next = characters.get(index + 1).copied();
        if characters[index] == '/' && next == Some('/') {
            while index < characters.len() && characters[index] != '\n' {
                out.push(' ');
                index += 1;
            }
        } else if characters[index] == '/' && next == Some('*') {
            let mut depth = 0_usize;
            while index < characters.len() {
                let following = characters.get(index + 1).copied();
                if characters[index] == '/' && following == Some('*') {
                    depth += 1;
                    out.extend_from_slice(&[' ', ' ']);
                    index += 2;
                } else if characters[index] == '*' && following == Some('/') {
                    depth -= 1;
                    out.extend_from_slice(&[' ', ' ']);
                    index += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blanked(characters[index]));
                    index += 1;
                }
            }
        } else if let Some(hashes) = raw_string_hashes(&characters, index) {
            let closing: Vec<char> = format!("\"{}", "#".repeat(hashes)).chars().collect();
            out.extend(std::iter::repeat_n(' ', hashes + 2));
            index += hashes + 2;
            while index < characters.len() && !starts_with(&characters, index, &closing) {
                out.push(blanked(characters[index]));
                index += 1;
            }
            let closer = closing.len().min(characters.len().saturating_sub(index));
            out.extend(std::iter::repeat_n(' ', closer));
            index += closer;
        } else if characters[index] == '"' || is_char_literal_start(&characters, index) {
            let quote = characters[index];
            out.push(' ');
            index += 1;
            while index < characters.len() && characters[index] != quote {
                if characters[index] == '\\' && index + 1 < characters.len() {
                    out.push(blanked(characters[index]));
                    out.push(blanked(characters[index + 1]));
                    index += 2;
                    continue;
                }
                out.push(blanked(characters[index]));
                index += 1;
            }
            if index < characters.len() {
                out.push(' ');
                index += 1;
            }
        } else {
            out.push(characters[index]);
            index += 1;
        }
    }
    out
}

/// Keep a newline, replace every other blanked character with a space.
const fn blanked(character: char) -> char {
    if character == '\n' { '\n' } else { ' ' }
}

fn starts_with(characters: &[char], index: usize, literal: &[char]) -> bool {
    literal
        .iter()
        .enumerate()
        .all(|(offset, expected)| characters.get(index + offset) == Some(expected))
}

/// Number of `#` in a raw string opener at `index`, if one starts there.
fn raw_string_hashes(characters: &[char], index: usize) -> Option<usize> {
    if characters.get(index) != Some(&'r') {
        return None;
    }
    if index > 0 && (characters[index - 1].is_alphanumeric() || characters[index - 1] == '_') {
        return None;
    }
    let mut hashes = 0;
    while characters.get(index + 1 + hashes) == Some(&'#') {
        hashes += 1;
    }
    if characters.get(index + 1 + hashes) == Some(&'"') {
        Some(hashes)
    } else {
        None
    }
}

/// A `'` opens a character literal only when it is not a lifetime (`'a`) or a
/// label (`'outer:`): a character literal holds one character, or one escape,
/// and then closes.
fn is_char_literal_start(characters: &[char], index: usize) -> bool {
    if characters[index] != '\'' {
        return false;
    }
    match characters.get(index + 1) {
        Some('\\') => true,
        Some(_) => characters.get(index + 2) == Some(&'\''),
        None => false,
    }
}

fn find_from(characters: &[char], from: usize, literal: &[char]) -> Option<usize> {
    (from..characters.len()).find(|index| starts_with(characters, *index, literal))
}

fn line_of(characters: &[char], offset: usize) -> usize {
    characters[..offset.min(characters.len())]
        .iter()
        .filter(|character| **character == '\n')
        .count()
        + 1
}

/// Offset of the `)` closing the `(` at `open`.
fn matching_parenthesis(characters: &[char], open: usize) -> Option<usize> {
    let mut depth = 0_i32;
    for (offset, character) in characters[open..].iter().enumerate() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// Offset of the `(` that opens the parameter list of the `fn` keyword at
/// `keyword`, skipping any `(` inside a generic list such as
/// `fn run<F: Fn(u8) -> u8>(..)`.
fn parameter_list_start(characters: &[char], keyword: usize) -> Option<usize> {
    let mut angle = 0_i32;
    let mut index = keyword + 3;
    while index < characters.len() {
        match characters[index] {
            '<' => angle += 1,
            // `->` inside a generic bound is not a closing angle bracket.
            '>' if index > 0 && characters[index - 1] == '-' => {}
            '>' => angle -= 1,
            '(' if angle == 0 => return Some(index),
            '(' => index = matching_parenthesis(characters, index)?,
            '{' | '}' | ';' => return None,
            _ => {}
        }
        index += 1;
    }
    None
}

/// The binding names declared by every `fn` parameter list, as `(name, line)`.
/// Comments and literals must already be blanked out.
fn parameter_bindings(characters: &[char]) -> Vec<(String, usize)> {
    let keyword_characters = ['f', 'n', ' '];
    let mut bindings = Vec::new();
    let mut index = 0;
    while let Some(keyword) = find_from(characters, index, &keyword_characters) {
        index = keyword + 3;
        if keyword > 0
            && (characters[keyword - 1].is_alphanumeric() || characters[keyword - 1] == '_')
        {
            continue;
        }
        let Some(open) = parameter_list_start(characters, keyword) else {
            continue;
        };
        let Some(close) = matching_parenthesis(characters, open) else {
            continue;
        };
        for (name, offset) in split_top_level(&characters[open + 1..close], open + 1) {
            bindings.push((name, line_of(characters, offset)));
        }
        index = close;
    }
    bindings
}

/// Split a parameter list on its top-level commas and return the binding name
/// of each parameter with the offset it was declared at.
fn split_top_level(parameters: &[char], base: usize) -> Vec<(String, usize)> {
    let mut names = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0_usize;
    let mut pieces: Vec<(usize, usize)> = Vec::new();
    for (offset, character) in parameters.iter().enumerate() {
        match character {
            '(' | '[' | '<' | '{' => depth += 1,
            ')' | ']' | '>' | '}' => depth -= 1,
            ',' if depth == 0 => {
                pieces.push((start, offset));
                start = offset + 1;
            }
            _ => {}
        }
    }
    pieces.push((start, parameters.len()));

    for (from, to) in pieces {
        let piece = &parameters[from..to];
        // `self`, and anything else without a `pattern: type` split, declares
        // no name the heuristic can match.
        let mut depth = 0_i32;
        let colon = piece.iter().position(|character| match character {
            '(' | '[' | '<' | '{' => {
                depth += 1;
                false
            }
            ')' | ']' | '>' | '}' => {
                depth -= 1;
                false
            }
            ':' => depth == 0,
            _ => false,
        });
        let Some(colon) = colon else { continue };
        let leading = piece[..colon]
            .iter()
            .position(|character| !character.is_whitespace())
            .unwrap_or(0);
        let pattern: String = piece[leading..colon].iter().collect();
        let name = pattern
            .trim_end()
            .trim_start_matches('&')
            .trim_start()
            .trim_start_matches("mut ")
            .trim_start();
        if name.is_empty()
            || !name
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_')
        {
            continue;
        }
        names.push((name.to_owned(), base + from + leading));
    }
    names
}

/// The lowercase word segments of an identifier, split on `_` and on
/// camel-case boundaries.
fn segments(name: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    for character in name.chars() {
        if character == '_' {
            if !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
        } else if character.is_uppercase() && !current.is_empty() {
            segments.push(std::mem::take(&mut current));
            current.push(character.to_ascii_lowercase());
        } else {
            current.push(character.to_ascii_lowercase());
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

/// Holds if `name` is what `nameIndicatesSensitiveData` calls account
/// information: it carries an account-info word and nothing that marks the
/// value as already hashed, encoded or a location.
fn reads_as_account_information(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    if NOT_SENSITIVE_FRAGMENTS
        .iter()
        .any(|fragment| lowered.contains(fragment))
    {
        return false;
    }
    let segments = segments(name);
    ACCOUNT_INFO_SEGMENTS.iter().any(|wanted| {
        segments.windows(wanted.len()).any(|window| {
            window
                .iter()
                .zip(wanted.iter())
                .all(|(have, want)| have == want)
        })
    })
}

/// Every identifier a logging macro writes, as `(identifier, line)`.
///
/// `code` must be the blanked form and `raw` the original, character for
/// character — [`code_only`] preserves the count, so an offset means the same
/// place in both. The macro's extent is found in `code`, where a `)` inside a
/// string cannot end it early; the arguments are then read from both:
///
/// * from `code`, the identifiers passed as arguments, and
/// * from `raw`, the identifiers captured inline by a format string.
///
/// The inline half is not optional. `formal-ai improve` wrote
/// `eprintln!("… {session_id}")` with no argument list at all, which is the
/// ordinary spelling in this edition and precisely the alert being guarded
/// against.
fn logged_identifiers(code: &[char], raw: &[char]) -> Vec<(String, usize)> {
    let mut logged = Vec::new();
    for macro_name in LOGGING_MACROS {
        let opener: Vec<char> = format!("{macro_name}!(").chars().collect();
        let mut index = 0;
        while let Some(start) = find_from(code, index, &opener) {
            index = start + opener.len();
            if start > 0 && (code[start - 1].is_alphanumeric() || code[start - 1] == '_') {
                continue;
            }
            let open = index - 1;
            let Some(close) = matching_parenthesis(code, open) else {
                continue;
            };
            logged.extend(argument_identifiers(code, open, close));
            logged.extend(interpolated_identifiers(raw, open, close));
            index = close;
        }
    }
    logged
}

/// The identifiers written between `open` and `close` of a macro call.
fn argument_identifiers(code: &[char], open: usize, close: usize) -> Vec<(String, usize)> {
    let mut identifiers = Vec::new();
    let mut identifier = String::new();
    let mut identifier_start = open;
    for (offset, character) in code[open..close].iter().enumerate() {
        if character.is_alphanumeric() || *character == '_' {
            if identifier.is_empty() {
                identifier_start = open + offset;
            }
            identifier.push(*character);
        } else if !identifier.is_empty() {
            identifiers.push((
                std::mem::take(&mut identifier),
                line_of(code, identifier_start),
            ));
        }
    }
    if !identifier.is_empty() {
        identifiers.push((identifier, line_of(code, identifier_start)));
    }
    identifiers
}

/// The identifiers a `{name}` or `{name:spec}` placeholder captures, read from
/// the unblanked source between `open` and `close`.
fn interpolated_identifiers(raw: &[char], open: usize, close: usize) -> Vec<(String, usize)> {
    let mut identifiers = Vec::new();
    let mut index = open;
    while index < close && index < raw.len() {
        if raw[index] != '{' {
            index += 1;
            continue;
        }
        // `{{` is an escaped brace, not a placeholder.
        if raw.get(index + 1) == Some(&'{') {
            index += 2;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < close && raw[end] != '}' && raw[end] != ':' {
            end += 1;
        }
        let name: String = raw[start..end].iter().collect();
        let captures_a_binding = !name.is_empty()
            && !name.starts_with(|character: char| character.is_ascii_digit())
            && name
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_');
        if captures_a_binding {
            identifiers.push((name, line_of(raw, start)));
        }
        index = end + 1;
    }
    identifiers
}

/// No `fn` parameter in the analysed tree is named after a
/// `rust/hard-coded-cryptographic-value` sink.
///
/// This repository performs no cryptography, so the honest allowlist is empty:
/// a parameter named `salt`, `password`, `iv` or `nonce` here is a misnomer,
/// and it turns every constant that can reach it into a critical alert.
#[test]
fn no_function_parameter_is_named_after_a_hard_coded_cryptographic_sink() {
    let mut offenders = Vec::new();
    for (path, source) in analysed_rust_sources() {
        let code = code_only(&source);
        for (name, line) in parameter_bindings(&code) {
            if CRYPTOGRAPHIC_SINK_PARAMETERS.contains(&name.as_str()) {
                offenders.push(format!("{path}:{line}: parameter `{name}`"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "CodeQL's rust/hard-coded-cryptographic-value makes a sink of every argument reaching a \
         parameter named {CRYPTOGRAPHIC_SINK_PARAMETERS:?}, so each constant that can reach one \
         of these is reported as a hard-coded cryptographic value. Rename the parameter after \
         what it is (`seed` for a deterministic draw, for instance):\n  {}",
        offenders.join("\n  ")
    );
}

/// No logging macro in the analysed tree is handed a binding whose name
/// `CodeQL`'s sensitive-data heuristic reads as account information.
///
/// The values this repository prints on those paths are content-addressed
/// digests it publishes as evidence, not credentials — so naming one
/// `session_id` states something untrue about it, and `rust/cleartext-logging`
/// believes the name.
#[test]
fn no_logging_macro_is_handed_a_name_that_reads_as_account_information() {
    let mut offenders = Vec::new();
    for (path, source) in analysed_rust_sources() {
        let raw: Vec<char> = source.chars().collect();
        let code = code_only(&source);
        for (identifier, line) in logged_identifiers(&code, &raw) {
            if reads_as_account_information(&identifier) {
                offenders.push(format!("{path}:{line}: `{identifier}`"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "CodeQL's rust/cleartext-logging reads a name matching \
         `.*(acc(ou)?nt|puid|user.?(name|id)|session.?(id|key)).*` as account information and \
         reports writing it to a log. Name the value what it is, or do not log it:\n  {}",
        offenders.join("\n  ")
    );
}

/// The scan above and `CodeQL` must agree on which files are in scope, or the
/// test passes over exactly the directory that produces the next alert.
#[test]
fn the_scan_skips_the_same_directories_the_codeql_config_ignores() {
    let config = fs::read_to_string(repository_root().join(".github/codeql/codeql-config.yml"))
        .expect("read the CodeQL configuration");
    let ignored: Vec<String> = config
        .lines()
        .skip_while(|line| !line.starts_with("paths-ignore:"))
        .skip(1)
        .take_while(|line| line.starts_with("  - "))
        .map(|line| {
            line.trim_start_matches("  - ")
                .trim_end_matches("**")
                .to_owned()
        })
        .collect();

    assert_eq!(
        ignored, PATHS_IGNORED_BY_CODEQL,
        "`.github/codeql/codeql-config.yml` no longer ignores the directories this test skips; \
         update PATHS_IGNORED_BY_CODEQL so the local scan covers everything CodeQL extracts"
    );
}

#[cfg(test)]
mod parser {
    use super::{code_only, logged_identifiers, parameter_bindings, reads_as_account_information};

    fn bindings(source: &str) -> Vec<(String, usize)> {
        parameter_bindings(&code_only(source))
    }

    fn logged(source: &str) -> Vec<(String, usize)> {
        let raw: Vec<char> = source.chars().collect();
        logged_identifiers(&code_only(source), &raw)
    }

    /// The acceptance case: the declaration that produced 98 critical alerts,
    /// and the name that replaced it.
    #[test]
    fn a_parameter_named_salt_is_found_and_the_seed_that_replaced_it_is_not() {
        let before = "fn sample_index(weights: &[f32], impulse: &str, salt: &str) -> usize {}";
        assert_eq!(
            bindings(before),
            vec![
                ("weights".to_owned(), 1),
                ("impulse".to_owned(), 1),
                ("salt".to_owned(), 1),
            ]
        );

        let after = "fn sample_index(weights: &[f32], impulse: &str, seed: &str) -> usize {}";
        assert_eq!(
            bindings(after),
            vec![
                ("weights".to_owned(), 1),
                ("impulse".to_owned(), 1),
                ("seed".to_owned(), 1),
            ]
        );
    }

    /// Prose about the heuristic is not a declaration of it — this file would
    /// otherwise fail its own test.
    #[test]
    fn a_salt_in_a_comment_or_a_string_declares_nothing() {
        let source =
            "/// Not named `salt`.\n// fn f(salt: &str) {}\nlet s = \"fn g(salt: &str) {}\";\n";
        assert!(bindings(source).is_empty());
    }

    /// A parameter list spanning several lines reports the line each parameter
    /// is written on, which is what a reader needs in order to fix it.
    #[test]
    fn a_multi_line_signature_reports_the_line_of_each_parameter() {
        let source = "fn select(\n    candidates: &[u8],\n    salt: &str,\n) -> usize {}";
        assert_eq!(
            bindings(source),
            vec![("candidates".to_owned(), 2), ("salt".to_owned(), 3)]
        );
    }

    /// `self`, closure-typed parameters and generic bounds must not be mistaken
    /// for a binding name, and a `(` inside a generic list is not the start of
    /// the parameter list.
    #[test]
    fn self_and_nested_types_do_not_produce_spurious_names() {
        let source = "fn run<F: Fn(u8) -> u8>(&self, apply: F, pairs: Vec<(u8, u8)>) {}";
        assert_eq!(
            bindings(source),
            vec![("apply".to_owned(), 1), ("pairs".to_owned(), 1)]
        );
    }

    /// `mut` and reference patterns belong to the pattern, not to the name.
    #[test]
    fn patterns_are_stripped_down_to_the_binding_name() {
        let source = "fn f(mut salt: String, &nonce: &u64) {}";
        assert_eq!(
            bindings(source),
            vec![("salt".to_owned(), 1), ("nonce".to_owned(), 1)]
        );
    }

    /// The line that produced the one alert this pull request would have added,
    /// in the spelling it was actually written in: an inline capture, with no
    /// argument list to read the name from.
    #[test]
    fn an_inline_capture_is_read_out_of_the_format_string() {
        let source = "eprintln!(\"Formal AI Agent session evidence: {session_id}\");";
        assert_eq!(logged(source), vec![("session_id".to_owned(), 1)]);
        assert!(reads_as_account_information("session_id"));

        let fixed = "eprintln!(\"Formal AI Agent session evidence: {digest}\");";
        assert_eq!(logged(fixed), vec![("digest".to_owned(), 1)]);
        assert!(!reads_as_account_information("digest"));
    }

    /// A positional placeholder, a width specifier and an escaped brace capture
    /// no binding, and a `)` inside the format string does not end the call.
    // The fixture spells out a format string; clippy reads its placeholders as
    // if this were a formatting macro, which is the one thing it is not.
    #[allow(clippy::literal_string_with_formatting_args)]
    #[test]
    fn placeholders_that_capture_nothing_are_not_mistaken_for_bindings() {
        assert_eq!(
            logged("println!(\"{0} {{literal}} {label:>8} (done)\", label);"),
            vec![("label".to_owned(), 1), ("label".to_owned(), 1)]
        );
    }

    /// The names this repository actually uses are not account information:
    /// `agent_session_digests` never matches the pattern, and a name carrying
    /// `hash` is excluded by the upstream heuristic itself.
    #[test]
    fn digests_and_hashes_are_not_read_as_account_information() {
        assert!(!reads_as_account_information("agent_session_digests"));
        assert!(!reads_as_account_information("session_id_hash"));
        assert!(!reads_as_account_information("impulse"));
        assert!(reads_as_account_information("user_name"));
        assert!(reads_as_account_information("nativeSessionId"));
        assert!(reads_as_account_information("AccountNumber"));
    }

    /// What word-segment anchoring buys: `accounted_for` carries `account` as a
    /// substring but is not an account, and `CodeQL`'s own scan of this tree does
    /// not report it.
    #[test]
    fn a_substring_that_is_not_a_word_is_not_account_information() {
        assert!(!reads_as_account_information("accounted_for"));
        assert!(!reads_as_account_information("user_prompt"));
        assert!(reads_as_account_information("accounts"));
    }
}
