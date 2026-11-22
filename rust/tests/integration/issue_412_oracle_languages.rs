//! The shipped runtime composes output programs without a bundled example.

use formal_ai::{SolverConfig, UniversalSolver};

fn offline_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    })
}

#[test]
fn standard_output_programs_resolve_without_publisher_bytes() {
    let solver = offline_solver();
    for (language, fence_and_code, command) in [
        (
            "kotlin",
            "kotlin\nfun main() {\n    println(\"Hello, world!\")\n}\n",
            "java -jar Main.jar",
        ),
        (
            "swift",
            "swift\nprint(\"Hello, world!\")\n",
            "swift hello.swift",
        ),
        ("lua", "lua\nprint(\"Hello, world!\")\n", "lua main.lua"),
        (
            "php",
            "php\n<?php\n\necho \"Hello, world!\", PHP_EOL;\n",
            "php main.php",
        ),
        (
            "rust",
            "rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n",
            "./main",
        ),
    ] {
        let response = solver.solve(&format!("Write a hello world program in {language}"));
        assert_eq!(response.intent, "write_program", "{response:?}");
        assert_eq!(
            response.answer.split("```").nth(1),
            Some(fence_and_code),
            "{response:?}"
        );
        assert!(response.answer.contains(command), "{response:?}");
        assert!(
            response.answer.contains("Execution status:"),
            "{response:?}"
        );
        assert!(
            !response.answer.contains("compiled and ran"),
            "{response:?}"
        );
    }
}

#[test]
fn offline_renderer_quotes_a_held_out_literal_for_each_target() {
    let solver = offline_solver();
    for (language, fence_and_code) in [
        ("bash", "bash\nprintf '%s\\n' 'Three {plums} $4'\n"),
        (
            "haskell",
            "haskell\nmain :: IO ()\nmain = putStrLn \"Three {plums} $4\"\n",
        ),
        (
            "rust",
            "rust\nfn main() {\n    println!(\"{}\", \"Three {plums} $4\");\n}\n",
        ),
    ] {
        let response = solver.solve(&format!(
            "Write a program in {language} to print \"Three {{plums}} $4\""
        ));
        assert_eq!(response.intent, "write_program", "{response:?}");
        assert_eq!(
            response.answer.split("```").nth(1),
            Some(fence_and_code),
            "{response:?}"
        );
        assert!(
            !response.answer.contains("compiled and ran"),
            "{response:?}"
        );
    }
}
