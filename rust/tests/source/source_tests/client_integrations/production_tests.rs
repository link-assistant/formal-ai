use std::fs;

use clap::{Args as _, Command, FromArgMatches as _};

use super::{
    DEFAULT_BASE_URL, TempConfigDir, WithFormalAiArgs, backup_path, ensure_backup, restore_backup,
};

#[test]
fn backup_round_trip_restores_original_file_presence_and_bytes() {
    let directory = TempConfigDir::new("backup-round-trip").unwrap();
    let path = directory.path.join("settings");
    let backup = backup_path(&path, ".bak");

    ensure_backup(&path, &backup).unwrap();
    fs::write(&path, "added").unwrap();
    ensure_backup(&path, &backup).unwrap();
    restore_backup(&path, &backup).unwrap();
    assert!(!path.exists());
    assert!(!backup.exists());

    fs::write(&path, "original\n").unwrap();
    ensure_backup(&path, &backup).unwrap();
    fs::write(&path, "overridden").unwrap();
    ensure_backup(&path, &backup).unwrap();
    restore_backup(&path, &backup).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), "original\n");
    assert!(!backup.exists());
}

#[test]
fn client_arguments_use_the_configured_default_server_url() {
    let command = WithFormalAiArgs::augment_args(Command::new("with"));
    let matches = command.try_get_matches_from(["with", "codex"]).unwrap();
    let args = WithFormalAiArgs::from_arg_matches(&matches).unwrap();
    assert_eq!(args.base_url, DEFAULT_BASE_URL);
}
