//! Constraint-satisfying option networks — issue #781.
//!
//! Issue #781 arrived as a shopping question ("find me a charger for this
//! laptop"), but the reviewer's requirement is explicitly **not** a shopping
//! feature: *"everything must be solved by generalization, not specific to only
//! ability to search chargers, it can be any product with any constraints and
//! requirements"*. So nothing in this module knows what a charger, a laptop, or
//! a marketplace is. It knows only:
//!
//! * a [`Constraint`] — some attribute the answer must supply, either as a
//!   nominal value or as a quantity with a unit and a comparison;
//! * a [`Candidate`] — something discovered by research that *supplies* some
//!   attributes and optionally carries an [`Offer`] (price, seller, source URL);
//! * a [`Plan`] — one *or more* candidates that jointly satisfy every
//!   constraint.
//!
//! The plan arity is the point. The reviewer asked that we *"provide not only
//! the most expensive option that has charger + multiple adapters"* but also
//! *"options with 2 separate items — conversion adapter and charger"*. A plan is
//! therefore a **set** of candidates, and [`OptionNetwork::ranked_plans`]
//! enumerates every *minimal* satisfying set, not just the single-item ones.
//!
//! ## Why this is a links network and not a scoring model
//!
//! [`VISION.md`](../VISION.md) requires the reasoning state to be an inspectable
//! associative network, and the reviewer restated it: *"building associative
//! links network (not neural net), just knowledge network with all facts we know
//! and possible options"*. So the network is not a private struct graph — it
//! projects onto [`crate::world_model::Context`], the project's existing
//! links-network state container:
//!
//! * [`OptionNetwork::target_context`] holds the constraints as links;
//! * [`OptionNetwork::current_context`] holds the facts research has established;
//! * [`OptionNetwork::unmet`] is their [`ContextDiff`] — the still-open part of
//!   the question, which is exactly what the next research turn should go and
//!   look for.
//!
//! That last point is what makes the research loop multi-turn without a
//! hardcoded turn script: research continues while the difference is non-empty
//! and the tier ladder is not exhausted.
//!
//! ## Tiers
//!
//! [`Tier`] ranks *provenance*, not price: an original part from the device's
//! own maker outranks a compatible part from an endorsed maker, which outranks
//! an unaffiliated part that merely matches the numbers. Research walks the
//! ladder in that order — the reviewer asked that we *"first prioritize finding
//! original/authentic"* and only *"if everything in that search fails"* fall
//! back. Presentation order is the opposite axis and is independent: plans are
//! listed **cheapest first**, per *"by default we should list cheaper options
//! first"*. Tier only breaks price ties.
//!
//! Every number here is fixed-point integer arithmetic, so a plan ranking is
//! byte-for-byte reproducible and carries no float drift.

use std::collections::{BTreeMap, BTreeSet};

use crate::links_format::format_lino_record;
use crate::world_model::{Context, ContextDiff};

/// Scale for fixed-point quantities and prices: thousandths of a unit.
///
/// Prices and electrical quantities both need sub-unit precision without float
/// drift, and a shared scale keeps the comparison and the total in the same
/// arithmetic. Thousandths cover currency minor units (2 places) and the
/// tolerances a spec sheet actually quotes (e.g. 2.375 A) with room to spare.
pub const SCALE: i64 = 1_000;

/// Convert a whole number of units into the fixed-point scale.
#[must_use]
pub const fn units(whole: i64) -> i64 {
    whole * SCALE
}

/// Convert a decimal written as `whole.thousandths` into the fixed-point scale.
#[must_use]
pub const fn milli(whole: i64, thousandths: i64) -> i64 {
    whole * SCALE + thousandths
}

/// How a candidate's supply must compare to the required value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Comparison {
    /// The supplied quantity must equal the requirement exactly.
    ///
    /// Correct for attributes where more is not better and is often harmful —
    /// a supply voltage, a socket size.
    Equal,
    /// The supplied quantity must be at least the requirement.
    ///
    /// Correct for headroom attributes — deliverable current, capacity.
    AtLeast,
    /// The supplied quantity must be at most the requirement.
    ///
    /// Correct for budget-shaped attributes — mass, footprint.
    AtMost,
}

