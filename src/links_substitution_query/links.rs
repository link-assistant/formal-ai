//! The link operand domain of the substitution query language.
//!
//! Operands are doublets, matching link-cli's own syntax. See the
//! [module docs](super) for how this domain relates to the text one.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use super::{err, escape, Parser, SubstitutionQueryError, TWO_SIDES};
use crate::link_store::DoubletLink;
use crate::normal_markov::RewriteHalt;
use crate::substitution::CrudEvent;

const LINK_SLOTS: &str = "a link is written (source target) or (index: source target), \
                          with each slot a value or a `$` variable";

/// One slot of a link: a literal value, or a variable that binds to whatever
/// the matched link holds there.
///
/// link-cli names its variables for the positions they usually occupy: "`$i`
/// stands for variable named i, that stands for index. `$s` is for source and
/// `$t` is for target." Nothing enforces that convention, so any name is
/// accepted in any slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot {
    Value(String),
    Variable(String),
}

/// One `(source target)` or `(index: source target)` link operand.
///
/// An absent `index` means the two-slot form. On a matching side it matches any
/// index; on a substitution side it keeps the matched link's index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkPattern {
    pub index: Option<Slot>,
    pub source: Slot,
    pub target: Slot,
}

impl LinkPattern {
    fn slots(&self) -> impl Iterator<Item = &Slot> {
        self.index.iter().chain([&self.source, &self.target])
    }

    fn variables(&self) -> BTreeSet<&str> {
        self.slots()
            .filter_map(|slot| match slot {
                Slot::Variable(name) => Some(name.as_str()),
                Slot::Value(_) => None,
            })
            .collect()
    }
}

/// One rule of a link rewrite: what to match, and what to leave behind.
///
/// `None` is the empty link, and it is what makes CRUD fall out of substitution
/// exactly as it does over text. An absent `pattern` matches nothing and so
/// creates; an absent `replacement` leaves nothing and so deletes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRewriteRule {
    pub pattern: Option<LinkPattern>,
    pub replacement: Option<LinkPattern>,
}

/// One observable substitution over the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRewriteStep {
    /// Selected rule index.
    pub rule_index: usize,
    /// What the rule did.
    pub effect: CrudEvent,
    /// The link written, or the link removed.
    pub link: DoubletLink,
}

/// Immutable result and audit trace for one link rewrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRewriteOutcome {
    pub links: Vec<DoubletLink>,
    pub trace: Vec<LinkRewriteStep>,
    pub halt: RewriteHalt,
}

/// An ordered link rewrite under a caller-supplied step bound.
///
/// The control model is [`crate::normal_markov`]'s, unchanged: rules are tried
/// in order, the first applicable one fires, and selection restarts at rule
/// zero. Only the operand domain differs, so the Turing completeness argument is
/// the same one — this is a Markov algorithm over an associative store, which is
/// how LinksQL describes its own single rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRewriteProgram {
    pub rules: Vec<LinkRewriteRule>,
    pub max_steps: usize,
}

impl LinkRewriteProgram {
    #[must_use]
    pub const fn new(rules: Vec<LinkRewriteRule>, max_steps: usize) -> Self {
        Self { rules, max_steps }
    }

    /// Every link any rule's matching side selects, in store order.
    ///
    /// This answers a read. link-cli documents `((($i: $s $t)) (($i: $s $t)))`
    /// as reading "all links without modification", so a read cannot be an
    /// execution step: over a set-valued store, a substitution that leaves the
    /// store unchanged is not a state transition. Reads are matches, not
    /// rewrites.
    #[must_use]
    pub fn matched_links(&self, links: &[DoubletLink]) -> Vec<DoubletLink> {
        links
            .iter()
            .filter(|link| {
                self.rules.iter().any(|rule| {
                    rule.pattern
                        .as_ref()
                        .is_some_and(|pattern| match_link(pattern, link).is_some())
                })
            })
            .cloned()
            .collect()
    }

