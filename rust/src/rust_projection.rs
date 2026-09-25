//! Grammar projection between the three committed corpora (issue #1138,
//! plan 16 L8).
//!
//! The engine parses each grammar into a leaf-up network while its
//! renderer walks references downward, so template expansion here is an
//! owned leaf-up walk over the dependency's rule set — the same
//! owned-walk-over-dependency-primitives shape as every plan-16 leaf:
//! links are claimed through the engine's public `query_matches`, and
//! templates expand through `{placeholder}` substitution over the
//! captures each rule's query binds. The variadic form enumerates the
//! named syntax children only — a parse also carries anonymous
//! punctuation leaves and whitespace gap tokens, and no child list
//! enumerates those. Every syntax kind a projection meets
//! is either ruled for the target or declared refused in the seed, and a
//! refused kind splices its source span through verbatim — that is the
//! contract; anything else refuses the whole projection naming the kind.
//! The seed is generated (`generate_grammar_projection_rules` composes the
//! rule set with the refusal rows) and never hand-written.

use std::collections::BTreeSet;

use crate::es_meta::Refusal;

/// The language tag that marks a seed row as a declared refusal.
pub const REFUSAL_LANGUAGE: &str = "grammar-projection-refusal";

/// The refusal-row target that applies to every projection direction.
pub const ANY_TARGET: &str = "any";

const RULE_SET_TERM: &str = "translation-rule-set";
const RULE_TERM: &str = "translation-rule";
const MATCH_TERM: &str = "translation-rule-match";
const MAX_WALK_DEPTH: usize = 1024;

/// What a projection produced: rendered target source, or every refusal
/// that stopped it (empty output is never silently returned).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionOutcome {
    /// The target label and the rendered target source.
    Rendered {
        /// The grammar label the source was projected into.
        target: String,
        /// The rendered target source.
        source: String,
    },
    /// Every construct that had no rule and no refusal row; one is enough
    /// to refuse the whole projection.
    Refused {
        /// The refusals, one per offending kind.
        refusals: Vec<Refusal>,
    },
}

/// Project `source` from one owned grammar into another by label.
///
/// The seed and the grammar labels are data: the rust, javascript and
/// typescript labels the cycle owns are the ones
/// [`crate::grammar_kinds::CORPORA`] declares.
#[must_use]
pub fn project(from_label: &str, target_label: &str, source: &str) -> ProjectionOutcome {
    projection_project(from_label, target_label, source)
}

#[cfg(feature = "meta-language")]
fn projection_project(from_label: &str, target_label: &str, source: &str) -> ProjectionOutcome {
    use crate::grammar_kinds::corpus_by_label;

    let unknown = Refusal {
        construct: from_label.to_owned(),
        detail: "unknown grammar label; the cycle owns rust, javascript and typescript".to_owned(),
    };
    if corpus_by_label(from_label).is_none() || corpus_by_label(target_label).is_none() {
        return ProjectionOutcome::Refused {
            refusals: vec![unknown],
        };
    }
    if from_label == target_label {
        return ProjectionOutcome::Refused {
            refusals: vec![Refusal {
                construct: from_label.to_owned(),
                detail: "same-grammar projection is the identity; the source already is the target"
                    .to_owned(),
            }],
        };
    }

    let projection = embedded();
    let network = crate::grammar_kinds::parse_network(from_label, source);
    // Anonymous keyword leaves and punctuation ride the template of the
    // kind that rules them — the coverage table answers for named kinds.
    let kinds: BTreeSet<String> = network
        .links()
        .filter(|link| link.metadata().link_type() == Some(meta_language::LinkType::Syntax))
        .filter(|link| link.metadata().is_named())
        .filter_map(|link| link.metadata().term())
        .map(str::to_owned)
        .collect();
    let refusals = projection.coverage_kinds(&kinds, target_label);
    if !refusals.is_empty() {
        return ProjectionOutcome::Refused { refusals };
    }
    match projection.render(&network, source, target_label) {
        Ok(rendered) => ProjectionOutcome::Rendered {
            target: target_label.to_owned(),
            source: rendered,
        },
        Err(refusal) => ProjectionOutcome::Refused {
            refusals: vec![refusal],
        },
    }
}

