#!/usr/bin/env bash
# Issue #906 reproduction: the language router takes the word after "in" as the
# target language. Run from the repository root after `cargo build`.
set -u

BIN="${BIN:-./target/debug/formal-ai}"

prompts=(
  "Create a file named hello.txt in the current directory whose entire content is the single line: Hello World."
  "Write a program that prints hello world."
  "Fix the failing CI job in Rust."
  "Create a file named hello.txt containing Hello World, in JavaScript."
  "hello world in elvish"
  "write a hello world program in python"
  "напиши программу hello world на python"
)

for prompt in "${prompts[@]}"; do
  printf '=== PROMPT: %s\n' "$prompt"
  "$BIN" chat --silent --prompt "$prompt" 2>&1
  printf '\n'
done
