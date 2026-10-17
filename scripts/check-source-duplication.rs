#!/usr/bin/env rust-script
//! Strict structural Rust clone candidates; execute only in hosted CI.
//! No baseline, maximum body size, same-file exclusion or generated-file exemption.
//! Opaque item macros and unreadable/unparseable inputs fail closed.
//! Usage: rust-script scripts/check-source-duplication.rs [--inventory] [ROOT...]
//! Native controls: rust-script --test scripts/check-source-duplication.rs
//! A structural candidate is not proof of semantic equivalence.
//!
//! cargo dependencies use versions already present in rust/Cargo.lock.
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! syn = { version = "=2.0.119", features = ["full", "visit", "visit-mut", "clone-impls"] }
//! quote = "=1.0.47"
//! proc-macro2 = { version = "=1.0.107", features = ["span-locations"] }
//! serde_json = "=1.0.151"
//! ```
use quote::ToTokens;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

#[derive(Default)]
struct Normalize {
    scopes: Vec<BTreeMap<String, syn::Ident>>,
    next: usize,
}
impl Normalize {
    fn bind(&mut self, ident: &mut syn::Ident) {
        let replacement = syn::Ident::new(&format!("__local_{}", self.next), ident.span());
        self.next += 1;
        self.scopes
            .last_mut()
            .expect("binding scope")
            .insert(ident.to_string(), replacement.clone());
        *ident = replacement;
    }
    fn resolve(&self, ident: &syn::Ident) -> Option<syn::Ident> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&ident.to_string()).cloned())
    }
}
impl VisitMut for Normalize {
    fn visit_pat_ident_mut(&mut self, node: &mut syn::PatIdent) {
        self.bind(&mut node.ident);
        if let Some((_, pattern)) = &mut node.subpat {
            self.visit_pat_mut(pattern);
        }
    }
    fn visit_expr_path_mut(&mut self, node: &mut syn::ExprPath) {
        if node.qself.is_none()
            && node.path.segments.len() == 1
            && let Some(replacement) = self.resolve(&node.path.segments[0].ident)
        {
            node.path.segments[0].ident = replacement;
        }
        visit_mut::visit_expr_path_mut(self, node);
    }
    fn visit_expr_let_mut(&mut self, node: &mut syn::ExprLet) {
        self.visit_expr_mut(&mut node.expr);
        self.visit_pat_mut(&mut node.pat);
    }
    fn visit_expr_for_loop_mut(&mut self, node: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut node.expr);
        self.scopes.push(BTreeMap::new());
        self.visit_pat_mut(&mut node.pat);
        self.visit_block_mut(&mut node.body);
        self.scopes.pop();
    }
    fn visit_arm_mut(&mut self, node: &mut syn::Arm) {
        self.scopes.push(BTreeMap::new());
        self.visit_pat_mut(&mut node.pat);
        if let Some((_, guard)) = &mut node.guard {
            self.visit_expr_mut(guard);
        }
        self.visit_expr_mut(&mut node.body);
        self.scopes.pop();
    }
    fn visit_expr_if_mut(&mut self, node: &mut syn::ExprIf) {
        self.scopes.push(BTreeMap::new());
        self.visit_expr_mut(&mut node.cond);
        self.visit_block_mut(&mut node.then_branch);
        self.scopes.pop();
        if let Some((_, otherwise)) = &mut node.else_branch {
            self.visit_expr_mut(otherwise);
        }
    }
    fn visit_expr_while_mut(&mut self, node: &mut syn::ExprWhile) {
        self.scopes.push(BTreeMap::new());
        self.visit_expr_mut(&mut node.cond);
        self.visit_block_mut(&mut node.body);
        self.scopes.pop();
    }
    fn visit_local_mut(&mut self, node: &mut syn::Local) {
        // The initializer sees outer bindings, not the declaration it initializes.
        if let Some(init) = &mut node.init {
            self.visit_expr_mut(&mut init.expr);
            if let Some((_, diverge)) = &mut init.diverge {
                self.visit_expr_mut(diverge);
            }
        }
        self.visit_pat_mut(&mut node.pat);
    }
    fn visit_block_mut(&mut self, node: &mut syn::Block) {
        self.scopes.push(BTreeMap::new());
        visit_mut::visit_block_mut(self, node);
        self.scopes.pop();
    }
    fn visit_expr_closure_mut(&mut self, node: &mut syn::ExprClosure) {
        self.scopes.push(BTreeMap::new());
        for pattern in &mut node.inputs {
            self.visit_pat_mut(pattern);
        }
        self.visit_expr_mut(&mut node.body);
        self.scopes.pop();
    }
    // Nested function items own their own binding environment.
    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {
        let mut inner = Self {
            scopes: vec![BTreeMap::new()],
            next: 0,
        };
        for input in &mut node.sig.inputs {
            if let syn::FnArg::Typed(input) = input {
                inner.visit_pat_mut(&mut input.pat);
            }
        }
        inner.visit_block_mut(&mut node.block);
    }
}

