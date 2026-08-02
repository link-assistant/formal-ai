//! Probe: can the workspace self-AST census resolve the entry-point symbol of
//! every method the registry knows (issue #673)?

use std::fs;

use formal_ai::self_ast_census::workspace;

fn dispatch_sources() -> String {
    let mut joined = String::new();
    for path in ["src/solver_dispatch.rs", "src/meta_method_dispatch.rs"] {
        joined.push_str(&fs::read_to_string(path).expect("dispatch source"));
    }
    joined
}

/// The identifier the dispatch source binds to `name`.
fn entry_point(source: &str, name: &str, declared: &dyn Fn(&str) -> bool) -> Option<String> {
    let quoted = format!("\"{name}\"");
    let mut cursor = 0;
    while let Some(offset) = source[cursor..].find(&quoted) {
        let after = cursor + offset + quoted.len();
        let rest = source[after..].trim_start();
        let rest = rest
            .strip_prefix("=>")
            .or_else(|| rest.strip_prefix(','))
            .map(|rest| rest.trim_start().trim_start_matches('{').trim_start());
        if let Some(rest) = rest {
            let identifier: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if declared(&identifier) {
                return Some(identifier);
            }
        }
        cursor = after;
    }
    None
}

fn main() {
    let source = dispatch_sources();
    let census = workspace();
    let registry = formal_ai::method_registry::MethodRegistry::from_dispatch();
    let mut unresolved = 0;
    for method in &registry.methods {
        let declared = |identifier: &str| {
            !identifier.is_empty()
                && census
                    .modules_declaring(identifier)
                    .iter()
                    .any(|module| {
                        module
                            .symbol(identifier)
                            .is_some_and(|symbol| symbol.kind == "function")
                    })
        };
        match entry_point(&source, &method.name, &declared) {
            None => {
                println!("NO-ENTRY  {} ({})", method.name, method.surface.slug());
                unresolved += 1;
            }
            Some(symbol) => {
                let declaring = census.modules_declaring(&symbol);
                if declaring.is_empty() {
                    println!("NO-SYMBOL {} -> {symbol}", method.name);
                    unresolved += 1;
                } else {
                    let reference = format!("{}:{symbol}", declaring[0].path);
                    assert!(census.resolve(&reference).is_some(), "{reference}");
                }
            }
        }
    }
    println!(
        "{} methods, {unresolved} unresolved",
        registry.methods.len()
    );
}
