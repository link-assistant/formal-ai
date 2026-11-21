use super::*;

#[test]
fn forwards_only_well_formed_recipe_chat_prompts() {
    let request =
        |prompt| serde_json::json!({"messages":[{"role":"user","content":prompt}]}).to_string();
    assert!(is_recipe_chat(&request(
        "Run recipe example/normalize on \"teh\"."
    )));
    assert!(is_recipe_chat(&request(
        "Use recipe example/normalize to rewrite \"teh\"."
    )));
    assert!(!is_recipe_chat(&request(
        "Run recipe example/normalize on \"a\" or \"b\"."
    )));
    assert!(!is_recipe_chat(&request(
        "Describe recipe example/normalize"
    )));
}

#[test]
fn rejects_incomplete_upstream_responses() {
    let response = decode_response(
        b"HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
    )
    .unwrap();
    assert_eq!(response.status_code, 403);
    assert_eq!(response.body, "{}");
    assert!(
        decode_response(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: 2\r\n\r\n!"
        )
        .is_err()
    );
}
