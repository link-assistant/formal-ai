//! Capability is a function of the object in the request, the act asked for,
//! and where the effect lands (#1138 B10, plan 10 Architecture 1-2).
//!
//! Verbs select the *act*; they never select the capability. A triple with no
//! row in `data/seed/capability-routing.lino` asks rather than guesses, so a
//! silent UNKNOWN is unreachable by construction.
//!
//! The three derivations are deliberately asymmetric about vocabulary. The
//! *object* is structural -- a URL parses, a path carries a separator or an
//! extension, a clock time is digits around a colon -- so it carries no
//! natural-language branch, and the two shapes that cannot be read off the
//! characters are grounded in seed roles rather than spelled here. The *act* is
//! the one place per-language verb surfaces remain, and it is shared by every
//! capability. The *locus* is derived from the objects and the seed's own scope
//! nouns, so a possessive is never evidence about where an effect lands.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::seed::parser::parse_lino;
use crate::seed::{
    self, ROLE_ASSISTANT_MECHANISM_INQUIRY, ROLE_CALENDAR_DAY_REFERENCE,
    ROLE_CAPABILITY_ACT_COMPOSE, ROLE_CAPABILITY_ACT_DEMONSTRATE, ROLE_CAPABILITY_ACT_ENUMERATE,
    ROLE_CAPABILITY_ACT_EXPLAIN, ROLE_CAPABILITY_ACT_RECORD, ROLE_CAPABILITY_ACT_RETRIEVE,
    ROLE_CAPABILITY_ACT_SCHEDULE, ROLE_CAPABILITY_ACT_TRANSFORM, ROLE_CAPABILITY_CLOCK_REFERENCE,
    ROLE_CAPABILITY_CONTAINER_SCOPE, ROLE_CAPABILITY_CONTENT_ASSIGNMENT,
    ROLE_CAPABILITY_CONTENT_INTRODUCER, ROLE_CAPABILITY_FRESHNESS_LIVE,
    ROLE_CAPABILITY_LANGUAGE_REFERENCE, ROLE_CAPABILITY_PRIOR_TURN_REFERENCE,
    ROLE_CAPABILITY_QUANTITY_INTERROGATIVE, ROLE_CAPABILITY_SELF_SURFACE_NOUN,
    ROLE_CAPABILITY_WEB_HOST_SUFFIX, ROLE_CAPABILITY_WEB_SCOPE, ROLE_CAPABILITY_WORKSPACE_SCOPE,
    ROLE_LOCAL_PATH_SCOPE_CURRENT, ROLE_LOCAL_PATH_SCOPE_DESKTOP, ROLE_LOCAL_PATH_SCOPE_HOME,
    ROLE_TRANSLATION_LANGUAGE,
};
use crate::web_engine_core::normalize_prompt;

/// What the request is *about*, derived structurally with no natural-language
/// vocabulary in the derivation (plan 10 Architecture 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ObjectType {
    /// A token parsing as an absolute URL.
    Url,
    /// A token with a path separator, a known extension, or a filesystem noun.
    Path,
    /// A filesystem scope noun (`desktop`, `home`, the current directory).
    PathScope,
    /// A balanced quoted span longer than one token.
    QuotedContent,
    /// A clock time, a date, or a weekday/relative-day reference.
    TimeExpression,
    /// A term resolving to a registered language in `data/seed/languages.lino`.
    LanguageName,
    /// An interrogative whose expected answer is a magnitude.
    QuantityQuestion,
    /// One or more content tokens that are none of the above.
    BareTerm,
    /// The subject is the assistant's own UI, answer or behaviour.
    SelfSurface,
    /// No content token survives.
    #[default]
    None,
}

/// Every object type, in the order the decision table's axis declares them.
const OBJECT_TYPES: [ObjectType; 10] = [
    ObjectType::Url,
    ObjectType::Path,
    ObjectType::PathScope,
    ObjectType::QuotedContent,
    ObjectType::TimeExpression,
    ObjectType::LanguageName,
    ObjectType::QuantityQuestion,
    ObjectType::BareTerm,
    ObjectType::SelfSurface,
    ObjectType::None,
];

