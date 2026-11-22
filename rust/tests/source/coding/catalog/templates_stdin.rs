//! Templates for the standard-stream task added in issue #863 — copy standard
//! input to standard output — in every supported language. Split from
//! [`super::templates_core`] only to keep each file well under the repository's
//! per-file line limit; the groups are concatenated in [`super`].
//!
//! These programs copy the bytes the caller supplies on standard input.
//! Verification provides an explicit input stream and compares the output.

use super::types::ProgramTemplate;

pub(super) const TEMPLATES_STDIN: &[ProgramTemplate] = &[
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "rust",
        code: r"use std::io;

fn main() -> io::Result<()> {
    io::copy(&mut io::stdin().lock(), &mut io::stdout().lock())?;
    Ok(())
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "python",
        code: r"import shutil
import sys

shutil.copyfileobj(sys.stdin.buffer, sys.stdout.buffer)",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "javascript",
        code: r"process.stdin.pipe(process.stdout);",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "typescript",
        code: r"process.stdin.pipe(process.stdout);",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "go",
        code: r#"package main

import (
    "io"
    "os"
)

func main() {
    if _, err := io.Copy(os.Stdout, os.Stdin); err != nil {
        panic(err)
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "c",
        code: r"#include <stdio.h>

int main(void) {
    int character;
    while ((character = getchar()) != EOF) {
        putchar(character);
    }
    return 0;
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "cpp",
        code: r"#include <iostream>

int main() {
    std::cout << std::cin.rdbuf();
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "java",
        code: r"import java.io.IOException;

public class Main {
    public static void main(String[] args) throws IOException {
        System.in.transferTo(System.out);
    }
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "csharp",
        code: r"using System;
using System.IO;

class Program {
    static void Main() {
        using (Stream input = Console.OpenStandardInput())
        using (Stream output = Console.OpenStandardOutput()) {
            input.CopyTo(output);
        }
    }
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "ruby",
        code: r"IO.copy_stream($stdin.binmode, $stdout.binmode)",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "scala",
        code: r"object Main {
  def main(args: Array[String]): Unit = {
    System.in.transferTo(System.out)
  }
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "kotlin",
        code: r"fun main() {
    System.`in`.copyTo(System.out)
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "php",
        code: r"<?php

fpassthru(STDIN);",
    },
];
