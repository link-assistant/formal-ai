//! Issue #781 review — sourcing an answer that must satisfy constraints.
//!
//! The review on pull request #795 asked for four properties the previous
//! research path did not have: a tier ladder that tries the authentic part
//! before any substitute, plans made of *more than one* purchase, cheapest-first
//! presentation, and an open-question signal that a multi-turn loop can be
//! driven by. It also asked that none of it be about chargers — so the charger
//! case appears here once, as the issue's own regression, and the generality is
//! proven separately on a subject that shares no vocabulary with it.

use formal_ai::option_network::{
    milli, units, Candidate, Comparison, Constraint, Offer, OptionNetwork, Supply, Tier,
    MAX_PLAN_ITEMS,
};

/// The issue's own case: a replacement power supply for a specific laptop.
///
/// Three constraints, because that is what actually decides fit — the voltage
/// must match exactly, the current must have headroom, and the plug has to go
/// into the socket.
fn laptop_supply_network() -> OptionNetwork {
    let mut network = OptionNetwork::new("acer_aspire_3_a325_45");
    network.require(
        Constraint::quantity("output_voltage", milli(19, 500), "V", Comparison::Equal)
            .from_source("https://example.invalid/spec"),
    );
    network.require(Constraint::quantity(
        "output_current",
        milli(2, 310),
        "A",
        Comparison::AtLeast,
    ));
    network.require(Constraint::nominal("connector", "barrel_5.5x1.7"));
    network
}

#[test]
fn authentic_part_is_found_before_any_substitute() {
    let mut network = laptop_supply_network();
    network.observe(
        Candidate::new("generic_supply", Tier::GenericCompatible)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(3, 420), "A"))
            .supplying("connector", Supply::nominal("barrel_5.5x1.7")),
    );
    network.observe(
        Candidate::new("original_supply", Tier::Authentic)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(2, 310), "A"))
            .supplying("connector", Supply::nominal("barrel_5.5x1.7")),
    );

    // Research order is the ladder, independent of what was observed first.
    assert_eq!(
        Tier::LADDER,
        [
            Tier::Authentic,
            Tier::OfficialCompatible,
            Tier::GenericCompatible
        ]
    );
    assert_eq!(Tier::Authentic.next(), Some(Tier::OfficialCompatible));
    assert_eq!(Tier::GenericCompatible.next(), None);

    let authentic = network.at_tier(Tier::Authentic);
    assert_eq!(authentic.len(), 1);
    assert_eq!(authentic[0].id, "original_supply");
}

#[test]
fn exact_constraints_reject_a_near_miss_and_headroom_constraints_accept_one() {
    let voltage = Constraint::quantity("output_voltage", milli(19, 500), "V", Comparison::Equal);
    assert!(voltage.satisfied_by(&Supply::quantity(milli(19, 500), "V")));
    // 19 V is close, and closeness is not fit: an exact-match attribute has no
    // tolerance, which is the whole reason it is modelled as Equal.
    assert!(!voltage.satisfied_by(&Supply::quantity(units(19), "V")));
    // A matching number in the wrong unit is not a match either.
    assert!(!voltage.satisfied_by(&Supply::quantity(milli(19, 500), "A")));

    let current = Constraint::quantity("output_current", milli(2, 310), "A", Comparison::AtLeast);
    assert!(current.satisfied_by(&Supply::quantity(milli(2, 310), "A")));
    assert!(current.satisfied_by(&Supply::quantity(milli(3, 420), "A")));
    assert!(!current.satisfied_by(&Supply::quantity(milli(1, 500), "A")));

    let connector = Constraint::nominal("connector", "barrel_5.5x1.7");
    assert!(connector.satisfied_by(&Supply::nominal("Barrel_5.5x1.7")));
    assert!(!connector.satisfied_by(&Supply::nominal("usb_c")));
    // A nominal demand is not satisfied by a quantity, or the other way round.
    assert!(!connector.satisfied_by(&Supply::quantity(units(1), "V")));
}