#[cfg(not(feature = "meta-language"))]
fn projection_project(from_label: &str, _target_label: &str, _source: &str) -> ProjectionOutcome {
    ProjectionOutcome::Refused {
        refusals: vec![Refusal {
            construct: from_label.to_owned(),
            detail: "grammar projection requires the meta-language engine".to_owned(),
        }],
    }
}

#[cfg(feature = "meta-language")]
mod engine {
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use std::sync::OnceLock;

    use super::{
        ANY_TARGET, MATCH_TERM, MAX_WALK_DEPTH, REFUSAL_LANGUAGE, RULE_SET_TERM, RULE_TERM,
    };
    use crate::es_meta::Refusal;

    /// One seeded refusal detail (R379): the sentences live in
    /// data/seed/multilingual-responses-translate.lino under
    /// `projection_refusal_*` intents, with the intent id as the fallback
    /// so a missing record stays visible instead of rendering empty.
    fn seeded_detail(intent: &str, values: &[(&str, &str)]) -> String {
        crate::seed::render_response(intent, "en", values).unwrap_or_else(|| intent.to_string())
    }

    use meta_language::{
        LinkId, LinkNetwork, LinkType, QueryMatch, TranslationRule, TranslationRuleSet,
    };

    /// A rule set plus the refusal rows that ship beside it in one seed
    /// document.
    ///
    /// Built by [`super::projection_from`]; the refusal rows and the root
    /// kinds ride the same lino network the dependency's `from_lino`
    /// already tolerates as extra root children.
    #[derive(Debug, Clone)]
    pub struct GrammarProjection {
        rule_set: TranslationRuleSet,
        root_kinds: Vec<Option<String>>,
        ruled_kinds: BTreeMap<String, Vec<String>>,
        refused_kinds: BTreeMap<String, Vec<String>>,
    }

    /// One claim: the rule that won a link, and the captures its query
    /// bound for it, in match order, with how many of those bindings are
    /// named syntax content.
    struct Claim {
        rule_index: usize,
        content: usize,
        captures: Vec<(String, LinkId)>,
    }

