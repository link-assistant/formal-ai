//! Issue #1138 B7 (plan 03, L6): find what the requirement names, or nothing.
//!
//! None of the five held-out prompts names a file or a constant. Resolving them
//! must come from the census, not from a pre-authored rule, and the requirement
//! arrives as the need's `subject` in the need's own language so five languages
//! are one code path. A requirement matching two declarations resolves to
//! nothing and names both candidates: a guess is never returned.

use std::path::{Path, PathBuf};
use std::process::Command;

use formal_ai::needs::{Need, NeedKind, NeedState};
use formal_ai::repository_workspace::RepositoryWorkspace;
use formal_ai::repository_workspace::locate::{LocationEvidence, locate_ambiguity, locate_targets};

/// The same requirement in five languages. None of them names
/// `src/web_search_core.rs` or `WEB_SEARCH_PROVIDERS`.
const PROMPTS: &[(&str, &str)] = &[
    (
        "en",
        "In the repository at commit {base}, the list of trusted search providers is missing the encyclopaedia of quotations. Add it, keep the file valid, and run the tests that cover that list.",
    ),
    (
        "ru",
        "В репозитории на коммите {base} в списке доверенных поисковых провайдеров нет энциклопедии цитат. Добавь её, не сломай файл и запусти тесты, которые покрывают этот список.",
    ),
    (
        "hi",
        "कमिट {base} पर मौजूद रिपॉज़िटरी में भरोसेमंद खोज प्रदाताओं की सूची में उद्धरणों का विश्वकोश नहीं है। उसे जोड़ो, फ़ाइल को वैध रखो, और उस सूची को कवर करने वाले परीक्षण चलाओ।",
    ),
    (
        "zh",
        "在提交 {base} 的仓库里，可信搜索来源的列表缺少引语百科。请把它加进去，保持文件有效，并运行覆盖该列表的测试。",
    ),
    (
        "es",
        "En el repositorio en el commit {base}, la lista de proveedores de búsqueda de confianza no incluye la enciclopedia de citas. Añádela, mantén el archivo válido y ejecuta las pruebas que cubren esa lista.",
    ),
];

