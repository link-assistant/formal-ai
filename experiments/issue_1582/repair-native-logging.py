"""Preserve the temporary-server test while asserting quiet, persisted logs."""
from pathlib import Path
path = Path('rust/tests/integration/with_formal_ai.rs')
text = path.read_text()
old = '''    assert!(
        structured_stderr::messages(&output.stderr).contains("temporary server in agent mode"),
        "stderr: {}",
        structured_stderr::messages(&output.stderr)
    );'''
assert old in text
text = text.replace(old, '''    assert!(output.stderr.is_empty(), "routine startup stays out of the harness");
    let logs = std::fs::read_dir(home.join(".formal-ai/logs"))
        .expect("default diagnostic directory")
        .map(|entry| std::fs::read(entry.expect("log entry").path()).expect("log"))
        .flatten()
        .collect::<Vec<_>>();
    assert!(structured_stderr::messages(&logs).contains("temporary server in agent mode"));''')
path.write_text(text)
