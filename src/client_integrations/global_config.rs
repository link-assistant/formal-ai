//! Writing and undoing the global client configuration files.
//!
//! `formal-ai with --global <tool>` edits the client's own settings file in
//! place, so the rendering of those files — and the backups that make `--undo`
//! exact — lives apart from the per-run invocation rendering.

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde_json::Value;
use toml_edit::{value as toml_value, DocumentMut, Item, Table};

use crate::seed::{ClientIntegration, ConfigFormat};

use super::command::ensure_trailing_newline;
use super::{
    backup_path, codex_model_catalog, global_config_path, render_context, render_template,
    write_file, RenderContext, WithFormalAiArgs, EMPTY_BACKUP_SENTINEL, ERROR_PLACEHOLDER,
    RENDERED_PLACEHOLDER,
};

pub(super) fn write_global_config(
    integration: &ClientIntegration,
    args: &WithFormalAiArgs,
) -> Result<(), Box<dyn Error>> {
    let mut context = render_context(integration, args)?;
    let global_config = integration.global_config_for(&context.protocol);
    if !global_config.model_catalog_path.is_empty() {
        let catalog_path = global_config_path(&global_config.model_catalog_path)?;
        let catalog_backup = backup_path(&catalog_path, &global_config.backup_suffix);
        ensure_backup(&catalog_path, &catalog_backup)?;
        context.model_catalog_path = catalog_path.display().to_string();
        write_file(&catalog_path, &codex_model_catalog(&context.model)?)?;
    }
    let path = global_config_path(&global_config.path)?;
    let backup_path = backup_path(&path, &global_config.backup_suffix);
    ensure_backup(&path, &backup_path)?;
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let next = match global_config.format {
        ConfigFormat::Toml => {
            render_toml_settings(&global_config.toml_settings, &existing, &context)?
        }
        ConfigFormat::Json => {
            merge_json_settings(&global_config.json_settings, &existing, &context)?
        }
        ConfigFormat::ShellEnv => {
            merge_shell_env_config(&integration.id, global_config, &existing, &context)
        }
    };
    if next == existing {
        println!(
            "{} already configured at {}",
            integration.id,
            path.display()
        );
    } else {
        write_file(&path, &next)?;
        println!("configured {} at {}", integration.id, path.display());
    }
    for companion in &global_config.companion_files {
        write_companion_file(&integration.id, companion, &context)?;
    }
    Ok(())
}

/// Materialise a companion settings file belonging to the same global install.
///
/// Environment alone is not always a configuration: gemini-cli reads
/// `GEMINI_DEFAULT_AUTH_TYPE` as a *default* and still refuses to start headlessly
/// unless a settings file records the selection, so `--global` reported success and
/// the client then failed with `Invalid auth method selected.` (issue #909). The
/// companion is backed up before it is touched, exactly like the main file, so
/// `--undo` stays exact.
fn write_companion_file(
    integration_id: &str,
    companion: &crate::seed::ClientIntegrationCompanionFile,
    context: &RenderContext,
) -> Result<(), Box<dyn Error>> {
    let path = global_config_path(&companion.path)?;
    ensure_backup(&path, &backup_path(&path, &companion.backup_suffix))?;
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let next = match companion.format {
        ConfigFormat::Toml => render_toml_settings(&companion.toml_settings, &existing, context)?,
        ConfigFormat::Json => merge_json_settings(&companion.json_settings, &existing, context)?,
        // A companion carries settings; environment belongs to the profile block the
        // main config writes. The seed parser rejects a `shell_env` companion, so
        // reaching here means there is nothing of its own to write.
        ConfigFormat::ShellEnv => existing.clone(),
    };
    if next == existing {
        println!("{integration_id} already configured at {}", path.display());
    } else {
        write_file(&path, &next)?;
        println!("configured {integration_id} at {}", path.display());
    }
    Ok(())
}

pub(super) fn undo_global_config(
    integration: &ClientIntegration,
    args: &WithFormalAiArgs,
) -> Result<(), Box<dyn Error>> {
    let context = render_context(integration, args)?;
    let global_config = integration.global_config_for(&context.protocol);
    let path = global_config_path(&global_config.path)?;
    let config_backup_path = backup_path(&path, &global_config.backup_suffix);
    let mut restored = if config_backup_path.exists() {
        restore_backup(&path, &config_backup_path)?;
        true
    } else {
        false
    };
    if !global_config.model_catalog_path.is_empty() {
        let catalog_path = global_config_path(&global_config.model_catalog_path)?;
        let catalog_backup_path = backup_path(&catalog_path, &global_config.backup_suffix);
        if catalog_backup_path.exists() {
            restore_backup(&catalog_path, &catalog_backup_path)?;
            restored = true;
        }
    }
    for companion in &global_config.companion_files {
        let companion_path = global_config_path(&companion.path)?;
        let companion_backup = backup_path(&companion_path, &companion.backup_suffix);
        if companion_backup.exists() {
            restore_backup(&companion_path, &companion_backup)?;
            restored = true;
        }
    }
    if !restored {
        println!(
            "no formal-ai backup for {} at {}",
            integration.id,
            path.display()
        );
        return Ok(());
    }
    println!(
        "restored {} from {}",
        integration.id,
        config_backup_path.display()
    );
    Ok(())
}

