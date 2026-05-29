fn main() {
    for expr in [
        "2+2",
        "2+2. what is 3+3",
        "2+2. 3+3",
        "8% of 500",
        "8% of 500. show me the code and the final result",
    ] {
        eprintln!("=== evaluating: {expr:?} ===");
        let mut c = link_calculator::Calculator::new();
        match c.calculate_with_value(expr) {
            Ok((_e, v, _s, _l)) => eprintln!("OK: {}", v.to_display_string()),
            Err(e) => eprintln!("ERR: {e}"),
        }
    }
}
