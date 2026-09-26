//! Any-direction translation through the meta language as the single pivot
//! (issue #1138, plan 16 L2).
//!
//! `translate(X → Y)` is always `extract(X → meta)` followed by
//! `render(meta → Y)`; the dispatcher holds no per-pair code. A direction
//! opens by carrying a leg, and a leg that is not materialized yet reports
//! the plan-16 leaf that owes it — the same honesty the capability table
//! practices: a gap is stated, never papered over with a silent no-op.
//!
//! Live legs: `rust → meta` (the self-AST signature projection) and, since
//! plan 16 L2, the whole ES quadrant — `js ↔ meta`, `ts ↔ meta`,
//! `js → ts`, `ts → js` — carried by the token-tree pivot in
//! [`crate::es_meta`] under the seed's projection rules. Every other
//! direction runs through the pivot in principle and is owed by exactly one
//! leaf: the dogfood back-translation into Rust by L3, and the
//! CST-equal round trip including the full `meta → rust` inverse by L5.
//! `L0` marks a same-root non-direction: no leaf owes it because it is not
//! a translation, and callers reject it before listing.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::es_meta::{ProjectionTarget, Refusal, SourceLanguage, render_document, render_source};

/// The four trees the pivot connects; `Meta` is the pivot itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceRoot {
    Rust,
    JavaScript,
    TypeScript,
    Meta,
}

impl SourceRoot {
    /// Parse the CLI spelling of a root, accepting the long language name
    /// beside the directory spelling.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "js" | "javascript" => Some(Self::JavaScript),
            "ts" | "typescript" => Some(Self::TypeScript),
            "meta" | "lino" => Some(Self::Meta),
            _ => None,
        }
    }

    /// The canonical CLI spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::JavaScript => "js",
            Self::TypeScript => "ts",
            Self::Meta => "meta",
        }
    }
}

/// What one translate call produced.
///
/// The rendered target, the honest gap naming the plan-16 leaf that owes the
/// missing half of the leg, the seed's refusal of a construct the target
/// does not accept, or the reason the source could not be normalized at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationOutcome {
    /// The pivot carried the source into the target language; `carried`
    /// counts what crossed (tokens for the ES legs).
    Rendered { target: String, carried: usize },
    /// The seed's projection rules refuse named constructs; nothing was
    /// produced, because carrying the rest would silently change the tree.
    Refused { refusals: Vec<Refusal> },
    /// The leg is registered but not materialized; the leaf owes it.
    Pending { plan_leaf: &'static str },
    /// The source could not be normalized into the pivot.
    Invalid { reason: String },
}

/// The plan-16 leaf that owes a leg, or `None` when the leg is live.
///
/// Live: `rust → meta` (self-AST census), the ES quadrant — `js ↔ meta`,
/// `ts ↔ meta`, `js → ts`, `ts → js` (plan 16 L2's token-tree pivot) —
/// `meta → rust` (plan 16 L7's network serialization, rendered back with
/// `reconstruct_text`), and the grammar projection legs both ways (plan
/// 16 L8: every corpus kind is ruled, spliced or declared no-form, so
/// the walk renders or refuses by name — never guesses; js/ts → rust
/// since the no-form batch, rust → js/ts since the type-position tail
/// closed). `L0` marks a same-root non-direction. The seed's
/// `root_projection` rows mirror this table; [`root_projections`] reads
/// them, and the round-trip projection test pins the two surfaces in
/// agreement.
#[must_use]
pub const fn pending_leg(from: SourceRoot, to: SourceRoot) -> Option<&'static str> {
    match (from, to) {
        (SourceRoot::Rust | SourceRoot::JavaScript | SourceRoot::TypeScript, SourceRoot::Meta)
        | (SourceRoot::Meta, SourceRoot::JavaScript | SourceRoot::TypeScript | SourceRoot::Rust)
        | (SourceRoot::JavaScript, SourceRoot::TypeScript)
        | (SourceRoot::TypeScript, SourceRoot::JavaScript)
        | (SourceRoot::JavaScript | SourceRoot::TypeScript, SourceRoot::Rust)
        | (SourceRoot::Rust, SourceRoot::JavaScript | SourceRoot::TypeScript) => None,
        (SourceRoot::Rust, SourceRoot::Rust)
        | (SourceRoot::JavaScript, SourceRoot::JavaScript)
        | (SourceRoot::TypeScript, SourceRoot::TypeScript)
        | (SourceRoot::Meta, SourceRoot::Meta) => Some("L0"),
    }
}