    /// Run the program over a store until it stops changing, or the bound runs
    /// out.
    ///
    /// [`RewriteHalt::TerminalRule`] is never returned: link rules cannot be
    /// terminal, because link-cli spends the pre-colon slot on the index rather
    /// than on a name this dialect could borrow.
    #[must_use]
    pub fn execute(&self, links: &[DoubletLink]) -> LinkRewriteOutcome {
        let mut store = links.to_vec();
        let mut trace = Vec::new();
        for _ in 0..self.max_steps {
            let Some((rule_index, transition)) = self.next_transition(&store) else {
                return LinkRewriteOutcome {
                    links: store,
                    trace,
                    halt: RewriteHalt::NoApplicableRule,
                };
            };
            let link = apply(&mut store, transition);
            trace.push(LinkRewriteStep {
                rule_index,
                effect: link_substitution_effect(&self.rules[rule_index]),
                link,
            });
        }
        LinkRewriteOutcome {
            links: store,
            trace,
            halt: RewriteHalt::StepLimit,
        }
    }

    fn next_transition(&self, store: &[DoubletLink]) -> Option<(usize, Transition)> {
        self.rules
            .iter()
            .enumerate()
            .find_map(|(index, rule)| rule_transition(rule, store).map(|found| (index, found)))
    }
}

/// Classify what a link rule does to the store it matches.
#[must_use]
pub fn link_substitution_effect(rule: &LinkRewriteRule) -> CrudEvent {
    match (&rule.pattern, &rule.replacement) {
        // Neither side holds a link, so nothing is substituted for nothing.
        (None, None) => CrudEvent::Manual,
        (None, Some(_)) => CrudEvent::Create,
        (Some(_), None) => CrudEvent::Delete,
        (Some(pattern), Some(replacement)) => {
            if pattern == replacement {
                CrudEvent::Read
            } else {
                CrudEvent::Update
            }
        }
    }
}

/// Parse a link substitution query into a bounded rewrite over doublets.
///
/// This is link-cli's syntax as documented: `() ((1 1))` creates, `((1 1)) ()`
/// deletes, `((1: 1 1)) ((1: 1 2))` updates, and `((1: 1 1)) ((1: 1 1))` reads.
///
/// # Errors
/// Returns [`SubstitutionQueryError`] when the two sides are malformed, their
/// operand counts cannot be paired, a substitution uses a variable the matching
/// side never bound, or a creation tries to force an index.
pub fn parse_link_substitution_query(
    query: &str,
    max_steps: usize,
) -> Result<LinkRewriteProgram, SubstitutionQueryError> {
    let mut parser = Parser::new(query);
    let matching = parser.parse_link_side()?;
    parser.skip_whitespace();
    if parser.at_end() {
        return Err(err(format!("{TWO_SIDES}; found only one")));
    }
    let substituting = parser.parse_link_side()?;
    parser.skip_whitespace();
    if !parser.at_end() {
        return Err(err(format!("{TWO_SIDES}; found trailing input")));
    }
    Ok(LinkRewriteProgram::new(
        pair_links(matching, substituting)?,
        max_steps,
    ))
}

/// Render a link rewrite as a substitution query.
///
/// Output is canonical: [`parse_link_substitution_query`] round-trips it back to
/// an equal program. link-cli's `()` shorthand is used whenever a whole side is
/// empty; a program that mixes effects renders the empty link per operand
/// instead, which is the general form the shorthand abbreviates.
#[must_use]
pub fn render_link_substitution_query(program: &LinkRewriteProgram) -> String {
    let rules = &program.rules;
    let patterns_empty = rules.iter().all(|rule| rule.pattern.is_none());
    let replacements_empty = rules.iter().all(|rule| rule.replacement.is_none());
    // As in the text dialect, a side may only be elided when the other side
    // still carries the operands; otherwise the rule count would be lost.
    let elide_matching = !rules.is_empty() && patterns_empty && !replacements_empty;
    let elide_substituting = !rules.is_empty() && replacements_empty && !patterns_empty;

    let matching = if elide_matching {
        String::from("()")
    } else {
        render_link_side(rules.iter().map(|rule| rule.pattern.as_ref()))
    };
    let substituting = if elide_substituting {
        String::from("()")
    } else {
        render_link_side(rules.iter().map(|rule| rule.replacement.as_ref()))
    };
    format!("{matching} {substituting}")
}

