use super::{
    extract_quantities, extract_requested_expression, extract_variable_assignments,
    substitute_variables,
};

#[test]
fn extracts_assignments_and_substitutes_expression() {
    let prompt = "If x = 2 and y = 5, what is the value of (x^4 + 2y^2) / 6?";
    let assignments = extract_variable_assignments(prompt);
    assert_eq!(
        assignments,
        vec![
            (String::from("x"), String::from("2")),
            (String::from("y"), String::from("5"))
        ],
    );
    let expression = extract_requested_expression(prompt).expect("expression");
    assert_eq!(
        substitute_variables(&expression, &assignments),
        "(2^4 + 2*5^2) / 6"
    );
}

#[test]
fn extracts_number_words_for_remainder_sale() {
    let values = extract_quantities(
        "Ducks lay 16 eggs. She eats three, bakes with four, and sells each for $2.",
    );
    assert_eq!(values, vec![16, 3, 4, 2]);
}
