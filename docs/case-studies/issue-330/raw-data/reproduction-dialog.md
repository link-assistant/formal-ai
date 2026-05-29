# Reproduction dialog — Issue #330

> Legend: `U` = user, `A` = agent. Captured verbatim from the issue body
> (filed against version 0.149.0, GitHub Pages WASM worker, manual mode,
> UI language `en-US`, locale `en-US` / `Asia/Calcutta`).

````
U: Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории
A: Вот минимальная программа на языке Rust (list files in the current directory):

   ```rust
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
   ```

   Статус выполнения: не запущено - браузерная песочница не может вызвать инструментарий rust. Скопируйте фрагмент в среду rust, чтобы проверить.
   Вывод зависит от каталога; для образца каталога, содержащего ровно `Cargo.toml`, `README.md` и `main.rs`, он такой:
   ```text
   Cargo.toml
   README.md
   main.rs
   ```
U: Сделай так, чтобы программа принимала путь как аргумент
A (intent: write_program, reported): Вот минимальная программа на языке Rust (list files in the directory given as a path argument):

   ```rust
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
   ```

   Статус выполнения: не запущено - браузерная песочница не может вызвать инструментарий rust. Скопируйте фрагмент в среду rust, чтобы проверить.
   Вывод зависит от каталога; для образца каталога, содержащего ровно `Cargo.toml`, `README.md` и `main.rs`, он такой:
   ```text
   Cargo.toml
   README.md
   main.rs
   ```
````

## Note on the dialog

The dialog itself is **already correct** as of the predecessor work (issue
[#324](https://github.com/link-assistant/formal-ai/issues/324) / PR
[#325](https://github.com/link-assistant/formal-ai/pull/325)): the response is
in Russian, the follow-up "make it accept a path argument" is honored, and the
program is valid. Issue #330 quotes this *working* exchange not to report a
regression but to point at the **presentation layer**: those fenced code blocks
render as flat, monochrome `<pre><code>` with no syntax highlighting and no way
to copy them. The actual requirements live in the **Description** section of the
issue, not the dialog.