impl ObjectType {
    /// The seed slug, so the decision table is data.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::Path => "path",
            Self::PathScope => "path_scope",
            Self::QuotedContent => "quoted_content",
            Self::TimeExpression => "time_expression",
            Self::LanguageName => "language_name",
            Self::QuantityQuestion => "quantity_question",
            Self::BareTerm => "bare_term",
            Self::SelfSurface => "self_surface",
            Self::None => "none",
        }
    }

    /// Read an object type back from its seed slug.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        OBJECT_TYPES
            .into_iter()
            .find(|object| object.slug() == slug)
    }

    /// Where this object sits when a prompt carries several.
    ///
    /// A parsed URL or a path literal is the least ambiguous thing a request can
    /// name, so it outranks everything else; a bare term is what is left when
    /// nothing more specific survived, so it ranks last but one.
    const fn rank(self) -> u8 {
        match self {
            Self::Url => 0,
            Self::Path => 1,
            Self::SelfSurface => 2,
            Self::LanguageName => 3,
            Self::TimeExpression => 4,
            Self::QuantityQuestion => 5,
            Self::QuotedContent => 6,
            Self::PathScope => 7,
            Self::BareTerm => 8,
            Self::None => 9,
        }
    }
}

/// The eight acts, one meaning per act, five languages, seeded in
/// `data/seed/meanings-acts.lino`. The act vocabulary is the one place
/// per-language surfaces remain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Act {
    Retrieve,
    Enumerate,
    Transform,
    Compose,
    Schedule,
    Explain,
    Demonstrate,
    Record,
    #[default]
    Unresolved,
}

/// The acts in the order they are tested, most specific first.
///
/// `retrieve` is last because it is the most general: nearly every request
/// retrieves something on the way to what it actually asks for, so a prompt that
/// also evidences a narrower act means the narrower one.
const ACTS_IN_PRECEDENCE: [Act; 8] = [
    Act::Schedule,
    Act::Demonstrate,
    Act::Compose,
    Act::Enumerate,
    Act::Transform,
    Act::Explain,
    Act::Record,
    Act::Retrieve,
];

impl Act {
    /// The seed slug, so the decision table is data.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Retrieve => "retrieve",
            Self::Enumerate => "enumerate",
            Self::Transform => "transform",
            Self::Compose => "compose",
            Self::Schedule => "schedule",
            Self::Explain => "explain",
            Self::Demonstrate => "demonstrate",
            Self::Record => "record",
            Self::Unresolved => "unresolved",
        }
    }

    /// Read an act back from its seed slug.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        ACTS_IN_PRECEDENCE
            .into_iter()
            .chain([Self::Unresolved])
            .find(|act| act.slug() == slug)
    }

    /// The seed role whose surfaces evidence this act.
    const fn role(self) -> &'static str {
        match self {
            Self::Retrieve => ROLE_CAPABILITY_ACT_RETRIEVE,
            Self::Enumerate => ROLE_CAPABILITY_ACT_ENUMERATE,
            Self::Transform => ROLE_CAPABILITY_ACT_TRANSFORM,
            Self::Compose => ROLE_CAPABILITY_ACT_COMPOSE,
            Self::Schedule => ROLE_CAPABILITY_ACT_SCHEDULE,
            Self::Explain => ROLE_CAPABILITY_ACT_EXPLAIN,
            Self::Demonstrate => ROLE_CAPABILITY_ACT_DEMONSTRATE,
            Self::Record => ROLE_CAPABILITY_ACT_RECORD,
            Self::Unresolved => "",
        }
    }
}

/// Where the effect lands. Derived, never stated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Locus {
    Workspace,
    Web,
    Dialogue,
    SelfSurface,
    /// Unresolved: the table's `ask` outcome applies.
    #[default]
    Unresolved,
}

impl Locus {
    /// The seed slug, so the decision table is data.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Web => "web",
            Self::Dialogue => "dialogue",
            Self::SelfSurface => "self",
            Self::Unresolved => "unresolved",
        }
    }

    /// Read a locus back from its seed slug.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        [
            Self::Workspace,
            Self::Web,
            Self::Dialogue,
            Self::SelfSurface,
            Self::Unresolved,
        ]
        .into_iter()
        .find(|locus| locus.slug() == slug)
    }
}

