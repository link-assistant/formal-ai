//! Read release identity from Cargo's package table and update only its version token.
//! Keep source spans so unrelated comments, line endings and dependency tables survive.

#![allow(dead_code)] // Each release script imports only the operations it needs.

use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{Document, Item, TableLike};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

fn parse(content: &str) -> Result<Document<&str>, String> {
    Document::parse(content).map_err(|error| format!("Invalid Cargo manifest: {error}"))
}

fn package(document: &Document<impl AsRef<str>>) -> Result<&dyn TableLike, String> {
    document
        .get("package")
        .and_then(Item::as_table_like)
        .ok_or_else(|| "Could not find a [package] table".to_owned())
}

fn literal(table: &dyn TableLike, key: &str) -> Result<String, String> {
    table
        .get(key)
        .and_then(Item::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "[package].{key} must be a non-empty literal string; inherited \
                 values ({key}.workspace = true) are not supported by release scripts"
            )
        })
}

pub fn package_info_from_str(content: &str) -> Result<PackageInfo, String> {
    let document = parse(content)?;
    let table = package(&document)?;
    Ok(PackageInfo {
        name: literal(table, "name")?,
        version: literal(table, "version")?,
    })
}

/// Badges need the package name even when the version is workspace-inherited.
pub fn read_package_name(manifest: &Path) -> Result<String, String> {
    let content = fs::read_to_string(manifest)
        .map_err(|error| format!("Failed to read {}: {error}", manifest.display()))?;
    let document = parse(&content).map_err(|error| format!("{}: {error}", manifest.display()))?;
    literal(package(&document)?, "name").map_err(|error| format!("{}: {error}", manifest.display()))
}

pub fn read_package_info(manifest: &Path) -> Result<PackageInfo, String> {
    let content = fs::read_to_string(manifest)
        .map_err(|error| format!("Failed to read {}: {error}", manifest.display()))?;
    package_info_from_str(&content).map_err(|error| format!("{}: {error}", manifest.display()))
}

pub fn update_package_version(manifest: &Path, version: &str) -> Result<(), String> {
    let mut content = fs::read_to_string(manifest)
        .map_err(|error| format!("Failed to read {}: {error}", manifest.display()))?;
    // Refuse missing or inherited identity before any write. Never replace a
    // dependency's version or convert a workspace inheritance declaration.
    package_info_from_str(&content).map_err(|error| format!("{}: {error}", manifest.display()))?;
    if version.is_empty() {
        return Err("The new package version must not be empty".to_owned());
    }
    let document = parse(&content)?;
    let span = package(&document)?
        .get("version")
        .and_then(Item::as_value)
        .and_then(toml_edit::Value::span)
        .ok_or_else(|| "The literal package version has no source span".to_owned())?;
    // Only the parsed value token changes. Rendering the whole document can
    // normalize CRLF, reorder dotted keys or add a missing final newline.
    content.replace_range(span, &toml_edit::Value::from(version).to_string());
    fs::write(manifest, content)
        .map_err(|error| format!("Failed to write {}: {error}", manifest.display()))
}

pub fn get_package_manifest_path(root_manifest: &Path) -> Result<PathBuf, String> {
    let content = fs::read_to_string(root_manifest)
        .map_err(|error| format!("Failed to read {}: {error}", root_manifest.display()))?;
    let document =
        parse(&content).map_err(|error| format!("{}: {error}", root_manifest.display()))?;
    if document
        .get("package")
        .and_then(Item::as_table_like)
        .is_some()
    {
        return Ok(root_manifest.to_path_buf());
    }
    let workspace = document
        .get("workspace")
        .and_then(Item::as_table_like)
        .ok_or_else(|| {
            format!(
                "Could not find [package] or [workspace] in {}",
                root_manifest.display()
            )
        })?;
    let members = workspace
        .get("members")
        .and_then(Item::as_array)
        .ok_or_else(|| {
            format!(
                "Could not find workspace members in {}",
                root_manifest.display()
            )
        })?;
    let base = root_manifest.parent().ok_or_else(|| {
        format!(
            "Could not determine parent directory for {}",
            root_manifest.display()
        )
    })?;
    for member in members {
        let member = member.as_str().ok_or_else(|| {
            format!(
                "Workspace members must be strings in {}",
                root_manifest.display()
            )
        })?;
        let manifest = base.join(member).join("Cargo.toml");
        let Ok(content) = fs::read_to_string(&manifest) else {
            continue;
        };
        let document =
            parse(&content).map_err(|error| format!("{}: {error}", manifest.display()))?;
        let Ok(table) = package(&document) else {
            continue;
        };
        if table.get("publish").is_some_and(|item| {
            item.as_bool() == Some(false)
                || item
                    .as_array()
                    .is_some_and(|registries| registries.is_empty())
        }) {
            continue;
        }
        return Ok(manifest);
    }
    Err(format!(
        "No publishable workspace members found in {}",
        root_manifest.display()
    ))
}
