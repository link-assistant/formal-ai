//! Nested symbolic contexts with explicit inheritance — issue #702.
//!
//! A dialogue turn, task, pull request, issue, repository, or organization can
//! be represented by the same [`Context`] type. [`ContextHierarchy`] composes
//! those contexts without copying facts between them:
//!
//! * nesting has no configured depth bound;
//! * every child chooses full, isolated, or conditional inheritance;
//! * reference lookup starts locally and walks outward only until it resolves;
//! * an unresolved lookup reports whether outside research is permitted but
//!   never performs that research itself.
//!
//! The implementation is iterative and cycle-safe. All returned links and
//! traces are deterministic and use Links Notation.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;

use crate::links_format::format_lino_value_verbatim;
use crate::substitution::{LinkPattern, SubstitutionLink};
use crate::world_model::Context;

/// What a child context may see from its parent and more distant ancestors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InheritancePolicy {
    /// Inherit every ancestor link.
    Full,
    /// Keep the child local; parent lookup stops at this boundary.
    Isolated,
    /// Inherit only links accepted by at least one pattern.
    ///
    /// When lookup crosses more than one conditional boundary, a link must
    /// satisfy at least one pattern at every crossed boundary.
    Conditional(Vec<LinkPattern>),
}

impl InheritancePolicy {
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Isolated => "isolated",
            Self::Conditional(_) => "conditional",
        }
    }
}

/// Whether a failed local hierarchy lookup may be handed to outside research.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalLookup {
    Denied,
    Allowed,
}

/// The deterministic outcome of resolving one reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceResolutionKind {
    /// A nearest visible context supplied one or more matching links.
    Resolved,
    /// No local context resolved it and policy allows an outside lookup.
    ExternalLookupRequired,
    /// No local context resolved it and outside lookup is denied.
    Unresolved,
}

impl ReferenceResolutionKind {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::ExternalLookupRequired => "external_lookup_required",
            Self::Unresolved => "unresolved",
        }
    }
}

/// Traceable result of a nearest-context-first reference lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceResolution {
    pub reference: String,
    pub start_context: String,
    pub kind: ReferenceResolutionKind,
    /// The nearest context that resolved the reference.
    pub context_id: Option<String>,
    /// Number of parent boundaries crossed to reach `context_id`.
    pub depth: Option<usize>,
    /// Every matching link from the nearest resolving context.
    pub links: Vec<SubstitutionLink>,
    /// Context ids actually inspected, in order. A successful lazy lookup does
    /// not contain unneeded ancestors.
    pub visited: Vec<String>,
}

impl ReferenceResolution {
    /// Render the decision as deterministic Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("reference_resolution\n");
        let _ = writeln!(
            out,
            "  reference {}",
            format_lino_value_verbatim(&self.reference)
        );
        let _ = writeln!(
            out,
            "  start_context {}",
            format_lino_value_verbatim(&self.start_context)
        );
        let _ = writeln!(
            out,
            "  outcome {}",
            format_lino_value_verbatim(self.kind.slug())
        );
        if let Some(context_id) = &self.context_id {
            let _ = writeln!(
                out,
                "  resolved_in {}",
                format_lino_value_verbatim(context_id)
            );
        }
        if let Some(depth) = self.depth {
            let _ = writeln!(
                out,
                "  depth {}",
                format_lino_value_verbatim(&depth.to_string())
            );
        }
        let visited = self.visited.join("|");
        let _ = writeln!(out, "  visited {}", format_lino_value_verbatim(&visited));
        for link in &self.links {
            let _ = writeln!(
                out,
                "  link {}",
                format_lino_value_verbatim(&link.pattern_text())
            );
        }
        out.trim_end().to_owned()
    }
}

/// A child-to-parent inheritance declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentContext {
    pub parent_id: String,
    pub policy: InheritancePolicy,
}

/// Errors that would make a context hierarchy ambiguous or cyclic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextHierarchyError {
    DuplicateContext(String),
    MissingContext(String),
    SelfParent(String),
    ParentCycle { child: String, parent: String },
}

impl fmt::Display for ContextHierarchyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateContext(id) => write!(formatter, "context `{id}` already exists"),
            Self::MissingContext(id) => write!(formatter, "context `{id}` does not exist"),
            Self::SelfParent(id) => write!(formatter, "context `{id}` cannot inherit from itself"),
            Self::ParentCycle { child, parent } => {
                write!(
                    formatter,
                    "making `{parent}` the parent of `{child}` creates a cycle"
                )
            }
        }
    }
}

impl Error for ContextHierarchyError {}

/// Named contexts plus explicit child-to-parent inheritance declarations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextHierarchy {
    contexts: BTreeMap<String, Context>,
    parents: BTreeMap<String, ParentContext>,
}

