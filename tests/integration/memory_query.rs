use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir() -> std::path::PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let thread_id = format!("{:?}", std::thread::current().id())
        .replace(|c: char| !c.is_ascii_alphanumeric(), "");
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-memory-query-{}-{}-{seq}",
        std::process::id(),
        thread_id,
    ));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn memory_query(path: &std::path::Path, prompt: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "memory",
            "query",
            "--path",
            path.to_str().expect("memory path should be utf-8"),
            "--prompt",
            prompt,
        ])
        .output()
        .expect("memory query")
}

#[test]
fn cli_memory_query_can_append_natural_language_memory() {
    let dir = tmpdir();
    let memory_path = dir.join("memory.lino");

    let write_output = memory_query(&memory_path, "remember issue 529 associative memory");
    let write_stderr = String::from_utf8_lossy(&write_output.stderr);
    assert!(
        write_output.status.success(),
        "write stderr: {write_stderr}"
    );
    let write_stdout = String::from_utf8_lossy(&write_output.stdout);
    assert!(
        write_stdout.contains("Recorded memory: issue 529 associative memory"),
        "write output: {write_stdout}"
    );

    let memory_text = std::fs::read_to_string(&memory_path).expect("read appended memory");
    assert!(memory_text.contains("intent \"memory_write\""));
    assert!(memory_text.contains("content \"issue 529 associative memory\""));

    let recall_output = memory_query(&memory_path, "recall issue 529");
    let recall_stderr = String::from_utf8_lossy(&recall_output.stderr);
    assert!(
        recall_output.status.success(),
        "recall stderr: {recall_stderr}"
    );
    let recall_stdout = String::from_utf8_lossy(&recall_output.stdout);
    assert!(
        recall_stdout.contains("issue 529 associative memory"),
        "recall output: {recall_stdout}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_memory_query_records_substitution_style_memory_write() {
    let dir = tmpdir();
    let memory_path = dir.join("memory.lino");
    std::fs::write(
        &memory_path,
        "demo_memory\n\
         \x20\x20event \"a1\"\n\
         \x20\x20\x20\x20kind \"message\"\n\
         \x20\x20\x20\x20role \"user\"\n\
         \x20\x20\x20\x20content \"alpha\"\n",
    )
    .expect("seed memory file");

    let output = memory_query(&memory_path, "replace alpha with beta in memory");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "substitution stderr: {stderr}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Replaced \"alpha\" with \"beta\" in memory (1 occurrence(s) updated)."),
        "substitution output: {stdout}"
    );

    let memory_text = std::fs::read_to_string(&memory_path).expect("read substitution memory");
    assert!(memory_text.contains("kind \"memory_substitution\""));
    assert!(memory_text.contains("inputs \"replace:alpha\""));
    assert!(memory_text.contains("outputs \"with:beta\""));
    assert!(memory_text.contains("evidence \"substitution_event:update|substitution:applied=1\""));
    // The transform is a real read+write: the *original* event's content must
    // have been rewritten in place from "alpha" to "beta".
    assert!(
        memory_text.contains("content \"beta\""),
        "original content should be rewritten: {memory_text}"
    );
    assert!(
        !memory_text.contains("content \"alpha\""),
        "original content should no longer hold the old value: {memory_text}"
    );

    let recall_output = memory_query(&memory_path, "recall beta");
    let recall_stderr = String::from_utf8_lossy(&recall_output.stderr);
    assert!(
        recall_output.status.success(),
        "substitution recall stderr: {recall_stderr}"
    );
    let recall_stdout = String::from_utf8_lossy(&recall_output.stdout);
    assert!(
        recall_stdout.contains("outputs: with:beta"),
        "substitution recall output: {recall_stdout}"
    );
    assert!(
        recall_stdout.contains("user: beta"),
        "rewritten original event should surface in recall: {recall_stdout}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
