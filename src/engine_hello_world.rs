#[derive(Clone, Copy)]
pub struct ProgramLanguage {
    pub slug: &'static str,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub code_fence: &'static str,
    pub execution: ProgramExecution,
    pub source: &'static str,
    /// File name a novice should save the snippet as before running it (issue
    /// #330). The check/run commands above already reference this name.
    pub save_as: &'static str,
    /// One-line, novice-friendly hint for installing the toolchain (issue
    /// #330). URLs and shell commands stay canonical; only the surrounding
    /// prose is localized in `program_test_instructions`.
    pub setup_hint: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProgramTask {
    pub slug: &'static str,
    pub label: &'static str,
    pub aliases: &'static [&'static str],
    pub output: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProgramTemplate {
    pub task_slug: &'static str,
    pub language_slug: &'static str,
    pub code: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProgramSpec {
    pub language: &'static ProgramLanguage,
    pub task: &'static ProgramTask,
    pub template: &'static ProgramTemplate,
}

impl ProgramSpec {
    #[must_use]
    pub fn response_link(self) -> String {
        format!(
            "response:write_program:{}:{}",
            self.task.slug, self.language.slug
        )
    }

    #[must_use]
    pub fn parameter_summary(self) -> String {
        format!(
            "write_program(language={}, task={})",
            self.language.slug, self.task.slug
        )
    }

    #[must_use]
    pub fn legacy_intent(self) -> String {
        if self.task.slug == "hello_world" {
            format!("hello_world_{}", self.language.slug)
        } else {
            format!("write_program_{}_{}", self.task.slug, self.language.slug)
        }
    }
}

#[derive(Clone, Copy)]
pub struct ProgramExecution {
    pub status: ExecutionStatus,
    pub environment: &'static str,
    pub check_command: Option<&'static str>,
    pub run_command: &'static str,
    pub notes: &'static str,
}

#[derive(Clone, Copy)]
pub enum ExecutionStatus {
    Verified,
    Unavailable,
}

impl ExecutionStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Verified => "compiled and ran",
            Self::Unavailable => "not compiled or run",
        }
    }
}

pub const WRITE_PROGRAM_INTENT: &str = "write_program";

pub const PROGRAM_LANGUAGES: &[ProgramLanguage] = &[
    ProgramLanguage {
        slug: "rust",
        name: "Rust",
        aliases: &["rust", "rs", "раст", "расте"],
        code_fence: "rust",
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("rustc main.rs -o main"),
            run_command: "./main",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.rs",
        setup_hint: "the Rust toolchain from https://rustup.rs",
    },
    ProgramLanguage {
        slug: "python",
        name: "Python",
        aliases: &["python", "py", "питон", "питоне"],
        code_fence: "python",
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("python3 -m py_compile main.py"),
            run_command: "python3 main.py",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.py",
        setup_hint: "Python 3 from https://www.python.org/downloads/",
    },
    ProgramLanguage {
        slug: "javascript",
        name: "JavaScript",
        aliases: &["javascript", "js", "node", "джаваскрипт"],
        code_fence: "javascript",
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("node --check main.js"),
            run_command: "node main.js",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.js",
        setup_hint: "Node.js from https://nodejs.org/",
    },
    ProgramLanguage {
        slug: "typescript",
        name: "TypeScript",
        aliases: &["typescript", "ts", "тайпскрипт"],
        code_fence: "typescript",
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "TypeScript compiler is not configured in this repository runtime",
            check_command: Some("tsc hello.ts"),
            run_command: "node hello.js",
            notes: "The TypeScript seed is returned with this warning until a tsc-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "hello.ts",
        setup_hint: "Node.js from https://nodejs.org/ plus TypeScript via `npm install -g typescript`",
    },
    ProgramLanguage {
        slug: "go",
        name: "Go",
        aliases: &["go", "golang", "го"],
        code_fence: "go",
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: None,
            run_command: "go run main.go",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.go",
        setup_hint: "Go from https://go.dev/dl/",
    },
    ProgramLanguage {
        slug: "c",
        name: "C",
        aliases: &["c"],
        code_fence: "c",
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("gcc main.c -o main"),
            run_command: "./main",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.c",
        setup_hint: "a C compiler such as GCC from https://gcc.gnu.org/ or your package manager",
    },
    ProgramLanguage {
        slug: "cpp",
        name: "C++",
        aliases: &["cpp", "c++", "cplusplus"],
        code_fence: "cpp",
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "C++ toolchain is not configured in this repository runtime",
            check_command: Some("g++ main.cpp -o main"),
            run_command: "./main",
            notes: "The C++ seed is returned with this warning until a g++-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.cpp",
        setup_hint: "a C++ compiler such as g++ from https://gcc.gnu.org/ or your package manager",
    },
    ProgramLanguage {
        slug: "java",
        name: "Java",
        aliases: &["java", "джава"],
        code_fence: "java",
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "Java toolchain is not configured in this repository runtime",
            check_command: Some("javac Main.java"),
            run_command: "java Main",
            notes: "The Java seed is returned with this warning until a javac-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "Main.java",
        setup_hint: "a JDK from https://adoptium.net/",
    },
    ProgramLanguage {
        slug: "csharp",
        name: "C#",
        aliases: &["csharp", "c#", "cs", "dotnet"],
        code_fence: "csharp",
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "C# / dotnet toolchain is not configured in this repository runtime",
            check_command: Some("dotnet build"),
            run_command: "dotnet run",
            notes: "The C# seed is returned with this warning until a dotnet-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "Program.cs",
        setup_hint: "the .NET SDK from https://dotnet.microsoft.com/download",
    },
    ProgramLanguage {
        slug: "ruby",
        name: "Ruby",
        aliases: &["ruby", "rb", "руби"],
        code_fence: "ruby",
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "Ruby interpreter is not configured in this repository runtime",
            check_command: Some("ruby -c main.rb"),
            run_command: "ruby main.rb",
            notes: "The Ruby seed is returned with this warning until a ruby-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.rb",
        setup_hint: "Ruby from https://www.ruby-lang.org/en/downloads/",
    },
];