/// What a constraint demands of one attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Demand {
    /// A nominal value that must match after normalization.
    Nominal(String),
    /// A quantity with a unit, compared per [`Comparison`].
    Quantity {
        /// Fixed-point value at [`SCALE`].
        value: i64,
        /// Unit symbol; supply and demand must agree on it.
        unit: String,
        /// How the supplied value must compare to `value`.
        comparison: Comparison,
    },
}

/// One requirement the answer must satisfy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    /// Attribute identifier, e.g. `output_voltage`. Never user-facing prose.
    pub attribute: String,
    /// What the attribute must supply.
    pub demand: Demand,
    /// Where this constraint came from, for auditability.
    pub source: Option<String>,
}

impl Constraint {
    /// A constraint on a nominal attribute.
    #[must_use]
    pub fn nominal(attribute: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            attribute: attribute.into(),
            demand: Demand::Nominal(value.into()),
            source: None,
        }
    }

    /// A constraint on a quantity attribute.
    #[must_use]
    pub fn quantity(
        attribute: impl Into<String>,
        value: i64,
        unit: impl Into<String>,
        comparison: Comparison,
    ) -> Self {
        Self {
            attribute: attribute.into(),
            demand: Demand::Quantity {
                value,
                unit: unit.into(),
                comparison,
            },
            source: None,
        }
    }

    /// Attach the evidence this constraint was derived from.
    #[must_use]
    pub fn from_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Whether `supply` meets this constraint.
    #[must_use]
    pub fn satisfied_by(&self, supply: &Supply) -> bool {
        match (&self.demand, supply) {
            (Demand::Nominal(required), Supply::Nominal(offered)) => {
                normalize(required) == normalize(offered)
            }
            (
                Demand::Quantity {
                    value,
                    unit,
                    comparison,
                },
                Supply::Quantity {
                    value: offered,
                    unit: offered_unit,
                },
            ) => {
                normalize(unit) == normalize(offered_unit)
                    && match comparison {
                        Comparison::Equal => offered == value,
                        Comparison::AtLeast => offered >= value,
                        Comparison::AtMost => offered <= value,
                    }
            }
            _ => false,
        }
    }
}

/// What a candidate supplies for one attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Supply {
    /// A nominal value.
    Nominal(String),
    /// A quantity at [`SCALE`] with its unit.
    Quantity {
        /// Fixed-point value at [`SCALE`].
        value: i64,
        /// Unit symbol.
        unit: String,
    },
}

impl Supply {
    /// A nominal supply.
    #[must_use]
    pub fn nominal(value: impl Into<String>) -> Self {
        Self::Nominal(value.into())
    }

    /// A quantity supply.
    #[must_use]
    pub fn quantity(value: i64, unit: impl Into<String>) -> Self {
        Self::Quantity {
            value,
            unit: unit.into(),
        }
    }

    /// Render for the links projection.
    fn render(&self) -> String {
        match self {
            Self::Nominal(value) => value.clone(),
            Self::Quantity { value, unit } => format!("{} {unit}", render_fixed(*value)),
        }
    }
}

/// Provenance rank of a candidate. Lower is more authentic.
///
/// The ladder is the reviewer's search order: the device maker's own part, then
/// a part the maker or the part's maker declares compatible, then an
/// unaffiliated part that only matches the numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// The original part from the subject's own manufacturer.
    Authentic,
    /// A part an official source declares compatible.
    OfficialCompatible,
    /// An unaffiliated part that matches the required attributes.
    GenericCompatible,
}

impl Tier {
    /// Every tier in research order, most authentic first.
    pub const LADDER: [Self; 3] = [
        Self::Authentic,
        Self::OfficialCompatible,
        Self::GenericCompatible,
    ];

