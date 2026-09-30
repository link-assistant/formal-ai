<!-- Trimmed capture in the shape of a Rust Book page (issue #1163
     fixture): Markdown-served documentation with rust-fenced code. -->
# Running a Program

To run a Rust program, compile it and execute the binary.

```rust
fn main() {
    println!("Hello, world!");
}
```

The `rustc` command compiles a single crate:

```shell
rustc main.rs
```

- Compilation produces a binary named after the source file.
- The binary lands next to the source unless `-o` says otherwise.