#[test]
fn two_separate_items_form_one_plan_when_neither_suffices_alone() {
    let mut network = laptop_supply_network();
    // A supply with the right electrical output but the wrong plug.
    network.observe(
        Candidate::new("usb_c_supply", Tier::OfficialCompatible)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(3, 420), "A"))
            .supplying("connector", Supply::nominal("usb_c"))
            .offered(Offer::new(
                units(1_800),
                "INR",
                "seller_a",
                "https://a.invalid",
            )),
    );
    // A conversion adapter that supplies only the plug.
    network.observe(
        Candidate::new("plug_adapter", Tier::GenericCompatible)
            .supplying("connector", Supply::nominal("barrel_5.5x1.7"))
            .offered(Offer::new(
                units(300),
                "INR",
                "seller_b",
                "https://b.invalid",
            )),
    );

    let plans = network.ranked_plans();
    assert_eq!(plans.len(), 1, "{plans:?}");
    let plan = &plans[0];
    assert!(plan.is_composite());
    assert_eq!(plan.items, vec!["plug_adapter", "usb_c_supply"]);
    assert_eq!(plan.total, Some(units(2_100)));
    assert_eq!(plan.currency.as_deref(), Some("INR"));
    // A plan is only as authentic as its least authentic part.
    assert_eq!(plan.tier, Tier::GenericCompatible);
}

#[test]
fn cheaper_options_are_listed_first_and_bundles_are_not_padded() {
    let mut network = laptop_supply_network();
    // An expensive original that needs nothing else.
    network.observe(
        Candidate::new("original_supply", Tier::Authentic)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(2, 310), "A"))
            .supplying("connector", Supply::nominal("barrel_5.5x1.7"))
            .offered(Offer::new(
                units(4_500),
                "INR",
                "official",
                "https://o.invalid",
            )),
    );
    // A cheaper generic that also needs nothing else.
    network.observe(
        Candidate::new("generic_supply", Tier::GenericCompatible)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(3, 420), "A"))
            .supplying("connector", Supply::nominal("barrel_5.5x1.7"))
            .offered(Offer::new(
                units(1_200),
                "INR",
                "market",
                "https://g.invalid",
            )),
    );
    // An unrelated extra that would pad a bundle if minimality were not enforced.
    network.observe(
        Candidate::new("plug_adapter", Tier::GenericCompatible)
            .supplying("connector", Supply::nominal("barrel_5.5x1.7"))
            .offered(Offer::new(units(300), "INR", "market", "https://b.invalid")),
    );

    let plans = network.ranked_plans();
    // Exactly the two self-sufficient supplies. The adapter satisfies a
    // constraint each of them already satisfies, so pairing it with either is
    // non-minimal and must not be offered as a third, worse option.
    assert_eq!(
        plans
            .iter()
            .map(|plan| plan.items.clone())
            .collect::<Vec<_>>(),
        vec![vec!["generic_supply"], vec!["original_supply"]],
        "{plans:?}"
    );
    // Cheapest first — the generic outranks the authentic here, because price is
    // the primary axis and tier only breaks ties.
    assert_eq!(plans[0].total, Some(units(1_200)));
    assert_eq!(plans[1].total, Some(units(4_500)));
    assert_eq!(
        network.best_plan().map(|plan| plan.items),
        Some(vec!["generic_supply".to_owned()])
    );
}

#[test]
fn an_unpriced_option_is_still_reported_but_ranks_after_priced_ones() {
    let mut network = OptionNetwork::new("subject");
    network.require(Constraint::nominal("attribute", "value"));
    network.observe(
        Candidate::new("unpriced", Tier::Authentic)
            .supplying("attribute", Supply::nominal("value")),
    );
    network.observe(
        Candidate::new("priced", Tier::GenericCompatible)
            .supplying("attribute", Supply::nominal("value"))
            .offered(Offer::new(units(10), "INR", "seller", "https://p.invalid")),
    );

    let plans = network.ranked_plans();
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].items, vec!["priced"]);
    // Reported, not dropped: an option whose price could not be established is
    // still a real option, it just cannot be compared on the primary axis.
    assert_eq!(plans[1].items, vec!["unpriced"]);
    assert_eq!(plans[1].total, None);
}