    /// Stable identifier used in the links projection and in seed lookups.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Authentic => "authentic",
            Self::OfficialCompatible => "official_compatible",
            Self::GenericCompatible => "generic_compatible",
        }
    }

    /// The next tier to try when this one yields no satisfying plan.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Authentic => Some(Self::OfficialCompatible),
            Self::OfficialCompatible => Some(Self::GenericCompatible),
            Self::GenericCompatible => None,
        }
    }
}

/// A purchasable listing for a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// Price at [`SCALE`] in `currency`.
    pub price: i64,
    /// Currency code, e.g. `INR`.
    pub currency: String,
    /// Seller or marketplace identifier.
    pub seller: String,
    /// Source URL the listing was read from.
    pub url: String,
    /// Whether the listing was observed to be purchasable. `None` means the
    /// capture could not establish it — reported, never guessed.
    pub available: Option<bool>,
}

impl Offer {
    /// A listing with unknown availability.
    #[must_use]
    pub fn new(
        price: i64,
        currency: impl Into<String>,
        seller: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            price,
            currency: currency.into(),
            seller: seller.into(),
            url: url.into(),
            available: None,
        }
    }

    /// Record whether the listing was observed purchasable.
    #[must_use]
    pub const fn with_available(mut self, available: bool) -> Self {
        self.available = Some(available);
        self
    }
}

/// Something research found that supplies attributes and may be purchasable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Stable identifier, unique within a network.
    pub id: String,
    /// Provenance rank.
    pub tier: Tier,
    /// Attributes this candidate supplies.
    pub supplies: BTreeMap<String, Supply>,
    /// Listing, when one was found.
    pub offer: Option<Offer>,
}

impl Candidate {
    /// A candidate with no supplies and no offer yet.
    #[must_use]
    pub fn new(id: impl Into<String>, tier: Tier) -> Self {
        Self {
            id: id.into(),
            tier,
            supplies: BTreeMap::new(),
            offer: None,
        }
    }

    /// Record an attribute this candidate supplies.
    #[must_use]
    pub fn supplying(mut self, attribute: impl Into<String>, supply: Supply) -> Self {
        self.supplies.insert(attribute.into(), supply);
        self
    }

    /// Attach a listing.
    #[must_use]
    pub fn offered(mut self, offer: Offer) -> Self {
        self.offer = Some(offer);
        self
    }

    /// The constraints in `constraints` this candidate satisfies on its own.
    fn covers(&self, constraints: &[Constraint]) -> BTreeSet<String> {
        constraints
            .iter()
            .filter(|constraint| {
                self.supplies
                    .get(&constraint.attribute)
                    .is_some_and(|supply| constraint.satisfied_by(supply))
            })
            .map(|constraint| constraint.attribute.clone())
            .collect()
    }
}

/// One or more candidates that jointly satisfy every constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Candidate ids, sorted for determinism.
    pub items: Vec<String>,
    /// Sum of the items' prices, when every item carries an offer in one
    /// currency. `None` when any price is unknown — an unpriced plan is still a
    /// real option and is reported, it simply cannot be ranked by price.
    pub total: Option<i64>,
    /// Currency of `total`.
    pub currency: Option<String>,
    /// Weakest tier among the items — a plan is only as authentic as its least
    /// authentic part.
    pub tier: Tier,
}

impl Plan {
    /// Whether this plan needs more than one purchase.
    #[must_use]
    pub const fn is_composite(&self) -> bool {
        self.items.len() > 1
    }
}

/// The associative network of constraints, facts, and candidate options.
#[derive(Debug, Clone, Default)]
pub struct OptionNetwork {
    /// Identifier of the thing being sourced, e.g. a device model.
    subject: String,
    constraints: Vec<Constraint>,
    candidates: Vec<Candidate>,
}

