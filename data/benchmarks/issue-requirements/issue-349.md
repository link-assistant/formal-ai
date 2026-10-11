## Environment

- **Version**: 0.152.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: Manual mode
- **Diagnostics**: off
- **Timestamp**: 2026-05-30T09:45:51.653Z

## User Context

- **UI languages**: *en-US*
- **Theme**: auto (dark)
- **UI**: 1712x923 viewport, 1728x1117 @2x screen, Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Safari/605.1.15 browser, MacIntel platform
- **Locale**: en-US (Asia/Calcutta)
- **Guess probability**: 80%
- **Temperature**: 0.7
- **Follow-up probability**: 75%
- **Location**: inferred from time zone / locale only

## Reproduction of dialog

Legend: `U` = user, `A` = agent.

```
Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории
```

```
Вот минимальная программа на языке Rust (list files in the current directory):

\```rust
use std::fs;

fn main() -> std::io::Result<()> {
    let mut names: Vec<String> = fs::read_dir(".")?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in names {
        println!("{name}");
    }
    Ok(())
}
\```

Статус выполнения: не запущено - браузерная песочница не может вызвать инструментарий rust. Скопируйте фрагмент в среду rust, чтобы проверить.
Вывод зависит от каталога; для образца каталога, содержащего ровно `Cargo.toml`, `README.md` и `main.rs`, он такой:
\```text
Cargo.toml
README.md
main.rs
\```

Как это работает:
Программа читает содержимое текущего каталога, оставляет только обычные файлы, собирает их имена в список, сортирует список по алфавиту и печатает каждое имя на отдельной строке.

Как проверить это самостоятельно:
1. Установите инструментарий: the Rust toolchain from https://rustup.rs.
2. Сохраните приведённый выше код в файл `main.rs`.
3. Проверьте, что код компилируется: `rustc main.rs -o main`.
4. Запустите программу: `./main`.
5. Сравните вывод с разделом ожидаемого вывода выше.
```

```
Сделай так, чтобы программа принимала путь как аргумент
```

```
Вот минимальная программа на языке Rust (list files in the directory given as a path argument):

/```rust
use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| String::from("."));
    let mut names: Vec<String> = fs::read_dir(&path)
        .expect("failed to read directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in names {
        println!("{name}");
    }
}
/```

Статус выполнения: не запущено - браузерная песочница не может вызвать инструментарий rust. Скопируйте фрагмент в среду rust, чтобы проверить.
Вывод зависит от каталога; для образца каталога, содержащего ровно `Cargo.toml`, `README.md` и `main.rs`, он такой:
/```text
Cargo.toml
README.md
main.rs
/```

Как это работает:
Программа берёт путь к каталогу из первого аргумента командной строки (если аргумент не задан, используется текущий каталог), читает содержимое этого каталога, оставляет только обычные файлы, сортирует их имена по алфавиту и печатает каждое имя на отдельной строке.

Проверьте обновлённую программу так же, как и раньше: сохраните код в файл `main.rs` и снова выполните `./main`.
```

...

```
... omitted 3 earlier messages ...
A: Вот минимальная программа на языке Rust (list files in the directory given as a path argument):
   ... omitted 31 lines ...
   Проверьте обновлённую программу так же, как и раньше: сохраните код в файл `main.rs` и снова выполните `./main`.
U: Сделай сортировку результатов в обратном порядке
A (intent: unknown, reported): Я не уверен, как на это ответить. Я пока не могу ответить на это по локальным правилам Links Notation. Чтобы посмотреть текущие правила, отправьте `List behavior rules`, затем `Show behavior rule unknown`. Чтобы научить этот диалог ответу, отправьте: When I say `ваш запрос`, answer `ваш ответ`. Чтобы сделать это постоянным, экспортируйте память или используйте Report issue, чтобы разработчики добавили факт или правило в seed-данные Links Notation.
```

## Description

It will sure will not work, if I have to report issue on each and every message. We need deeply rethink how we do it, and make sure our system deeply understands each symbol, word, that it refers to, that is the meaning, that are requirements, request, question and so on. Yet, we don't store the entire internet or all possible messages. We need to collect the deepest possible details, but as they are needed.

That should be done according to our vision using reasoning on top of semantic meta language, that is represented by binary links or links notation.

We should do actual reasoning, not some memorized only rules, expecially if we don't have yet any rules, we should try our best to reason how to construct such rule, as human would do.

So current solution is still fake, and to stop that, we need to plan issues in this repository using gh tool, to fully cover coding - initial drafts, code editing, iterating until error solved and so on.

We should use tests, examples, datasets from AI projects, that we can legally use in open-source and so on. If the dataset is not public domain, we should download it or part of it only when we execute our testing.

Based on previous issues we more or less understand what our users want, and if we solve coding in general as specific case of universal problem solving algorithm, we should be able to actually solve everything.

We already planned issues to fully implement our roadmap multiple times, and here we are, still not fully implemented.

We should rethink our architecture, make sure we use the best practices possible, if no such practices in the internet and no similar projects exist we must implement it from scratch. By using bulk test suites with large datasets, to make sure our system really work, and really deeply reasons, and we provide in diagnostics mode all the details about each reasoning step.

We need to implement not black box as neutral networks, but white box, in both cases it is machine learning on data, but as soon as our system can start coding itself, we can use it to improve itself.

As the result of this pull request, I want to have full plan as issues on GitHub in this pull requests, where each issue clearly blocked via GitHub API by other issues that it depends on. Each issue plan should be detailed as possible, so even weakest AI systems can implement them.

We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration.

If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code. Also double check to fully apply requirements to entire codebase, so if we have issue in multiple places, it should be fixed in all them.



