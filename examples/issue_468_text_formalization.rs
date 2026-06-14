//! Issue #468: a worked tour of deterministic text-to-knowledge formalization.
//! Run with:
//!   `cargo run --example issue_468_text_formalization`
//!
//! This walks Igor Martynov's "Formal protocol for translating texts into a
//! knowledge base" end to end, on the issue's canonical input — «Сказка о
//! рыбаке и рыбке». It shows, in order:
//!
//!   1. a curated knowledge base that exercises all nine primitives;
//!   2. that every primitive reduces to plain links/doublets — *everything is a
//!      link* — across three interchangeable renderings (Links Notation, the
//!      protocol's JSON wire format, and the fully reduced doublet stream);
//!   3. the protocol's declarative conjunctive query (article §9);
//!   4. a constrained, closed-class extractor reproducing the article's own
//!      worked example «Пётр открыл магазин в Москве в 2019 году.»;
//!   5. that the extractor never guesses — out-of-template input yields nothing;
//!   6. that the JSON wire format round-trips losslessly.
//!
//! Everything here is deterministic: there is no neural-network inference. The
//! case study scopes general open-domain extraction as future work and surveys
//! the prior art in `docs/case-studies/issue-468/`.

use formal_ai::{formalize_sentence, tale_knowledge_base, KbFormat, KnowledgeBase, Query};

fn rule(title: &str) {
    println!("\n=== {title} ===");
}

fn main() {
    // --- 1. Curated knowledge base: all nine primitives ---------------------
    let tale = tale_knowledge_base();
    let coverage = tale.coverage();
    rule("1. Curated tale knowledge base — coverage");
    println!("doc_id: {}", tale.doc_id);
    println!("covers all nine primitives: {}", coverage.covers_all_nine());
    println!(
        "  concepts={} entities={} predicates={} assertions={}",
        coverage.concepts, coverage.entities, coverage.predicates, coverage.assertions,
    );
    println!(
        "  procedures={} contexts={} temporals={} modals={} annotations={}",
        coverage.procedures,
        coverage.contexts,
        coverage.temporals,
        coverage.modals,
        coverage.annotations,
    );

    // --- 2. Everything is a link: three interchangeable renderings ----------
    rule("2a. Links Notation (structured, one record per primitive) — head");
    for line in tale.to_lino().lines().take(12) {
        println!("{line}");
    }

    rule("2b. Protocol JSON wire format — head");
    for line in KbFormat::Json.render(&tale).lines().take(12) {
        println!("{line}");
    }

    rule("2c. Fully reduced doublet stream — every primitive is source/target links");
    println!("total links (doublets): {}", tale.link_count());
    for line in tale.to_links_lino().lines().take(9) {
        println!("{line}");
    }

    // --- 3. Declarative conjunctive query (article §9) ----------------------
    rule("3. Declarative query — what did the old man act upon?");
    let acted_on = tale
        .query_text("SELECT ?target WHERE subject = ent:old_man")
        .expect("query parses");
    for assertion in &acted_on {
        println!(
            "  {} : {} -> {}",
            assertion.id,
            assertion.predicate_id(),
            assertion
                .object
                .iter()
                .map(formal_ai::Term::node_id)
                .collect::<Vec<_>>()
                .join(", "),
        );
    }

    // A programmatic query is equivalent to the textual form above.
    let possibilities = tale.query(&Query::new().with_subject("ent:golden_fish"));
    println!(
        "  golden fish is the subject of {} assertion(s); one is modal: {:?}",
        possibilities.len(),
        tale.assertion("b:make_ruler")
            .map(|a| a.modal.kind.as_str()),
    );

    // --- 4. Constrained deterministic extractor (article worked example) ----
    rule("4. Deterministic extractor — «Пётр открыл магазин в Москве в 2019 году.»");
    let sentence = "Пётр открыл магазин в Москве в 2019 году.";
    match formalize_sentence("doc-0001", sentence, KbFormat::Json) {
        Some(rendered) => {
            for line in rendered.lines().take(18) {
                println!("{line}");
            }
        }
        None => println!("  (no extraction)"),
    }

    // --- 5. The extractor never guesses -------------------------------------
    rule("5. Out-of-template input yields nothing (the extractor never guesses)");
    for probe in ["Кошка спит.", "Пётр закрыл магазин в Москве в 2019 году."]
    {
        let outcome = formalize_sentence("doc-0001", probe, KbFormat::Json);
        println!(
            "  {probe:<48} -> {}",
            if outcome.is_some() {
                "extracted"
            } else {
                "None"
            }
        );
    }

    // --- 6. Lossless JSON round-trip ----------------------------------------
    rule("6. JSON wire format round-trips losslessly");
    let json = tale.to_json_pretty();
    let reparsed = KnowledgeBase::from_json(&json).expect("valid protocol JSON");
    println!(
        "  tale.to_json -> from_json -> equal: {}",
        reparsed.to_json_pretty() == json,
    );
}
