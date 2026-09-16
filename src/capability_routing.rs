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
    ROLE_CAPABILITY_ACT_SCHEDULE, ROLE_CAPABILITY_ACT_TRANSFORM,
    ROLE_CAPABILITY_QUANTITY_INTERROGATIVE, ROLE_CAPABILITY_SELF_SURFACE_NOUN,
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
        OBJECT_TYPES.into_iter().find(|object| object.slug() == slug)
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

/// Whether any surface of `role` occurs in `prompt`.
///
/// Both the token-bounded and the raw-substring readings are consulted, because
/// the roles this module shares with older recognisers record some surfaces as
/// whole phrases and some as inflectable stems.
fn evidences(role: &str, normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(role, normalized) || lexicon.mentions_role_raw(role, normalized)
}

/// Whether a token parses as an absolute URL.
fn is_url(token: &str) -> bool {
    let token = token.trim_matches(|character: char| !character.is_alphanumeric());
    token.starts_with("http://") || token.starts_with("https://")
}

/// Whether a token reads as a workspace path: it carries a separator, or it is
/// a `name.extension` pair with a short alphabetic extension.
fn is_path(token: &str) -> bool {
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
            && extension.chars().all(|character| character.is_alphanumeric())
            && extension.chars().any(char::is_alphabetic)
    })
}

/// Whether the prompt carries a clock time: digits, a colon, digits.
fn has_clock_time(prompt: &str) -> bool {
    let characters: Vec<char> = prompt.chars().collect();
    characters.iter().enumerate().any(|(index, character)| {
        *character == ':'
            && index > 0
            && characters[index - 1].is_ascii_digit()
            && characters
                .get(index + 1)
                .is_some_and(char::is_ascii_digit)
    })
}

/// Whether the prompt carries a balanced quoted span of more than one token.
fn has_quoted_content(prompt: &str) -> bool {
    for delimiter in ['"', '\u{201c}', '\u{00ab}'] {
        let closing = match delimiter {
            '\u{201c}' => '\u{201d}',
            '\u{00ab}' => '\u{00bb}',
            other => other,
        };
        if let Some(open) = prompt.find(delimiter) {
            let rest = &prompt[open + delimiter.len_utf8()..];
            if let Some(close) = rest.find(closing) {
                if rest[..close].split_whitespace().count() > 1 {
                    return true;
                }
            }
        }
    }
    false
}

/// Whether the prompt names a filesystem scope: the desktop, the home directory,
/// or the current directory. A possessive is not consulted, which is the single
/// change that makes `on my desktop` and `on desktop` one request.
fn has_path_scope(normalized: &str) -> bool {
    [
        ROLE_LOCAL_PATH_SCOPE_DESKTOP,
        ROLE_LOCAL_PATH_SCOPE_HOME,
        ROLE_LOCAL_PATH_SCOPE_CURRENT,
    ]
    .iter()
    .any(|role| evidences(role, normalized))
}

/// Whether the subject is a piece of the assistant's own surface.
fn is_self_surface(normalized: &str) -> bool {
    evidences(ROLE_CAPABILITY_SELF_SURFACE_NOUN, normalized)
        || evidences(ROLE_ASSISTANT_MECHANISM_INQUIRY, normalized)
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
        tokens.iter().any(|token| is_path(token)),
        &mut found,
    );
    note(ObjectType::SelfSurface, is_self_surface(&normalized), &mut found);
    note(
        ObjectType::LanguageName,
        evidences(ROLE_TRANSLATION_LANGUAGE, &normalized),
        &mut found,
    );
    note(
        ObjectType::TimeExpression,
        has_clock_time(prompt) || evidences(ROLE_CALENDAR_DAY_REFERENCE, &normalized),
        &mut found,
    );
    note(
        ObjectType::QuantityQuestion,
        evidences(ROLE_CAPABILITY_QUANTITY_INTERROGATIVE, &normalized),
        &mut found,
    );
    note(
        ObjectType::QuotedContent,
        has_quoted_content(prompt),
        &mut found,
    );
    note(ObjectType::PathScope, has_path_scope(&normalized), &mut found);
    note(
        ObjectType::BareTerm,
        tokens.iter().any(|token| {
            token
                .chars()
                .any(|character| character.is_alphanumeric())
        }),
        &mut found,
    );
    if found.is_empty() {
        return vec![ObjectType::None];
    }
    found.sort_by_key(|object| object.rank());
    found
}

/// The act the prompt asks for, resolved through `data/seed/meanings-acts.lino`.
#[must_use]
pub fn act(prompt: &str) -> Act {
    let normalized = normalize_prompt(prompt);
    ACTS_IN_PRECEDENCE
        .into_iter()
        .find(|candidate| evidences(candidate.role(), &normalized))
        .unwrap_or(Act::Unresolved)
}

/// Where the effect lands, derived from the objects and the scope nouns.
#[must_use]
pub fn locus(prompt: &str) -> Locus {
    let normalized = normalize_prompt(prompt);
    let objects = object_type(prompt);
    if objects.contains(&ObjectType::SelfSurface) {
        return Locus::SelfSurface;
    }
    if has_path_scope(&normalized) || objects.contains(&ObjectType::Path) {
        return Locus::Workspace;
    }
    if objects.contains(&ObjectType::Url) {
        return Locus::Web;
    }
    Locus::Unresolved
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
            let act =
                Act::from_slug(act_slug).ok_or_else(|| format!("route:act:{act_slug}"))?;
            let locus = Locus::from_slug(locus_slug)
                .ok_or_else(|| format!("route:locus:{locus_slug}"))?;
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
#[must_use]
pub fn route(prompt: &str, advertised: &[&str]) -> RoutingOutcome {
    let table = routing_table();
    let objects = object_type(prompt);
    let act = act(prompt);
    let locus = locus(prompt);
    for object in &objects {
        let outcome = route_with(&table, *object, act, locus, advertised);
        if !matches!(outcome, RoutingOutcome::Ask { .. }) {
            return outcome;
        }
    }
    let highest = objects.first().copied().unwrap_or(ObjectType::None);
    route_with(&table, highest, act, locus, advertised)
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
    if let Some(fallback) = &row.fallback {
        if advertised.contains(&fallback.as_str()) {
            return RoutingOutcome::Lowered {
                preferred: row.capability.clone(),
                capability: fallback.clone(),
            };
        }
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
    let mut readings = vec![format!(
        "{}:{}:{}",
        object.slug(),
        act.slug(),
        locus.slug()
    )];
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
