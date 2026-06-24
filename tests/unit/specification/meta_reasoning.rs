//! Issue #559: white-box recursive reasoning per work unit (R337).
//!
//! The recursive downward pass records *what* the meta core did at each node; the
//! reasoning records *why*. These tests pin that the reasoning is a faithful
//! parallel of the work-unit tree (one step per unit, same shape), that every
//! step carries a human-readable downward and upward thought, that an atomic leaf
//! names the method it resolves to (through the same route→method bridge the
//! evidence join uses), and that the whole thing is trace-only — it serializes to
//! Links Notation and changes neither the unit tree nor the resolved methods.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::WorkUnit;
use formal_ai::meta_reasoning::WorkUnitReasoning;
use formal_ai::method_registry::MethodRegistry;
use formal_ai::translation::formalize_prompt;

fn reasoning_for(prompt: &str, max_depth: u8) -> (WorkUnit, WorkUnitReasoning) {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let root = WorkUnit::from_formalization(&formalization, max_depth);
    let registry = MethodRegistry::from_dispatch();
    let reasoning = WorkUnitReasoning::for_unit(&root, &registry);
    (root, reasoning)
}

fn for_each<'a>(reasoning: &'a WorkUnitReasoning, visit: &mut dyn FnMut(&'a WorkUnitReasoning)) {
    visit(reasoning);
    for child in &reasoning.children {
        for_each(child, visit);
    }
}

#[test]
fn reasoning_tree_mirrors_the_work_unit_tree() {
    let (root, reasoning) = reasoning_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    // Same shape: one reasoning step per work unit, same ids, same children order.
    assert_eq!(
        reasoning.step_count(),
        root.unit_count(),
        "there must be exactly one reasoning step per work unit"
    );
    assert_eq!(reasoning.unit_id, root.unit_id);
    assert_eq!(reasoning.children.len(), root.children.len());
    for (child, unit) in reasoning.children.iter().zip(&root.children) {
        assert_eq!(child.unit_id, unit.unit_id);
        assert_eq!(child.depth, unit.depth);
    }
}

#[test]
fn every_step_carries_a_downward_and_upward_thought() {
    let (_root, reasoning) = reasoning_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    for_each(&reasoning, &mut |step| {
        assert!(
            !step.observation.is_empty(),
            "step {} must observe its span",
            step.unit_id
        );
        assert!(
            !step.decision.is_empty(),
            "step {} must record a decision slug",
            step.unit_id
        );
        assert!(
            !step.downward_rationale.is_empty(),
            "step {} must explain its downward decision",
            step.unit_id
        );
        assert!(
            !step.upward_rationale.is_empty(),
            "step {} must explain how its answer is composed",
            step.unit_id
        );
    });
}

#[test]
fn decision_slugs_are_drawn_from_the_known_set() {
    let (_root, reasoning) = reasoning_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let known = ["decompose", "direct_method", "single_need", "depth_bound"];
    for_each(&reasoning, &mut |step| {
        assert!(
            known.contains(&step.decision.as_str()),
            "step {} has an unknown decision slug `{}`",
            step.unit_id,
            step.decision
        );
        // A decomposed unit must have children; an atomic decision must not.
        if step.decision == "decompose" {
            assert!(
                !step.children.is_empty(),
                "a decomposed step must reason about sub-units"
            );
        } else {
            assert!(
                step.children.is_empty(),
                "an atomic step must be a reasoning leaf"
            );
        }
    });
}

#[test]
fn atomic_leaf_names_the_method_it_resolves_to() {
    // The program-writing leaf must reason its way to write_script via the alias
    // bridge, exactly as the solution-evidence join resolves it.
    let (_root, reasoning) = reasoning_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let mut methods = Vec::new();
    for_each(&reasoning, &mut |step| {
        if let Some(method) = &step.method {
            methods.push(method.clone());
        }
    });
    assert!(
        methods.iter().any(|m| m == "write_script"),
        "the program-writing leaf must reason to the write_script method, got {methods:?}"
    );
    assert!(
        methods.iter().any(|m| m == "translation"),
        "the translation leaf must reason to the translation method, got {methods:?}"
    );
}

#[test]
fn reasoning_serializes_to_links_notation_records() {
    let (_root, reasoning) = reasoning_for(
        "translate apple to Russian and write a hello world program in Python",
        4,
    );
    let lino = reasoning.to_links_notation();
    assert!(
        lino.contains("record_type \"work_unit_reasoning\""),
        "the reasoning must serialize as work_unit_reasoning records:\n{lino}"
    );
    assert!(
        lino.contains("downward_rationale"),
        "the serialized reasoning must carry the downward thought:\n{lino}"
    );
    assert!(
        lino.contains("upward_rationale"),
        "the serialized reasoning must carry the upward thought:\n{lino}"
    );
    // One record per reasoning step.
    let records = lino.matches("record_type \"work_unit_reasoning\"").count();
    assert_eq!(
        records,
        reasoning.step_count(),
        "every reasoning step must serialize to exactly one record"
    );
}

#[test]
fn reasoning_is_trace_only_and_does_not_alter_resolution() {
    // Building the reasoning must not change the work-unit tree or the methods the
    // registry resolves: the reasoning observes, it does not steer.
    let prompt = "translate apple to Russian";
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let root = WorkUnit::from_formalization(&formalization, 4);
    let registry = MethodRegistry::from_dispatch();

    let before = root.unit_count();
    let reasoning = WorkUnitReasoning::for_unit(&root, &registry);
    let after = WorkUnit::from_formalization(&formalization, 4).unit_count();

    assert_eq!(
        before, after,
        "reasoning must not mutate the work-unit tree"
    );
    // A single-intent prompt is one atomic leaf: one reasoning step that resolves
    // to the same method the registry would resolve directly.
    assert_eq!(reasoning.step_count(), 1);
    assert_eq!(
        reasoning.method.as_deref(),
        registry
            .method_for_route(root.route.as_deref().unwrap_or(""))
            .map(|m| m.name.as_str()),
        "the leaf's reasoned method must match the registry's direct resolution"
    );
}