#[test]
fn mixed_currencies_leave_a_plan_total_unknown_rather_than_summed() {
    let mut network = OptionNetwork::new("subject");
    network.require(Constraint::nominal("left", "value"));
    network.require(Constraint::nominal("right", "value"));
    network.observe(
        Candidate::new("a", Tier::GenericCompatible)
            .supplying("left", Supply::nominal("value"))
            .offered(Offer::new(units(10), "INR", "seller", "https://a.invalid")),
    );
    network.observe(
        Candidate::new("b", Tier::GenericCompatible)
            .supplying("right", Supply::nominal("value"))
            .offered(Offer::new(units(10), "USD", "seller", "https://b.invalid")),
    );

    let plans = network.ranked_plans();
    assert_eq!(plans.len(), 1);
    // Adding 10 INR to 10 USD would produce a number that means nothing. No
    // conversion rate is available offline, so the total is reported unknown.
    assert_eq!(plans[0].total, None);
    assert_eq!(plans[0].currency, None);
}

#[test]
fn the_open_question_is_what_drives_the_next_research_turn() {
    let mut network = laptop_supply_network();
    assert!(!network.is_closed());
    assert_eq!(
        network.open_attributes(),
        vec!["output_voltage", "output_current", "connector"]
    );
    assert!(!network.unmet().is_empty());

    network.observe(
        Candidate::new("partial", Tier::OfficialCompatible)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(3, 420), "A")),
    );
    // The connector is still open, so a loop keyed on this signal keeps going —
    // and knows precisely what to look for next, without a scripted turn count.
    assert_eq!(network.open_attributes(), vec!["connector"]);
    assert!(!network.is_closed());
    assert!(network.ranked_plans().is_empty());

    network.observe(
        Candidate::new("plug_adapter", Tier::GenericCompatible)
            .supplying("connector", Supply::nominal("barrel_5.5x1.7")),
    );
    assert!(network.is_closed());
    assert!(network.unmet().is_empty());
    assert!(!network.ranked_plans().is_empty());
}

#[test]
fn restating_a_constraint_or_a_candidate_replaces_it_instead_of_duplicating() {
    let mut network = OptionNetwork::new("subject");
    network.require(Constraint::nominal("attribute", "old"));
    network.require(Constraint::nominal("attribute", "new"));
    assert_eq!(network.constraints().len(), 1);

    network.observe(Candidate::new("item", Tier::GenericCompatible));
    network.observe(
        Candidate::new("item", Tier::Authentic).supplying("attribute", Supply::nominal("new")),
    );
    assert_eq!(network.candidates().len(), 1);
    assert_eq!(network.candidates()[0].tier, Tier::Authentic);
    // Later evidence wins, so a second research turn that finds a better
    // description of the same thing corrects the network rather than forking it.
    assert!(network.is_closed());
}

#[test]
fn the_network_projects_onto_links_notation_with_its_sources_intact() {
    let mut network = laptop_supply_network();
    network.observe(
        Candidate::new("original_supply", Tier::Authentic)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(2, 310), "A"))
            .supplying("connector", Supply::nominal("barrel_5.5x1.7"))
            .offered(
                Offer::new(units(4_500), "INR", "official", "https://o.invalid")
                    .with_available(true),
            ),
    );

    let notation = network.links_notation();
    assert!(notation.contains("option_network"), "{notation}");
    assert!(notation.contains("acer_aspire_3_a325_45"), "{notation}");
    assert!(notation.contains("authentic"), "{notation}");
    // Fixed point, rendered without float drift.
    assert!(notation.contains("19.5"), "{notation}");
    assert!(notation.contains("2.31"), "{notation}");
    assert!(notation.contains("4500"), "{notation}");
    // Provenance survives into the artifact, so a reader can re-derive the claim.
    assert!(notation.contains("https://o.invalid"), "{notation}");
    assert!(
        notation.contains("https://example.invalid/spec"),
        "{notation}"
    );
    assert!(notation.contains("plan"), "{notation}");

    // The projection is a pure function of the network.
    assert_eq!(notation, network.links_notation());
}