impl ContextHierarchy {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            contexts: BTreeMap::new(),
            parents: BTreeMap::new(),
        }
    }

    /// Insert an independent context.
    pub fn insert(&mut self, context: Context) -> Result<(), ContextHierarchyError> {
        if self.contexts.contains_key(&context.id) {
            return Err(ContextHierarchyError::DuplicateContext(context.id));
        }
        self.contexts.insert(context.id.clone(), context);
        Ok(())
    }

    /// Insert a context and attach it to an existing parent atomically.
    pub fn nest(
        &mut self,
        context: Context,
        parent_id: &str,
        policy: InheritancePolicy,
    ) -> Result<(), ContextHierarchyError> {
        let id = context.id.clone();
        self.insert(context)?;
        if let Err(error) = self.set_parent(&id, parent_id, policy) {
            self.contexts.remove(&id);
            return Err(error);
        }
        Ok(())
    }

    /// Set or replace one context's parent after validating both ids and cycles.
    pub fn set_parent(
        &mut self,
        child_id: &str,
        parent_id: &str,
        policy: InheritancePolicy,
    ) -> Result<(), ContextHierarchyError> {
        self.require_context(child_id)?;
        self.require_context(parent_id)?;
        if child_id == parent_id {
            return Err(ContextHierarchyError::SelfParent(child_id.to_owned()));
        }

        let mut cursor = parent_id;
        let mut seen = BTreeSet::new();
        while seen.insert(cursor.to_owned()) {
            if cursor == child_id {
                return Err(ContextHierarchyError::ParentCycle {
                    child: child_id.to_owned(),
                    parent: parent_id.to_owned(),
                });
            }
            let Some(relation) = self.parents.get(cursor) else {
                break;
            };
            cursor = &relation.parent_id;
        }

        self.parents.insert(
            child_id.to_owned(),
            ParentContext {
                parent_id: parent_id.to_owned(),
                policy,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn context(&self, id: &str) -> Option<&Context> {
        self.contexts.get(id)
    }

    #[must_use]
    pub fn context_mut(&mut self, id: &str) -> Option<&mut Context> {
        self.contexts.get_mut(id)
    }

    #[must_use]
    pub fn parent(&self, child_id: &str) -> Option<&ParentContext> {
        self.parents.get(child_id)
    }

    /// Resolve an exact link `from` value, nearest visible context first.
    ///
    /// Lookup is lazy: once a visible context contains matching links, more
    /// distant parents are not inspected. Conditional filters accumulate while
    /// crossing boundaries, and an isolated boundary stops the walk.
    pub fn resolve(
        &self,
        start_context: &str,
        reference: &str,
        external_lookup: ExternalLookup,
    ) -> Result<ReferenceResolution, ContextHierarchyError> {
        self.require_context(start_context)?;

        let mut cursor = start_context;
        let mut visited = Vec::new();
        let mut seen = BTreeSet::new();
        let mut conditional_boundaries: Vec<&[LinkPattern]> = Vec::new();

        loop {
            if !seen.insert(cursor.to_owned()) {
                return Err(ContextHierarchyError::ParentCycle {
                    child: start_context.to_owned(),
                    parent: cursor.to_owned(),
                });
            }
            visited.push(cursor.to_owned());

            let context = self
                .contexts
                .get(cursor)
                .ok_or_else(|| ContextHierarchyError::MissingContext(cursor.to_owned()))?;
            let links: Vec<SubstitutionLink> = context
                .links()
                .into_iter()
                .filter(|link| {
                    link.from == reference
                        && conditional_boundaries
                            .iter()
                            .all(|patterns| patterns.iter().any(|pattern| pattern.matches(link)))
                })
                .collect();
            if !links.is_empty() {
                return Ok(ReferenceResolution {
                    reference: reference.to_owned(),
                    start_context: start_context.to_owned(),
                    kind: ReferenceResolutionKind::Resolved,
                    context_id: Some(cursor.to_owned()),
                    depth: Some(visited.len() - 1),
                    links,
                    visited,
                });
            }

            let Some(relation) = self.parents.get(cursor) else {
                break;
            };
            match &relation.policy {
                InheritancePolicy::Full => {}
                InheritancePolicy::Isolated => break,
                InheritancePolicy::Conditional(patterns) => {
                    conditional_boundaries.push(patterns);
                }
            }
            cursor = &relation.parent_id;
        }

        Ok(ReferenceResolution {
            reference: reference.to_owned(),
            start_context: start_context.to_owned(),
            kind: match external_lookup {
                ExternalLookup::Allowed => ReferenceResolutionKind::ExternalLookupRequired,
                ExternalLookup::Denied => ReferenceResolutionKind::Unresolved,
            },
            context_id: None,
            depth: None,
            links: Vec::new(),
            visited,
        })
    }

    /// Render context membership and inheritance declarations as Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("context_hierarchy\n");
        for (id, context) in &self.contexts {
            let _ = writeln!(out, "  context {}", format_lino_value_verbatim(id));
            for link in context.links() {
                let _ = writeln!(
                    out,
                    "    link {}",
                    format_lino_value_verbatim(&link.pattern_text())
                );
            }
            if let Some(parent) = self.parents.get(id) {
                let _ = writeln!(
                    out,
                    "    parent {}",
                    format_lino_value_verbatim(&parent.parent_id)
                );
                let _ = writeln!(
                    out,
                    "    inheritance {}",
                    format_lino_value_verbatim(parent.policy.slug())
                );
                if let InheritancePolicy::Conditional(patterns) = &parent.policy {
                    for pattern in patterns {
                        let _ = writeln!(
                            out,
                            "    condition {}",
                            format_lino_value_verbatim(&pattern.to_string())
                        );
                    }
                }
            }
        }
        out.trim_end().to_owned()
    }

    fn require_context(&self, id: &str) -> Result<(), ContextHierarchyError> {
        if self.contexts.contains_key(id) {
            Ok(())
        } else {
            Err(ContextHierarchyError::MissingContext(id.to_owned()))
        }
    }
}
