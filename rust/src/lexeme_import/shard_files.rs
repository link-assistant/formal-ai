use std::fs;
use std::path::Path;

use super::{Shard, diagnostic};

/// Replace the complete generated shard set without leaving stale files.
///
/// Each new file is staged in the destination directory before being renamed,
/// then obsolete importer-owned shards are removed. Unrelated seed files are
/// never touched.
pub fn write_shards(directory: &Path, shards: &[Shard]) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| {
        diagnostic(
            "lexeme_import_cannot_create",
            &[
                ("path", &directory.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;

    for shard in shards {
        let staged = directory.join(format!(".{}.staged", shard.file_name));
        fs::write(&staged, &shard.content).map_err(|error| {
            diagnostic(
                "lexeme_import_cannot_stage",
                &[
                    ("path", &staged.display().to_string()),
                    ("error", &error.to_string()),
                ],
            )
        })?;
        fs::rename(&staged, directory.join(&shard.file_name)).map_err(|error| {
            diagnostic(
                "lexeme_import_cannot_install",
                &[("file", &shard.file_name), ("error", &error.to_string())],
            )
        })?;
    }

    let retained: std::collections::BTreeSet<&str> = shards
        .iter()
        .map(|shard| shard.file_name.as_str())
        .collect();
    let entries = fs::read_dir(directory).map_err(|error| {
        diagnostic(
            "lexeme_import_cannot_list",
            &[
                ("path", &directory.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            diagnostic(
                "lexeme_import_cannot_inspect",
                &[("error", &error.to_string())],
            )
        })?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if file_name.starts_with("meanings-lexicon-import")
            && Path::new(file_name)
                .extension()
                .is_some_and(|ext| ext == "lino")
            && !retained.contains(file_name)
        {
            fs::remove_file(entry.path()).map_err(|error| {
                diagnostic(
                    "lexeme_import_cannot_remove_stale",
                    &[("file", file_name), ("error", &error.to_string())],
                )
            })?;
        }
    }
    Ok(())
}