/// Render one stored link in the same `(index: source target)` form a query
/// writes it in.
///
/// A read answers with links, so the answer has to be written in the notation
/// the question was asked in: the reader that parses `((1: 1 1))` should be able
/// to parse what comes back. Sharing [`render_slot`] with the query renderer is
/// what keeps the two from drifting into separate quoting rules.
#[must_use]
pub fn render_link(link: &DoubletLink) -> String {
    let pattern = LinkPattern {
        index: Some(Slot::Value(link.index.clone())),
        source: Slot::Value(link.from.clone()),
        target: Slot::Value(link.to.clone()),
    };
    let rendered = render_link_side([Some(&pattern)].into_iter());
    // `render_link_side` renders a whole side, so it wraps the operands in the
    // side's own parentheses; a single link is that side without the wrapper.
    rendered[1..rendered.len() - 1].to_owned()
}

fn render_link_side<'a>(patterns: impl Iterator<Item = Option<&'a LinkPattern>>) -> String {
    let mut rendered = String::from("(");
    for (position, pattern) in patterns.enumerate() {
        if position > 0 {
            rendered.push(' ');
        }
        match pattern {
            None => rendered.push_str("()"),
            Some(pattern) => {
                rendered.push('(');
                if let Some(index) = &pattern.index {
                    let _ = write!(rendered, "{}: ", render_slot(index));
                }
                let _ = write!(
                    rendered,
                    "{} {})",
                    render_slot(&pattern.source),
                    render_slot(&pattern.target)
                );
            }
        }
    }
    rendered.push(')');
    rendered
}

fn render_slot(slot: &Slot) -> String {
    match slot {
        Slot::Variable(name) => format!("${name}"),
        // A value that reads back as a single bare token needs no quotes; one
        // that would not — `a b`, `x:`, `` — must be quoted to survive.
        Slot::Value(value) => {
            if !value.is_empty() && value.chars().all(is_token_char) {
                value.clone()
            } else {
                format!("\"{}\"", escape(value))
            }
        }
    }
}

/// Pair the two sides into ordered rules, distributing an elided side.
fn pair_links(
    matching: Vec<Option<LinkPattern>>,
    substituting: Vec<Option<LinkPattern>>,
) -> Result<Vec<LinkRewriteRule>, SubstitutionQueryError> {
    let rules: Vec<LinkRewriteRule> = match (matching.is_empty(), substituting.is_empty()) {
        (true, true) => Vec::new(),
        (true, false) => substituting
            .into_iter()
            .map(|replacement| LinkRewriteRule {
                pattern: None,
                replacement,
            })
            .collect(),
        (false, true) => matching
            .into_iter()
            .map(|pattern| LinkRewriteRule {
                pattern,
                replacement: None,
            })
            .collect(),
        (false, false) => {
            if matching.len() != substituting.len() {
                return Err(err(format!(
                    "operand count must match across sides, or one side must be `()`; \
                     the matching side has {} and the substitution side has {}",
                    matching.len(),
                    substituting.len()
                )));
            }
            matching
                .into_iter()
                .zip(substituting)
                .map(|(pattern, replacement)| LinkRewriteRule {
                    pattern,
                    replacement,
                })
                .collect()
        }
    };
    for rule in &rules {
        validate_link_rule(rule)?;
    }
    Ok(rules)
}

