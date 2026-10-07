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

use std::borrow::Cow;

use super::types::{ProgramExecution, ProgramLanguage};

pub const PROGRAM_LANGUAGES: &[ProgramLanguage] = &[
    ProgramLanguage {
        slug: "rust",
        name: "Rust",
        code_fence: "rust",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("rustc main.rs -o main")),
            run_command: Cow::Borrowed("./main"),
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.rs"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "python",
        name: "Python",
        code_fence: "python",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("python3 -m py_compile main.py")),
            run_command: Cow::Borrowed("python3 main.py"),
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.py"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "javascript",
        name: "JavaScript",
        code_fence: "javascript",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("node --check main.js")),
            run_command: Cow::Borrowed("node main.js"),
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.js"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "typescript",
        name: "TypeScript",
        code_fence: "typescript",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("tsc hello.ts")),
            run_command: Cow::Borrowed("node hello.js"),
            notes: "The TypeScript seed is returned with this warning until a tsc-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("hello.ts"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "go",
        name: "Go",
        code_fence: "go",
        execution: ProgramExecution {
            check_command: None,
            run_command: Cow::Borrowed("go run main.go"),
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.go"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "c",
        name: "C",
        code_fence: "c",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("gcc main.c -o main")),
            run_command: Cow::Borrowed("./main"),
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.c"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "cpp",
        name: "C++",
        code_fence: "cpp",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("g++ main.cpp -o main")),
            run_command: Cow::Borrowed("./main"),
            notes: "The C++ seed is returned with this warning until a g++-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.cpp"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "java",
        name: "Java",
        code_fence: "java",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("javac Main.java")),
            run_command: Cow::Borrowed("java Main"),
            notes: "The Java seed is returned with this warning until a javac-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("Main.java"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "csharp",
        name: "C#",
        code_fence: "csharp",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("dotnet build")),
            run_command: Cow::Borrowed("dotnet run"),
            notes: "The C# seed is returned with this warning until a dotnet-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("Program.cs"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "ruby",
        name: "Ruby",
        code_fence: "ruby",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("ruby -c main.rb")),
            run_command: Cow::Borrowed("ruby main.rb"),
            notes: "The Ruby seed is returned with this warning until a ruby-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.rb"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "scala",
        name: "Scala",
        code_fence: "scala",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("scalac Main.scala")),
            run_command: Cow::Borrowed("scala Main"),
            notes: "The Scala seed is returned with this warning until a scalac-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("Main.scala"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "kotlin",
        name: "Kotlin",
        code_fence: "kotlin",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed(
                "kotlinc Main.kt -include-runtime -d Main.jar",
            )),
            run_command: Cow::Borrowed("java -jar Main.jar"),
            notes: "The Kotlin seed is returned with this warning until a kotlinc-backed execution profile is available.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("Main.kt"),
        framework_of: None,
    },
    ProgramLanguage {
        slug: "php",
        name: "PHP",
        code_fence: "php",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("php -l main.php")),
            run_command: Cow::Borrowed("php main.php"),
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("main.php"),
        framework_of: None,
    },
    // Issue #1167: meta-language 0.58.2 ships a Swift tree-sitter grammar and
    // data/meta/hello-world-languages.lino carries a verified Hello World for
    // it, so Swift joins the catalog the way the seed's own rule demands — a
    // row with its fences and execution metadata, not a special case.
    ProgramLanguage {
        slug: "swift",
        name: "Swift",
        code_fence: "swift",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("swiftc -parse hello.swift")),
            run_command: Cow::Borrowed("swift hello.swift"),
            // Stated in the response seed: `program_execution_notes_swift`.
            notes: "",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("hello.swift"),
        framework_of: None,
    },
    // Issue #1167: the R grammar ships in the same meta-language revision.
    // R is one of the nine languages whose Hello World could not be executed
    // on the reference machine (no R toolchain installed), so the row states
    // its run command and that verification is owed to CI, never claims it.
    ProgramLanguage {
        slug: "r",
        name: "R",
        code_fence: "r",
        execution: ProgramExecution {
            check_command: Some(Cow::Borrowed("Rscript -e 'invisible(parse(\"hello.R\"))'")),
            run_command: Cow::Borrowed("Rscript hello.R"),
            // Stated in the response seed: `program_execution_notes_r`.
            notes: "",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("hello.R"),
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
            check_command: Some(Cow::Borrowed("php -l app/Console/Commands/HelloWorld.php")),
            run_command: Cow::Borrowed("php artisan hello:world"),
            notes: "Laravel Framework 13.26.1 on PHP 8.3.31: `composer create-project laravel/laravel`, then the command above printed the expected output exactly.",
        },
        source: "local Links Notation write-program seed",
        save_as: Cow::Borrowed("app/Console/Commands/HelloWorld.php"),
        framework_of: Some("php"),
    },
];