/// One row of `data/seed/capability-routing.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRow {
    pub object: ObjectType,
    pub act: Act,
    pub locus: Locus,
    /// The preferred capability, e.g. `web_fetch`.
    pub capability: String,
    /// The capability used when the preferred one is not advertised (#758's
    /// specialized-first, bash-as-universal-fallback policy, stated as data).
    pub fallback: Option<String>,
    /// The written reason the row exists.
    pub because: String,
}

/// The four exhaustive outcomes (plan 10 Architecture 5). Every path ends in
/// one of them, so "silent UNKNOWN" is unreachable by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingOutcome {
    /// The triple has a row and the capability's tool is advertised.
    Routed { capability: String },
    /// The preferred capability is not advertised and the row names a fallback.
    Lowered {
        preferred: String,
        capability: String,
    },
    /// The triple has a row but neither the capability nor its fallback is
    /// advertised: the answer names what was needed and what was missing.
    HonestGap { needed: String, missing: String },
    /// The triple has no row, or two rows tie: one question naming both readings.
    Ask { readings: Vec<String> },
}

/// The fully-grounded routing decision for one request.
///
/// Keeping the three axes beside the outcome lets every execution surface use
/// the same decision *and* record why it was selected.  In particular, the
/// symbolic solver must not re-derive a handler from the prompt after the
/// agentic planner has already derived a capability from these axes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    /// Highest-ranked object whose table row resolved.
    pub object: ObjectType,
    /// Most-specific evidenced act whose table row resolved.
    pub act: Act,
    /// The derived effect locus for `object`.
    pub locus: Locus,
    /// Capability availability after resolving the selected table row.
    pub outcome: RoutingOutcome,
}

/// Whether any surface of `role` occurs in `prompt`.
///
/// Both the token-bounded and the raw-substring readings are consulted, because
/// the roles this module shares with older recognisers record some surfaces as
/// whole phrases and some as inflectable stems.
fn evidences(role: &str, normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(role, normalized) || lexicon.mentions_role_raw(role, normalized)
}

/// How *specific* the evidence for `role` in `normalized` is: the character
/// length of the longest surface of the role the prompt carries, or zero.
///
/// A longer surface is a more specific observation about the same sentence.
/// "echo" is evidence of composing; "echo the contents of" is evidence of
/// retrieving, and it is the same four letters plus fifteen more, so it is the
/// reading the sentence actually supports. Without this, act resolution would
/// depend on the order the acts happen to be declared in, which is exactly the
/// declaration-order dependence plan 10 refuses in the table itself.
fn evidence_strength(role: &str, normalized: &str) -> usize {
    seed::lexicon()
        .words_for_role(role)
        .into_iter()
        .filter(|word| normalized.contains(word.as_str()))
        .map(|word| word.chars().count())
        .max()
        .unwrap_or(0)
}

/// Whether a token parses as an absolute URL.
fn is_url(token: &str) -> bool {
    let token = token.trim_matches(|character: char| !character.is_alphanumeric());
    token
        .split_once("://")
        .is_some_and(|(scheme, rest)| matches!(scheme, "http" | "https") && !rest.is_empty())
}

/// Whether a token reads as a workspace path in this request.
///
/// A dotted API member and a filename have the same punctuation. A
/// lower-to-upper boundary in the left side (`TypeName.member`) is structural
/// evidence for a qualified symbol, unless the surrounding request explicitly
/// places that token in a filesystem scope. This prevents a documentation
/// subject from becoming a read-file request without maintaining an extension
/// allowlist in Rust.
fn is_path(token: &str, normalized: &str) -> bool {
    let token = token.trim_matches(|character: char| matches!(character, ',' | ';' | '"' | '\''));
    if is_url(token) {
        return false;
    }
    if token.contains('/') || token.contains('\\') {
        return true;
    }
    token.rsplit_once('.').is_some_and(|(stem, extension)| {
        !stem.is_empty()
            && (1..=5).contains(&extension.chars().count())
            && extension.chars().all(char::is_alphanumeric)
            && extension.chars().any(char::is_alphabetic)
            && !is_web_host_suffix(extension)
            && (!looks_like_qualified_member(stem, extension) || has_workspace_scope(normalized))
    })
}

