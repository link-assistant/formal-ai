use super::{recipe_ci_result, staged_ready_result};
use serde_json::json;

#[test]
fn staged_readiness_never_substitutes_for_hosted_ci_evidence() {
    let reference = "https://github.com/fixture/project/pull/2";
    let path = "ci/test.yml";
    let receipt = json!({"schema":"recipe-staged-ready-v1","success":true,"complete":true,
        "reference":reference,"workflow":"test","path":path,"ready":true,"isDraft":false,
        "head":"a".repeat(40),"branch":"owned","checks":0,"ciVerified":false});
    assert!(staged_ready_result(&receipt.to_string(), reference, "test", path, true).is_some());
    assert!(recipe_ci_result(&receipt.to_string(), reference, "test", true).is_none());
    for (key, value) in [
        ("ciVerified", json!(true)),
        ("checks", json!(1)),
        ("path", json!(".github/workflows/test.yml")),
        ("ready", json!(false)),
        ("complete", json!(false)),
        ("success", json!(false)),
    ] {
        let mut changed = receipt.clone();
        changed[key] = value;
        assert!(staged_ready_result(&changed.to_string(), reference, "test", path, true).is_none());
    }
}