/// Translate one document from `from` into `to` through the meta pivot.
///
/// `display_path` names the source for the rendered document's target header,
/// and `source` is its text. Calling this on a pending leg returns the gap
/// rather than failing, so a caller can list and translate through one shape.
#[must_use]
pub fn translate(
    from: SourceRoot,
    to: SourceRoot,
    display_path: &str,
    source: &str,
) -> TranslationOutcome {
    if let Some(plan_leaf) = pending_leg(from, to) {
        return TranslationOutcome::Pending { plan_leaf };
    }
    match (from, to) {
        (SourceRoot::Rust, SourceRoot::Meta) => TranslationOutcome::Rendered {
            target: crate::agentic_coding::self_ast::render_ast_document(display_path, source),
            carried: 0,
        },
        (SourceRoot::Meta, SourceRoot::Rust) => {
            match crate::agentic_coding::self_ast::render_network_source(source) {
                Ok(target) => TranslationOutcome::Rendered { target, carried: 0 },
                Err(reason) => TranslationOutcome::Invalid { reason },
            }
        }
        (SourceRoot::JavaScript | SourceRoot::TypeScript, SourceRoot::Meta) => {
            let language = es_language(from);
            match crate::es_meta::extract(display_path, language, source) {
                Ok(document) => TranslationOutcome::Rendered {
                    target: render_document(&document),
                    carried: document.token_count,
                },
                Err(error) => TranslationOutcome::Invalid {
                    reason: error.to_string(),
                },
            }
        }
        (SourceRoot::JavaScript, SourceRoot::TypeScript)
        | (SourceRoot::TypeScript, SourceRoot::JavaScript) => {
            let language = es_language(from);
            match crate::es_meta::extract(display_path, language, source) {
                Ok(document) => es_render(&document, projection_target(to)),
                Err(error) => TranslationOutcome::Invalid {
                    reason: error.to_string(),
                },
            }
        }
        (SourceRoot::Meta, SourceRoot::JavaScript | SourceRoot::TypeScript) => {
            match crate::es_meta::parse_document(source) {
                Ok(document) => es_render(&document, projection_target(to)),
                Err(error) => TranslationOutcome::Invalid {
                    reason: error.to_string(),
                },
            }
        }
        // The L8 grammar projection legs: the seed's rule table answers
        // for every corpus kind, so the walk renders rust or refuses the
        // constructs that have no rust spelling — objects, classes,
        // imports, the dynamic family — by name.
        (SourceRoot::JavaScript | SourceRoot::TypeScript, SourceRoot::Rust) => {
            match crate::rust_projection::project(es_grammar_label(from), "rust", source) {
                crate::rust_projection::ProjectionOutcome::Rendered { source, .. } => {
                    TranslationOutcome::Rendered {
                        target: source,
                        carried: 0,
                    }
                }
                crate::rust_projection::ProjectionOutcome::Refused { refusals } => {
                    TranslationOutcome::Refused { refusals }
                }
            }
        }
        // The mirror legs: the same rule table answers for the rust
        // corpus, so the walk renders the plain subset and refuses the
        // constructs with no js/ts spelling — enums, impls, traits, the
        // dyn family, labels — by name.
        (SourceRoot::Rust, SourceRoot::JavaScript | SourceRoot::TypeScript) => {
            match crate::rust_projection::project("rust", es_grammar_label(to), source) {
                crate::rust_projection::ProjectionOutcome::Rendered { source, .. } => {
                    TranslationOutcome::Rendered {
                        target: source,
                        carried: 0,
                    }
                }
                crate::rust_projection::ProjectionOutcome::Refused { refusals } => {
                    TranslationOutcome::Refused { refusals }
                }
            }
        }
        _ => TranslationOutcome::Pending {
            plan_leaf: pending_leg(from, to).unwrap_or("L2"),
        },
    }
}

const fn es_language(root: SourceRoot) -> SourceLanguage {
    match root {
        SourceRoot::TypeScript => SourceLanguage::TypeScript,
        SourceRoot::Rust | SourceRoot::JavaScript | SourceRoot::Meta => SourceLanguage::JavaScript,
    }
}

