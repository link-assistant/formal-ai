//! Issue #932: the image survey is authored by Formal AI through the Agent CLI,
//! not hand-edited, and the decomposition that says so must keep holding.

const CANONICAL: &str = include_str!("../../data/meta/box-image-survey.lino");
const AUTHORED: &str =
    include_str!("../../docs/case-studies/issue-932/self-hosting-authorship/box-image-survey.lino");
const DECOMPOSITION: &str =
    include_str!("../../docs/case-studies/issue-932/self-hosting-authorship/decomposition.lino");
const AGENT_CLI_LOG: &str =
    include_str!("../../docs/case-studies/issue-932/self-hosting-authorship/agent-cli.log");
const RECIPE: &str = include_str!("../../experiments/issue_932_self_authoring/run.sh");
const CASE_STUDY: &str = include_str!("../../docs/case-studies/issue-932/README.md");
const REQUIREMENTS: &str =
    include_str!("../../docs/requirements/issue-0932-box-language-projects.md");

#[test]
fn issue_932_formal_ai_authored_survey_is_preserved_byte_for_byte() {
    assert_eq!(
        CANONICAL.as_bytes(),
        AUTHORED.as_bytes(),
        "data/meta/box-image-survey.lino drifted from the captured Agent-CLI artifact; \
         re-run experiments/issue_932_self_authoring/run.sh instead of editing it by hand"
    );
    for invariant in [
        "record_type \"box_image_survey\"",
        "namespace \"konard\"",
        "pinned_tag \"2.4.0\"",
        "evidence \"docs/case-studies/issue-932/raw-data/box-image-tags.log\"",
        "published \"true\"",
        "published \"false\"",
        "box-c",
    ] {
        assert!(CANONICAL.contains(invariant), "missing {invariant}");
    }
}

#[test]
fn issue_932_recipe_reproduces_the_artifact_through_the_real_agent_cli() {
    assert!(
        RECIPE.contains("experiments/agent_cli_e2e/run_agent_cli.sh"),
        "the recipe must drive the real Agent CLI, not a mock"
    );
    assert!(
        RECIPE.contains("cmp \"$ARTIFACT_DIR/box-image-survey.lino\" \"$CANONICAL\""),
        "the recipe must prove the committed file is the authored one"
    );
    for line in CANONICAL.lines() {
        assert!(
            RECIPE.contains(line),
            "the recipe's task does not ask for `{line}`"
        );
    }
    assert!(
        AGENT_CLI_LOG.contains("ses_"),
        "the captured session log must carry the real session id"
    );
}

#[test]
fn issue_932_decomposition_assigns_one_of_five_leaves_to_formal_ai() {
    assert!(DECOMPOSITION.contains("leaf_count \"5\""));
    assert!(DECOMPOSITION.contains("formal_ai_authored_leaf_count \"1\""));
    assert!(DECOMPOSITION.contains("formal_ai_authored_percent \"20\""));
    assert_eq!(
        DECOMPOSITION
            .matches("owner \"formal_ai_agent_cli\"")
            .count(),
        1
    );
    assert_eq!(
        DECOMPOSITION
            .matches("record_type \"smallest_leaf\"")
            .count(),
        5
    );
    assert_eq!(DECOMPOSITION.matches("status \"complete\"").count(), 5);
    assert!(
        DECOMPOSITION.contains("artifact \"data/meta/box-image-survey.lino\""),
        "the authored leaf must name the artifact the byte-for-byte test guards"
    );
}

#[test]
fn issue_932_documents_every_requirement() {
    for requirement in 1..=13 {
        let id = format!("R932-{requirement}");
        assert!(CASE_STUDY.contains(&id), "case study is missing {id}");
        assert!(
            REQUIREMENTS.contains(&id),
            "requirement shard is missing {id}"
        );
    }
}
