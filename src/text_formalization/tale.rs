//! A curated knowledge base for «Сказка о рыбаке и рыбке».
//!
//! This formalizes **plot facts** of Alexander Pushkin's *The Tale of the
//! Fisherman and the Fish* (1833, public domain) — who did what to whom, in what
//! order, under what modality — **not** the poem's verse, which is neither
//! reproduced nor required. The one short grounded span ([`TALE_GLOSS`]) is a
//! plain factual gloss authored here, used to demonstrate the [`Annotation`]
//! primitive with real character offsets.
//!
//! The result deliberately exercises at least one instance of every one of the
//! nine primitives, including a nested higher-order assertion (a demand whose
//! object is itself an assertion) and a [`Procedure`]; the
//! [`KnowledgeBase::coverage`] of the returned base satisfies
//! [`PrimitiveCoverage::covers_all_nine`].
//!
//! [`PrimitiveCoverage::covers_all_nine`]: super::knowledge_base::PrimitiveCoverage::covers_all_nine

use super::knowledge_base::KnowledgeBase;
use super::primitives::{
    Annotation, Assertion, Concept, Context, Entity, Modal, Predicate, PredicateRef, Procedure,
    Provenance, Temporal, Term,
};

/// Document identifier for the curated tale knowledge base.
pub const TALE_DOC_ID: &str = "tale:fisherman-and-fish";

/// A plain factual gloss (not Pushkin's verse) used to ground one assertion with
/// real character offsets via the [`Annotation`] primitive.
pub const TALE_GLOSS: &str = "Старик поймал золотую рыбку.";

/// Build the curated knowledge base for the tale.
///
/// The base carries concepts, entities, predicate declarations, a procedure,
/// contexts, a source annotation and a set of assertions (one of them nested),
/// so every primitive is represented.
#[must_use]
pub fn tale_knowledge_base() -> KnowledgeBase {
    let mut kb = KnowledgeBase::new(TALE_DOC_ID);

    // Concepts — abstract units of meaning that drive the plot.
    kb.push_concept(Concept::new("concept:greed", "жадность", "trait"));
    kb.push_concept(Concept::new("concept:wish", "желание", "mental_state"));
    kb.push_concept(Concept::new("concept:ransom", "выкуп", "abstract"));

    // Entities — the concrete referents of the tale.
    kb.push_entity(
        Entity::new("ent:old_man", "старик")
            .with_form("рыбак")
            .with_attribute("role", "protagonist"),
    );
    kb.push_entity(Entity::new("ent:old_woman", "старуха").with_attribute("role", "antagonist"));
    kb.push_entity(
        Entity::new("ent:golden_fish", "золотая рыбка")
            .with_form("рыбка")
            .with_attribute("power", "magical"),
    );
    kb.push_entity(Entity::new("ent:sea", "море").with_form("синее море"));
    kb.push_entity(Entity::new("ent:trough", "корыто").with_form("разбитое корыто"));

    // Predicate declarations — the reference catalogue of relations.
    kb.push_predicate(Predicate::new("pred:catch", "поймал"));
    kb.push_predicate(Predicate::new("pred:promise", "обещала"));
    kb.push_predicate(Predicate::new("pred:release", "отпустил"));
    kb.push_predicate(Predicate::new("pred:demand", "потребовала"));
    kb.push_predicate(
        Predicate::new("pred:make", "сделать").with_semantics("make(agent, patient, role)"),
    );
    kb.push_predicate(Predicate::new("pred:grant", "исполнила"));
    kb.push_predicate(Predicate::new("pred:remain", "осталась"));

    // Procedure — the escalation rule that the plot embodies.
    kb.push_procedure(
        Procedure::new(
            "proc:escalate",
            "escalate(wish) -> larger_wish",
            "после исполнения желания следующее требование возрастает",
        )
        .with_trigger("pred:grant"),
    );

    // Contexts — situations / bounds of validity.
    kb.push_context(
        Context::new("ctx:seaside")
            .with_label("У синего моря")
            .with_description("место действия сказки")
            .with_property("location", "у моря"),
    );
    kb.push_context(
        Context::new("ctx:final")
            .with_label("Финал")
            .with_description("возврат к исходному состоянию"),
    );

    // Annotation — a grounded source span for one fact.
    let gloss_len = TALE_GLOSS.chars().count();
    kb.push_annotation(
        Annotation::new("ann:tale:catch", TALE_DOC_ID, 0, gloss_len)
            .with_language("ru")
            .with_tokens(["Старик", "поймал", "золотую", "рыбку"]),
    );

    // Assertions — the atomic facts.

    // The old man catches the golden fish (grounded, temporally first).
    kb.push_assertion(
        Assertion::new(
            "a:catch",
            Term::entity("ent:old_man", "старик"),
            PredicateRef::new("pred:catch", "поймал"),
            Term::entity("ent:golden_fish", "золотая рыбка"),
        )
        .with_time(Temporal::relative("в начале сказки"))
        .with_context(Context::new("ctx:seaside").with_property("location", "у моря"))
        .with_provenance(
            Provenance::source(TALE_DOC_ID)
                .with_offsets(0, gloss_len)
                .with_extractor("curated_v1"),
        ),
    );

    // The fish promises a ransom (the abstract concept) to be set free.
    kb.push_assertion(
        Assertion::new(
            "a:promise",
            Term::entity("ent:golden_fish", "золотая рыбка"),
            PredicateRef::new("pred:promise", "обещала"),
            Term::concept("concept:ransom", "выкуп"),
        )
        .with_modal(Modal::assertion(0.95)),
    );

    // The old man releases the fish.
    kb.push_assertion(
        Assertion::new(
            "a:release",
            Term::entity("ent:old_man", "старик"),
            PredicateRef::new("pred:release", "отпустил"),
            Term::entity("ent:golden_fish", "золотая рыбка"),
        )
        .with_time(Temporal::relative("после того как поймал")),
    );

    // A nested, higher-order assertion: the old woman demands that the fish make
    // her the ruler of the sea. The object of the demand is itself an assertion.
    kb.push_assertion(
        Assertion::new(
            "b:make_ruler",
            Term::entity("ent:golden_fish", "золотая рыбка"),
            PredicateRef::new("pred:make", "сделать"),
            Term::literal("string", "владычица морская"),
        )
        .with_objects(vec![
            Term::entity("ent:old_woman", "старуха"),
            Term::literal("string", "владычица морская"),
        ])
        .with_modal(Modal::new("possibility", 0.5)),
    );
    kb.push_assertion(
        Assertion::new(
            "a:demand_sea",
            Term::entity("ent:old_woman", "старуха"),
            PredicateRef::new("pred:demand", "потребовала"),
            Term::assertion_ref("b:make_ruler"),
        )
        .with_modal(Modal::new("desire", 0.9)),
    );

    // The fish grants a wish (the abstract concept).
    kb.push_assertion(
        Assertion::new(
            "a:grant",
            Term::entity("ent:golden_fish", "золотая рыбка"),
            PredicateRef::new("pred:grant", "исполнила"),
            Term::concept("concept:wish", "желание"),
        )
        .with_modal(Modal::assertion(0.9)),
    );

    // The ending: the old woman is left with the broken trough.
    kb.push_assertion(
        Assertion::new(
            "a:final",
            Term::entity("ent:old_woman", "старуха"),
            PredicateRef::new("pred:remain", "осталась"),
            Term::entity("ent:trough", "разбитое корыто"),
        )
        .with_context(
            Context::new("ctx:final")
                .with_property("state", "исходное")
                .with_property("moral", "наказанная жадность"),
        ),
    );

    kb
}