/// The grammar label the L8 projection parses an ES root as: the label
/// [`crate::grammar_kinds::CORPORA`] declares, not the pivot's language.
const fn es_grammar_label(root: SourceRoot) -> &'static str {
    match root {
        SourceRoot::TypeScript => "typescript",
        SourceRoot::Rust | SourceRoot::JavaScript | SourceRoot::Meta => "javascript",
    }
}

const fn projection_target(root: SourceRoot) -> ProjectionTarget {
    match root {
        SourceRoot::TypeScript => ProjectionTarget::TypeScript,
        SourceRoot::Rust | SourceRoot::JavaScript | SourceRoot::Meta => {
            ProjectionTarget::JavaScript
        }
    }
}

fn es_render(
    document: &crate::es_meta::PivotDocument,
    target: ProjectionTarget,
) -> TranslationOutcome {
    let rendered = render_source(document, target);
    match rendered.output {
        Some(target_text) => TranslationOutcome::Rendered {
            target: target_text,
            carried: rendered.report.carried,
        },
        None => TranslationOutcome::Refused {
            refusals: rendered.report.refused,
        },
    }
}

/// Every ordered pair of distinct roots with its leg status, in a stable
/// order, so the CLI can state what the cycle covers and what it still owes.
#[must_use]
pub fn directions() -> Vec<(SourceRoot, SourceRoot, Option<&'static str>)> {
    let roots = [
        SourceRoot::Rust,
        SourceRoot::JavaScript,
        SourceRoot::TypeScript,
        SourceRoot::Meta,
    ];
    let mut listed = Vec::new();
    for from in roots {
        for to in roots {
            if from != to {
                listed.push((from, to, pending_leg(from, to)));
            }
        }
    }
    listed
}

/// A declared root projection from the committed seed (plan 16 L5): the
/// data-side mirror of the leg table, so the seed — not code — answers
/// "which projections exist and at what fidelity".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootProjection {
    /// The leg's from-root.
    pub from: SourceRoot,
    /// The leg's to-root.
    pub to: SourceRoot,
    /// The pivot fidelity the projection carries (`signature`,
    /// `token_tree`), spelled as the seed spells it.
    pub fidelity: String,
    /// The plan-16 leaf that owes this leg; `None` when it is live.
    pub owed_by: Option<String>,
}

/// The declared root projections from the committed projection seed.
///
/// Rows whose roots or status the table does not know are dropped, so the
/// agreement check (every directed pair exactly one row, live ⇔
/// [`pending_leg`] is `None`) is what surfaces seed drift — a silently
/// skipped row is a missing row there.
#[must_use]
pub fn root_projections() -> Vec<RootProjection> {
    crate::es_meta::projection_rules()
        .roots
        .iter()
        .filter_map(|rule| {
            let from = SourceRoot::parse(&rule.from)?;
            let to = SourceRoot::parse(&rule.to)?;
            let owed_by = if rule.status == "live" {
                None
            } else {
                Some(
                    rule.status
                        .strip_prefix("owed_by ")
                        .filter(|leaf| leaf.starts_with('L'))
                        .map(str::to_string)?,
                )
            };
            Some(RootProjection {
                from,
                to,
                fidelity: rule.fidelity.clone(),
                owed_by,
            })
        })
        .collect()
}

/// Why a write target could not be mapped (plan 16 L2g).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteTargetError {
    /// The path does not live under the source root with its owned
    /// extension, or escapes it with `..`.
    WrongRoot { path: String },
    /// The leg is live for reading, but its target is not a committed
    /// source tree `--write` maps into (the pivot document and the rust
    /// legs owe their writes to L3/L5).
    UnsupportedLeg { from: SourceRoot, to: SourceRoot },
}

impl SourceRoot {
    /// The directory spelling of the root inside the repository, for the
    /// sibling-root write mapping (`js/`, `ts/`).
    #[must_use]
    pub const fn directory(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::JavaScript => "js",
            Self::TypeScript => "ts",
            Self::Meta => "meta",
        }
    }

    /// The file extension this root's owned source files carry.
    #[must_use]
    pub const fn owned_extension(self) -> Option<&'static str> {
        match self {
            Self::JavaScript => Some("js"),
            Self::TypeScript => Some("ts"),
            Self::Rust | Self::Meta => None,
        }
    }
}

