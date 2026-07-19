//! Reading options out of fetched pages — issue #781.
//!
//! [`formal_ai::option_network`] is only useful if something can fill it from
//! what research actually returns. These tests pin that reading step: which
//! numbers it takes, which it refuses, and that a whole set of purchase options
//! can be assembled from nothing but page text.

use formal_ai::option_evidence::{
    candidate_from_page, price_in, quantity_in, with_offer_from_page,
};
use formal_ai::option_network::{milli, units, Comparison, Constraint, OptionNetwork, Tier};

/// The unit carries the meaning, so the surrounding language does not matter.
///
/// This is the issue's own situation: the question arrived in Russian and the
/// useful spec sheets are written in it too.
#[test]
fn a_specification_is_read_the_same_whatever_language_surrounds_the_number() {
    let russian = "Блок питания для ноутбука. Выходное напряжение: 19.5 V, ток 2.31 A.";
    let english = "Laptop power adapter. Output voltage: 19.5 V, current 2.31 A.";

    assert_eq!(quantity_in(russian, "V"), Some(milli(19, 500)));
    assert_eq!(quantity_in(russian, "A"), Some(milli(2, 310)));
    assert_eq!(quantity_in(russian, "V"), quantity_in(english, "V"));
    assert_eq!(quantity_in(russian, "A"), quantity_in(english, "A"));
}

/// When the symbol itself is localised there is nothing to match, and the right
/// answer is to report nothing rather than to reach for the nearest number.
#[test]
fn a_localised_unit_symbol_yields_no_reading_instead_of_a_wrong_one() {
    let cyrillic = "Выходное напряжение: 19.5 В";

    assert_eq!(
        quantity_in(cyrillic, "V"),
        None,
        "a Cyrillic В is not a Latin V and must not be read as one"
    );
}

/// A unit symbol has to end where it ends, or every short symbol swallows the
/// first word that happens to start with it.
#[test]
fn a_unit_does_not_match_the_beginning_of_a_longer_word() {
    assert_eq!(quantity_in("shipped 2 Apr 2026", "A"), None);
    assert_eq!(quantity_in("cable 100 mm long", "m"), None);
    assert_eq!(quantity_in("cable 100 mm long", "mm"), Some(units(100)));
    assert_eq!(quantity_in("12 Volts", "V"), None);
}

/// Part numbers are alphanumeric runs that contain digits, and those digits are
/// not measurements. This one is real: Acer's `A13-045N2A` ends in `2A`, which
/// reads exactly like a 2-ampere rating and would have credited the part with a
/// current its page never stated — enough to make a genuine adapter look
/// underpowered and drop it out of the answer entirely.
#[test]
fn a_digit_inside_a_part_number_is_not_read_as_a_measurement() {
    assert_eq!(quantity_in("Acer adapter A13-045N2A", "A"), None);
    assert_eq!(quantity_in("model X200V revision", "V"), None);
    assert_eq!(
        quantity_in("A13-045N2A rated 2.31 A", "A"),
        Some(milli(2, 310)),
        "the real rating is still found once the part number is passed over"
    );
}

/// Prices are written with digit groups, and specifications with decimals; both
/// have to survive the trip into fixed point.
#[test]
fn grouped_digits_and_decimals_both_survive_parsing() {
    assert_eq!(price_in("Price: 1,299 INR", "INR"), Some(units(1299)));
    assert_eq!(price_in("Price: ₹1,299", "₹"), Some(units(1299)));
    assert_eq!(price_in("₹ 1 299 today", "₹"), Some(units(1299)));
    assert_eq!(quantity_in("output 3.25 A", "A"), Some(milli(3, 250)));
    assert_eq!(
        quantity_in("output 3.2519 A", "A"),
        Some(milli(3, 251)),
        "a fourth decimal is dropped, not rounded up across a boundary"
    );
}

/// The page decides what it states. An attribute it is silent about must stay
/// open, because a silently-filled attribute would close the question on a
/// candidate nobody verified.
#[test]
fn an_attribute_the_page_does_not_state_is_left_open() {
    let constraints = vec![
        Constraint::quantity("output_voltage", milli(19, 500), "V", Comparison::Equal),
        Constraint::quantity("output_current", milli(2, 310), "A", Comparison::AtLeast),
        Constraint::nominal("plug", "5.5x1.7"),
    ];
    let silent_about_current = "Adapter, output 19.5 V, barrel plug 5.5x1.7 mm.";

    let candidate = candidate_from_page("psu", Tier::Authentic, silent_about_current, &constraints);

    assert!(candidate.supplies.contains_key("output_voltage"));
    assert!(candidate.supplies.contains_key("plug"));
    assert!(
        !candidate.supplies.contains_key("output_current"),
        "the page never stated a current, so nothing may be recorded for it"
    );

    let mut network = OptionNetwork::new("charger");
    for constraint in constraints {
        network.require(constraint);
    }
    network.observe(candidate);

    assert_eq!(
        network.open_attributes(),
        vec!["output_current".to_string()]
    );
    assert!(
        network.best_plan().is_none(),
        "an unverified candidate must not be offered as a complete answer"
    );
}

