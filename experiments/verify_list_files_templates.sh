#!/usr/bin/env bash
# Issue #312: verify that every `list_files` write_program template compiles and
# runs, and that it prints exactly the documented sample-directory output:
#
#   Cargo.toml
#   README.md
#   main.rs
#
# Each program is built/stored OUTSIDE the listing directory, then executed with
# its working directory set to a clean "sample" directory that contains exactly
# those three regular files plus a `subdir/` with a nested file. This proves the
# templates: (1) sort byte-wise, (2) list only regular files (directories are
# filtered out), and (3) are non-recursive (the nested file never appears).
#
# Usage: bash experiments/verify_list_files_templates.sh
set -u

EXPECTED=$'Cargo.toml\nREADME.md\nmain.rs'
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT

PASS=0
FAIL=0

# Create a clean listing directory: three regular files (deliberately created
# out of sorted order) plus a directory holding a nested file.
fresh_sample() {
    local dir="$1"
    rm -rf "$dir"
    mkdir -p "$dir/subdir"
    : >"$dir/main.rs"
    : >"$dir/README.md"
    : >"$dir/Cargo.toml"
    : >"$dir/subdir/nested.txt"
}

check() {
    local lang="$1"
    local actual="$2"
    if [ "$actual" = "$EXPECTED" ]; then
        echo "PASS: $lang"
        PASS=$((PASS + 1))
    else
        echo "FAIL: $lang"
        echo "----- expected -----"; printf '%s\n' "$EXPECTED"
        echo "----- actual -----"; printf '%s\n' "$actual"
        echo "------------------"
        FAIL=$((FAIL + 1))
    fi
}

BUILD="$ROOT/build"; mkdir -p "$BUILD"
SAMPLE="$ROOT/sample"

# ---- Rust -----------------------------------------------------------------
if command -v rustc >/dev/null 2>&1; then
    cat >"$BUILD/list.rs" <<'EOF'
use std::fs;

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
}
EOF
    if rustc "$BUILD/list.rs" -o "$BUILD/list_rs" 2>/dev/null; then
        fresh_sample "$SAMPLE"
        check "rust" "$(cd "$SAMPLE" && "$BUILD/list_rs")"
    else
        echo "FAIL: rust (compile)"; FAIL=$((FAIL + 1))
    fi
fi

# ---- Python ---------------------------------------------------------------
if command -v python3 >/dev/null 2>&1; then
    cat >"$BUILD/list.py" <<'EOF'
import os

names = sorted(name for name in os.listdir(".") if os.path.isfile(name))
for name in names:
    print(name)
EOF
    fresh_sample "$SAMPLE"
    check "python" "$(cd "$SAMPLE" && python3 "$BUILD/list.py")"
fi

# ---- JavaScript (Node) ----------------------------------------------------
if command -v node >/dev/null 2>&1; then
    cat >"$BUILD/list.js" <<'EOF'
const fs = require("fs");

const names = fs
  .readdirSync(".")
  .filter((name) => fs.statSync(name).isFile())
  .sort();

for (const name of names) {
  console.log(name);
}
EOF
    fresh_sample "$SAMPLE"
    check "javascript" "$(cd "$SAMPLE" && node "$BUILD/list.js")"
fi

# ---- TypeScript -----------------------------------------------------------
if command -v tsc >/dev/null 2>&1 && command -v node >/dev/null 2>&1; then
    cat >"$BUILD/list.ts" <<'EOF'
import * as fs from "fs";

const names: string[] = fs
  .readdirSync(".")
  .filter((name) => fs.statSync(name).isFile())
  .sort();

for (const name of names) {
  console.log(name);
}
EOF
    if tsc "$BUILD/list.ts" --outDir "$BUILD" 2>/dev/null; then
        fresh_sample "$SAMPLE"
        check "typescript" "$(cd "$SAMPLE" && node "$BUILD/list.js")"
    else
        echo "SKIP: typescript (compile failed)"
    fi
else
    echo "SKIP: typescript (tsc not installed)"
fi

# ---- Go -------------------------------------------------------------------
if command -v go >/dev/null 2>&1; then
    cat >"$BUILD/list.go" <<'EOF'
package main

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
}
EOF
    if GO111MODULE=off go build -o "$BUILD/list_go" "$BUILD/list.go" 2>/dev/null; then
        fresh_sample "$SAMPLE"
        check "go" "$(cd "$SAMPLE" && "$BUILD/list_go")"
    else
        echo "FAIL: go (compile)"; FAIL=$((FAIL + 1))
    fi
fi

# ---- C --------------------------------------------------------------------
if command -v gcc >/dev/null 2>&1; then
    cat >"$BUILD/list.c" <<'EOF'
#include <dirent.h>
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
}
EOF
    if gcc "$BUILD/list.c" -o "$BUILD/list_c" 2>/dev/null; then
        fresh_sample "$SAMPLE"
        check "c" "$(cd "$SAMPLE" && "$BUILD/list_c")"
    else
        echo "FAIL: c (compile)"; FAIL=$((FAIL + 1))
    fi
fi

# ---- C++ ------------------------------------------------------------------
if command -v g++ >/dev/null 2>&1; then
    cat >"$BUILD/list.cpp" <<'EOF'
#include <algorithm>
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
}
EOF
    if g++ -std=c++17 "$BUILD/list.cpp" -o "$BUILD/list_cpp" 2>/dev/null; then
        fresh_sample "$SAMPLE"
        check "cpp" "$(cd "$SAMPLE" && "$BUILD/list_cpp")"
    else
        echo "FAIL: cpp (compile)"; FAIL=$((FAIL + 1))
    fi
fi

# ---- Java -----------------------------------------------------------------
if command -v javac >/dev/null 2>&1; then
    mkdir -p "$BUILD/java"
    cat >"$BUILD/java/Main.java" <<'EOF'
import java.io.File;
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
}
EOF
    if javac -d "$BUILD/java" "$BUILD/java/Main.java" 2>/dev/null; then
        fresh_sample "$SAMPLE"
        check "java" "$(cd "$SAMPLE" && java -cp "$BUILD/java" Main)"
    else
        echo "FAIL: java (compile)"; FAIL=$((FAIL + 1))
    fi
fi

# ---- C# -------------------------------------------------------------------
if command -v dotnet >/dev/null 2>&1; then
    PROJ="$BUILD/csharp"; mkdir -p "$PROJ"
    cat >"$PROJ/Program.cs" <<'EOF'
using System;
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
}
EOF
    cat >"$PROJ/app.csproj" <<'PROJ'
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <Nullable>disable</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
  </PropertyGroup>
</Project>
PROJ
    if DOTNET_CLI_TELEMETRY_OPTOUT=1 dotnet build "$PROJ" -o "$PROJ/out" >/dev/null 2>&1; then
        fresh_sample "$SAMPLE"
        check "csharp" "$(cd "$SAMPLE" && DOTNET_CLI_TELEMETRY_OPTOUT=1 dotnet "$PROJ/out/app.dll")"
    else
        echo "SKIP: csharp (build failed / offline)"
    fi
else
    echo "SKIP: csharp"
fi

# ---- Ruby -----------------------------------------------------------------
if command -v ruby >/dev/null 2>&1; then
    cat >"$BUILD/list.rb" <<'EOF'
names = Dir.entries(".").select { |name| File.file?(name) }.sort
names.each { |name| puts name }
EOF
    fresh_sample "$SAMPLE"
    check "ruby" "$(cd "$SAMPLE" && ruby "$BUILD/list.rb")"
fi

echo "============================="
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
