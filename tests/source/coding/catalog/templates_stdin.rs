//! Templates for the standard-stream task added in issue #863 — copy standard
//! input to standard output — in every supported language. Split from
//! [`super::templates_core`] only to keep each file well under the repository's
//! per-file line limit; the groups are concatenated in [`super`].
//!
//! Unlike every other group, these programs are not self-contained: their
//! output is whatever arrives on standard input. The bytes they are verified
//! against are named once, on the task itself
//! (`ProgramTask::input` in [`super::tasks`]), and
//! [`super::types::ProgramSpec::run_command_line`] pipes them into the run
//! command so the answer a reader copies is the command that was checked
//! (`experiments/issue-1021-copy-stdin`).

use super::types::ProgramTemplate;

pub(super) const TEMPLATES_STDIN: &[ProgramTemplate] = &[
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "rust",
        code: r"use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    io::stdout().write_all(&input)
}",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "python",
        code: r"import sys

sys.stdout.write(sys.stdin.read())",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "javascript",
        code: r#"const fs = require("fs");

process.stdout.write(fs.readFileSync(0));"#,
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "typescript",
        code: r#"import * as fs from "fs";

process.stdout.write(fs.readFileSync(0));"#,
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
        code: r"$stdout.write($stdin.read)",
    },
    ProgramTemplate {
        task_slug: "copy_stdin_to_stdout",
        language_slug: "scala",
        code: r"object Main {
  def main(args: Array[String]): Unit = {
    print(scala.io.Source.stdin.mkString)
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