/// Map one repo-relative source path to its write target under the sibling
/// root: `js/app/foo.js` → `ts/app/foo.ts`, and the reverse (plan 16 L2g).
///
/// The mapping is total on the two ES roots because those are the committed
/// sibling trees the cycle turns; every other leg is refused by name rather
/// than guessed, and the refusal carries the roots so the seed can say what
/// is owed.
pub fn write_target(
    from: SourceRoot,
    to: SourceRoot,
    repo_relative: &str,
) -> Result<String, WriteTargetError> {
    if !matches!(
        (from, to),
        (SourceRoot::JavaScript, SourceRoot::TypeScript)
            | (SourceRoot::TypeScript, SourceRoot::JavaScript)
    ) {
        return Err(WriteTargetError::UnsupportedLeg { from, to });
    }
    let directory = from.directory();
    let extension = from
        .owned_extension()
        .expect("the ES roots declare their owned extension");
    let mapped = repo_relative
        .strip_prefix(&format!("{directory}/"))
        .and_then(|sub| sub.strip_suffix(&format!(".{extension}")))
        .filter(|_| !escapes_root(repo_relative))
        .ok_or_else(|| WriteTargetError::WrongRoot {
            path: repo_relative.to_owned(),
        })?;
    Ok(format!(
        "{}/{mapped}.{}",
        to.directory(),
        to.owned_extension().expect("the ES roots declare one")
    ))
}

/// A request to translate one source tree file between the ES roots,
/// recognized from the path token and the named target (plan 16 L2g).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTreeRequest {
    pub from: SourceRoot,
    pub to: SourceRoot,
    pub path: String,
}

/// A repo-relative path escapes its root when any segment is the parent
/// directory — a structural check on segments, not a phrase literal.
fn escapes_root(repo_relative: &str) -> bool {
    repo_relative.split('/').any(|segment| segment == "..")
}

/// Recognize a source-tree translation request in a prompt: a path token
/// under `js/` or `ts/` with its owned extension, plus a named ES target.
///
/// The path token is structural vocabulary (the source roots' directory
/// names), not a natural-language phrase, and the target names are the same
/// spellings [`SourceRoot::parse`] accepts — so this adds no phrase table;
/// the meaning gate (the translation-action lexicon) stays in the caller.
#[must_use]
pub fn source_tree_request(prompt: &str) -> Option<SourceTreeRequest> {
    let folded = prompt.to_ascii_lowercase();
    // The path token's shape is the two ES roots' own data — their directory
    // spellings and owned extensions — so a future root joins by declaring
    // itself, not by another literal here.
    let es_roots = [SourceRoot::JavaScript, SourceRoot::TypeScript];
    let mut path_token = None;
    for token in folded.split_whitespace() {
        for root in es_roots {
            let prefix = format!("{}/", root.directory());
            let suffix = format!(
                ".{}",
                root.owned_extension().expect("an ES root owns files")
            );
            if token.starts_with(&prefix) && token.ends_with(&suffix) {
                path_token = Some((root, token));
                break;
            }
        }
        if path_token.is_some() {
            break;
        }
    }
    let (from, token) = path_token?;
    let path = token
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '.')
        .to_owned();
    if escapes_root(&path) {
        return None;
    }
    let words = folded.split_whitespace().collect::<Vec<_>>();
    let to_alias = ["typescript", "javascript"].into_iter().find(|alias| {
        words
            .windows(2)
            .any(|pair| pair[0] == "to" && pair[1] == *alias)
    });
    let to =
        to_alias.map(|alias| SourceRoot::parse(alias).expect("the alias is a root spelling"))?;
    if from == to {
        return None;
    }
    Some(SourceTreeRequest { from, to, path })
}

/// The one-line CLI rendering of a leg row; the row text is carried by the
/// seed (`translate_leg_live` / `translate_leg_pending`).
#[must_use]
pub fn describe_leg(from: SourceRoot, to: SourceRoot, pending: Option<&'static str>) -> String {
    pending
        .map_or_else(
            || {
                crate::seed::render_response(
                    "translate_leg_live",
                    "en",
                    &[("from", from.name()), ("to", to.name())],
                )
            },
            |leaf| {
                crate::seed::render_response(
                    "translate_leg_pending",
                    "en",
                    &[("from", from.name()), ("to", to.name()), ("leaf", leaf)],
                )
            },
        )
        .unwrap_or_else(|| "translate_leg".to_string())
}
