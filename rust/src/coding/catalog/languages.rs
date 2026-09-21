//! The catalog of supported programming languages. Adding a language is a
//! matter of extending [`PROGRAM_LANGUAGES`] with its fences and verified
//! execution metadata, then declaring its `program_language_<slug>` meaning
//! (with the alias surfaces, role `program_language_alias`) in the seed lexicon
//! — the engine does not change.
//!
//! Issue #1138 plan 06 leaf L3: **no row states its own availability.** The
//! fourteen `setup_hint` strings, the five `environment` strings and the
//! per-row `status` left Rust for `data/seed/toolchains.lino`, beside the probe
//! argv that says whether the toolchain is actually on this machine. A row that
//! asserted `ExecutionStatus::Verified` could not be wrong about its
//! environment because it never looked, and could not become right, because
//! becoming right would have been a source edit.

use super::types::{ProgramExecution, ProgramLanguage};

pub const PROGRAM_LANGUAGES: &[ProgramLanguage] = &[
    ProgramLanguage {
        slug: "rust",
        name: "Rust",
        code_fence: "rust",
        execution: ProgramExecution {
            check_command: Some("rustc main.rs -o main"),
            run_command: "./main",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.rs",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "python",
        name: "Python",
        code_fence: "python",
        execution: ProgramExecution {
            check_command: Some("python3 -m py_compile main.py"),
            run_command: "python3 main.py",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.py",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "javascript",
        name: "JavaScript",
        code_fence: "javascript",
        execution: ProgramExecution {
            check_command: Some("node --check main.js"),
            run_command: "node main.js",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.js",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "typescript",
        name: "TypeScript",
        code_fence: "typescript",
        execution: ProgramExecution {
            check_command: Some("tsc hello.ts"),
            run_command: "node hello.js",
            notes: "The TypeScript seed is returned with this warning until a tsc-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "hello.ts",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "go",
        name: "Go",
        code_fence: "go",
        execution: ProgramExecution {
            check_command: None,
            run_command: "go run main.go",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.go",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "c",
        name: "C",
        code_fence: "c",
        execution: ProgramExecution {
            check_command: Some("gcc main.c -o main"),
            run_command: "./main",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.c",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "cpp",
        name: "C++",
        code_fence: "cpp",
        execution: ProgramExecution {
            check_command: Some("g++ main.cpp -o main"),
            run_command: "./main",
            notes: "The C++ seed is returned with this warning until a g++-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.cpp",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "java",
        name: "Java",
        code_fence: "java",
        execution: ProgramExecution {
            check_command: Some("javac Main.java"),
            run_command: "java Main",
            notes: "The Java seed is returned with this warning until a javac-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "Main.java",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "csharp",
        name: "C#",
        code_fence: "csharp",
        execution: ProgramExecution {
            check_command: Some("dotnet build"),
            run_command: "dotnet run",
            notes: "The C# seed is returned with this warning until a dotnet-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "Program.cs",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "ruby",
        name: "Ruby",
        code_fence: "ruby",
        execution: ProgramExecution {
            check_command: Some("ruby -c main.rb"),
            run_command: "ruby main.rb",
            notes: "The Ruby seed is returned with this warning until a ruby-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.rb",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "scala",
        name: "Scala",
        code_fence: "scala",
        execution: ProgramExecution {
            check_command: Some("scalac Main.scala"),
            run_command: "scala Main",
            notes: "The Scala seed is returned with this warning until a scalac-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "Main.scala",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "kotlin",
        name: "Kotlin",
        code_fence: "kotlin",
        execution: ProgramExecution {
            check_command: Some("kotlinc Main.kt -include-runtime -d Main.jar"),
            run_command: "java -jar Main.jar",
            notes: "The Kotlin seed is returned with this warning until a kotlinc-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: "Main.kt",
        framework_of: None,
    },
    ProgramLanguage {
        slug: "php",
        name: "PHP",
        code_fence: "php",
        execution: ProgramExecution {
            check_command: Some("php -l main.php"),
            run_command: "php main.php",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: "main.php",
        framework_of: None,
    },
    // Issue #723 reported `напиши мне код на PHP Laravel` and got an answer that
    // named no language at all; issue #1021 then answered it in PHP, which is
    // the language Laravel is written in but not the thing that was asked for.
    // The row below is the difference: an implementation target that is a
    // framework of `php`, carrying the file a Laravel application actually keeps
    // its code in and the command Artisan actually runs. Nothing else in the
    // catalog changed shape — a framework is resolved, rendered and verified by
    // the same code paths as a language, which is why the fix is one row and not
    // a rule about Laravel.
    ProgramLanguage {
        slug: "laravel",
        name: "Laravel",
        code_fence: "php",
        execution: ProgramExecution {
            check_command: Some("php -l app/Console/Commands/HelloWorld.php"),
            run_command: "php artisan hello:world",
            notes: "Laravel Framework 13.26.1 on PHP 8.3.31: `composer create-project laravel/laravel`, then the command above printed the expected output exactly.",
        },
        source: "local Links Notation write-program seed",
        save_as: "app/Console/Commands/HelloWorld.php",
        framework_of: Some("php"),
    },
];
