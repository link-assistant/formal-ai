//! Print the whole-repository source ↔ links projection document the source-links
//! agentic recipe emits (issue #558) — the fast representative-slice view, not the
//! exhaustive projection (see `project_source_links` for that).

fn main() {
    print!(
        "{}",
        formal_ai::agentic_coding::source_links::render_document()
    );
}
