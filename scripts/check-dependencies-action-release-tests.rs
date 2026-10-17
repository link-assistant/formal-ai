use super::*;

const OLD: &str = "1111111111111111111111111111111111111111";
const CURRENT: &str = "2222222222222222222222222222222222222222";
const TAG: &str = "3333333333333333333333333333333333333333";

fn dependency() -> Dependency {
    Dependency {
        ecosystem: Ecosystem::GitHubAction,
        name: "owner/action/path".to_string(),
        declared: Some(OLD.to_string()),
        resolved: OLD.to_string(),
        path: PathBuf::from("workflow.yml"),
        line: 1,
        blocked: None,
    }
}

fn release() -> ActionRelease {
    ActionRelease {
        tag: "v2.3.4".to_string(),
        commit: CURRENT.to_string(),
        source_url: action_release_url("owner/action/path"),
        tag_source_url: tag_reference_url("owner/action/path", "v2.3.4"),
    }
}

fn snapshot() -> Snapshot {
    Snapshot {
        refreshed_at: "2026-10-17T00:00:00Z".to_string(),
        entries: [(
            (Ecosystem::GitHubAction, "owner/action/path".to_string()),
            (
                "v2.3.4".to_string(),
                action_release_url("owner/action/path"),
            ),
        )]
        .into_iter()
        .collect(),
        action_releases: [("owner/action/path".to_string(), release())]
            .into_iter()
            .collect(),
    }
}

#[test]
fn immutable_current_stale_missing_and_unresolved_are_distinct() {
    let mut item = dependency();
    let mut captured = snapshot();
    let (stale, unknown) = compare(&[item.clone()], &captured);
    assert_eq!(stale[0].latest, CURRENT);
    assert!(unknown.is_empty());
    assert_eq!(comparison_exit_code(&stale, &unknown), 1);
    item.resolved = CURRENT.to_string();
    let (drift, unknown) = compare(&[item.clone()], &captured);
    assert!(drift.is_empty() && unknown.is_empty());
    assert_eq!(comparison_exit_code(&drift, &unknown), 0);
    captured.action_releases.clear();
    let (drift, unknown) = compare(&[item.clone()], &captured);
    assert!(drift.is_empty());
    assert!(unknown[0].contains("commit unresolved"));
    assert_eq!(comparison_exit_code(&drift, &unknown), EXIT_UNVERIFIABLE);
    captured.entries.clear();
    let (drift, unknown) = compare(&[item], &captured);
    assert!(unknown[0].contains("absent from snapshot"));
    assert_eq!(comparison_exit_code(&drift, &unknown), EXIT_UNVERIFIABLE);
}

#[test]
fn lightweight_and_nested_annotated_tags_bind_same_repository_commit() {
    let mut calls = Vec::new();
    let result = resolve_action_release("owner/action/path", "v2.3.4", |url| {
        calls.push(url.to_string());
        if url.ends_with("/git/ref/tags/v2.3.4") {
            Ok(format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"type":"tag","sha":"{TAG}","url":"https://evil.invalid/ignored"}}}}"#))
        } else {
            assert_eq!(url, format!("https://api.github.com/repos/owner/action/git/tags/{TAG}"));
            Ok(format!(r#"{{"sha":"{TAG}","object":{{"sha":"{CURRENT}","type":"commit"}}}}"#))
        }
    }).unwrap();
    assert_eq!(result, release());
    assert_eq!(calls.len(), 2);
    let lightweight = resolve_action_release("owner/action/path", "v2.3.4", |_| {
        Ok(format!(
            r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{CURRENT}","type":"commit"}}}}"#
        ))
    })
    .unwrap();
    assert_eq!(lightweight, result);
}

