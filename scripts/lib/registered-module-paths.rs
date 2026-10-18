//! Resolve real Rust module registrations from a crate root, including private
//! external test children. Comments and string literals cannot register modules.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn registered_paths(root: &Path, directory: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let Some(crate_root) = directory
        .ancestors()
        .find(|base| base.join("src/lib.rs").is_file() || base.join("src/main.rs").is_file())
    else {
        return Ok(BTreeSet::new());
    };
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let mut visited = BTreeSet::new();
    for entry in ["src/lib.rs", "src/main.rs"] {
        walk_file(&root, &crate_root.join(entry), None, &mut visited)?;
    }
    Ok(visited)
}

fn walk_file(
    root: &Path,
    file: &Path,
    module_dir: Option<&Path>,
    visited: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    if !file.is_file() {
        return Ok(());
    }
    let file = file.canonicalize().map_err(|error| error.to_string())?;
    if !file.starts_with(root) {
        return Err("ModuleRegistrationOutsideRepository".to_owned());
    }
    if !visited.insert(file.clone()) {
        return Ok(());
    }
    let source = fs::read_to_string(&file).map_err(|error| error.to_string())?;
    let syntax =
        syn::parse_file(&source).map_err(|error| format!("{}: {error}", file.display()))?;
    let parent = file
        .parent()
        .ok_or_else(|| "ModuleParentRequired".to_owned())?;
    let conventional = if matches!(
        file.file_name().and_then(|name| name.to_str()),
        Some("lib.rs" | "main.rs" | "mod.rs")
    ) {
        parent.to_path_buf()
    } else {
        parent.join(
            file.file_stem()
                .ok_or_else(|| "ModuleStemRequired".to_owned())?,
        )
    };
    walk_items(
        root,
        &syntax.items,
        parent,
        module_dir.unwrap_or(&conventional),
        visited,
    )
}

fn walk_items(
    root: &Path,
    items: &[syn::Item],
    source_dir: &Path,
    module_dir: &Path,
    visited: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    for item in items {
        match item {
            syn::Item::Mod(module) => {
                let explicit = module.attrs.iter().find_map(|attribute| {
                    if !attribute.path().is_ident("path") {
                        return None;
                    }
                    let syn::Meta::NameValue(value) = &attribute.meta else {
                        return None;
                    };
                    let syn::Expr::Lit(value) = &value.value else {
                        return None;
                    };
                    let syn::Lit::Str(value) = &value.lit else {
                        return None;
                    };
                    Some(value.value())
                });
                if let Some((_, items)) = &module.content {
                    let child = explicit.map_or_else(
                        || module_dir.join(module.ident.to_string()),
                        |path| module_dir.join(path),
                    );
                    walk_items(root, items, &child, &child, visited)?;
                } else {
                    let child = explicit.map_or_else(
                        || {
                            let path = module_dir.join(format!("{}.rs", module.ident));
                            if path.is_file() {
                                path
                            } else {
                                module_dir.join(module.ident.to_string()).join("mod.rs")
                            }
                        },
                        |path| source_dir.join(path),
                    );
                    walk_file(root, &child, None, visited)?;
                }
            }
            syn::Item::Macro(item) if item.mac.path.is_ident("include") => {
                // Dynamic OUT_DIR includes remain unknown and supply no registration witness.
                if let Ok(path) = syn::parse2::<syn::LitStr>(item.mac.tokens.clone()) {
                    walk_file(
                        root,
                        &source_dir.join(path.value()),
                        Some(module_dir),
                        visited,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_include_and_external_child_are_real_registration() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("module-owner-{stamp}"));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("tests/unit")).unwrap();
        fs::write(root.join("src/lib.rs"), "include!(\"modules.rs\");\n").unwrap();
        fs::write(root.join("src/modules.rs"), "mod owner;\n").unwrap();
        fs::write(
            root.join("src/owner.rs"),
            "#[cfg(test)]\n#[path = \"../tests/unit/child.rs\"]\nmod child;\n",
        )
        .unwrap();
        fs::write(root.join("tests/unit/child.rs"), "// real child\n").unwrap();
        let child = root.join("tests/unit/child.rs").canonicalize().unwrap();
        let targets = registered_paths(&root, &root.join("tests/unit")).unwrap();
        assert!(targets.contains(&child));
        fs::write(
            root.join("src/lib.rs"),
            "// mod owner;\nconst TEXT: &str = \"mod owner;\";\n",
        )
        .unwrap();
        assert!(
            !registered_paths(&root, &root.join("tests/unit"))
                .unwrap()
                .contains(&child),
            "an orphan module or quoted declaration supplies no witness"
        );
        fs::write(root.join("src/lib.rs"), "mod owner;\n").unwrap();
        fs::write(
            root.join("src/owner.rs"),
            "// #[path = \"../tests/unit/child.rs\"]\n// mod child;\n",
        )
        .unwrap();
        assert!(
            !registered_paths(&root, &root.join("tests/unit"))
                .unwrap()
                .contains(&child),
            "commented declarations supply no witness"
        );
        fs::write(
            root.join("src/lib.rs"),
            "include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));\n",
        )
        .unwrap();
        assert!(
            !registered_paths(&root, &root.join("tests/unit"))
                .unwrap()
                .contains(&child),
            "unknown generated includes supply no fabricated witness"
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
#[test]
fn binary_root_owns_external_test_children_without_a_library() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("binary-module-owner-{stamp}"));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests/unit")).unwrap();
    fs::write(root.join("src/main.rs"), "mod cli;\nfn main() {}\n").unwrap();
    fs::write(
        root.join("src/cli.rs"),
        "#[cfg(test)]\n#[path = \"../tests/unit/child.rs\"]\nmod child;\n",
    )
    .unwrap();
    fs::write(root.join("tests/unit/child.rs"), "// real binary child\n").unwrap();
    let child = root.join("tests/unit/child.rs").canonicalize().unwrap();
    assert!(
        registered_paths(&root, &root.join("tests/unit"))
            .unwrap()
            .contains(&child)
    );
    fs::write(root.join("src/lib.rs"), "// independent library\n").unwrap();
    assert!(
        registered_paths(&root, &root.join("tests/unit"))
            .unwrap()
            .contains(&child)
    );
    fs::write(root.join("src/main.rs"), "// mod cli;\nfn main() {}\n").unwrap();
    assert!(
        !registered_paths(&root, &root.join("tests/unit"))
            .unwrap()
            .contains(&child)
    );
    fs::remove_dir_all(root).unwrap();
}