/// A dotted identifier whose left side carries a type/module-style word
/// boundary and whose right side is an identifier, rather than an extension.
fn looks_like_qualified_member(stem: &str, member: &str) -> bool {
    if stem.contains('.')
        || !stem
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
        || !member
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
    {
        return false;
    }
    stem.chars()
        .zip(stem.chars().skip(1))
        .any(|(left, right)| left.is_lowercase() && right.is_uppercase())
}

/// Whether `extension` is a registrable domain suffix rather than a file
/// extension, so `amazon.in` is a site and `main.rs` is a file.
fn is_web_host_suffix(extension: &str) -> bool {
    let extension = extension.to_lowercase();
    seed::lexicon()
        .words_for_role(ROLE_CAPABILITY_WEB_HOST_SUFFIX)
        .contains(&extension)
}

/// Whether the prompt carries a clock time: digits, a colon, digits.
fn has_clock_time(prompt: &str) -> bool {
    let characters: Vec<char> = prompt.chars().collect();
    characters.iter().enumerate().any(|(index, character)| {
        *character == ':'
            && index > 0
            && characters[index - 1].is_ascii_digit()
            && characters.get(index + 1).is_some_and(char::is_ascii_digit)
    })
}

/// Whether the prompt carries a balanced quoted span of more than one token.
fn has_quoted_span(prompt: &str) -> bool {
    for delimiter in ['"', '\u{201c}', '\u{00ab}'] {
        let closing = match delimiter {
            '\u{201c}' => '\u{201d}',
            '\u{00ab}' => '\u{00bb}',
            other => other,
        };
        if let Some(open) = prompt.find(delimiter) {
            let rest = &prompt[open + delimiter.len_utf8()..];
            if let Some(close) = rest.find(closing)
                && rest[..close].split_whitespace().count() > 1
            {
                return true;
            }
        }
    }
    false
}

/// Whether the request carries *explicit content*: a quoted span, or a seeded
/// content introducer (`containing`, `с текстом`, `内容为`, `con el texto`).
///
/// Quotation marks are one way a person hands over a literal; naming the
/// content with the phrase the language uses for it is the other, and a request
/// that carries one carries the same object as a request that carries the other.
fn has_explicit_content(prompt: &str, normalized: &str) -> bool {
    has_quoted_span(prompt) || evidences(ROLE_CAPABILITY_CONTENT_INTRODUCER, normalized)
}

/// Whether the prompt names a filesystem container: the desktop, the home
/// directory, the current directory, or the folder/file/path nouns themselves.
/// A possessive is not consulted, which is the single change that makes
/// `on my desktop` and `on desktop` one request.
fn has_container_scope(normalized: &str) -> bool {
    [
        ROLE_LOCAL_PATH_SCOPE_DESKTOP,
        ROLE_LOCAL_PATH_SCOPE_HOME,
        ROLE_LOCAL_PATH_SCOPE_CURRENT,
        ROLE_CAPABILITY_CONTAINER_SCOPE,
    ]
    .iter()
    .any(|role| evidences(role, normalized))
}

/// Whether the prompt scopes its effect to the machine the task is being done
/// on. Wider than [`has_container_scope`]: the code, the repository and the
/// project are the workspace without being a directory the request names.
fn has_workspace_scope(normalized: &str) -> bool {
    has_container_scope(normalized) || evidences(ROLE_CAPABILITY_WORKSPACE_SCOPE, normalized)
}

/// Whether the prompt scopes its effect to the open web, either by naming it or
/// by asking for what is true *now*, which nothing but a live source can answer
/// (plan 10 leaf 12's `freshness: live` qualifier).
fn has_web_scope(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_WEB_SCOPE, normalized) || is_freshness_live(normalized)
}

/// Whether the request names the open web itself -- the web, the internet, a
/// search engine, or what is true right now.
///
/// The planner asks this before letting the table answer a bare term at the end
/// of its cascade: at that position every route that reads the conversation and
/// the workspace has already declined, and a request that never mentioned the
/// open web is one the symbolic engine should still get its turn at.
#[must_use]
pub fn names_open_web(prompt: &str) -> bool {
    has_web_scope(&normalize_prompt(prompt))
}

