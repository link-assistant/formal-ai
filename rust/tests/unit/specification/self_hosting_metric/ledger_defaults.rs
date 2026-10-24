//! Shared release fields remain exact measurements through writes and archives.
use super::{commit, fixture_repo, metric_script};
use std::fs;

const SHARED: &str =
    "  release-defaults\n    metric_version \"3\"\n    target_percentage_basis_points \"389\"\n";

fn row(tag: &str, overrides: &str) -> String {
    [
        format!("  release\n    tag \"{tag}\"\n"),
        "    since \"v0\"\n    until \"source\"\n".to_owned(),
        "    self_authored_lines \"0\"\n    changed_lines \"100\"\n".to_owned(),
        "    self_authored_commits \"0\"\n    commits \"2\"\n".to_owned(),
        "    percentage_basis_points \"0\"\n    trailing_window \"3\"\n".to_owned(),
        "    trailing_percentage_basis_points \"0\"\n".to_owned(),
        overrides.to_owned(),
    ]
    .concat()
}

#[test]
fn archive_defaults_are_scoped_and_explicit_rows_override_them() {
    let repo = fixture_repo();
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    let directory = repo.join("data/meta/self-hosting-ledger-parts");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("0001.lino"),
        format!(
            "self-hosting-ledger-archive\n{SHARED}{}{}",
            row("v1", ""),
            row(
                "v2",
                "    metric_version \"2\"\n    target_percentage_basis_points \"17\"\n"
            )
        ),
    )
    .unwrap();
    fs::write(
        &ledger,
        format!(
            "self_hosting_ledger\n  archive \"self-hosting-ledger-parts/0001.lino\"\n{}",
            row("v3", "")
        ),
    )
    .unwrap();
    let rows = metric_script::read_release_rows(&ledger).unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row.metric_version)
            .collect::<Vec<_>>(),
        [3, 2, 1]
    );
    assert_eq!(
        rows.iter()
            .map(|row| row.target_percentage_basis_points)
            .collect::<Vec<_>>(),
        [Some(389), Some(17), None]
    );
    assert_eq!(
        rows.iter().map(|row| row.tag.as_str()).collect::<Vec<_>>(),
        ["v1", "v2", "v3"]
    );
    assert!(
        rows.iter()
            .all(|row| row.changed_lines == 100 && row.commits == 2)
    );
    fs::remove_dir_all(repo).unwrap();
}

#[test]
fn release_writer_omits_only_fields_that_match_declared_defaults() {
    let repo = fixture_repo();
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    fs::write(&ledger, format!("self_hosting_ledger\n{SHARED}")).unwrap();
    fs::write(repo.join("code.txt"), "changed source\n").unwrap();
    commit(&repo, "human change");
    let row = metric_script::record_release(&repo, &ledger, "v1.1.0", "v1.0.0", "HEAD", 3).unwrap();
    let text = fs::read_to_string(&ledger).unwrap();
    let written = text.split("  release\n").nth(1).unwrap();
    assert!(!written.contains("metric_version"));
    assert!(written.contains("target_percentage_basis_points \"0\""));
    assert_eq!(metric_script::read_release_rows(&ledger).unwrap(), [row]);
    fs::remove_dir_all(repo).unwrap();
}
