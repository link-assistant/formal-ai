use super::tokenize;

#[test]
fn grouped_references_keep_their_quotes_and_spaces() {
    let tokens = tokenize(r#"row sample ("a b" "c")"#);
    assert_eq!(tokens, ["row", "sample", r#"("a b" "c")"#]);
    assert_eq!(
        crate::seed::parser::split_pipe_list(&tokens[2]),
        ["a b", "c"]
    );
    assert_eq!(
        tokenize(r#"row sample "a (b) c""#),
        ["row", "sample", "a (b) c"]
    );
}
