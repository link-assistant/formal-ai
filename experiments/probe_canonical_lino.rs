// Ad-hoc check (issue #663): the shipped handler-precedence seed parses under the
// strict canonical Links Notation parser used by tests/unit/data_files.rs.
fn main() {
    let c = std::fs::read_to_string("data/seed/handler-precedence.lino").unwrap();
    match links_notation::parse_lino(c.trim()) {
        Ok(_) => println!("handler-precedence.lino: canonical parse OK"),
        Err(e) => { println!("handler-precedence.lino: ERR {e}"); std::process::exit(1); }
    }
}