/// A listing with no readable price is still an option — it is ranked after the
/// priced ones, not discarded, because "we could not read the price" is not the
/// same as "this does not exist".
#[test]
fn a_candidate_without_a_readable_price_is_kept_rather_than_dropped() {
    let constraints = vec![Constraint::quantity(
        "output_voltage",
        milli(19, 500),
        "V",
        Comparison::Equal,
    )];
    let page = "Genuine Acer adapter, 19.5 V. Contact seller for pricing.";

    let candidate = candidate_from_page("acer-oem", Tier::Authentic, page, &constraints);
    let candidate =
        with_offer_from_page(candidate, page, "INR", "acer", "https://acer.example/psu");

    assert!(candidate.offer.is_none());

    let mut network = OptionNetwork::new("charger");
    network.require(constraints[0].clone());
    network.observe(candidate);

    let plans = network.ranked_plans();
    assert_eq!(
        plans.len(),
        1,
        "the option still exists and must be offered"
    );
    assert!(plans[0].total.is_none());
}

/// The whole task, assembled from page text alone.
///
/// Three sources are read: the authentic Acer part, a cheaper official-brand
/// unit whose plug is the wrong size, and the conversion ring that makes that
/// plug fit. The expected result is the reviewer's: every option present, the
/// cheapest first, and the cheapest one made of **two** separate purchases
/// rather than a single expensive bundle.
#[test]
fn every_purchase_option_is_assembled_from_pages_and_listed_cheapest_first() {
    let mut network = OptionNetwork::new("acer aspire 3 a325-45 power supply");
    let constraints = vec![
        Constraint::quantity("output_voltage", milli(19, 500), "V", Comparison::Equal),
        Constraint::quantity("output_current", milli(2, 310), "A", Comparison::AtLeast),
        Constraint::nominal("plug", "5.5x1.7"),
    ];
    for constraint in &constraints {
        network.require(constraint.clone());
    }

    // The authentic part: it fits, and it is the most expensive thing here.
    let authentic = "Acer original adapter A13-045N2A: 19.5 V, 2.31 A, plug 5.5x1.7. ₹3,400";
    network.observe(with_offer_from_page(
        candidate_from_page("acer-a13-045n2a", Tier::Authentic, authentic, &constraints),
        authentic,
        "₹",
        "acer",
        "https://acer.example/a13",
    ));

    // An official unit from another manufacturer — right power, wrong plug.
    let official = "Delta ADP-45HE: output 19.5 V, 2.31 A, plug 5.5x2.5. ₹1,100";
    network.observe(with_offer_from_page(
        candidate_from_page(
            "delta-adp-45he",
            Tier::OfficialCompatible,
            official,
            &constraints,
        ),
        official,
        "₹",
        "delta",
        "https://delta.example/45he",
    ));

    // The ring that makes the wrong plug right.
    let ring = "Barrel conversion ring 5.5x2.5 to 5.5x1.7. ₹150";
    network.observe(with_offer_from_page(
        candidate_from_page(
            "ring-55x25-to-55x17",
            Tier::GenericCompatible,
            ring,
            &constraints,
        ),
        ring,
        "₹",
        "market",
        "https://market.example/ring",
    ));

    let plans = network.ranked_plans();

    assert_eq!(
        plans.len(),
        2,
        "both the authentic single purchase and the two-item combination are options"
    );

    let cheapest = &plans[0];
    assert!(
        cheapest.is_composite(),
        "the cheapest answer is two separate items, and must not be hidden behind the bundled one"
    );
    assert_eq!(
        cheapest.items,
        vec![
            "delta-adp-45he".to_string(),
            "ring-55x25-to-55x17".to_string()
        ]
    );
    assert_eq!(cheapest.total, Some(units(1250)));

    let authentic_plan = &plans[1];
    assert_eq!(authentic_plan.items, vec!["acer-a13-045n2a".to_string()]);
    assert_eq!(authentic_plan.total, Some(units(3400)));
    assert_eq!(authentic_plan.tier, Tier::Authentic);

    assert!(
        network.is_closed(),
        "every requirement was met by evidence read from the pages"
    );
}