pub const PROGRAM_TASKS: &[ProgramTask] = &[
    ProgramTask {
        slug: "hello_world",
        label: "hello world",
        aliases: &["hello world", "хелло ворлд"],
        output: "Hello, world!",
    },
    ProgramTask {
        slug: "count_to_three",
        label: "count to three",
        aliases: &[
            "count to three",
            "count to 3",
            "counts to three",
            "counts to 3",
        ],
        output: "1\n2\n3",
    },
    ProgramTask {
        slug: "list_files",
        label: "list files in the current directory",
        // English, Russian, Hindi and Chinese phrasings of "list the files in
        // the current directory" (issue #312). The Russian reporter wrote
        // "выдаёт список файлов в текущей директории"; competitors answered with
        // full code. Every supported prompt language (en, ru, hi, zh) is covered
        // so the whole class of list-files requests resolves, not just Russian.
        aliases: &[
            "list files in the current directory",
            "list files in current directory",
            "list files in the directory",
            "list the files in the current directory",
            "lists files in the current directory",
            "lists the files in the current directory",
            "list files in a directory",
            "list directory files",
            "list files",
            "lists files",
            "files in the current directory",
            "files in current directory",
            "список файлов в текущей директории",
            "список файлов в текущем каталоге",
            "список файлов в директории",
            "список файлов в каталоге",
            "выдаёт список файлов",
            "выдает список файлов",
            "выводит список файлов",
            "вывести список файлов",
            "вывод списка файлов",
            "список файлов",
            "файлы в текущей директории",
            "файлы в текущем каталоге",
            // Hindi: "list of files (in the current directory)".
            "फ़ाइलों की सूची",
            "फाइलों की सूची",
            "वर्तमान निर्देशिका की फ़ाइलें",
            "वर्तमान निर्देशिका की फाइलें",
            "निर्देशिका की फ़ाइलें",
            // Chinese: "list the files in the current directory".
            "列出当前目录中的文件",
            "列出当前目录中文件",
            "列出当前目录的文件",
            "列出当前目录文件",
            "列出目录中的文件",
            "列出文件",
        ],
        // Verified output for the documented sample directory containing exactly
        // `Cargo.toml`, `README.md`, and `main.rs`. Every template sorts names in
        // byte order, so the output is identical across languages.
        output: "Cargo.toml\nREADME.md\nmain.rs",
    },
    ProgramTask {
        slug: "list_files_arg",
        label: "list files in the directory given as a path argument",
        // Issue #324 follow-up: "Сделай так, чтобы программа принимала путь как
        // аргумент" (make the program accept a path as an argument). This task is
        // the path-argument variant of `list_files`; conversation context maps a
        // bare "accept a path argument" modification onto it (see
        // `program_path_argument_modifier`). Aliases let an explicit, single-turn
        // request resolve here directly too. Every supported prompt language
        // (en, ru, hi, zh) is covered.
        aliases: &[
            "list files in the directory given as a path argument",
            "list files in a directory given as an argument",
            "list files in the directory passed as an argument",
            "list files in a path argument",
            "list files with a path argument",
            "list files accepting a path argument",
            "список файлов в каталоге переданном как аргумент",
            "список файлов в директории переданной как аргумент",
            "список файлов по пути из аргумента",
            // Hindi: "list of files in the directory given as a path argument".
            "पथ तर्क के रूप में दी गई निर्देशिका की फ़ाइलों की सूची",
            // Chinese: "list the files in the directory given as a path argument".
            "列出作为路径参数给出的目录中的文件",
            "列出路径参数指定目录中的文件",
        ],
        // When no argument is supplied the templates fall back to "." so the
        // documented sample directory still produces the verified listing.
        output: "Cargo.toml\nREADME.md\nmain.rs",
    },
];