/// The `freshness: live` qualifier: the request is about what is true at the
/// moment it is asked, so a stored answer cannot satisfy it (issue #720).
#[must_use]
pub fn is_freshness_live(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_FRESHNESS_LIVE, normalized)
}

/// Whether the subject is a piece of the assistant's own surface.
fn is_self_surface(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_SELF_SURFACE_NOUN, normalized)
        || evidences(ROLE_ASSISTANT_MECHANISM_INQUIRY, normalized)
        || is_prior_turn_reference(normalized)
}

/// Whether the object of the request is the assistant's *previous turn*: the
/// closed class of comprehension-failure surfaces (issue #721).
///
/// This is the one class with no structural signal at all -- "that went over my
/// head" names nothing the characters can be read for -- so it is grounded in a
/// seed role in five languages rather than in a Rust branch, and none of the
/// three reported strings is among its surfaces.
#[must_use]
pub fn is_prior_turn_reference(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_PRIOR_TURN_REFERENCE, normalized)
}

/// The literal content a request hands over, and the path it hands it to.
///
/// Both halves are read from the same two sources the `quoted_content` object
/// is derived from -- a balanced quoted span, or a seeded content introducer --
/// so the planner writes what the table said was there rather than guessing
/// again with a second rule.
#[must_use]
pub fn explicit_content(prompt: &str) -> Option<String> {
    if let Some(span) = quoted_span(prompt) {
        return Some(span);
    }
    let characters: Vec<char> = prompt.chars().collect();
    let lowered: Vec<char> = prompt.chars().flat_map(char::to_lowercase).collect();
    if lowered.len() != characters.len() {
        // A case fold that changes the character count would misplace every
        // offset below, so the request is left to the planner's own extractor.
        return None;
    }
    let mut best: Option<usize> = None;
    for surface in seed::lexicon().words_for_role(ROLE_CAPABILITY_CONTENT_INTRODUCER) {
        let needle: Vec<char> = surface.chars().collect();
        if needle.is_empty() || needle.len() > lowered.len() {
            continue;
        }
        for start in 0..=lowered.len() - needle.len() {
            if lowered[start..start + needle.len()] == needle[..] {
                let end = start + needle.len();
                if best.is_none_or(|current| end > current) {
                    best = Some(end);
                }
            }
        }
    }
    let Some(end) = best else {
        return assigned_content(prompt);
    };
    let tail: String = characters[end..].iter().collect();
    let tail = tail.trim().trim_matches(|character: char| {
        matches!(
            character,
            ':' | '"' | '\u{201c}' | '\u{201d}' | '.' | '\u{3002}'
        )
    });
    (!tail.trim().is_empty()).then(|| tail.trim().to_owned())
}

/// The content of a request that names its destination first: the text after
/// the assignment connector that follows the destination path.
///
/// "set the contents of note.txt to hello" and "pon el contenido de note.txt en
/// hello" are the same request, and neither quotes its content nor introduces
/// it. The connector is only read *after* the path, because on its own it is
/// far too common a word to be evidence of anything.
fn assigned_content(prompt: &str) -> Option<String> {
    let path = first_path(prompt)?;
    let after = prompt.split_once(path.as_str()).map(|(_, tail)| tail)?;
    let lowered = after.to_lowercase();
    let mut best: Option<usize> = None;
    for surface in seed::lexicon().words_for_role(ROLE_CAPABILITY_CONTENT_ASSIGNMENT) {
        if let Some(position) = lowered.find(surface.as_str()) {
            let end = position + surface.len();
            if best.is_none_or(|current| end < current) && after.is_char_boundary(end) {
                best = Some(end);
            }
        }
    }
    let content = after[best?..].trim().trim_matches('"');
    (!content.is_empty()).then(|| content.to_owned())
}