/// Reject rules that could not be executed, at parse time rather than mid-run.
///
/// With this check every parsed program is total: each variable on a
/// substitution side is guaranteed to be bound by the time it is resolved.
fn validate_link_rule(rule: &LinkRewriteRule) -> Result<(), SubstitutionQueryError> {
    let Some(replacement) = &rule.replacement else {
        return Ok(());
    };
    let bound = rule
        .pattern
        .as_ref()
        .map(LinkPattern::variables)
        .unwrap_or_default();
    if let Some(unbound) = replacement
        .variables()
        .into_iter()
        .find(|name| !bound.contains(name))
    {
        return Err(err(format!(
            "`${unbound}` is used on the substitution side but never bound on the matching side"
        )));
    }
    if rule.pattern.is_none() && replacement.index.is_some() {
        return Err(err(
            "creation cannot force an index: write `() ((source target))` \
             and let the store assign one",
        ));
    }
    Ok(())
}

fn rule_transition(rule: &LinkRewriteRule, store: &[DoubletLink]) -> Option<Transition> {
    match (&rule.pattern, &rule.replacement) {
        (None, None) => None,
        (None, Some(replacement)) => {
            let empty = BTreeMap::new();
            let link = DoubletLink {
                index: allocate_index(store),
                from: resolve(&replacement.source, &empty),
                to: resolve(&replacement.target, &empty),
            };
            // link-cli deduplicates: "Identical sub-links are created once and
            // reused". Re-creating a link the store already holds is therefore
            // not a state transition, which is also what lets a creation rule
            // terminate instead of appending forever.
            (!store
                .iter()
                .any(|held| held.from == link.from && held.to == link.to))
            .then_some(Transition::Insert(link))
        }
        (Some(pattern), None) => store
            .iter()
            .position(|link| match_link(pattern, link).is_some())
            .map(Transition::Remove),
        (Some(pattern), Some(replacement)) => {
            store.iter().enumerate().find_map(|(position, link)| {
                let bindings = match_link(pattern, link)?;
                let next = substitute(replacement, &bindings, link);
                // A substitution that returns the same link is a read, not a
                // step. Skipping it is what makes the identity query halt.
                (next != *link).then_some(Transition::Replace(position, next))
            })
        }
    }
}

enum Transition {
    Insert(DoubletLink),
    Remove(usize),
    Replace(usize, DoubletLink),
}

/// Apply one transition and report the link it wrote or removed.
fn apply(store: &mut Vec<DoubletLink>, transition: Transition) -> DoubletLink {
    match transition {
        Transition::Insert(link) => {
            store.push(link.clone());
            link
        }
        Transition::Remove(position) => store.remove(position),
        Transition::Replace(position, link) => {
            store.remove(position);
            // The store is a set, so an update onto a link it already holds
            // merges rather than duplicates.
            if !store.contains(&link) {
                store.insert(position, link.clone());
            }
            link
        }
    }
}

fn match_link(pattern: &LinkPattern, link: &DoubletLink) -> Option<BTreeMap<String, String>> {
    let mut bindings = BTreeMap::new();
    if let Some(index) = &pattern.index {
        bind(index, &link.index, &mut bindings)?;
    }
    bind(&pattern.source, &link.from, &mut bindings)?;
    bind(&pattern.target, &link.to, &mut bindings)?;
    Some(bindings)
}

/// Match one slot, binding a free variable or checking a bound one.
///
/// Rebinding is a check, not an overwrite, so `(($i: $s $s))` matches only the
/// links whose source and target are equal.
fn bind(slot: &Slot, value: &str, bindings: &mut BTreeMap<String, String>) -> Option<()> {
    match slot {
        Slot::Value(expected) => (expected == value).then_some(()),
        // A free variable binds and so always matches; a bound one only matches
        // the value it already holds. `or_insert_with` is both cases at once.
        Slot::Variable(name) => {
            let bound = bindings
                .entry(name.clone())
                .or_insert_with(|| value.to_owned());
            (bound == value).then_some(())
        }
    }
}

