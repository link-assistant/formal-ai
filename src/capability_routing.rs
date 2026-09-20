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
    ROLE_CALENDAR_HOUR_REFERENCE, ROLE_CALENDAR_SCHEDULE_ACTION, ROLE_CAPABILITY_ACT_COMPOSE,
    ROLE_CAPABILITY_ACT_DEMONSTRATE, ROLE_CAPABILITY_ACT_ENUMERATE, ROLE_CAPABILITY_ACT_EXPLAIN,
    ROLE_CAPABILITY_ACT_RECORD, ROLE_CAPABILITY_ACT_RETRIEVE, ROLE_CAPABILITY_ACT_SCHEDULE,
    ROLE_CAPABILITY_ACT_TRANSFORM, ROLE_CAPABILITY_CLOCK_REFERENCE,
    ROLE_CAPABILITY_CONTAINER_SCOPE, ROLE_CAPABILITY_CONTENT_ASSIGNMENT,
    ROLE_CAPABILITY_CONTENT_INTRODUCER, ROLE_CAPABILITY_DELEGATION_MARKER,
    ROLE_CAPABILITY_FRESHNESS_LIVE, ROLE_CAPABILITY_LANGUAGE_REFERENCE,
    ROLE_CAPABILITY_PRIOR_TURN_REFERENCE, ROLE_CAPABILITY_QUANTITY_INTERROGATIVE,
    ROLE_CAPABILITY_SELF_SURFACE_NOUN, ROLE_CAPABILITY_TASK_LIST_NOUN,
    ROLE_CAPABILITY_WEB_HOST_SUFFIX, ROLE_CAPABILITY_WEB_SCOPE, ROLE_CAPABILITY_WORKSPACE_SCOPE,
    ROLE_LOCAL_PATH_SCOPE_CURRENT, ROLE_LOCAL_PATH_SCOPE_DESKTOP, ROLE_LOCAL_PATH_SCOPE_HOME,
    ROLE_TRANSLATION_LANGUAGE,
};
use crate::web_engine_core::normalize_prompt;

mod evidence;
pub use evidence::*;

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
    /// A token carrying a wildcard (`*`, `?`, `[`): a pattern, not a file.
    Pattern,
    /// Strictly more than one path-shaped token: a set of files, not one file.
    PathSet,
    /// A balanced quoted span longer than one token.
    QuotedContent,
    /// A clock time, a date, or a weekday/relative-day reference.
    TimeExpression,
    /// A relative period measured in hours — what a fresh-period digest
    /// ("summarise the last few hours, with links") asks about. Kept apart
    /// from `TimeExpression` because weekday questions are engine calendar
    /// reasoning while a period digest needs fresh web events, and the table
    /// routes objects, not spellings.
    RelativePeriod,
    /// A term resolving to a registered language in `data/seed/languages.lino`.
    LanguageName,
    /// An interrogative whose expected answer is a magnitude.
    QuantityQuestion,
    /// The subject is a list of tasks to track (`todo`).
    TaskList,
    /// The work is handed to another agent (`subagent`).
    Delegation,
    /// One or more content tokens that are none of the above.
    BareTerm,
    /// The subject is the assistant's own UI, answer or behaviour.
    SelfSurface,
    /// No content token survives.
    #[default]
    None,
}