/// The first balanced quoted span of more than one token, if any.
fn quoted_span(prompt: &str) -> Option<String> {
    for delimiter in ['"', '\u{201c}', '\u{00ab}'] {
        let closing = match delimiter {
            '\u{201c}' => '\u{201d}',
            '\u{00ab}' => '\u{00bb}',
            other => other,
        };
        if let Some(open) = prompt.find(delimiter) {
            let rest = &prompt[open + delimiter.len_utf8()..];
            if let Some(close) = rest.find(closing) {
                let inner = &rest[..close];
                if inner.split_whitespace().count() > 1 {
                    return Some(inner.to_owned());
                }
            }
        }
    }
    None
}

/// The first token of `prompt` that reads as a workspace path.
#[must_use]
pub fn first_path(prompt: &str) -> Option<String> {
    let normalized = normalize_prompt(prompt);
    prompt
        .split_whitespace()
        .find(|token| is_path(token, &normalized))
        .map(|token| {
            token
                .trim_matches(|character: char| {
                    matches!(character, ',' | ';' | '"' | '\'' | '(' | ')' | '\u{3002}')
                })
                .to_owned()
        })
}

/// The first token of `prompt` that parses as an absolute URL.
#[must_use]
pub fn first_url(prompt: &str) -> Option<String> {
    prompt
        .split_whitespace()
        .find(|token| is_url(token))
        .map(|token| {
            token
                .trim_matches(|character: char| matches!(character, ',' | ';' | '"' | '\'' | '.'))
                .to_owned()
        })
}

/// Whether the shipped table is the routing authority (plan 10 leaves 9 and 11).
///
/// A data field rather than a Rust constant, so turning the table off to compare
/// it against the cue-phrase path it replaces is a seed edit like every other
/// routing change.
#[must_use]
pub fn table_routing_enabled() -> bool {
    let tree = parse_lino(CAPABILITY_ROUTING_LINO);
    for document in &tree.children {
        if document.find_child_value("routing_enabled") == "false" {
            return false;
        }
    }
    true
}

/// Every object the prompt carries, ranked; the table is consulted for the
/// highest-ranked first.
#[must_use]
pub fn object_type(prompt: &str) -> Vec<ObjectType> {
    let normalized = normalize_prompt(prompt);
    let tokens: Vec<&str> = prompt.split_whitespace().collect();
    let mut found: Vec<ObjectType> = Vec::new();
    let note = |object: ObjectType, present: bool, found: &mut Vec<ObjectType>| {
        if present && !found.contains(&object) {
            found.push(object);
        }
    };
    note(
        ObjectType::Url,
        tokens.iter().any(|token| is_url(token)),
        &mut found,
    );
    note(
        ObjectType::Path,
        tokens.iter().any(|token| is_path(token, &normalized)),
        &mut found,
    );
    note(
        ObjectType::SelfSurface,
        is_self_surface(&normalized),
        &mut found,
    );
    note(
        ObjectType::LanguageName,
        evidences(ROLE_TRANSLATION_LANGUAGE, &normalized)
            || evidences(ROLE_CAPABILITY_LANGUAGE_REFERENCE, &normalized),
        &mut found,
    );
    note(
        ObjectType::TimeExpression,
        has_clock_time(prompt)
            || evidences(ROLE_CALENDAR_DAY_REFERENCE, &normalized)
            || evidences(ROLE_CAPABILITY_CLOCK_REFERENCE, &normalized),
        &mut found,
    );
    note(
        ObjectType::QuantityQuestion,
        evidences(ROLE_CAPABILITY_QUANTITY_INTERROGATIVE, &normalized),
        &mut found,
    );
    note(
        ObjectType::QuotedContent,
        has_explicit_content(prompt, &normalized),
        &mut found,
    );
    note(
        ObjectType::PathScope,
        has_container_scope(&normalized),
        &mut found,
    );
    note(
        ObjectType::BareTerm,
        tokens
            .iter()
            .any(|token| token.chars().any(char::is_alphanumeric)),
        &mut found,
    );
    if found.is_empty() {
        return vec![ObjectType::None];
    }
    found.sort_by_key(|object| object.rank());
    found
}