fn substitute(
    replacement: &LinkPattern,
    bindings: &BTreeMap<String, String>,
    matched: &DoubletLink,
) -> DoubletLink {
    DoubletLink {
        // An elided index on a substitution keeps the matched link's own index,
        // which is what makes `(($i: $s $t)) (($t $s))` a pure swap.
        index: replacement
            .index
            .as_ref()
            .map_or_else(|| matched.index.clone(), |slot| resolve(slot, bindings)),
        from: resolve(&replacement.source, bindings),
        to: resolve(&replacement.target, bindings),
    }
}

/// Resolve a slot to a value.
///
/// `validate_link_rule` guarantees a parsed program never resolves a free
/// variable. A hand-built [`LinkRewriteProgram`] can, so the variable renders as
/// itself rather than panicking.
fn resolve(slot: &Slot, bindings: &BTreeMap<String, String>) -> String {
    match slot {
        Slot::Value(value) => value.clone(),
        Slot::Variable(name) => bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| format!("${name}")),
    }
}

/// The index a creation takes: one past the highest the store holds.
fn allocate_index(store: &[DoubletLink]) -> String {
    store
        .iter()
        .filter_map(|link| link.index.parse::<u64>().ok())
        .max()
        .map_or(1, |highest| highest.saturating_add(1))
        .to_string()
}

/// Characters a slot value may use without quoting.
///
/// Unicode-aware, so a link between non-Latin values still reads and renders
/// bare; anything holding a separator (`:`, `(`, `)`, a space) must be quoted.
fn is_token_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-' | '.')
}

impl Parser<'_> {
    /// Read one side of a link query: `()`, or a run of link operands.
    fn parse_link_side(&mut self) -> Result<Vec<Option<LinkPattern>>, SubstitutionQueryError> {
        self.skip_whitespace();
        if !self.eat('(') {
            return Err(err(format!("{TWO_SIDES}; each side opens with `(`")));
        }
        let mut patterns = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(')') => {
                    self.bump();
                    return Ok(patterns);
                }
                Some('(') => patterns.push(self.parse_link_pattern()?),
                Some(unexpected) => {
                    return Err(err(format!("unexpected `{unexpected}`; {LINK_SLOTS}")));
                }
                None => {
                    return Err(err("unbalanced parentheses: a side is missing its `)`"));
                }
            }
        }
    }

    /// Read one link operand. `()` is the empty link, which creates or deletes
    /// depending on the side it appears on.
    fn parse_link_pattern(&mut self) -> Result<Option<LinkPattern>, SubstitutionQueryError> {
        self.bump(); // the operand's `(`
        self.skip_whitespace();
        if self.eat(')') {
            return Ok(None);
        }
        let first = self.parse_link_slot()?;
        self.skip_whitespace();
        // `(index: source target)` and `(source target)` diverge only here: a
        // colon demotes the slot just read from source to index.
        let (index, source) = if self.eat(':') {
            (Some(first), self.parse_link_slot()?)
        } else {
            (None, first)
        };
        let target = self.parse_link_slot()?;
        self.skip_whitespace();
        if !self.eat(')') {
            return Err(err(format!(
                "unbalanced parentheses, or too many slots: {LINK_SLOTS}"
            )));
        }
        Ok(Some(LinkPattern {
            index,
            source,
            target,
        }))
    }

    fn parse_link_slot(&mut self) -> Result<Slot, SubstitutionQueryError> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') => Ok(Slot::Value(self.parse_string()?)),
            Some('$') => {
                self.bump();
                let name = self.parse_token();
                if name.is_empty() {
                    return Err(err("`$` must be followed by a variable name, as in `$i`"));
                }
                Ok(Slot::Variable(name))
            }
            _ => {
                let token = self.parse_token();
                if token.is_empty() {
                    return Err(err(LINK_SLOTS));
                }
                Ok(Slot::Value(token))
            }
        }
    }

    fn parse_token(&mut self) -> String {
        let start = self.cursor;
        while self.peek().is_some_and(is_token_char) {
            self.bump();
        }
        self.input[start..self.cursor].to_owned()
    }
}