    /// Everything one expansion step needs besides the placeholder
    /// data — the walk's shared context.
    struct Walk<'a> {
        network: &'a LinkNetwork,
        source: &'a str,
        target: &'a str,
        claims: &'a HashMap<LinkId, Claim>,
    }

    impl GrammarProjection {
        /// How many rules the seed carries.
        #[must_use]
        pub fn rule_count(&self) -> usize {
            self.rule_set.rules().len()
        }

        /// How many refusal rows the seed carries (one row per kind and
        /// target).
        #[must_use]
        pub fn refusal_count(&self) -> usize {
            self.refused_kinds.values().map(Vec::len).sum()
        }

        /// Whether `kind` is ruled with a template for `target`.
        #[must_use]
        pub fn is_ruled(&self, kind: &str, target: &str) -> bool {
            self.ruled_kinds
                .get(kind)
                .is_some_and(|targets| targets.iter().any(|ruled| ruled == target))
        }

        /// Whether `kind` is declared refused for `target` (or for every
        /// target).
        #[must_use]
        pub fn is_refused(&self, kind: &str, target: &str) -> bool {
            self.refused_kinds.get(kind).is_some_and(|targets| {
                targets
                    .iter()
                    .any(|refused| refused == target || refused == ANY_TARGET)
            })
        }

        /// The kinds in `kinds` that are neither ruled for `target` nor
        /// declared refused — one refusal per kind.
        ///
        /// Kind-set arithmetic, not query matching: this is the corpus
        /// ratchet's engine, and it stays fast enough to run over every
        /// committed module.
        #[must_use]
        pub fn coverage_kinds(&self, kinds: &BTreeSet<String>, target: &str) -> Vec<Refusal> {
            kinds
                .iter()
                .filter(|kind| !self.is_ruled(kind, target) && !self.is_refused(kind, target))
                .map(|kind| Refusal {
                    construct: kind.clone(),
                    detail: seeded_detail(
                        "projection_refusal_uncovered_kind",
                        &[("target", target)],
                    ),
                })
                .collect()
        }

        /// Render the network's syntax root(s) into `target` source.
        ///
        /// Callers run [`Self::coverage_kinds`] first; render still
        /// refuses on its own if a link the walk meets is neither claimed
        /// by a rule carrying a `target` template nor declared refused.
        pub fn render(
            &self,
            network: &LinkNetwork,
            source: &str,
            target: &str,
        ) -> Result<String, Refusal> {
            let claims = self.claim_map(network, target);
            let walk = Walk {
                network,
                source,
                target,
                claims: &claims,
            };
            let mut roots = Vec::new();
            for link in network.links() {
                if link.metadata().link_type() != Some(LinkType::Syntax) {
                    continue;
                }
                let parent_is_document = link
                    .references()
                    .first()
                    .and_then(|parent| network.link(*parent))
                    .is_some_and(|parent| {
                        parent.metadata().link_type() == Some(LinkType::Document)
                    });
                if parent_is_document {
                    roots.push(link.id());
                }
            }
            if roots.is_empty() {
                return Err(Refusal {
                    construct: "document".to_owned(),
                    detail: "the parse has no syntax root under the document".to_owned(),
                });
            }
            let mut rendered = Vec::with_capacity(roots.len());
            for root in roots {
                rendered.push(self.expand_link(&walk, root, 0)?);
            }
            Ok(rendered.join("\n"))
        }

        /// Claims every link a rule's query matches, first rule in seed
        /// order winning the link. Only rules carrying a template for
        /// `target` compete: the grammars share kind names
        /// (`binary_expression` is rust's and javascript's), and a rule
        /// that cannot render the target must not steal the link from the
        /// direction's own row.
        fn claim_map(&self, network: &LinkNetwork, target: &str) -> HashMap<LinkId, Claim> {
            let kind_list = syntax_kinds(network);
            let present_kinds: BTreeSet<&str> = kind_list.iter().map(String::as_str).collect();
            let mut claims: HashMap<LinkId, Claim> = HashMap::new();
            for (rule_index, rule) in self.rule_set.rules().iter().enumerate() {
                if rule.templates().get(target).is_none() {
                    continue;
                }
                if self
                    .root_kinds
                    .get(rule_index)
                    .and_then(Option::as_deref)
                    .is_some_and(|kind| !present_kinds.contains(kind))
                {
                    continue;
                }
                for query_match in network.query_matches(rule.query()) {
                    let matched = query_match.link_id();
                    let captures = bound_captures(network, rule, &query_match);
                    let content = content_count(network, &captures);
                    // A wildcard child pattern also binds the anonymous
                    // keyword and punctuation wrappers a parse carries, so
                    // a match whose every binding is one of those binds no
                    // content at all — structural noise the claim pass
                    // skips, which is what lets a bare `(return_statement)`
                    // row claim `return;` instead of a wildcard match that
                    // happened to bind the `return` keyword itself.
                    if !captures.is_empty() && content == 0 {
                        continue;
                    }
                    // A quantified child pattern makes the engine return
                    // one match per binding count — zero items, one item,
                    // and so on — so the greedy match is the one with the
                    // most captures, and that is the claim the template
                    // expands. A single unquantified wildcard enumerates
                    // the children one match at a time, all with one
                    // capture, so among equally greedy matches the one
                    // binding more named content wins: the anonymous
                    // wrapper is structure, the named child is what the
                    // template means. The first rule in seed order still
                    // wins the link outright.
                    let better = claims.get(&matched).is_none_or(|existing| {
                        existing.rule_index == rule_index
                            && (existing.content, existing.captures.len())
                                < (content, captures.len())
                    });
                    if better {
                        claims.insert(
                            matched,
                            Claim {
                                rule_index,
                                content,
                                captures,
                            },
                        );
                    }
                }
            }
            claims
        }

        /// Expand one link into target source: the leaf-up walk itself.
        fn expand_link(
            &self,
            walk: &Walk<'_>,
            link_id: LinkId,
            depth: usize,
        ) -> Result<String, Refusal> {
            let network = walk.network;
            let source = walk.source;
            let target = walk.target;
            let claims = walk.claims;
            if depth > MAX_WALK_DEPTH {
                return Err(Refusal {
                    construct: "walk".to_owned(),
                    detail: seeded_detail(
                        "projection_refusal_walk_depth",
                        &[("depth", &MAX_WALK_DEPTH.to_string())],
                    ),
                });
            }
            let Some(link) = network.link(link_id) else {
                return Err(Refusal {
                    construct: "walk".to_owned(),
                    detail: seeded_detail(
                        "projection_refusal_dangling_capture",
                        &[("link", &format!("{link_id:?}"))],
                    ),
                });
            };
            let metadata = link.metadata();
            match metadata.link_type() {
                Some(LinkType::Token) => Ok(metadata.term().unwrap_or_default().to_owned()),
                Some(LinkType::Trivia) => Ok(String::new()),
                Some(LinkType::Syntax) => {
                    let kind = metadata.term().unwrap_or("?").to_owned();
                    if let Some(claim) = claims.get(&link_id) {
                        let rule = &self.rule_set.rules()[claim.rule_index];
                        if let Some(template) = rule.templates().get(target) {
                            return self.expand_template(
                                walk,
                                link_id,
                                &kind,
                                template.source(),
                                depth,
                            );
                        }
                        if self.is_refused(&kind, target) {
                            return Ok(span_text(source, metadata.span()));
                        }
                        return Err(Refusal {
                            construct: kind,
                            detail: seeded_detail(
                                "projection_refusal_missing_template",
                                &[("rule", rule.name()), ("target", target)],
                            ),
                        });
                    }
                    if self.is_refused(&kind, target) {
                        return Ok(span_text(source, metadata.span()));
                    }
                    Err(Refusal {
                        construct: kind,
                        detail: seeded_detail(
                            "projection_refusal_unruled_kind",
                            &[("target", target)],
                        ),
                    })
                }
                _ => Ok(span_text(source, metadata.span())),
            }
        }

        /// Expand a template's placeholders against one match.
        fn expand_template(
            &self,
            walk: &Walk<'_>,
            matched: LinkId,
            kind: &str,
            template: &str,
            depth: usize,
        ) -> Result<String, Refusal> {
            let bytes = template.as_bytes();
            let mut out: Vec<u8> = Vec::with_capacity(template.len());
            let mut index = 0;
            while index < bytes.len() {
                if bytes[index] == b'{' && bytes.get(index + 1) == Some(&b'{') {
                    out.push(b'{');
                    index += 2;
                } else if bytes[index] == b'}' && bytes.get(index + 1) == Some(&b'}') {
                    out.push(b'}');
                    index += 2;
                } else if bytes[index] == b'{' {
                    let Some(relative) = template[index + 1..].find('}') else {
                        out.push(b'{');
                        index += 1;
                        continue;
                    };
                    let inner = &template[index + 1..index + 1 + relative];
                    let replacement = self.expand_placeholder(walk, matched, kind, inner, depth)?;
                    out.extend_from_slice(replacement.as_bytes());
                    index += 1 + relative + 1;
                } else {
                    out.push(bytes[index]);
                    index += 1;
                }
            }
            Ok(String::from_utf8_lossy(&out).into_owned())
        }

        /// Expand one placeholder body (the text between the braces).
        fn expand_placeholder(
            &self,
            walk: &Walk<'_>,
            matched: LinkId,
            kind: &str,
            inner: &str,
            depth: usize,
        ) -> Result<String, Refusal> {
            let network = walk.network;
            let claims = walk.claims;
            let unknown = |placeholder: &str| Refusal {
                construct: kind.to_owned(),
                detail: seeded_detail(
                    "projection_refusal_unknown_placeholder",
                    &[("placeholder", placeholder)],
                ),
            };

            if inner == ".:text" {
                return Ok(span_of(walk, matched));
            }

            let capture_text = |name: &str| -> Option<String> {
                let claim = claims.get(&matched)?;
                let binding = claim
                    .captures
                    .iter()
                    .find(|(bound, _)| bound == name)
                    .map(|(_, link_id)| *link_id)?;
                Some(span_of(walk, binding))
            };

            if let Some(rest) = inner.strip_prefix('*') {
                let Some((name_mode, separator)) = rest.split_once('|') else {
                    return Err(unknown(inner));
                };
                let (name, splice) = match name_mode.split_once(':') {
                    None => (name_mode, false),
                    Some((name, "text" | "source")) => (name, true),
                    Some((_, other)) => {
                        return Err(Refusal {
                            construct: kind.to_owned(),
                            detail: seeded_detail(
                                "projection_refusal_variadic_mode",
                                &[("mode", other)],
                            ),
                        });
                    }
                };
                let bindings: Vec<LinkId> = claims
                    .get(&matched)
                    .map(|claim| {
                        claim
                            .captures
                            .iter()
                            .filter(|(bound, _)| bound == name)
                            .map(|(_, link_id)| *link_id)
                            .collect()
                    })
                    .unwrap_or_default();
                // The variadic form enumerates child lists, so it keeps the
                // named syntax links only: wildcard children also bind the
                // anonymous punctuation leaves and whitespace gap tokens a
                // parse carries, and no child list enumerates those.
                let bindings: Vec<LinkId> = bindings
                    .into_iter()
                    .filter(|link_id| {
                        network.link(*link_id).is_some_and(|link| {
                            link.metadata().link_type() == Some(LinkType::Syntax)
                                && link.metadata().is_named()
                        })
                    })
                    .collect();
                let mut pieces = Vec::with_capacity(bindings.len());
                for binding in bindings {
                    let piece = if splice {
                        span_of(walk, binding)
                    } else {
                        self.expand_link(walk, binding, depth + 1)?
                    };
                    pieces.push(piece);
                }
                return Ok(pieces.join(&unescape_separator(separator)));
            }

            if let Some((name, mode)) = inner.split_once(':') {
                return match mode {
                    "text" | "source" => Ok(capture_text(name).unwrap_or_default()),
                    "term" => {
                        let Some(claim) = claims.get(&matched) else {
                            return Ok(String::new());
                        };
                        let term = claim
                            .captures
                            .iter()
                            .find(|(bound, _)| bound == name)
                            .and_then(|(_, link_id)| network.link(*link_id))
                            .and_then(|link| link.metadata().term())
                            .unwrap_or_default()
                            .to_owned();
                        Ok(term)
                    }
                    other => Err(Refusal {
                        construct: kind.to_owned(),
                        detail: seeded_detail(
                            "projection_refusal_placeholder_mode",
                            &[("mode", other)],
                        ),
                    }),
                };
            }

            let Some(claim) = claims.get(&matched) else {
                return Ok(String::new());
            };
            match claim.captures.iter().find(|(bound, _)| bound == inner) {
                Some((_, binding)) => self.expand_link(walk, *binding, depth + 1),
                // A capture an optional quantifier did not bind expands to
                // nothing; the corpus re-parse ratchet catches templates
                // that lean on it by mistake.
                None => Ok(String::new()),
            }
        }
    }

    /// Load a projection from seed lino text: the rule set through the
    /// dependency's own loader, the refusal rows and root kinds from the
    /// same document's network.
    ///
    /// The tests and the generator both go through this function, so the
    /// behavior provably follows the data.
    pub fn projection_from(lino_text: &str) -> Result<GrammarProjection, String> {
        let rule_set =
            TranslationRuleSet::from_lino(lino_text).map_err(|error| error.to_string())?;
        let network = LinkNetwork::from_lino(lino_text).map_err(|error| error.to_string())?;

        let mut root_kinds: Vec<Option<String>> = Vec::new();
        let mut ruled_kinds: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut refused_kinds: BTreeMap<String, Vec<String>> = BTreeMap::new();

        let Some(root) = network.links().find(|link| {
            link.metadata().link_type() == Some(LinkType::Semantic)
                && link.metadata().term() == Some(RULE_SET_TERM)
        }) else {
            return Err("seed has no translation-rule-set root".to_owned());
        };

        let mut rule_links: Vec<_> = network
            .links()
            .filter(|link| {
                link.references().first().copied() == Some(root.id())
                    && link.metadata().term() == Some(RULE_TERM)
            })
            .collect();
        rule_links.sort_by_key(|link| link.id());
        for rule_link in rule_links {
            let spec = network
                .links()
                .find(|link| {
                    link.references().first().copied() == Some(rule_link.id())
                        && link.metadata().term() == Some(MATCH_TERM)
                })
                .and_then(|link| link.metadata().definition())
                .unwrap_or_default();
            root_kinds.push(root_kind_of_spec(spec));
        }
        if root_kinds.len() != rule_set.rules().len() {
            root_kinds = vec![None; rule_set.rules().len()];
        }

        for (rule, root_kind) in rule_set.rules().iter().zip(&root_kinds) {
            let Some(kind) = root_kind else {
                continue;
            };
            for target in rule.templates().keys() {
                ruled_kinds
                    .entry(kind.clone())
                    .or_default()
                    .push(target.clone());
            }
        }

        for link in network.links().filter(|link| {
            link.references().first().copied() == Some(root.id())
                && link.metadata().language() == Some(REFUSAL_LANGUAGE)
        }) {
            if let (Some(kind), Some(target)) =
                (link.metadata().term(), link.metadata().definition())
            {
                refused_kinds
                    .entry(kind.to_owned())
                    .or_default()
                    .push(target.to_owned());
            }
        }

        Ok(GrammarProjection {
            rule_set,
            root_kinds,
            ruled_kinds,
            refused_kinds,
        })
    }

    /// The embedded grammar-projection seed, parsed once.
    #[must_use]
    pub fn embedded() -> &'static GrammarProjection {
        static EMBEDDED: OnceLock<GrammarProjection> = OnceLock::new();
        EMBEDDED.get_or_init(|| {
            projection_from(crate::seed::GRAMMAR_PROJECTION_RULES_LINO)
                .expect("the embedded grammar projection seed is well-formed")
        })
    }

    /// How many of a match's bindings are named syntax links — the
    /// content of the match, as opposed to the anonymous wrappers and
    /// gap tokens a parse carries around it.
    fn content_count(network: &LinkNetwork, captures: &[(String, LinkId)]) -> usize {
        captures
            .iter()
            .filter(|(_, link_id)| {
                network.link(*link_id).is_some_and(|link| {
                    link.metadata().link_type() == Some(LinkType::Syntax)
                        && link.metadata().is_named()
                })
            })
            .count()
    }

    fn bound_captures(
        network: &LinkNetwork,
        rule: &TranslationRule,
        query_match: &QueryMatch,
    ) -> Vec<(String, LinkId)> {
        let mut captures: Vec<(String, LinkId)> = query_match
            .captures()
            .iter()
            .map(|capture| (capture.name().to_owned(), capture.link_id()))
            .collect();
        for (name, reference_index) in rule.reference_captures() {
            if captures.iter().any(|(bound, _)| bound == name) {
                continue;
            }
            if let Some(reference) = network
                .link(query_match.link_id())
                .and_then(|link| link.references().get(*reference_index))
            {
                captures.push((name.clone(), *reference));
            }
        }
        captures
    }

    fn root_kind_of_spec(spec: &str) -> Option<String> {
        let value: serde_json::Value = serde_json::from_str(spec).ok()?;
        let sexpression = value.get("sexpression").and_then(|s| s.as_str())?;
        // The serialized spec carries the pattern, never a term filter, so
        // the root kind is the head atom of the s-expression.
        let inner = sexpression.trim().strip_prefix('(')?;
        let end = inner
            .find(|c: char| c.is_whitespace() || c == ')')
            .unwrap_or(inner.len());
        let kind = &inner[..end];
        (!kind.is_empty() && kind != "_").then(|| kind.to_owned())
    }

    fn syntax_kinds(network: &LinkNetwork) -> Vec<String> {
        network
            .links()
            .filter(|link| link.metadata().link_type() == Some(LinkType::Syntax))
            .filter_map(|link| link.metadata().term())
            .map(str::to_owned)
            .collect()
    }

    fn span_text(source: &str, span: Option<meta_language::SourceSpan>) -> String {
        span.map(|span| {
            let range = span.byte_range();
            source
                .get(range.start()..range.end())
                .unwrap_or_default()
                .to_owned()
        })
        .unwrap_or_default()
    }

    fn span_of(walk: &Walk<'_>, link_id: LinkId) -> String {
        let Some(link) = walk.network.link(link_id) else {
            return String::new();
        };
        let metadata = link.metadata();
        match (metadata.span(), metadata.term()) {
            (Some(span), _) => {
                let range = span.byte_range();
                walk.source
                    .get(range.start()..range.end())
                    .unwrap_or_default()
                    .to_owned()
            }
            // Token links carry their text as the term even where the
            // parse recorded no span.
            (None, Some(term)) => term.to_owned(),
            (None, None) => String::new(),
        }
    }

    fn unescape_separator(separator: &str) -> String {
        separator
            .replace("\\n", "\n")
            .replace("\\s", " ")
            .replace("\\t", "\t")
    }
}

#[cfg(feature = "meta-language")]
pub use engine::{GrammarProjection, embedded, projection_from};