/// Upper bound on how many separate purchases one plan may combine.
///
/// The reviewer's worked example needs two (a conversion adapter plus a supply).
/// Enumeration is over subsets, so the bound is what keeps that search
/// polynomial rather than exponential in the candidate count.
pub const MAX_PLAN_ITEMS: usize = 3;

impl OptionNetwork {
    /// An empty network for `subject`.
    #[must_use]
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            constraints: Vec::new(),
            candidates: Vec::new(),
        }
    }

    /// Record a constraint the answer must satisfy.
    pub fn require(&mut self, constraint: Constraint) {
        if let Some(existing) = self
            .constraints
            .iter_mut()
            .find(|existing| existing.attribute == constraint.attribute)
        {
            *existing = constraint;
            return;
        }
        self.constraints.push(constraint);
    }

    /// Record a candidate discovered by research.
    pub fn observe(&mut self, candidate: Candidate) {
        if let Some(existing) = self
            .candidates
            .iter_mut()
            .find(|existing| existing.id == candidate.id)
        {
            *existing = candidate;
            return;
        }
        self.candidates.push(candidate);
    }

    /// The subject being sourced.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The recorded constraints.
    #[must_use]
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// The recorded candidates.
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// Candidates at `tier`.
    #[must_use]
    pub fn at_tier(&self, tier: Tier) -> Vec<&Candidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.tier == tier)
            .collect()
    }

    /// The target state: one link per constraint.
    ///
    /// This is the goal context in the project's existing world-model sense, so
    /// the option search shares the state container the rest of the system
    /// reasons with instead of inventing a private one.
    #[must_use]
    pub fn target_context(&self) -> Context {
        let mut context = Context::new(format!("{}_target", self.subject));
        for constraint in &self.constraints {
            context.assert_link(&self.subject, &constraint_link(constraint));
        }
        context
    }

    /// The established state: one link per constraint some candidate satisfies.
    #[must_use]
    pub fn current_context(&self) -> Context {
        let mut context = Context::new(format!("{}_current", self.subject));
        for constraint in &self.constraints {
            if self.candidates.iter().any(|candidate| {
                !candidate
                    .covers(std::slice::from_ref(constraint))
                    .is_empty()
            }) {
                context.assert_link(&self.subject, &constraint_link(constraint));
            }
        }
        context
    }

    /// The still-open part of the question.
    ///
    /// A non-empty difference is the signal to keep researching: some constraint
    /// has no candidate supplying it yet.
    #[must_use]
    pub fn unmet(&self) -> ContextDiff {
        self.current_context().difference(&self.target_context())
    }

    /// Attributes no candidate supplies yet, in constraint order.
    ///
    /// The multi-turn research loop turns these into its next queries, which is
    /// why the loop needs no hardcoded turn script: it is driven by what is
    /// still missing.
    #[must_use]
    pub fn open_attributes(&self) -> Vec<String> {
        self.constraints
            .iter()
            .filter(|constraint| {
                !self.candidates.iter().any(|candidate| {
                    !candidate
                        .covers(std::slice::from_ref(constraint))
                        .is_empty()
                })
            })
            .map(|constraint| constraint.attribute.clone())
            .collect()
    }

    /// Whether every constraint is satisfied by at least one candidate.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.open_attributes().is_empty()
    }

    /// Every *minimal* satisfying plan, ranked cheapest first.
    ///
    /// Minimality matters: without it, any satisfying set plus an unrelated
    /// extra item would also "satisfy", and the answer would be padded with
    /// strictly worse bundles. A plan is kept only when dropping any one of its
    /// items breaks satisfaction, so a single sufficient item never appears
    /// again inside a larger bundle.
    ///
    /// Ranking is the reviewer's presentation rule: priced plans ascending by
    /// total, then fewer items, then more authentic tier, then id — the last two
    /// only to make ties deterministic. Unpriced plans sort last because they
    /// cannot be compared on the primary axis, not because they are worse.
    #[must_use]
    pub fn ranked_plans(&self) -> Vec<Plan> {
        let mut plans = self.satisfying_plans();
        plans.sort_by(|left, right| {
            price_key(left)
                .cmp(&price_key(right))
                .then(left.items.len().cmp(&right.items.len()))
                .then(left.tier.cmp(&right.tier))
                .then(left.items.cmp(&right.items))
        });
        plans
    }

    /// The cheapest satisfying plan, when one exists.
    #[must_use]
    pub fn best_plan(&self) -> Option<Plan> {
        self.ranked_plans().into_iter().next()
    }

    /// Enumerate minimal satisfying subsets up to [`MAX_PLAN_ITEMS`].
    fn satisfying_plans(&self) -> Vec<Plan> {
        if self.constraints.is_empty() {
            return Vec::new();
        }
        let required: BTreeSet<String> = self
            .constraints
            .iter()
            .map(|constraint| constraint.attribute.clone())
            .collect();
        let coverage: Vec<(usize, BTreeSet<String>)> = self
            .candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| (index, candidate.covers(&self.constraints)))
            .collect();

        let mut plans = Vec::new();
        let mut combination = Vec::new();
        self.walk(0, &mut combination, &coverage, &required, &mut plans);
        plans
    }

    /// Depth-first subset walk. `combination` holds candidate indices.
    fn walk(
        &self,
        start: usize,
        combination: &mut Vec<usize>,
        coverage: &[(usize, BTreeSet<String>)],
        required: &BTreeSet<String>,
        plans: &mut Vec<Plan>,
    ) {
        if !combination.is_empty() {
            let covered = union_of(combination, coverage);
            if required.iter().all(|attribute| covered.contains(attribute)) {
                // Minimal by construction: any proper subset that already
                // satisfied everything was reached earlier in this walk and
                // recorded, so we only keep sets where every item is load-bearing.
                if is_minimal(combination, coverage, required) {
                    plans.push(self.plan_for(combination));
                }
                // A satisfying set stays satisfying when extended, and every such
                // extension is non-minimal, so stop descending this branch.
                return;
            }
        }
        if combination.len() == MAX_PLAN_ITEMS {
            return;
        }
        for index in start..coverage.len() {
            combination.push(index);
            self.walk(index + 1, combination, coverage, required, plans);
            combination.pop();
        }
    }

    /// Build the plan record for a satisfying combination.
    fn plan_for(&self, combination: &[usize]) -> Plan {
        let mut items: Vec<String> = combination
            .iter()
            .map(|index| self.candidates[*index].id.clone())
            .collect();
        items.sort();
        let tier = combination
            .iter()
            .map(|index| self.candidates[*index].tier)
            .max()
            .unwrap_or(Tier::GenericCompatible);
        let offers: Vec<&Offer> = combination
            .iter()
            .filter_map(|index| self.candidates[*index].offer.as_ref())
            .collect();
        let priced = offers.len() == combination.len();
        let currency = offers.first().map(|offer| offer.currency.clone());
        let single_currency = currency.as_ref().is_some_and(|currency| {
            offers
                .iter()
                .all(|offer| normalize(&offer.currency) == normalize(currency))
        });
        let total = (priced && single_currency)
            .then(|| offers.iter().map(|offer| offer.price).sum::<i64>());
        Plan {
            items,
            currency: total.is_some().then(|| currency.unwrap_or_default()),
            total,
            tier,
        }
    }

    /// Project the whole network — constraints, facts, candidates, and ranked
    /// plans — as Links Notation.
    ///
    /// This is the auditable artifact: everything the answer rests on, in the
    /// project's reviewable text format, with each candidate's source URL
    /// retained so a reader can re-derive the claim.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut sections = vec![format_lino_record(
            "option_network",
            &[
                ("subject", self.subject.clone()),
                ("constraints", self.constraints.len().to_string()),
                ("candidates", self.candidates.len().to_string()),
                ("open", self.open_attributes().join(" ")),
            ],
        )];
        for constraint in &self.constraints {
            sections.push(format_lino_record(
                "constraint",
                &[
                    ("attribute", constraint.attribute.clone()),
                    ("demand", render_demand(&constraint.demand)),
                    ("source", constraint.source.clone().unwrap_or_default()),
                ],
            ));
        }
        for candidate in &self.candidates {
            let supplies = candidate
                .supplies
                .iter()
                .map(|(attribute, supply)| format!("{attribute}={}", supply.render()))
                .collect::<Vec<_>>()
                .join(" ");
            sections.push(format_lino_record(
                "candidate",
                &[
                    ("id", candidate.id.clone()),
                    ("tier", candidate.tier.id().to_owned()),
                    ("supplies", supplies),
                    (
                        "price",
                        candidate
                            .offer
                            .as_ref()
                            .map(|offer| {
                                format!("{} {}", render_fixed(offer.price), offer.currency)
                            })
                            .unwrap_or_default(),
                    ),
                    (
                        "url",
                        candidate
                            .offer
                            .as_ref()
                            .map(|offer| offer.url.clone())
                            .unwrap_or_default(),
                    ),
                ],
            ));
        }
        for (rank, plan) in self.ranked_plans().iter().enumerate() {
            sections.push(format_lino_record(
                "plan",
                &[
                    ("rank", (rank + 1).to_string()),
                    ("items", plan.items.join(" ")),
                    (
                        "total",
                        plan.total
                            .map(|total| {
                                format!(
                                    "{} {}",
                                    render_fixed(total),
                                    plan.currency.clone().unwrap_or_default()
                                )
                            })
                            .unwrap_or_default(),
                    ),
                    ("tier", plan.tier.id().to_owned()),
                    ("composite", plan.is_composite().to_string()),
                ],
            ));
        }
        sections.join("\n")
    }
}

