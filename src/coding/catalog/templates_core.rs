//! Templates for the original coding tasks — hello world, count to three, and
//! the two directory-listing variants — in every supported language. Split from
//! [`super::templates_extended`] only to keep each file well under the
//! repository's per-file line limit; the two groups are concatenated in
//! [`super`].

use super::types::ProgramTemplate;

pub(super) const TEMPLATES_CORE: &[ProgramTemplate] = &[
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
