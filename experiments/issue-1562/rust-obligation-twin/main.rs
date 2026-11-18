// Issue #1562: run the crate-independent half of
// rust/src/agentic_coding/work_item_citations.rs (obligation_text,
// obligation_text_keeping, is_citation) without building the formal-ai crate,
// so its output can be compared byte for byte with the JavaScript twin. The
// operands of each text (bound_output_literals, which needs the crate) come
// from the JavaScript twin. Built by ../rust-obligation-twin.sh.
mod citations {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR_GENERATED"), "/citations.rs"));
}

fn main() {
    // Each argument is a `*.full.txt` file beside its `*.operands`; print its
    // obligation text and its binding text, each after a header line.
    for path in std::env::args().skip(1) {
        let text = std::fs::read_to_string(&path).expect("readable fixture");
        let operands: Vec<String> = std::fs::read_to_string(path.replace(".full.txt", ".operands"))
            .expect("readable operands")
            .split('\0')
            .filter(|operand| !operand.is_empty())
            .map(str::to_owned)
            .collect();
        let obligations = citations::obligation_text(&text);
        let binding = citations::obligation_text_keeping(&text, &operands);
        assert_eq!(obligations.len(), text.len(), "byte offsets are preserved for {path}");
        assert_eq!(binding.len(), text.len(), "byte offsets are preserved for {path}");
        print!("=== {path}\n{obligations}=== binding\n{binding}");
    }
}
