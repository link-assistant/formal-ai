//! End-to-end regression coverage for the issue #660 import command.

use std::fs;
use std::process::Command;

#[test]
fn rejected_import_is_durable_and_leaves_existing_shards_untouched() {
    let root = std::env::temp_dir().join(format!("formal-ai-issue-660-cli-{}", std::process::id()));
    let cache = root.join("cache");
    let out = root.join("out");
    let concepts = root.join("concepts.txt");
    let events = root.join("rejections.lino");

    fs::create_dir_all(&cache).unwrap();
    fs::create_dir_all(&out).unwrap();
    fs::write(&concepts, "concepts\n  missing Q999999999\n").unwrap();
    let shard = out.join("meanings-lexicon-import-01.lino");
    fs::write(&shard, "existing shard\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "import",
            "lexemes",
            "--concepts",
            concepts.to_str().unwrap(),
            "--cache-dir",
            cache.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--offline",
            "--events",
            events.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&shard).unwrap(), "existing shard\n");
    let event_log = fs::read_to_string(&events).unwrap();
    assert!(event_log.contains("kind \"import_rejected\""));
    assert!(event_log.contains("Q999999999"));

    fs::remove_dir_all(&root).unwrap();
}
