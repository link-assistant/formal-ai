#[derive(Clone, Copy)]
pub struct ProgramLanguage {
    pub slug: &'static str,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub code_fence: &'static str,
    pub execution: ProgramExecution,
    pub source: &'static str,
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

fn contains_token(normalized: &str, expected: &str) -> bool {
    normalized.split_whitespace().any(|token| token == expected)
}

fn contains_phrase(normalized: &str, expected: &str) -> bool {
    normalized == expected
        || normalized.starts_with(&format!("{expected} "))
        || normalized.ends_with(&format!(" {expected}"))
        || normalized.contains(&format!(" {expected} "))
}
