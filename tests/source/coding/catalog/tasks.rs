//! The catalog of supported coding tasks. Each task is realized in every
//! language by a matching template in [`super::templates_core`] /
//! [`super::templates_extended`]; adding a task is a matter of extending
//! [`PROGRAM_TASKS`], supplying those templates, and declaring its
//! `program_task_<slug>` meaning (with the request phrasings, role
//! `program_task_alias`) in the seed lexicon (issue #386).

use super::types::ProgramTask;

pub const PROGRAM_TASKS: &[ProgramTask] = &[
    ProgramTask {
        slug: "hello_world",
        label: "hello world",
        output: "Hello, world!",
        input: "",
    },
    ProgramTask {
        slug: "count_to_three",
        label: "count to three",
        output: "1\n2\n3",
        input: "",
    },
    ProgramTask {
        slug: "list_files",
        label: "list files in the current directory",
        // Fallback output for the Rust-flavoured sample. Rendered answers resolve
        // list-file samples from each language's `save_as` name (issue #440).
        output: "Cargo.toml\nREADME.md\nmain.rs",
        input: "",
    },
    ProgramTask {
        slug: "list_files_arg",
        label: "list files in the directory given as a path argument",
        // A bare "accept a path argument" modification also maps here through the
        // program-plan rules; the `program_task_list_files_arg` seed aliases let
        // an explicit single-turn request resolve directly (issue #324 follow-up).
        // When no argument is supplied the templates fall back to "."; rendered
        // answers still use a language-aware sample directory (issue #440).
        output: "Cargo.toml\nREADME.md\nmain.rs",
        input: "",
    },
    ProgramTask {
        slug: "list_files_reverse_sort",
        label: "list files in the current directory in reverse-sorted order",
        output: "main.rs\nREADME.md\nCargo.toml",
        input: "",
    },
    ProgramTask {
        slug: "list_files_arg_reverse_sort",
        label: "list files from a path argument in reverse-sorted order",
        output: "main.rs\nREADME.md\nCargo.toml",
        input: "",
    },
    // Issue #330: the catalog supports general coding tasks, not only
    // hello-world. The tasks below are classic, deterministic exercises that
    // exercise control flow (fizzbuzz), arithmetic (factorial, sum), and string
    // handling (reverse). Each has a fixed, self-describing scenario so the
    // verified output is unambiguous, and every supported prompt language
    // (en, ru, hi, zh) is covered.
    ProgramTask {
        slug: "fizzbuzz",
        label: "FizzBuzz",
        output: "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz",
        input: "",
    },
    ProgramTask {
        slug: "factorial",
        label: "factorial of 5",
        // Tied to the concrete value 5 (5! = 120) so the verified output is
        // unambiguous; the seed aliases keep the number 5 so a different
        // factorial is never answered with the 5! program.
        output: "120",
        input: "",
    },
    ProgramTask {
        slug: "reverse_string",
        label: "string reversal",
        // Reverses the literal string "hello" -> "olleh"; the scenario is fixed
        // so the output is verifiable, mirroring the hello-world philosophy.
        output: "olleh",
        input: "",
    },
    ProgramTask {
        slug: "sum_to_ten",
        label: "sum from 1 to 10",
        // Sums 1..=10 -> 55; the range is fixed so the output is verifiable.
        output: "55",
        input: "",
    },
    // Issue #334: the website demo asked for "a Python function that calculates
    // the Fibonacci sequence recursively" and then "the 10th Fibonacci number".
    // The catalog had no Fibonacci entry, so step one resolved to "I didn't
    // understand you". This task defines a recursive `fibonacci` function and
    // prints the 10th term (F(1)=F(2)=1 -> F(10)=55), so the scenario is fixed
    // and the verified output is unambiguous. Every supported prompt language
    // (en, ru, hi, zh) is covered, including the explicit "recursive" /
    // "function" phrasings from the report.
    ProgramTask {
        slug: "fibonacci",
        label: "recursive Fibonacci",
        output: "55",
        input: "",
    },
    // Issue #863 / #862: "copy stdin to stdout" is the smallest task whose
    // subject is a *stream* rather than a computation, and the catalog could not
    // express it: every task before this one produced its output from nothing,
    // so `output` alone described it completely. Naming the input the task is
    // defined against is what makes this one verifiable — the fixture below is
    // what `experiments/issue-1021-copy-stdin` pipes in, and it is the same
    // fixture the rendered answer's run command carries, so the reader runs the
    // check that was run here.
    ProgramTask {
        slug: "copy_stdin_to_stdout",
        label: "copy standard input to standard output",
        output: "hello\nworld",
        input: "hello\nworld\n",
    },
];
