//! Plan 00 §4.1's `Need` contract (issue #1138, plan 00 leaf C1).
//!
//! The need record is the unit every step connects through, so it has exactly
//! one Rust home (`src/needs.rs`) and exactly one seed vocabulary
//! (`data/meta/need-contract.lino`). These tests pin both halves: the record
//! round-trips through Links Notation, and the Rust `NeedKind` slugs are the
//! vocabulary the contract file declares — not a second list that can drift.

use std::fs;
use std::path::Path;

use formal_ai::needs::{Need, NeedKind, NeedState};

const CONTRACT: &str = "data/meta/need-contract.lino";

/// Every variant of the contract enum, so neither side can quietly gain one.
const EVERY_KIND: [NeedKind; 7] = [
    NeedKind::Concept,
    NeedKind::Procedure,
    NeedKind::Part,
    NeedKind::Prerequisite,
    NeedKind::Evidence,
    NeedKind::Decision,
    NeedKind::None,
];

fn contract() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CONTRACT);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The `kind` slugs the contract file declares, in declaration order.
fn declared_kinds(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("kind "))
        .map(|value| value.trim().trim_matches('"').to_owned())
        .collect()
}

fn concept_need() -> Need {
    Need {
        need_id: "need:isogram".to_owned(),
        kind: NeedKind::Concept,
        subject: "isogram".to_owned(),
        language: "en".to_owned(),
        raised_by: "obligation:1".to_owned(),
        source_span: "doc:requirement@11:18".to_owned(),
        depth: 0,
        state: NeedState::Open,
        satisfied_by: None,
    }
}

#[test]
fn the_need_record_round_trips_through_links_notation() {
    let need = concept_need();
    let projection = need.to_links_notation();

    assert_eq!(
        projection,
        concat!(
            "need need:isogram\n",
            "  kind concept\n",
            "  subject \"isogram\"\n",
            "  language en\n",
            "  raised_by obligation:1\n",
            "  source_span \"doc:requirement@11:18\"\n",
            "  depth 0\n",
            "  state open\n",
        ),
        "the need projection is the record every step is remembered as"
    );
    assert_eq!(
        Need::from_links_notation(&projection).as_ref(),
        Some(&need),
        "a projected need must read back as the same record"
    );
}

#[test]
fn a_satisfied_need_names_the_evidence_that_satisfied_it() {
    let mut need = concept_need();
    need.state = NeedState::Satisfied;
    need.satisfied_by = Some("evidence:abc".to_owned());

    let projection = need.to_links_notation();
    assert!(
        projection.contains("  state satisfied\n  satisfied_by evidence:abc\n"),
        "satisfaction is only ever recorded with its evidence id: {projection}"
    );
    assert_eq!(Need::from_links_notation(&projection).as_ref(), Some(&need));
}

#[test]
fn need_kind_slugs_match_the_seed_vocabulary() {
    let declared = declared_kinds(&contract());
    assert_eq!(
        declared,
        vec![
            "concept",
            "procedure",
            "part",
            "prerequisite",
            "evidence",
            "decision",
            "none",
        ],
        "`{CONTRACT}` declares the one need-kind vocabulary"
    );

    let slugs = EVERY_KIND.map(NeedKind::slug).to_vec();
    assert_eq!(
        slugs, declared,
        "the Rust enum and the contract file must be one vocabulary, not two"
    );
    for slug in &declared {
        assert_eq!(
            NeedKind::from_seed(slug).slug(),
            slug.as_str(),
            "`{slug}` must round-trip through the seed reader"
        );
    }
}