/// Whether dropping any single item breaks satisfaction.
fn is_minimal(
    combination: &[usize],
    coverage: &[(usize, BTreeSet<String>)],
    required: &BTreeSet<String>,
) -> bool {
    if combination.len() <= 1 {
        return true;
    }
    !combination.iter().enumerate().any(|(position, _)| {
        let reduced: Vec<usize> = combination
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != position)
            .map(|(_, index)| *index)
            .collect();
        let covered = union_of(&reduced, coverage);
        required.iter().all(|attribute| covered.contains(attribute))
    })
}

/// Union of the attributes covered by a combination of candidates.
fn union_of(combination: &[usize], coverage: &[(usize, BTreeSet<String>)]) -> BTreeSet<String> {
    combination
        .iter()
        .flat_map(|index| coverage[*index].1.iter().cloned())
        .collect()
}

/// Sort key that puts priced plans first, ascending, and unpriced plans last.
fn price_key(plan: &Plan) -> (u8, i64) {
    plan.total.map_or((1, 0), |total| (0, total))
}

/// The link a constraint asserts in a context.
fn constraint_link(constraint: &Constraint) -> String {
    format!(
        "{}:{}",
        constraint.attribute,
        render_demand(&constraint.demand)
    )
}

/// Render a demand as a stable identifier.
fn render_demand(demand: &Demand) -> String {
    match demand {
        Demand::Nominal(value) => value.clone(),
        Demand::Quantity {
            value,
            unit,
            comparison,
        } => {
            let symbol = match comparison {
                Comparison::Equal => "=",
                Comparison::AtLeast => ">=",
                Comparison::AtMost => "<=",
            };
            format!("{symbol}{} {unit}", render_fixed(*value))
        }
    }
}

/// Render a fixed-point value at [`SCALE`] without float arithmetic.
fn render_fixed(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let magnitude = value.abs();
    let whole = magnitude / SCALE;
    let fraction = magnitude % SCALE;
    if fraction == 0 {
        return format!("{sign}{whole}");
    }
    let rendered = format!("{fraction:03}");
    format!("{sign}{whole}.{}", rendered.trim_end_matches('0'))
}

/// Case- and space-insensitive comparison key.
fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