fn restore_backup(path: &Path, backup_path: &Path) -> Result<(), Box<dyn Error>> {
    if !backup_path.exists() {
        return Ok(());
    }
    let backup = fs::read_to_string(backup_path)?;
    if backup == EMPTY_BACKUP_SENTINEL {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    } else {
        write_file(path, &backup)?;
    }
    fs::remove_file(backup_path)?;
    Ok(())
}

fn ensure_backup(path: &Path, backup_path: &Path) -> Result<(), Box<dyn Error>> {
    if backup_path.exists() {
        return Ok(());
    }
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        fs::copy(path, backup_path)?;
    } else {
        fs::write(backup_path, EMPTY_BACKUP_SENTINEL)?;
    }
    Ok(())
}

pub(super) fn render_toml_settings(
    settings: &[(String, String)],
    existing: &str,
    context: &RenderContext,
) -> Result<String, Box<dyn Error>> {
    let mut document = if existing.trim().is_empty() {
        DocumentMut::new()
    } else {
        existing.parse::<DocumentMut>()?
    };
    for (path, value) in settings {
        set_toml_string(
            document.as_table_mut(),
            &render_template(path, context),
            &render_template(value, context),
        )?;
    }
    Ok(ensure_trailing_newline(document.to_string()))
}

fn set_toml_string(
    table: &mut Table,
    dotted_path: &str,
    value: &str,
) -> Result<(), Box<dyn Error>> {
    let parts = dotted_path
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return Err("empty TOML setting path".into());
    };
    let parent = table_at_path_mut(table, parents);
    parent[*last] = toml_value(value);
    Ok(())
}

fn table_at_path_mut<'a>(mut table: &'a mut Table, parts: &[&str]) -> &'a mut Table {
    for part in parts {
        let item = table
            .entry(part)
            .or_insert_with(|| Item::Table(Table::new()));
        if !item.is_table() {
            *item = Item::Table(Table::new());
        }
        table = item.as_table_mut().expect("table item");
    }
    table
}

fn merge_json_settings(
    settings: &[(String, String)],
    existing: &str,
    context: &RenderContext,
) -> Result<String, Box<dyn Error>> {
    let mut base = if existing.trim().is_empty() {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(existing)?
    };
    let overlay = json_settings_value(settings, context)?;
    merge_json_value(&mut base, overlay);
    Ok(format!("{}\n", serde_json::to_string_pretty(&base)?))
}

pub(super) fn render_json_settings(
    settings: &[(String, String)],
    context: &RenderContext,
) -> Result<String, Box<dyn Error>> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&json_settings_value(settings, context)?)?
    ))
}

fn json_settings_value(
    settings: &[(String, String)],
    context: &RenderContext,
) -> Result<Value, Box<dyn Error>> {
    let mut value = Value::Object(serde_json::Map::new());
    for (path, setting_value) in settings {
        set_json_setting(&mut value, path, setting_value, context)?;
    }
    Ok(value)
}

fn set_json_setting(
    root: &mut Value,
    dotted_path: &str,
    value: &str,
    context: &RenderContext,
) -> Result<(), Box<dyn Error>> {
    let parts = dotted_path
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| render_template(part, context))
        .collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return Err("empty JSON setting path".into());
    };

    let mut current = root;
    for part in parents {
        let object = current
            .as_object_mut()
            .ok_or("JSON setting path conflicts with a scalar value")?;
        current = object
            .entry(part.clone())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
    }
    let object = current
        .as_object_mut()
        .ok_or("JSON setting path conflicts with a scalar value")?;
    let rendered = render_template(value, context);
    let setting = if let Some(literal) = rendered.strip_prefix("json:") {
        serde_json::from_str(literal)
            .map_err(|error| invalid_typed_json_setting_error(&rendered, &error))?
    } else {
        Value::String(rendered)
    };
    object.insert(last.clone(), setting);
    Ok(())
}

fn invalid_typed_json_setting_error(rendered: &str, error: &serde_json::Error) -> String {
    crate::seed::response_for("client_integration_invalid_typed_json_setting", "en")
        .unwrap_or_else(|| "client_integration_invalid_typed_json_setting".to_owned())
        .replace(RENDERED_PLACEHOLDER, rendered)
        .replace(ERROR_PLACEHOLDER, &error.to_string())
}

fn merge_json_value(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                match base_map.get_mut(&key) {
                    Some(base_value) => merge_json_value(base_value, overlay_value),
                    None => {
                        base_map.insert(key, overlay_value);
                    }
                }
            }
        }
        (base_value, overlay_value) => *base_value = overlay_value,
    }
}

fn merge_shell_env_config(
    integration_id: &str,
    global_config: &crate::seed::ClientIntegrationGlobalConfig,
    existing: &str,
    context: &RenderContext,
) -> String {
    let mut next = remove_managed_block(existing, integration_id);
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    let _ = writeln!(next, "# >>> formal-ai {integration_id}");
    for env in &global_config.shell_env {
        next.push_str("export ");
        next.push_str(&render_template(&env.key, context));
        next.push('=');
        next.push_str(&shell_double_quote(&render_template(&env.value, context)));
        next.push('\n');
    }
    let _ = writeln!(next, "# <<< formal-ai {integration_id}");
    next
}

fn remove_managed_block(existing: &str, tool: &str) -> String {
    let start = format!("# >>> formal-ai {tool}");
    let end = format!("# <<< formal-ai {tool}");
    let mut out = String::new();
    let mut skipping = false;
    for line in existing.lines() {
        if line == start {
            skipping = true;
            continue;
        }
        if skipping {
            if line == end {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn shell_double_quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
