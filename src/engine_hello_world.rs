pub struct HelloWorldProgram {
    pub slug: &'static str,
    pub language: &'static str,
    pub aliases: &'static [&'static str],
    pub code_fence: &'static str,
    pub code: &'static str,
    pub execution: ProgramExecution,
    pub response_link: &'static str,
    pub source: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProgramExecution {
    pub status: ExecutionStatus,
    pub environment: &'static str,
    pub check_command: Option<&'static str>,
    pub run_command: &'static str,
    pub output: &'static str,
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

pub const HELLO_WORLD_PROGRAMS: &[HelloWorldProgram] = &[
    HelloWorldProgram {
        slug: "rust",
        language: "Rust",
        aliases: &["rust", "rs", "раст", "расте"],
        code_fence: "rust",
        code: r#"fn main() {
    println!("Hello, world!");
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("rustc main.rs -o main"),
            run_command: "./main",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:rust",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "python",
        language: "Python",
        aliases: &["python", "py", "питон", "питоне"],
        code_fence: "python",
        code: r#"print("Hello, world!")"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("python3 -m py_compile main.py"),
            run_command: "python3 main.py",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:python",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "javascript",
        language: "JavaScript",
        aliases: &["javascript", "js", "node", "джаваскрипт"],
        code_fence: "javascript",
        code: r#"console.log("Hello, world!");"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("node --check main.js"),
            run_command: "node main.js",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:javascript",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "typescript",
        language: "TypeScript",
        aliases: &["typescript", "ts"],
        code_fence: "typescript",
        code: r#"console.log("Hello, world!");"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "TypeScript compiler is not configured in this repository runtime",
            check_command: Some("tsc hello.ts"),
            run_command: "node hello.js",
            output: "Hello, world!",
            notes: "The TypeScript seed is returned with this warning until a tsc-backed execution profile is available.",
        },
        response_link: "response:hello_world:typescript",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "go",
        language: "Go",
        aliases: &["go", "golang"],
        code_fence: "go",
        code: r#"package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: None,
            run_command: "go run main.go",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:go",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "c",
        language: "C",
        aliases: &["c"],
        code_fence: "c",
        code: r#"#include <stdio.h>

int main(void) {
    puts("Hello, world!");
    return 0;
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness (isolated sandbox)",
            check_command: Some("gcc main.c -o main"),
            run_command: "./main",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:c",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "cpp",
        language: "C++",
        aliases: &["cpp", "c++", "cplusplus"],
        code_fence: "cpp",
        code: r#"#include <iostream>

int main() {
    std::cout << "Hello, world!" << std::endl;
    return 0;
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "C++ toolchain is not configured in this repository runtime",
            check_command: Some("g++ main.cpp -o main"),
            run_command: "./main",
            output: "Hello, world!",
            notes: "The C++ seed is returned with this warning until a g++-backed execution profile is available.",
        },
        response_link: "response:hello_world:cpp",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "java",
        language: "Java",
        aliases: &["java"],
        code_fence: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, world!");
    }
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "Java toolchain is not configured in this repository runtime",
            check_command: Some("javac Main.java"),
            run_command: "java Main",
            output: "Hello, world!",
            notes: "The Java seed is returned with this warning until a javac-backed execution profile is available.",
        },
        response_link: "response:hello_world:java",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "csharp",
        language: "C#",
        aliases: &["csharp", "c#", "cs", "dotnet"],
        code_fence: "csharp",
        code: r#"using System;

class Program {
    static void Main() {
        Console.WriteLine("Hello, world!");
    }
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "C# / dotnet toolchain is not configured in this repository runtime",
            check_command: Some("dotnet build"),
            run_command: "dotnet run",
            output: "Hello, world!",
            notes: "The C# seed is returned with this warning until a dotnet-backed execution profile is available.",
        },
        response_link: "response:hello_world:csharp",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "ruby",
        language: "Ruby",
        aliases: &["ruby", "rb"],
        code_fence: "ruby",
        code: r#"puts "Hello, world!""#,
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "Ruby interpreter is not configured in this repository runtime",
            check_command: Some("ruby -c main.rb"),
            run_command: "ruby main.rb",
            output: "Hello, world!",
            notes: "The Ruby seed is returned with this warning until a ruby-backed execution profile is available.",
        },
        response_link: "response:hello_world:ruby",
        source: "local Links Notation hello-world seed",
    },
];
