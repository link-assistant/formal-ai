//! Print the workspace self-AST census index and a sample module document
//! (issue #673), without writing anything.
//!
//! ```bash
//! cargo run --example dump_self_ast_census
//! cargo run --example dump_self_ast_census -- src/agentic_coding/source_links.rs
//! ```

use formal_ai::self_ast_census::workspace;

fn main() {
    let census = workspace();
    let mut args = std::env::args().skip(1);
    match args.next() {
        Some(reference) => match census.resolve(&reference) {
            Some(resolution) => {
                let module = census
                    .module(&resolution.module_path)
                    .expect("resolved module is censused");
                println!("# {}", resolution.reference());
                print!("{}", module.links_notation());
            }
            None => eprintln!("`{reference}` does not resolve in the workspace census"),
        },
        None => print!("{}", census.index_notation()),
    }
}