/// Every object type, in the order the decision table's axis declares them.
const OBJECT_TYPES: [ObjectType; 15] = [
    ObjectType::Url,
    ObjectType::Path,
    ObjectType::Pattern,
    ObjectType::PathSet,
    ObjectType::PathScope,
    ObjectType::QuotedContent,
    ObjectType::TimeExpression,
    ObjectType::RelativePeriod,
    ObjectType::LanguageName,
    ObjectType::QuantityQuestion,
    ObjectType::TaskList,
    ObjectType::Delegation,
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
            Self::Pattern => "pattern",
            Self::PathSet => "path_set",
            Self::QuotedContent => "quoted_content",
            Self::TimeExpression => "time_expression",
            Self::RelativePeriod => "relative_period",
            Self::LanguageName => "language_name",
            Self::QuantityQuestion => "quantity_question",
            Self::TaskList => "task_list",
            Self::Delegation => "delegation",
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
    /// A parsed URL is the least ambiguous thing a request can name, so it
    /// outranks everything else; a bare term is what is left when nothing more
    /// specific survived, so it ranks last but one.
    ///
    /// A wildcard pattern outranks the path reading of the very same token:
    /// `**/*.rs` parses as a path (it carries a separator) and is nonetheless
    /// not one file anybody can open, so the pattern is the reading the token
    /// supports. A multi-file set outranks any single member for the same
    /// reason -- two paths are a batch, not a document. A magnitude
    /// interrogative outranks a temporal cue on the same sentence ("how tall
    /// does X normally grow" asks for a measurement; "normally" is scenery),
    /// because the interrogative is a closed class and the temporal cue is
    /// open to mere mentions of hours or weeks. The task-list and delegation
    /// subjects are named by their own closed-class nouns, which nothing more
    /// structural outranks.
    const fn rank(self) -> u8 {
        match self {
            Self::Url => 0,
            Self::Pattern => 1,
            Self::PathSet => 2,
            Self::Path => 3,
            Self::SelfSurface => 4,
            Self::LanguageName => 5,
            Self::QuantityQuestion => 6,
            Self::TimeExpression | Self::RelativePeriod => 7,
            Self::TaskList => 8,
            Self::Delegation => 9,
            Self::QuotedContent => 10,
            Self::PathScope => 11,
            Self::BareTerm => 12,
            Self::None => 13,
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
        ObjectType::Pattern,
        tokens.iter().any(|token| is_pattern_token(token)),
        &mut found,
    );
    note(
        ObjectType::PathSet,
        path_token_count(prompt, &normalized) > 1,
        &mut found,
    );
    note(
        ObjectType::SelfSurface,
        is_self_surface(&normalized),
        &mut found,
    );
    // A language reference is the *object* only when an act makes the language
    // itself the goal -- a demonstration ("say something in Russian") or a
    // transform ("translate this to Russian"). Otherwise it is a response-
    // language marker on some other subject: "tell me about Telegram Ads in
    // Russian" is about Telegram Ads, in Russian (plan 10 leaf 15, issue
    // #724); the marker role keeps the binding and the object stays the term.
    // An obligation on the answer's form hides behind the same act evidence --
    // "answer" evidences `demonstrate` exactly as "say" does -- but the
    // concept question is what tells them apart: "What is a фуфломицин?
    // Answer in English." is a question about a term whose answer happens to
    // be due in English, and routing the language stole the subject from the
    // term the #840 ladder replays (plan 10 leaf 21).
    note(
        ObjectType::LanguageName,
        (evidences(ROLE_TRANSLATION_LANGUAGE, &normalized)
            || evidences(ROLE_CAPABILITY_LANGUAGE_REFERENCE, &normalized))
            && (evidences(ROLE_CAPABILITY_ACT_DEMONSTRATE, &normalized)
                || evidences(ROLE_CAPABILITY_ACT_TRANSFORM, &normalized))
            && !is_response_language_obligation(prompt, &normalized),
        &mut found,
    );
    note(
        ObjectType::TimeExpression,
        has_clock_time(prompt)
            || evidences(ROLE_CALENDAR_DAY_REFERENCE, &normalized)
            || evidences(ROLE_CAPABILITY_CLOCK_REFERENCE, &normalized),
        &mut found,
    );
    // A relative period is not a calendar question: it never names a weekday
    // or a date, and it is not anchored by a clock time — it is "the last few
    // hours" asking for what changed, which the fresh-web digest answers.
    // Scheduling an event at an hour stays out: the schedule action keeps it
    // with the calendar create route. The hour evidence is read token-bounded:
    // the role's surfaces are whole words in every language, and the raw
    // substring reading turns Spanish "ahora" (now) into "hora" (hour),
    // making a dialogue question about "temas ... hasta ahora" a web digest
    // (issue #1138 family suite, es_dialogue_state_query_05).
    note(
        ObjectType::RelativePeriod,
        crate::seed::lexicon().mentions_role(ROLE_CALENDAR_HOUR_REFERENCE, &normalized)
            && !evidences(ROLE_CALENDAR_DAY_REFERENCE, &normalized)
            && !has_clock_time(prompt)
            && !evidences(ROLE_CAPABILITY_CLOCK_REFERENCE, &normalized)
            && !evidences(ROLE_CALENDAR_SCHEDULE_ACTION, &normalized),
        &mut found,
    );
    note(
        ObjectType::QuantityQuestion,
        evidences(ROLE_CAPABILITY_QUANTITY_INTERROGATIVE, &normalized),
        &mut found,
    );
    note(
        ObjectType::TaskList,
        evidences(ROLE_CAPABILITY_TASK_LIST_NOUN, &normalized),
        &mut found,
    );
    note(
        ObjectType::Delegation,
        evidences(ROLE_CAPABILITY_DELEGATION_MARKER, &normalized),
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

/// Whether the request itself evidences the retrieve act, through a verb
/// surface of `data/seed/meanings-acts.lino`, rather than receiving `retrieve`
/// as the default appended to every request.
///
/// `acts` cannot make this distinction -- it always ends in `retrieve` -- but
/// the planner's container rule turns on it: a container the request only
/// *mentioned* is not a request to act on it (issue #907), while "busca X en
/// mi escritorio" and "… में खोजिए" ask a search of it in the seed's own
/// words (the #840 ladder's hi/es folder nodes, plan 10 leaf 21).
#[must_use]
pub fn evidences_retrieve_act(prompt: &str) -> bool {
    evidence_strength(Act::Retrieve.role(), &normalize_prompt(prompt)) > 0
}

/// Whether a language mention is a response-language obligation on some other
/// subject, rather than the goal itself.
///
/// A concept question in the prompt is the test: the question's term is the
/// subject and the language names the form of its answer ("What is a
/// фуфломицин? Answer in English."), where a demonstration with no question
/// ("say something in Russian") makes the language the whole goal. Both the
/// question shape and the marker role are read from the seed -- the concept
/// slot forms the concept lookup handler resolves, and
/// `ROLE_RESPONSE_LANGUAGE_MARKER` for the obligation.
fn is_response_language_obligation(prompt: &str, normalized: &str) -> bool {
    crate::concepts::extract_concept_query(prompt).is_some()
        && crate::translation::detect_response_language(normalized).is_some()
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
        ObjectType::Path | ObjectType::PathScope | ObjectType::Pattern | ObjectType::PathSet => {
            Locus::Workspace
        }
        ObjectType::TaskList
        | ObjectType::Delegation
        | ObjectType::LanguageName
        | ObjectType::None => Locus::Dialogue,
        ObjectType::SelfSurface => {
            if is_prior_turn_reference(&normalized) {
                Locus::Dialogue
            } else {
                Locus::SelfSurface
            }
        }
        ObjectType::TimeExpression | ObjectType::RelativePeriod => {
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

/// Whether one of the advertised tool names provides the capability or
/// fallback `slug` — literally, or through the shared alias registry
/// (`data/seed/agentic-tool-capabilities.lino`), so a client that advertises
/// `websearch` still satisfies the table's `web_search` and one that
/// advertises `bash` still satisfies the `shell` fallback (#758's expected
/// item 1: the capability resolves to whichever alias the client advertised).
fn advertised_provides(slug: &str, advertised: &[&str]) -> bool {
    advertised.iter().any(|name| {
        name.eq_ignore_ascii_case(slug)
            || crate::seed::agentic_tool_capabilities()
                .iter()
                .any(|entry| {
                    let names_slug =
                        entry.id == slug || entry.aliases.iter().any(|alias| alias == slug);
                    let names_tool = entry.id.eq_ignore_ascii_case(name)
                        || entry
                            .aliases
                            .iter()
                            .any(|alias| alias.eq_ignore_ascii_case(name));
                    names_slug && names_tool
                })
    })
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
    if advertised_provides(&row.capability, advertised) {
        return RoutingOutcome::Routed {
            capability: row.capability.clone(),
        };
    }
    if let Some(fallback) = &row.fallback
        && advertised_provides(fallback, advertised)
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
