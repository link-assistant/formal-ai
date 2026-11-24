use super::{format_lino_record, push_lino_node};
use crate::seed::parser::parse_lino_checked;

#[test]
fn nested_node_values_round_trip_through_the_production_seed_reader() {
    let value = "A \"quoted\" path \\n followed by a real newline\nwith a tab\t";
    let mut document = String::new();
    push_lino_node(&mut document, 0, "message", None);
    push_lino_node(&mut document, 2, "body", Some(value));
    assert_eq!(document.lines().count(), 2);

    let root = parse_lino_checked(&document).expect("parse formatted nested record");
    let message = &root.children[0];
    assert_eq!(message.name, "message");
    assert_eq!(message.find_child_value("body"), value);
}

#[test]
fn flat_record_values_preserve_literal_backslashes_and_line_breaks() {
    let value = "literal \\n\nreal line break";
    let document = format_lino_record("message", &[("body", value.to_owned())]);
    let root = parse_lino_checked(&document).expect("parse formatted flat record");
    assert_eq!(root.children[0].find_child_value("body"), value);
}
