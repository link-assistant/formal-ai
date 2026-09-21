//! Templates for the reverse-sorted directory-listing tasks
//! (`list_files_reverse_sort`, `list_files_arg_reverse_sort`) in every
//! supported language. Split from [`super::templates_core`] only to keep each
//! file well under the repository's per-file line limit; the groups are
//! concatenated in [`super`].

use super::types::ProgramTemplate;

pub(super) const TEMPLATES_LISTING: &[ProgramTemplate] = &[
    // Issue #358: reverse-sort variants are ordinary catalog tasks reached by
    // data-defined program-plan modifiers. The templates keep the same file
    // filtering as `list_files` / `list_files_arg`, but emit descending names.
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "rust",
        code: r#"use std::fs;

fn main() -> std::io::Result<()> {
    let mut names: Vec<String> = fs::read_dir(".")?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort_by(|a, b| b.cmp(a));
    for name in names {
        println!("{name}");
    }
    Ok(())
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "python",
        code: r#"import os

names = sorted(
    (name for name in os.listdir(".") if os.path.isfile(name)),
    reverse=True,
)
for name in names:
    print(name)"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "javascript",
        code: r#"const fs = require("fs");

const names = fs
  .readdirSync(".")
  .filter((name) => fs.statSync(name).isFile())
  .sort()
  .reverse();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "typescript",
        code: r#"import * as fs from "fs";

const names: string[] = fs
  .readdirSync(".")
  .filter((name) => fs.statSync(name).isFile())
  .sort()
  .reverse();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
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
    sort.Sort(sort.Reverse(sort.StringSlice(names)))
    for _, name := range names {
        fmt.Println(name)
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "c",
        code: r#"#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static int compare_desc(const void *a, const void *b) {
    return strcmp(*(const char *const *)b, *(const char *const *)a);
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
    qsort(names, count, sizeof(char *), compare_desc);
    for (size_t i = 0; i < count; i++) {
        printf("%s\n", names[i]);
        free(names[i]);
    }
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
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
    std::sort(names.rbegin(), names.rend());
    for (const auto &name : names) {
        std::cout << name << '\n';
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "java",
        code: r#"import java.io.File;
import java.util.Arrays;
import java.util.Comparator;

public class Main {
    public static void main(String[] args) {
        File[] entries = new File(".").listFiles();
        if (entries == null) {
            return;
        }
        String[] names = Arrays.stream(entries)
            .filter(File::isFile)
            .map(File::getName)
            .sorted(Comparator.reverseOrder())
            .toArray(String[]::new);
        for (String name : names) {
            System.out.println(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "csharp",
        code: r#"using System;
using System.IO;
using System.Linq;

class Program {
    static void Main() {
        var names = Directory.GetFiles(".")
            .Select(Path.GetFileName)
            .OrderByDescending(name => name, StringComparer.Ordinal);
        foreach (var name in names) {
            Console.WriteLine(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "ruby",
        code: r#"names = Dir.entries(".").select { |name| File.file?(name) }.sort.reverse
names.each { |name| puts name }"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "scala",
        code: r#"import java.io.File

object Main {
  def main(args: Array[String]): Unit = {
    val entries = Option(new File(".").listFiles()).getOrElse(Array.empty[File])
    entries.filter(_.isFile).map(_.getName).sorted.reverse.foreach(println)
  }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "kotlin",
        code: r#"import java.io.File

fun main() {
    val entries = File(".").listFiles() ?: return
    entries.filter { it.isFile }.map { it.name }.sortedDescending().forEach { println(it) }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_reverse_sort",
        language_slug: "php",
        code: r#"<?php

$names = array_filter(scandir("."), "is_file");
rsort($names);
foreach ($names as $name) {
    echo $name, PHP_EOL;
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
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
    names.sort_by(|a, b| b.cmp(a));
    for name in names {
        println!("{name}");
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "python",
        code: r#"import os
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "."
names = sorted(
    (
        name
        for name in os.listdir(path)
        if os.path.isfile(os.path.join(path, name))
    ),
    reverse=True,
)
for name in names:
    print(name)"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "javascript",
        code: r#"const fs = require("fs");
const path = require("path");

const dir = process.argv[2] || ".";
const names = fs
  .readdirSync(dir)
  .filter((name) => fs.statSync(path.join(dir, name)).isFile())
  .sort()
  .reverse();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "typescript",
        code: r#"import * as fs from "fs";
import * as path from "path";

const dir: string = process.argv[2] ?? ".";
const names: string[] = fs
  .readdirSync(dir)
  .filter((name) => fs.statSync(path.join(dir, name)).isFile())
  .sort()
  .reverse();

for (const name of names) {
  console.log(name);
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
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
    sort.Sort(sort.Reverse(sort.StringSlice(names)))
    for _, name := range names {
        fmt.Println(name)
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "c",
        code: r#"#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static int compare_desc(const void *a, const void *b) {
    return strcmp(*(const char *const *)b, *(const char *const *)a);
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
    qsort(names, count, sizeof(char *), compare_desc);
    for (size_t i = 0; i < count; i++) {
        printf("%s\n", names[i]);
        free(names[i]);
    }
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
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
    std::sort(names.rbegin(), names.rend());
    for (const auto &name : names) {
        std::cout << name << '\n';
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "java",
        code: r#"import java.io.File;
import java.util.Arrays;
import java.util.Comparator;

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
            .sorted(Comparator.reverseOrder())
            .toArray(String[]::new);
        for (String name : names) {
            System.out.println(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "csharp",
        code: r#"using System;
using System.IO;
using System.Linq;

class Program {
    static void Main(string[] args) {
        var path = args.Length > 0 ? args[0] : ".";
        var names = Directory.GetFiles(path)
            .Select(Path.GetFileName)
            .OrderByDescending(name => name, StringComparer.Ordinal);
        foreach (var name in names) {
            Console.WriteLine(name);
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "ruby",
        code: r#"path = ARGV[0] || "."
names = Dir.entries(path).select { |name| File.file?(File.join(path, name)) }.sort.reverse
names.each { |name| puts name }"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "scala",
        code: r#"import java.io.File

object Main {
  def main(args: Array[String]): Unit = {
    val path = if (args.nonEmpty) args(0) else "."
    val entries = Option(new File(path).listFiles()).getOrElse(Array.empty[File])
    entries.filter(_.isFile).map(_.getName).sorted.reverse.foreach(println)
  }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "kotlin",
        code: r#"import java.io.File

fun main(args: Array<String>) {
    val path = if (args.isNotEmpty()) args[0] else "."
    val entries = File(path).listFiles() ?: return
    entries.filter { it.isFile }.map { it.name }.sortedDescending().forEach { println(it) }
}"#,
    },
    ProgramTemplate {
        task_slug: "list_files_arg_reverse_sort",
        language_slug: "php",
        code: r#"<?php

$path = $argv[1] ?? ".";
$names = array_filter(scandir($path), fn($name) => is_file($path . DIRECTORY_SEPARATOR . $name));
rsort($names);
foreach ($names as $name) {
    echo $name, PHP_EOL;
}"#,
    },
];
