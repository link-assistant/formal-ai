use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::coding_function_catalog::rosetta_code::fetch_example;
use formal_ai::event_log::EventLog;
use formal_ai::{
    CachedSourceClient, FetchError, SourceTransport, try_rosetta_code_request_with_client,
};

const CAPTURE_SHA256: &str = "cc6a157ca2188be0089b3cd901c26e41e6ff63913e47c12f1fbb9ff3f8691398";
const EXAMPLE_ANSWERS: [&str; 5] = [
    r"This rust example for Copy stdin to stdout was retrieved from Rosetta Code; it was not generated:

```rust
use std::io;

fn main() {
    io::copy(&mut io::stdin().lock(), &mut io::stdout().lock());
}
```

Source: https://rosettacode.org/wiki/Copy_stdin_to_stdout
License: GFDL-1.2
Execution status: not requested.
Source capture: fetched cc6a157ca2188be0089b3cd901c26e41e6ff63913e47c12f1fbb9ff3f8691398.",
    r"Этот пример на rust для задачи Copy stdin to stdout получен с Rosetta Code, а не сгенерирован:

```rust
use std::io;

fn main() {
    io::copy(&mut io::stdin().lock(), &mut io::stdout().lock());
}
```

Источник: https://rosettacode.org/wiki/Copy_stdin_to_stdout
Лицензия: GFDL-1.2
Статус выполнения: запуск не запрашивался.
Снимок источника: получен cc6a157ca2188be0089b3cd901c26e41e6ff63913e47c12f1fbb9ff3f8691398.",
    r"Copy stdin to stdout के लिए यह rust उदाहरण Rosetta Code से लिया गया है; इसे generate नहीं किया गया:

```rust
use std::io;

fn main() {
    io::copy(&mut io::stdin().lock(), &mut io::stdout().lock());
}
```

स्रोत: https://rosettacode.org/wiki/Copy_stdin_to_stdout
लाइसेंस: GFDL-1.2
Execution status: चलाने का अनुरोध नहीं किया गया।
Source capture: fetch किया गया cc6a157ca2188be0089b3cd901c26e41e6ff63913e47c12f1fbb9ff3f8691398.",
    r"这个用于 Copy stdin to stdout 的 rust 示例来自 Rosetta Code，并非生成：

```rust
use std::io;

fn main() {
    io::copy(&mut io::stdin().lock(), &mut io::stdout().lock());
}
```

来源：https://rosettacode.org/wiki/Copy_stdin_to_stdout
许可证：GFDL-1.2
执行状态：未请求运行。
来源快照：已获取 cc6a157ca2188be0089b3cd901c26e41e6ff63913e47c12f1fbb9ff3f8691398。",
    r"Este ejemplo de rust para Copy stdin to stdout se obtuvo de Rosetta Code; no fue generado:

```rust
use std::io;

fn main() {
    io::copy(&mut io::stdin().lock(), &mut io::stdout().lock());
}
```

Fuente: https://rosettacode.org/wiki/Copy_stdin_to_stdout
Licencia: GFDL-1.2
Estado de ejecución: no solicitado.
Captura de fuente: se obtuvo cc6a157ca2188be0089b3cd901c26e41e6ff63913e47c12f1fbb9ff3f8691398.",
];
const EXECUTED_ANSWER: &str = r"This rust example for Copy stdin to stdout was retrieved from Rosetta Code; it was not generated:

```rust
use std::io;

fn main() {
    io::copy(&mut io::stdin().lock(), &mut io::stdout().lock());
}
```

Source: https://rosettacode.org/wiki/Copy_stdin_to_stdout
License: GFDL-1.2
Execution status: compiled and ran in an isolated bounded agent workspace.
Source capture: fetched cc6a157ca2188be0089b3cd901c26e41e6ff63913e47c12f1fbb9ff3f8691398.";