/// Return every act the prompt evidences, most specific first.
///
/// `retrieve` is always last: a request that evidences no narrower act is a retrieval, and saying so
/// is what keeps a verb the list has never seen from ending in an `Ask` that
/// names nothing (plan 10 risk 1).
#[must_use]
pub fn acts(prompt: &str) -> Vec<Act> {
    let normalized = normalize_prompt(prompt);
    if normalized.is_empty() {
        return vec![Act::Unresolved];
    }
    let mut scored: Vec<(usize, usize, Act)> = ACTS_IN_PRECEDENCE
        .into_iter()
        .enumerate()
        .filter_map(|(order, candidate)| {
            let strength = evidence_strength(candidate.role(), &normalized);
            (strength > 0).then_some((strength, order, candidate))
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    let mut ordered: Vec<Act> = scored.into_iter().map(|(_, _, act)| act).collect();
    if !ordered.contains(&Act::Retrieve) {
        ordered.push(Act::Retrieve);
    }
    ordered
}

/// The act the prompt asks for, resolved through `data/seed/meanings-acts.lino`.
#[must_use]
pub fn act(prompt: &str) -> Act {
    acts(prompt).first().copied().unwrap_or(Act::Unresolved)
}

/// Where the effect of acting on `object` lands, given the rest of the prompt.
///
/// The locus is a property of the object and the seed's own scope nouns, never
/// of the verb: the same object in the same scope lands in the same place
/// whatever it is asked to do.
#[must_use]
pub fn locus_of(object: ObjectType, prompt: &str) -> Locus {
    let normalized = normalize_prompt(prompt);
    match object {
        ObjectType::Url => Locus::Web,
        ObjectType::Path | ObjectType::PathScope => Locus::Workspace,
        ObjectType::SelfSurface => {
            if is_prior_turn_reference(&normalized) {
                Locus::Dialogue
            } else {
                Locus::SelfSurface
            }
        }
        ObjectType::LanguageName | ObjectType::None => Locus::Dialogue,
        ObjectType::TimeExpression => {
            if has_workspace_scope(&normalized) {
                Locus::Workspace
            } else {
                Locus::Dialogue
            }
        }
        ObjectType::QuotedContent => {
            if has_workspace_scope(&normalized) {
                Locus::Workspace
            } else if has_web_scope(&normalized) {
                Locus::Web
            } else {
                Locus::Dialogue
            }
        }
        ObjectType::QuantityQuestion | ObjectType::BareTerm => {
            if has_workspace_scope(&normalized) {
                Locus::Workspace
            } else {
                Locus::Web
            }
        }
    }
}

/// Where the effect lands, derived from the highest-ranked object and the scope
/// nouns.
#[must_use]
pub fn locus(prompt: &str) -> Locus {
    let objects = object_type(prompt);
    let highest = objects.first().copied().unwrap_or(ObjectType::None);
    locus_of(highest, prompt)
}

/// The shipped decision table.
#[must_use]
pub fn routing_table() -> Vec<RouteRow> {
    routing_table_from(CAPABILITY_ROUTING_LINO).unwrap_or_default()
}

/// The shipped decision table, embedded so the browser build reads the same data.
pub const CAPABILITY_ROUTING_LINO: &str = include_str!("../data/seed/capability-routing.lino");

/// Parse a routing table document, so a fixture row can change routing with no
/// Rust edit.
///
/// # Errors
/// Returns the offending row and field when a row names an axis value the
/// declared axes do not carry.
pub fn routing_table_from(text: &str) -> Result<Vec<RouteRow>, String> {
    let tree = parse_lino(text);
    let mut rows = Vec::new();
    for document in &tree.children {
        for record in &document.children {
            if record.name != "route" {
                continue;
            }
            let object_slug = record.find_child_value("object");
            let act_slug = record.find_child_value("act");
            let locus_slug = record.find_child_value("locus");
            let object = ObjectType::from_slug(object_slug)
                .ok_or_else(|| format!("route:object:{object_slug}"))?;
            let act = Act::from_slug(act_slug).ok_or_else(|| format!("route:act:{act_slug}"))?;
            let locus =
                Locus::from_slug(locus_slug).ok_or_else(|| format!("route:locus:{locus_slug}"))?;
            let capability = record.find_child_value("capability").to_owned();
            if capability.is_empty() {
                return Err(format!("route:{object_slug}:{act_slug}:capability"));
            }
            let fallback = record.find_child_value("fallback");
            rows.push(RouteRow {
                object,
                act,
                locus,
                capability,
                fallback: (!fallback.is_empty()).then(|| fallback.to_owned()),
                because: record.find_child_value("because").to_owned(),
            });
        }
    }
    Ok(rows)
}

/// Resolve one prompt against the table, given the capabilities the client
/// advertised.
///
/// Objects are consulted highest-ranked first and, within an object, the acts
/// the prompt evidences most specific first. Each object is resolved in the
/// locus *its own* effect lands in, because a prompt carries several objects at
/// once -- "what came out today" names both a day and a term -- and asking
/// where "the prompt" lands would force one answer on both.
#[must_use]
pub fn route(prompt: &str, advertised: &[&str]) -> RoutingOutcome {
    route_decision(prompt, advertised).outcome
}

/// Resolve one prompt and retain the `(object, act, locus)` that selected the
/// outcome.
///
/// [`route`] remains the compact compatibility projection.  Runtime dispatchers
/// use this form so the event log proves that a capability came from the shared
/// table rather than from a second phrase recognizer.
#[must_use]
pub fn route_decision(prompt: &str, advertised: &[&str]) -> RoutingDecision {
    let table = routing_table();
    let objects = object_type(prompt);
    let acts = acts(prompt);
    for object in &objects {
        let locus = locus_of(*object, prompt);
        for act in &acts {
            let outcome = route_with(&table, *object, *act, locus, advertised);
            if !matches!(outcome, RoutingOutcome::Ask { .. }) {
                return RoutingDecision {
                    object: *object,
                    act: *act,
                    locus,
                    outcome,
                };
            }
        }
    }
    let highest = objects.first().copied().unwrap_or(ObjectType::None);
    let act = acts.first().copied().unwrap_or(Act::Unresolved);
    let locus = locus_of(highest, prompt);
    RoutingDecision {
        object: highest,
        act,
        locus,
        outcome: route_with(&table, highest, act, locus, advertised),
    }
}

/// Resolve one triple against a supplied table.
#[must_use]
pub fn route_with(
    table: &[RouteRow],
    object: ObjectType,
    act: Act,
    locus: Locus,
    advertised: &[&str],
) -> RoutingOutcome {
    let matched: Vec<&RouteRow> = table
        .iter()
        .filter(|row| row.object == object && row.act == act && row.locus == locus)
        .collect();
    let [row] = matched.as_slice() else {
        return RoutingOutcome::Ask {
            readings: readings_for(table, object, act, locus, &matched),
        };
    };
    if advertised.contains(&row.capability.as_str()) {
        return RoutingOutcome::Routed {
            capability: row.capability.clone(),
        };
    }
    if let Some(fallback) = &row.fallback
        && advertised.contains(&fallback.as_str())
    {
        return RoutingOutcome::Lowered {
            preferred: row.capability.clone(),
            capability: fallback.clone(),
        };
    }
    RoutingOutcome::HonestGap {
        needed: row.capability.clone(),
        missing: row
            .fallback
            .clone()
            .unwrap_or_else(|| row.capability.clone()),
    }
}

/// The readings an `Ask` names.
///
/// Never fewer than two: a question that names one reading is not a question.
/// The first reading is always the triple that was derived, so the person asked
/// can see what the system thought the request was about; the rest are the
/// nearest rows -- those agreeing on the object, or on the act and the locus.
fn readings_for(
    table: &[RouteRow],
    object: ObjectType,
    act: Act,
    locus: Locus,
    matched: &[&RouteRow],
) -> Vec<String> {
    let mut readings = vec![format!("{}:{}:{}", object.slug(), act.slug(), locus.slug())];
    for row in matched {
        let reading = row.capability.clone();
        if !readings.contains(&reading) {
            readings.push(reading);
        }
    }
    for row in table {
        if row.object != object && !(row.act == act && row.locus == locus) {
            continue;
        }
        let reading = row.capability.clone();
        if !readings.contains(&reading) {
            readings.push(reading);
        }
    }
    if readings.len() < 2 {
        readings.push(DEFAULT_OUTCOME.to_string());
    }
    readings
}

/// The outcome the table declares for a triple no row claims.
const DEFAULT_OUTCOME: &str = "ask";