fn function_key(signature: &syn::Signature, block: &syn::Block) -> String {
    let mut normal = Normalize {
        scopes: vec![BTreeMap::new()],
        next: 0,
    };
    let mut patterns = Vec::new();
    for input in &signature.inputs {
        match input {
            syn::FnArg::Typed(input) => {
                let mut pattern = (*input.pat).clone();
                normal.visit_pat_mut(&mut pattern);
                patterns.push(pattern.to_token_stream().to_string());
            }
            syn::FnArg::Receiver(receiver) => patterns.push(receiver.to_token_stream().to_string()),
        }
    }
    let mut body = block.clone();
    normal.visit_block_mut(&mut body);
    format!(
        "{:?}|{}|{}|{}",
        patterns,
        signature.asyncness.is_some(),
        signature.unsafety.is_some(),
        body.to_token_stream()
    )
}
fn closure_key(closure: &syn::ExprClosure) -> String {
    let mut normal = Normalize {
        scopes: vec![BTreeMap::new()],
        next: 0,
    };
    let mut copy = closure.clone();
    for input in &mut copy.inputs {
        normal.visit_pat_mut(input);
    }
    normal.visit_expr_mut(&mut copy.body);
    // Capture mode and async/const/move distinctions are preserved.
    copy.to_token_stream().to_string()
}
#[derive(Default)]
struct Scan {
    sites: Vec<(String, String, usize)>,
    gaps: Vec<String>,
}
impl<'ast> Visit<'ast> for Scan {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.sites.push((
            function_key(&node.sig, &node.block),
            node.sig.ident.to_string(),
            node.sig.ident.span().start().line,
        ));
        visit::visit_item_fn(self, node);
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.sites.push((
            function_key(&node.sig, &node.block),
            node.sig.ident.to_string(),
            node.sig.ident.span().start().line,
        ));
        visit::visit_impl_item_fn(self, node);
    }
    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.sites.push((
                function_key(&node.sig, block),
                node.sig.ident.to_string(),
                node.sig.ident.span().start().line,
            ));
        }
        visit::visit_trait_item_fn(self, node);
    }
    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.sites.push((
            closure_key(node),
            String::from("<closure>"),
            node.or1_token.span.start().line,
        ));
        visit::visit_expr_closure(self, node);
    }
    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        self.gaps.push(format!(
            "opaque item macro at line {} requires source-bound expansion custody",
            node.mac.bang_token.span.start().line
        ));
        visit::visit_item_macro(self, node);
    }
}
fn scan(source: &str) -> Result<Scan, syn::Error> {
    let parsed = syn::parse_file(source)?;
    let mut result = Scan::default();
    result.visit_file(&parsed);
    Ok(result)
}
fn tracked_source_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
        ])
        .args(roots)
        .output()?;
    if !output.status.success() {
        return Err("cannot establish tracked/nonignored implementation source custody".into());
    }
    let listed = String::from_utf8(output.stdout)?;
    let mut sources = std::collections::BTreeSet::new();
    for file in listed.split('\0').filter(|file| !file.is_empty()) {
        let path = PathBuf::from(file);
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("implementation input requires regular-file custody".into());
        }
        sources.insert(path);
    }
    Ok(sources.into_iter().collect())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inventory = false;
    let mut roots = Vec::new();
    for argument in std::env::args().skip(1) {
        if argument == "--inventory" {
            inventory = true;
        } else if argument.starts_with('-') {
            return Err(format!(
                "unsupported option {argument}; baselines and exemptions are forbidden"
            )
            .into());
        } else {
            roots.push(PathBuf::from(argument));
        }
    }
    if roots.is_empty() {
        roots.push(PathBuf::from("rust/src"));
    }
    let sources = tracked_source_files(&roots)?;
    if sources.is_empty() {
        return Err("no implementation sources discovered".into());
    }
    let mut groups: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    let mut gaps = Vec::new();
    let mut count = 0;
    for source in &sources {
        let parsed = scan(&fs::read_to_string(source)?)?;
        for gap in parsed.gaps {
            gaps.push(serde_json::json!({"path":source,"reason":gap}));
        }
        for (key, name, line) in parsed.sites {
            count += 1;
            groups
                .entry(key)
                .or_default()
                .push(serde_json::json!({"path":source,"name":name,"line":line}));
        }
    }
    let clones: Vec<_> = groups.into_iter().filter(|(_, sites)| sites.len() > 1)
        .map(|(canonical_body,sites)|serde_json::json!({"canonical_body":canonical_body,"sites":sites})).collect();
    println!(
        "{}",
        serde_json::json!({"files":sources.len(),"callables":count,"clones":clones,"gaps":gaps,"native_validation":"hosted_execution"})
    );
    if !inventory && (!clones.is_empty() || !gaps.is_empty()) {
        return Err("strict source duplication policy failed".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn groups(source: &str) -> usize {
        let mut groups = BTreeMap::<String, usize>::new();
        for (key, _, _) in scan(source).expect("valid Rust fixture").sites {
            *groups.entry(key).or_default() += 1;
        }
        groups.values().filter(|count| **count > 1).count()
    }
    #[test]
    fn renamed_parameters_and_locals_and_comments_do_not_hide_same_file_clones() {
        assert_eq!(
            groups("fn a(x:u32)->u32{let y=x+1;y*2} fn b(v:u32)->u32{/* prose */let z=v+1;z*2}"),
            1
        );
    }
    #[test]
    fn methods_and_default_trait_methods_are_scanned() {
        assert_eq!(
            groups("struct A; impl A{fn a(x:u32)->u32{x+1}} trait B{fn b(y:u32)->u32{y+1}}"),
            1
        );
    }
    #[test]
    fn closures_are_scanned_independently() {
        assert_eq!(groups("fn host(){let a=|x:u32|x+1;let b=|y:u32|y+1;}"), 1);
    }
    #[test]
    fn long_bodies_cannot_escape() {
        let body = "x+=1;".repeat(100);
        assert_eq!(
            groups(&format!(
                "fn a(mut x:u32){{{body}}}fn b(mut x:u32){{{body}}}"
            )),
            1
        );
    }
    #[test]
    fn literals_fields_globals_operators_and_effects_are_preserved() {
        for (a, b) in [
            ("x+1", "x+2"),
            ("x.a", "x.b"),
            ("x+1", "x-1"),
            ("x+GLOBAL_A", "x+GLOBAL_B"),
            ("read(x)", "write(x)"),
        ] {
            assert_eq!(groups(&format!("fn a(x:u32){{{a}}}fn b(x:u32){{{b}}}")), 0);
        }
    }
    #[test]
    fn initializer_shadowing_retains_binding_identity() {
        assert_eq!(
            groups("fn a(x:u32){let x=x+1;let y=x+1;y} fn b(v:u32){let z=v+1;let q=z+1;q}"),
            1
        );
    }
    #[test]
    fn raw_strings_and_nested_comments_are_parsed_by_rust_grammar() {
        assert_eq!(
            groups(
                "fn a(){/* outer /* nested */ */r###\"{fn fake(){}}\"###}fn b(){r###\"{fn fake(){}}\"###}"
            ),
            1
        );
    }
    #[test]
    fn opaque_item_generation_is_an_explicit_gap() {
        assert!(
            !scan("macro_rules! generated{()=>{fn a(){}}} generated!();")
                .unwrap()
                .gaps
                .is_empty()
        );
    }
    #[test]
    fn invalid_rust_never_becomes_a_clean_inventory() {
        assert!(scan("fn broken( {").is_err());
    }
}