#[test]
fn malformed_wrong_ref_unavailable_cycle_and_depth_never_verify() {
    for body in [
        "{}".to_string(),
        format!(r#"{{"ref":"refs/tags/wrong","object":{{"sha":"{CURRENT}","type":"commit"}}}}"#),
        r#"{"ref":"refs/tags/v2.3.4","object":{"sha":"bad","type":"commit"}}"#.to_string(),
        format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{CURRENT}","type":"tree"}}}}"#),
    ] {
        assert!(resolve_action_release("owner/action", "v2.3.4", |_| Ok(body.clone())).is_err());
    }
    assert!(
        resolve_action_release("owner/action", "v2.3.4", |_| Err("offline".to_string())).is_err()
    );
    let mut count = 0;
    let cycle = resolve_action_release("owner/action", "v2.3.4", |_| {
        count += 1;
        Ok(if count == 1 {
            format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{TAG}","type":"tag"}}}}"#)
        } else {
            format!(r#"{{"sha":"{TAG}","object":{{"sha":"{TAG}","type":"tag"}}}}"#)
        })
    });
    assert!(cycle.is_err());
    assert_eq!(count, 2);
    count = 0;
    let depth = resolve_action_release("owner/action", "v2.3.4", |_| {
        let previous = format!("{:040x}", count);
        count += 1;
        let next = format!("{:040x}", count);
        Ok(if count == 1 {
            format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{next}","type":"tag"}}}}"#)
        } else {
            format!(r#"{{"sha":"{previous}","object":{{"sha":"{next}","type":"tag"}}}}"#)
        })
    });
    assert!(depth.is_err());
    assert_eq!(count, 8);
}

#[test]
fn snapshot_round_trip_retains_urls_commit_and_apply_keeps_immutable_pin() {
    let directory =
        std::env::temp_dir().join(format!("action-release-currency-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let snapshot_path = directory.join("registry.lino");
    let captured = snapshot();
    write_snapshot(&snapshot_path, &captured);
    let loaded = read_snapshot(&snapshot_path).unwrap();
    assert_eq!(loaded.action_releases, captured.action_releases);
    assert_eq!(loaded.entries, captured.entries);
    let workflow = directory.join("workflow.yml");
    fs::write(&workflow, format!("uses: owner/action/path@{OLD}\n")).unwrap();
    let mut item = dependency();
    item.path = workflow.clone();
    let (findings, unknown) = compare(&[item], &loaded);
    assert!(unknown.is_empty());
    apply_latest(&findings);
    assert_eq!(
        fs::read_to_string(&workflow).unwrap(),
        format!("uses: owner/action/path@{CURRENT}\n")
    );
    let mut untrusted = loaded;
    untrusted
        .action_releases
        .get_mut("owner/action/path")
        .unwrap()
        .source_url = "https://evil.invalid/latest".to_string();
    assert!(!compare(&[dependency()], &untrusted).1.is_empty());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn changed_tag_binding_or_wrong_annotated_object_never_verifies() {
    let mut captured = snapshot();
    captured
        .action_releases
        .get_mut("owner/action/path")
        .unwrap()
        .tag = "v1.0.0".to_string();
    assert!(!compare(&[dependency()], &captured).1.is_empty());
    let mut captured = snapshot();
    captured
        .action_releases
        .get_mut("owner/action/path")
        .unwrap()
        .tag_source_url = "https://evil.invalid/tag".to_string();
    assert!(!compare(&[dependency()], &captured).1.is_empty());
    let mut calls = 0;
    assert!(
        resolve_action_release("owner/action", "v2.3.4", |_| {
            calls += 1;
            Ok(if calls == 1 {
                format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"type":"tag","sha":"{TAG}"}}}}"#)
            } else {
                format!(r#"{{"sha":"{OLD}","object":{{"type":"commit","sha":"{CURRENT}"}}}}"#)
            })
        })
        .is_err()
    );
}

#[test]
fn typed_json_rejects_trailing_garbage_nested_decoys_and_identity_duplicates() {
    let valid =
        format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{CURRENT}","type":"commit"}}}}"#);
    let invalid = [
        format!("{valid} trailing garbage"),
        format!(r#"{{"metadata":{valid}}}"#),
        format!(
            r#"{{"ref":"refs/tags/wrong","metadata":{valid},"object":{{"sha":"{CURRENT}","type":"commit"}}}}"#
        ),
        format!(
            r#"{{"ref":"refs/tags/v2.3.4","metadata":{{"object":{{"sha":"{CURRENT}","type":"commit"}}}}}}"#
        ),
        format!(
            r#"{{"ref":"refs/tags/wrong","ref":"refs/tags/v2.3.4","object":{{"sha":"{CURRENT}","type":"commit"}}}}"#
        ),
        format!(
            r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{OLD}","sha":"{CURRENT}","type":"commit"}}}}"#
        ),
        format!(
            r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{CURRENT}","type":"tree","type":"commit"}}}}"#
        ),
        format!(
            r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{OLD}","type":"commit"}},"object":{{"sha":"{CURRENT}","type":"commit"}}}}"#
        ),
        format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":42,"type":"commit"}}}}"#),
    ];
    for body in invalid {
        assert!(
            resolve_action_release("owner/action", "v2.3.4", |_| Ok(body.clone())).is_err(),
            "accepted: {body}"
        );
    }
    let mut count = 0;
    assert!(resolve_action_release("owner/action", "v2.3.4", |_| {
        count += 1;
        Ok(if count == 1 { format!(r#"{{"ref":"refs/tags/v2.3.4","object":{{"sha":"{TAG}","type":"tag"}}}}"#) }
            else { format!(r#"{{"sha":"{OLD}","sha":"{TAG}","object":{{"sha":"{CURRENT}","type":"commit"}}}}"#) })
    }).is_err());
    assert!(github_release_tag(r#"{"tag_name":"v1","tag_name":"v2"}"#).is_err());
    assert!(github_release_tag(r#"{"metadata":{"tag_name":"v2"}}"#).is_err());
    assert!(github_release_tag(r#"{"tag_name":"v2"} garbage"#).is_err());
    assert_eq!(
        github_release_tag(r#"{"tag_name":"v2","body":"ordinary extra official metadata"}"#)
            .unwrap(),
        "v2"
    );
}
