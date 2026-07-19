use super::*;
use crate::seed::parser::{parse_lino, unescape_value};
use lino_objects_codec::format::parse_indented;

/// The writer and the reader have to agree on the escape alphabet.
///
/// `sanitize_lino_value` escapes `\r`, `\n` and `\t`, because a value that
/// carries one of them would otherwise break the line-oriented documents
/// `seed::parser` reads back. But the decoder only ever learned `\n`: an
/// unknown escape falls through to the catch-all arm, which pushes the
/// backslash *and* the letter back verbatim. So a tab survives the write and
/// dies on the read — it returns as the two characters `\` and `t`.
///
/// This matters for the subject of issue #715 rather than in the abstract.
/// A substitution rule set is written with these escapes and read back with
/// `SubstitutionRuleSet::from_links_notation`, and the rules carry fragments of
/// real code. Tabs are not exotic there: a Makefile recipe line is *required*
/// to begin with one, and Go is conventionally tab-indented. A rewrite of such
/// a fragment would round-trip into a literal backslash-t and silently stop
/// matching the code it was derived from.
#[test]
fn every_escape_the_writer_emits_is_one_the_reader_decodes() {
    let mut broken: Vec<String> = Vec::new();
    for (name, value) in [
        ("newline", "a\nb"),
        ("tab", "a\tb"),
        ("carriage return", "a\rb"),
        ("makefile recipe", "build:\n\tcargo build\n"),
        ("tab-indented go", "func main() {\n\tprintln(\"hi\")\n}"),
        // A backslash is content too, and the writer never escapes it — so
        // these arrive at the decoder looking exactly like its own escapes.
        ("rust newline literal", r#"println!("\n")"#),
        ("regex", r"\d+\s*"),
        ("windows path", r"C:\new\table"),
        ("latex", r"\rightarrow \ldots"),
    ] {
        let round_tripped = unescape_value(&sanitize_lino_value(value));
        if round_tripped != value {
            broken.push(format!("  {name}: {value:?} -> {round_tripped:?}"));
        }
    }
    assert!(
        broken.is_empty(),
        "every value the writer sanitized must decode back to itself, but:\n{}",
        broken.join("\n")
    );
}

/// The same asymmetry through a document, which is how the product hits it:
/// `push_lino_node` writes the value, `seed::parser` reads it back.
#[test]
fn a_tab_survives_a_document_round_trip() {
    let mut document = String::new();
    push_lino_node(&mut document, 0, "rule", None);
    push_lino_node(&mut document, 2, "pattern", Some("build:\n\tcargo build"));

    let tree = parse_lino(&document);
    let read_back = tree.children[0].find_child_value("pattern");
    assert_eq!(
        read_back, "build:\n\tcargo build",
        "the tab in a Makefile recipe must come back a tab, not a literal \
         backslash-t; document was {document:?}"
    );
}

/// A rule document has *two* readers, and the escape has to survive both.
///
/// `SubstitutionRuleSet::from_links_notation` runs the grammar
/// (`parse_indented`) as a well-formedness gate and only then decodes with
/// `seed::parser`, so a value that the seed parser round-trips is still lost if
/// the grammar rejects the document it arrived in. The two readers disagree
/// about the backslash by construction — the grammar has no backslash escape at
/// all, and treats one as an ordinary character — which is exactly why this
/// pairing needs asserting rather than reasoning about: the escape is only safe
/// because the grammar's tree is *discarded*, leaving one decoder, not two.
///
/// The values below are the ones where the two dialects meet: a quote forces
/// the codec off its default delimiter, and a backslash is the character the
/// dialects define differently.
#[test]
fn a_rule_document_satisfies_both_of_its_readers() {
    let mut broken: Vec<String> = Vec::new();
    for (name, value) in [
        ("rust newline literal", r#"println!("\n")"#),
        ("windows path", r"C:\new\table"),
        ("makefile recipe", "build:\n\tcargo build"),
        ("quote and backslash", r#"re.match("\d+", s)"#),
        ("both delimiters", r#"it's a "quote" and a \ backslash"#),
    ] {
        let mut document = String::new();
        push_lino_node(&mut document, 0, "rule", None);
        push_lino_node(&mut document, 2, "pattern", Some(value));

        if let Err(error) = parse_indented(document.trim()) {
            broken.push(format!("  {name}: the grammar rejects it: {error:?}"));
            continue;
        }
        let tree = parse_lino(&document);
        let read_back = tree.children[0].find_child_value("pattern");
        if read_back != value {
            broken.push(format!("  {name}: {value:?} -> {read_back:?}"));
        }
    }
    assert!(
        broken.is_empty(),
        "a rule document must pass the grammar gate and decode back to itself, but:\n{}",
        broken.join("\n")
    );
}