#[derive(Clone, Default)]
struct FixtureTransport {
    requests: Arc<AtomicUsize>,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        assert!(url.contains("action=parse"), "{url}");
        assert!(url.contains("page=Copy_stdin_to_stdout"), "{url}");
        self.requests.fetch_add(1, Ordering::SeqCst);
        fs::read(fixture_root().join("Copy_stdin_to_stdout.json"))
            .map_err(|error| FetchError::Transport(error.to_string()))
    }
}

fn fixture_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/coding-discovery/rosetta")
}

fn client(label: &str) -> CachedSourceClient<FixtureTransport> {
    let cache =
        std::env::temp_dir().join(format!("formal-ai-rosetta-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&cache);
    CachedSourceClient::new(cache, FixtureTransport::default())
        .with_online(true)
        .with_clock(|| 1_789_344_000)
}

#[test]
fn captured_mediawiki_section_yields_an_attributed_rust_example() {
    let example =
        fetch_example(&client("parser"), "Copy_stdin_to_stdout", "rust").expect("captured example");

    assert_eq!(example.task, "Copy stdin to stdout");
    assert_eq!(example.language, "rust");
    assert!(example.code.contains("io::copy"));
    assert_eq!(example.license, "GFDL-1.2");
    assert_eq!(example.sha256.len(), 64);
    assert_eq!(example.sha256, CAPTURE_SHA256);
    assert_eq!(
        example.source_url,
        "https://rosettacode.org/wiki/Copy_stdin_to_stdout"
    );
}

#[test]
fn exact_example_request_and_four_language_paraphrases_return_source_not_generated_code() {
    for (index, (prompt, source_marker)) in [
        (
            "Give me example of how to do copy stdin to stdout in Rust",
            "not generated",
        ),
        (
            "Покажи пример копирования stdin в stdout на Rust",
            "а не сгенерирован",
        ),
        (
            "Rust में stdin को stdout में कॉपी करने का उदाहरण दें",
            "generate नहीं",
        ),
        ("给我一个用 Rust 把 stdin 复制到 stdout 的示例", "并非生成"),
        (
            "Dame un ejemplo para copiar stdin a stdout en Rust",
            "no fue generado",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut log = EventLog::new();
        let answer = try_rosetta_code_request_with_client(
            prompt,
            &prompt.to_lowercase(),
            &mut log,
            &client(&format!("example-{index}")),
        )
        .unwrap_or_else(|| panic!("not handled: {prompt}"));

        assert_eq!(answer.intent, "coding_example");
        assert_eq!(answer.answer, EXAMPLE_ANSWERS[index]);
        assert!(answer.answer.contains("io::copy"), "{}", answer.answer);
        assert!(answer.answer.contains("GFDL-1.2"), "{}", answer.answer);
        assert!(answer.answer.contains(source_marker), "{}", answer.answer);
        assert!(
            answer
                .answer
                .contains("https://rosettacode.org/wiki/Copy_stdin_to_stdout")
        );
    }
}

#[test]
fn exact_execute_url_request_runs_only_the_fetched_rust_example_in_a_bounded_workspace() {
    let prompt = "Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust";
    let mut log = EventLog::new();
    let answer = try_rosetta_code_request_with_client(
        prompt,
        &prompt.to_lowercase(),
        &mut log,
        &client("execute"),
    )
    .expect("execute request");

    assert_eq!(answer.intent, "execute_coding_example");
    assert_eq!(answer.answer, EXECUTED_ANSWER);
    assert!(answer.answer.contains("GFDL-1.2"));
    assert!(
        answer.answer.contains("Execution status: compiled and ran"),
        "{}",
        answer.answer
    );
    assert!(log.events().iter().any(|event| {
        event.kind == "action_log:run_command" && event.payload.starts_with("rustc ")
    }));
    assert!(log.events().iter().any(|event| {
        event.kind == "action_log:run_command" && event.payload.starts_with("./main ")
    }));
}