/// The second held-out family targets a non-Rust tree, so literal occurrence is
/// exercised rather than the census.
const FOREIGN_PROMPTS: &[(&str, &str)] = &[
    (
        "en",
        "Clone this project at the given commit, find where the default timeout is defined, raise it to sixty seconds, and run only the test that asserts the default.",
    ),
    (
        "ru",
        "Склонируй проект на указанном коммите, найди, где задан таймаут по умолчанию, подними его до шестидесяти секунд и запусти только тот тест, который проверяет значение по умолчанию.",
    ),
    (
        "hi",
        "दिए गए कमिट पर इस प्रोजेक्ट को क्लोन करो, पता करो कि डिफ़ॉल्ट टाइमआउट कहाँ परिभाषित है, उसे साठ सेकंड करो, और केवल वही परीक्षण चलाओ जो डिफ़ॉल्ट की जाँच करता है।",
    ),
    (
        "zh",
        "在给定提交处克隆这个项目，找到默认超时在哪里定义，把它改成六十秒，只运行断言默认值的那个测试。",
    ),
    (
        "es",
        "Clona este proyecto en el commit indicado, localiza dónde se define el tiempo de espera por defecto, súbelo a sesenta segundos y ejecuta solo la prueba que comprueba ese valor por defecto.",
    ),
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn head_commit() -> String {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root().display().to_string(),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("git rev-parse should run");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn need_for(subject: &str, language: &str) -> Need {
    Need {
        need_id: String::from("issue-1138-locate"),
        kind: NeedKind::Part,
        subject: subject.to_owned(),
        language: language.to_owned(),
        raised_by: String::from("issue-1138-locate-targets"),
        source_span: format!("issue-1138@0:{}", subject.len()),
        depth: 0,
        state: NeedState::Open,
        satisfied_by: None,
    }
}

struct PythonFixture(PathBuf);

impl std::ops::Deref for PythonFixture {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for PythonFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A three-file Python tree with exactly one occurrence of `DEFAULT_TIMEOUT`,
/// plus a second fixture where two files declare it. The fixture removes its
/// temporary Git object store even when an assertion fails.
fn python_fixture(tag: &str, duplicate: bool) -> PythonFixture {
    let root = std::env::temp_dir().join(format!("formal-ai-issue-1138-python-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("fixture root should be creatable");
    std::fs::write(
        root.join("client.py"),
        "DEFAULT_TIMEOUT = 30\n\n\ndef fetch(url):\n    return url\n",
    )
    .expect("fixture write");
    std::fs::write(
        root.join("server.py"),
        if duplicate {
            "DEFAULT_TIMEOUT = 30\n\n\ndef serve():\n    return None\n"
        } else {
            "def serve():\n    return None\n"
        },
    )
    .expect("fixture write");
    std::fs::write(
        root.join("test_defaults.py"),
        "from client import DEFAULT_TIMEOUT\n\n\ndef test_default():\n    assert DEFAULT_TIMEOUT\n",
    )
    .expect("fixture write");

    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.email=t@example.org",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "fixture",
        ],
    ] {
        let mut command = Command::new("git");
        command.arg("-C").arg(&root).args(&args);
        let _ = command.output();
    }
    PythonFixture(root)
}

/// Each of the five prompts resolves to the declaration neither of them names.
#[test]
fn census_locates_a_declaration_the_prompt_never_names() {
    let commit = head_commit();
    let workspace = RepositoryWorkspace::adopt(&repo_root()).expect("the ambient checkout adopts");

    for (language, template) in PROMPTS {
        let subject = template.replace("{base}", &commit);
        let locations = locate_targets(&workspace, &need_for(&subject, language))
            .unwrap_or_else(|error| panic!("{language}: locating must not error: {error:?}"));
        assert!(
            locations
                .iter()
                .any(|location| location.relative_path == "src/web_search_core.rs"),
            "{language}: the requirement must resolve to the provider list's file, got {locations:?}"
        );
        assert!(
            locations
                .iter()
                .any(|location| location.symbol.as_deref() == Some("WEB_SEARCH_PROVIDERS")),
            "{language}: the requirement must resolve to the declaration, got {locations:?}"
        );
        assert!(
            locations
                .iter()
                .any(|location| location.how == LocationEvidence::Census),
            "{language}: a Rust tree resolves through the census, not a literal scan"
        );
    }
}

/// A tie is ambiguity, and ambiguity resolves to nothing rather than to a guess.
#[test]
fn ambiguity_resolves_to_nothing() {
    let root = python_fixture("ambiguous", true);
    let workspace = RepositoryWorkspace::adopt(&root).expect("the fixture tree adopts");
    let need = need_for(FOREIGN_PROMPTS[0].1, FOREIGN_PROMPTS[0].0);

    let locations = locate_targets(&workspace, &need).expect("locating must not error");
    assert!(
        locations.is_empty(),
        "two declarations are a tie; a tie resolves to nothing, got {locations:?}"
    );

    let report = locate_ambiguity(&workspace, &need).expect("the candidates are reported");
    assert!(
        report.candidates.len() >= 2,
        "the ambiguity is reported with its candidates, not swallowed: {report:?}"
    );
}

/// A tree with no Rust in it still resolves, by the literal the requirement
/// implies occurring in exactly one file.
#[test]
fn literal_occurrence_locates_in_a_python_tree() {
    let root = python_fixture("single", false);
    let workspace = RepositoryWorkspace::adopt(&root).expect("the fixture tree adopts");

    for (language, prompt) in FOREIGN_PROMPTS {
        let locations = locate_targets(&workspace, &need_for(prompt, language))
            .unwrap_or_else(|error| panic!("{language}: locating must not error: {error:?}"));
        assert_eq!(
            locations.len(),
            1,
            "{language}: exactly one file declares the default, got {locations:?}"
        );
        assert_eq!(
            locations[0].relative_path, "client.py",
            "{language}: the single declaration is the one that is resolved"
        );
        assert_eq!(
            locations[0].how,
            LocationEvidence::LiteralOccurrence,
            "{language}: a non-Rust tree resolves by literal occurrence"
        );
    }
}