pub const PROGRAM_TEMPLATES: &[ProgramTemplate] = &[
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "rust",
        code: r#"fn main() {
    println!("Hello, world!");
}"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "python",
        code: r#"print("Hello, world!")"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "javascript",
        code: r#"console.log("Hello, world!");"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "typescript",
        code: r#"console.log("Hello, world!");"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    puts("Hello, world!");
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "cpp",
        code: r#"#include <iostream>

int main() {
    std::cout << "Hello, world!" << std::endl;
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, world!");
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "csharp",
        code: r#"using System;

class Program {
    static void Main() {
        Console.WriteLine("Hello, world!");
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "hello_world",
        language_slug: "ruby",
        code: r#"puts "Hello, world!""#,
    },
    ProgramTemplate {
        task_slug: "count_to_three",
        language_slug: "rust",
        code: r#"fn main() {
    for number in 1..=3 {
        println!("{number}");
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "count_to_three",
        language_slug: "python",
        code: r"for number in range(1, 4):
    print(number)",
    },
    ProgramTemplate {
        task_slug: "count_to_three",
        language_slug: "javascript",
        code: r"for (let number = 1; number <= 3; number += 1) {
    console.log(number);
}",
    },
    ProgramTemplate {
        task_slug: "count_to_three",
        language_slug: "typescript",
        code: r"for (let number = 1; number <= 3; number += 1) {
    console.log(number);
}",
    },
    ProgramTemplate {
        task_slug: "count_to_three",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    for number := 1; number <= 3; number++ {
        fmt.Println(number)
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "count_to_three",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    for (int number = 1; number <= 3; number++) {
        printf("%d\n", number);
    }
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "rust",
        code: r#"use std::fs;

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
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "python",
        code: r#"import os

names = sorted(name for name in os.listdir(".") if os.path.isfile(name))
for name in names:
    print(name)"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "javascript",
        code: r#"const fs = require("fs");

const names = fs
  .readdirSync(".")
  .filter((name) => fs.statSync(name).isFile())
  .sort();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "typescript",
        code: r#"import * as fs from "fs";

const names: string[] = fs
  .readdirSync(".")
  .filter((name) => fs.statSync(name).isFile())
  .sort();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "go",
        code: r#"package main

import (
    "fmt"
    "os"
    "sort"
)

func main() {
    entries, err := os.ReadDir(".")
    if err != nil {
        panic(err)
    }
    var names []string
    for _, entry := range entries {
        if !entry.IsDir() {
            names = append(names, entry.Name())
        }
    }
    sort.Strings(names)
    for _, name := range names {
        fmt.Println(name)
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "c",
        code: r#"#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static int compare(const void *a, const void *b) {
    return strcmp(*(const char *const *)a, *(const char *const *)b);
}

int main(void) {
    DIR *dir = opendir(".");
    if (dir == NULL) {
        return 1;
    }
    char *names[1024];
    size_t count = 0;
    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL && count < 1024) {
        struct stat info;
        if (stat(entry->d_name, &info) == 0 && S_ISREG(info.st_mode)) {
            names[count++] = strdup(entry->d_name);
        }
    }
    closedir(dir);
    qsort(names, count, sizeof(char *), compare);
    for (size_t i = 0; i < count; i++) {
        printf("%s\n", names[i]);
        free(names[i]);
    }
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "cpp",
        code: r#"#include <algorithm>
#include <filesystem>
#include <iostream>
#include <string>
#include <vector>

int main() {
    namespace fs = std::filesystem;
    std::vector<std::string> names;
    for (const auto &entry : fs::directory_iterator(".")) {
        if (entry.is_regular_file()) {
            names.push_back(entry.path().filename().string());
        }
    }
    std::sort(names.begin(), names.end());
    for (const auto &name : names) {
        std::cout << name << '\n';
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "java",
        code: r#"import java.io.File;
import java.util.Arrays;

public class Main {
    public static void main(String[] args) {
        File[] entries = new File(".").listFiles();
        if (entries == null) {
            return;
        }
        String[] names = Arrays.stream(entries)
            .filter(File::isFile)
            .map(File::getName)
            .sorted()
            .toArray(String[]::new);
        for (String name : names) {
            System.out.println(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "csharp",
        code: r#"using System;
using System.IO;
using System.Linq;

class Program {
    static void Main() {
        var names = Directory.GetFiles(".")
            .Select(Path.GetFileName)
            .OrderBy(name => name, StringComparer.Ordinal);
        foreach (var name in names) {
            Console.WriteLine(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files",
        language_slug: "ruby",
        code: r#"names = Dir.entries(".").select { |name| File.file?(name) }.sort
names.each { |name| puts name }"#,
    },
    // Issue #324 follow-up: list files in the directory passed as the first
    // command-line argument, defaulting to "." when none is supplied. Each
    // template sorts names in byte order, so the verified output matches
    // `list_files` for the documented sample directory.
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "rust",
        code: r#"use std::env;
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
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "python",
        code: r#"import os
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "."
names = sorted(
    name for name in os.listdir(path) if os.path.isfile(os.path.join(path, name))
)
for name in names:
    print(name)"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "javascript",
        code: r#"const fs = require("fs");
const path = require("path");

const dir = process.argv[2] || ".";
const names = fs
  .readdirSync(dir)
  .filter((name) => fs.statSync(path.join(dir, name)).isFile())
  .sort();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "typescript",
        code: r#"import * as fs from "fs";
import * as path from "path";

const dir: string = process.argv[2] ?? ".";
const names: string[] = fs
  .readdirSync(dir)
  .filter((name) => fs.statSync(path.join(dir, name)).isFile())
  .sort();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "go",
        code: r#"package main

import (
    "fmt"
    "os"
    "sort"
)

func main() {
    dir := "."
    if len(os.Args) > 1 {
        dir = os.Args[1]
    }
    entries, err := os.ReadDir(dir)
    if err != nil {
        panic(err)
    }
    var names []string
    for _, entry := range entries {
        if !entry.IsDir() {
            names = append(names, entry.Name())
        }
    }
    sort.Strings(names)
    for _, name := range names {
        fmt.Println(name)
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "c",
        code: r#"#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static int compare(const void *a, const void *b) {
    return strcmp(*(const char *const *)a, *(const char *const *)b);
}

int main(int argc, char *argv[]) {
    const char *path = argc > 1 ? argv[1] : ".";
    DIR *dir = opendir(path);
    if (dir == NULL) {
        return 1;
    }
    char *names[1024];
    size_t count = 0;
    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL && count < 1024) {
        char full[4096];
        snprintf(full, sizeof(full), "%s/%s", path, entry->d_name);
        struct stat info;
        if (stat(full, &info) == 0 && S_ISREG(info.st_mode)) {
            names[count++] = strdup(entry->d_name);
        }
    }
    closedir(dir);
    qsort(names, count, sizeof(char *), compare);
    for (size_t i = 0; i < count; i++) {
        printf("%s\n", names[i]);
        free(names[i]);
    }
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "cpp",
        code: r#"#include <algorithm>
#include <filesystem>
#include <iostream>
#include <string>
#include <vector>

int main(int argc, char *argv[]) {
    namespace fs = std::filesystem;
    std::string path = argc > 1 ? argv[1] : ".";
    std::vector<std::string> names;
    for (const auto &entry : fs::directory_iterator(path)) {
        if (entry.is_regular_file()) {
            names.push_back(entry.path().filename().string());
        }
    }
    std::sort(names.begin(), names.end());
    for (const auto &name : names) {
        std::cout << name << '\n';
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "java",
        code: r#"import java.io.File;
import java.util.Arrays;

public class Main {
    public static void main(String[] args) {
        String path = args.length > 0 ? args[0] : ".";
        File[] entries = new File(path).listFiles();
        if (entries == null) {
            return;
        }
        String[] names = Arrays.stream(entries)
            .filter(File::isFile)
            .map(File::getName)
            .sorted()
            .toArray(String[]::new);
        for (String name : names) {
            System.out.println(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "csharp",
        code: r#"using System;
using System.IO;
using System.Linq;

class Program {
    static void Main(string[] args) {
        var path = args.Length > 0 ? args[0] : ".";
        var names = Directory.GetFiles(path)
            .Select(Path.GetFileName)
            .OrderBy(name => name, StringComparer.Ordinal);
        foreach (var name in names) {
            Console.WriteLine(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg",
        language_slug: "ruby",
        code: r#"path = ARGV[0] || "."
names = Dir.entries(path).select { |name| File.file?(File.join(path, name)) }.sort
names.each { |name| puts name }"#,
    },
];

#[must_use]
pub fn program_language_by_slug(slug: &str) -> Option<&'static ProgramLanguage> {
    PROGRAM_LANGUAGES
        .iter()
        .find(|language| language.slug == slug)
}

#[must_use]
pub fn program_task_by_slug(slug: &str) -> Option<&'static ProgramTask> {
    PROGRAM_TASKS.iter().find(|task| task.slug == slug)
}

#[must_use]
pub fn program_template(task_slug: &str, language_slug: &str) -> Option<&'static ProgramTemplate> {
    PROGRAM_TEMPLATES
        .iter()
        .find(|template| template.task_slug == task_slug && template.language_slug == language_slug)
}

#[must_use]
pub fn program_spec(task_slug: &str, language_slug: &str) -> Option<ProgramSpec> {
    Some(ProgramSpec {
        task: program_task_by_slug(task_slug)?,
        language: program_language_by_slug(language_slug)?,
        template: program_template(task_slug, language_slug)?,
    })
}

#[must_use]
pub fn program_language_by_alias(normalized: &str) -> Option<&'static ProgramLanguage> {
    PROGRAM_LANGUAGES.iter().find(|language| {
        language
            .aliases
            .iter()
            .any(|alias| contains_token(normalized, alias))
    })
}

#[must_use]
pub fn program_task_by_alias(normalized: &str) -> Option<&'static ProgramTask> {
    PROGRAM_TASKS.iter().find(|task| {
        task.aliases
            .iter()
            .any(|alias| contains_phrase(normalized, alias))
    })
}

#[must_use]
pub fn supported_program_languages() -> String {
    PROGRAM_LANGUAGES
        .iter()
        .map(|language| language.slug)
        .collect::<Vec<_>>()
        .join(", ")
}

#[must_use]
pub fn supported_program_tasks() -> String {
    PROGRAM_TASKS
        .iter()
        .map(|task| task.slug)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Chinese (and other CJK) text is written without spaces between words, so the
/// whitespace-based token/phrase matchers below never see an isolated word. When
/// the expected alias itself contains a CJK ideograph we fall back to a plain
/// substring test, which is what "token boundaries" effectively mean for those
/// scripts. Latin and Cyrillic aliases keep strict boundary matching so short
/// tokens like `rust` never match inside `trust`.
pub fn contains_cjk(text: &str) -> bool {
    text.chars().any(|ch| {
        let cp = ch as u32;
        (0x3400..=0x4DBF).contains(&cp)
            || (0x4E00..=0x9FFF).contains(&cp)
            || (0xF900..=0xFAFF).contains(&cp)
            || (0x3040..=0x30FF).contains(&cp)
            || (0x3100..=0x312F).contains(&cp)
    })
}

fn contains_token(normalized: &str, expected: &str) -> bool {
    if contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized.split_whitespace().any(|token| token == expected)
}

fn contains_phrase(normalized: &str, expected: &str) -> bool {
    if contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized == expected
        || normalized.starts_with(&format!("{expected} "))
        || normalized.ends_with(&format!(" {expected}"))
        || normalized.contains(&format!(" {expected} "))
}
