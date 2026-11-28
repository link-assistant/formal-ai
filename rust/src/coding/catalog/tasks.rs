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
    },
    ProgramTask {
        slug: "count_to_three",
        label: "count over a range",
        output: "",
    },
    ProgramTask {
        slug: "list_files",
        label: "list files in the current directory",
        output: "",
    },
    ProgramTask {
        slug: "list_files_arg",
        label: "list files in the directory given as a path argument",
        // A bare "accept a path argument" modification also maps here through the
        // program-plan rules; the `program_task_list_files_arg` seed aliases let
        // an explicit single-turn request resolve directly (issue #324 follow-up).
        output: "",
    },
    ProgramTask {
        slug: "list_files_reverse_sort",
        label: "list files in the current directory in reverse-sorted order",
        output: "",
    },
    ProgramTask {
        slug: "list_files_arg_reverse_sort",
        label: "list files from a path argument in reverse-sorted order",
        output: "",
    },
    // These operations take their operands from the typed request.
    ProgramTask {
        slug: "fizzbuzz",
        label: "FizzBuzz",
        output: "",
    },
    ProgramTask {
        slug: "factorial",
        label: "factorial",
        output: "",
    },
    ProgramTask {
        slug: "reverse_string",
        label: "string reversal",
        output: "",
    },
    ProgramTask {
        slug: "sum_to_ten",
        label: "sum an integer range",
        output: "",
    },
    ProgramTask {
        slug: "fibonacci",
        label: "recursive Fibonacci",
        output: "",
    },
    // Its output depends on the input stream supplied at runtime.
    ProgramTask {
        slug: "copy_stdin_to_stdout",
        label: "copy standard input to standard output",
        output: "",
    },
    // Issue #1173 R1173-3: "a program that prints hello" names a task whose
    // output is the operand of the request, not a memorized program. Its seed
    // row in data/seed/hello-world-programs.lino names the procedure it is
    // composed from (the hello_world print procedure the documentation route
    // rediscovers) and the verb role that introduces the operand; the program
    // binds the operand into the output literal of that procedure
    // (`crate::coding::operand_program`), so `{operand}` is a slot, not text.
    ProgramTask {
        slug: "print_text",
        label: "print text",
        output: "{operand}",
    },
];
