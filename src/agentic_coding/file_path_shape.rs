//! Whether a token written in a request names a file at all.
//!
//! Several routes have to answer this from a bare token: the read route looks
//! for the file to open, the write-request parser looks for the file to create,
//! and the shell route looks for the argument to pass along. They answer it in
//! their own terms — one wants a separator, another an extension — but they
//! must agree on what is *not* a path, or the same request is arbitrated
//! differently depending on which route reaches it first.

/// Whether `token` spells a number rather than a file name.
///
/// A dotted run of digits is a version, an ordinal address inside a tree, or a
/// dotted-decimal host address: `2.7.19`, `1.1.1.1.1`, `192.168.0.14`. None of
/// them names a file, but every one of them splits on its last dot into a
/// non-empty stem and a short non-empty "extension", which is exactly the shape
/// a file name has. The digits are the whole of the difference, so the check
/// has to be made explicitly.
///
/// A name that carries any other character is left alone: `access.log.1` is a
/// rotated log file and keeps its meaning.
pub(super) fn is_dotted_number(token: &str) -> bool {
    token.contains('.')
        && token.chars().any(|character| character.is_ascii_digit())
        && token
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
}

/// Strip the sentence's terminating dot from a path token.
///
/// Prose puts a path inside a sentence, so a path that ends a sentence carries
/// that sentence's full stop: *"Read the file Cargo.toml."* hands the routes the
/// token `Cargo.toml.`, which splits on its last dot into the stem `Cargo.toml`
/// and an empty extension — no longer file-shaped by any of the tests above, so
/// the read route declined the plainest read request there is and the router
/// fell through to web search.
///
/// The dot can be removed safely because no path component may end in one: a
/// trailing dot always belongs to the sentence. The exception is a token that is
/// nothing but dots — `.` and `..` name the current and the parent directory,
/// and there the dots are the whole name, so the token is returned untouched.
pub(super) fn trim_trailing_sentence_dot(token: &str) -> &str {
    let trimmed = token.trim_end_matches('.');
    if trimmed.is_empty() || trimmed.ends_with('/') {
        token
    } else {
        trimmed
    }
}

/// Peel the punctuation a sentence wrapped around a path token, in any order.
///
/// Prose nests its marks, and the writer picks the nesting: a fenced path that
/// ends a sentence arrives as `` `Cargo.toml`. `` -- a fence inside a full stop
/// -- while the same path mid-sentence arrives as `` `Cargo.toml`, ``. Stripping
/// each class once, in whichever order the code happened to write, clears the
/// outermost layer and leaves the inner one: `` `Cargo.toml`. `` kept its
/// closing fence, stopped being file-shaped, and the plainest read request there
/// is fell through to the open-web routers (issue #1066).
///
/// Peeling to a fixpoint is what stops the writer's ordering from mattering.
/// `strip` supplies the marks the calling route accepts; the sentence's
/// terminating dot is applied here so every route keeps drawing that boundary
/// the same way.
pub(super) fn peel_sentence_punctuation<'a>(
    token: &'a str,
    strip: impl Fn(&'a str) -> &'a str,
) -> &'a str {
    let mut current = token;
    loop {
        let next = trim_trailing_sentence_dot(strip(current));
        if next == current {
            return current;
        }
        current = next;
    }
}