#[test]
fn the_target_and_current_contexts_are_ordinary_world_model_contexts() {
    let mut network = laptop_supply_network();
    let target = network.target_context();
    assert_eq!(target.links().len(), 3);
    // Nothing is established before research runs.
    assert!(network.current_context().links().is_empty());

    network.observe(
        Candidate::new("original_supply", Tier::Authentic)
            .supplying("output_voltage", Supply::quantity(milli(19, 500), "V"))
            .supplying("output_current", Supply::quantity(milli(2, 310), "A"))
            .supplying("connector", Supply::nominal("barrel_5.5x1.7")),
    );
    assert_eq!(network.current_context().links().len(), 3);
    assert!(network.unmet().is_empty());
}

#[test]
fn plan_search_is_bounded_by_the_declared_item_limit() {
    let mut network = OptionNetwork::new("subject");
    // One attribute per candidate, one more attribute than a plan may combine.
    for index in 0..=MAX_PLAN_ITEMS {
        network.require(Constraint::nominal(format!("attribute_{index}"), "value"));
        network.observe(
            Candidate::new(format!("item_{index}"), Tier::GenericCompatible)
                .supplying(format!("attribute_{index}"), Supply::nominal("value")),
        );
    }
    assert!(network.is_closed(), "every attribute has a supplier");
    // Closed, yet no plan: satisfying it would need more separate purchases than
    // the bound allows. Reporting nothing is correct — reporting a partial
    // bundle as if it were sufficient would not be.
    assert!(network.ranked_plans().is_empty());
}

/// The generality check. Nothing below is a charger, a laptop, or a marketplace
/// listing; the vocabulary is disjoint from the issue's, and the engine is
/// unchanged. Per CONTRIBUTING, generality is proven with different wording,
/// not asserted.
#[test]
fn the_same_engine_sources_a_subject_that_shares_no_vocabulary_with_the_issue() {
    let mut network = OptionNetwork::new("darkroom_enlarger_lens_mount");
    network.require(Constraint::nominal("thread", "m39"));
    network.require(Constraint::quantity(
        "focal_length",
        units(50),
        "mm",
        Comparison::Equal,
    ));
    network.require(Constraint::quantity(
        "max_aperture",
        milli(2, 800),
        "f",
        Comparison::AtMost,
    ));

    // The manufacturer's own lens: complete, and expensive.
    network.observe(
        Candidate::new("factory_lens", Tier::Authentic)
            .supplying("thread", Supply::nominal("m39"))
            .supplying("focal_length", Supply::quantity(units(50), "mm"))
            .supplying("max_aperture", Supply::quantity(milli(2, 800), "f"))
            .offered(Offer::new(
                units(21_000),
                "JPY",
                "maker",
                "https://f.invalid",
            )),
    );
    // A third-party lens with the wrong thread, plus a ring that converts it.
    network.observe(
        Candidate::new("third_party_lens", Tier::GenericCompatible)
            .supplying("thread", Supply::nominal("m42"))
            .supplying("focal_length", Supply::quantity(units(50), "mm"))
            .supplying("max_aperture", Supply::quantity(milli(2, 000), "f"))
            .offered(Offer::new(units(6_000), "JPY", "shop", "https://t.invalid")),
    );
    network.observe(
        Candidate::new("thread_ring", Tier::GenericCompatible)
            .supplying("thread", Supply::nominal("m39"))
            .offered(Offer::new(units(900), "JPY", "shop", "https://r.invalid")),
    );

    let plans = network.ranked_plans();
    assert_eq!(plans.len(), 2, "{plans:?}");
    // The two-item conversion route is cheaper, so it leads — exactly the shape
    // the review asked for, on a subject the engine was never told about.
    assert!(plans[0].is_composite());
    assert_eq!(plans[0].items, vec!["third_party_lens", "thread_ring"]);
    assert_eq!(plans[0].total, Some(units(6_900)));
    assert!(!plans[1].is_composite());
    assert_eq!(plans[1].items, vec!["factory_lens"]);
    assert_eq!(plans[1].total, Some(units(21_000)));
    // An at-most constraint accepts a wider maximum aperture, and the ring's
    // thread overrides nothing — it supplies the one attribute the lens lacks.
    assert!(network.is_closed());
}
