use super::extract_javascript_program;

#[test]
fn colon_bodies_preserve_literals_and_existing_wrappers() {
    for (prompt, expected) in [
        (
            r#"Run this JavaScript: console.log("Hello, world!");"#,
            r#"console.log("Hello, world!");"#,
        ),
        (
            "Run this JavaScript: console.log(`Hello, world!`);",
            "console.log(`Hello, world!`);",
        ),
        (
            "Run this JavaScript: console.log(6 * 7);",
            "console.log(6 * 7);",
        ),
        (
            "Run this JavaScript: `console.log(42);`",
            "console.log(42);",
        ),
        (
            r#"Run this JavaScript: "console.log(42);""#,
            "console.log(42);",
        ),
        (r#"Execute this JS "console.log(42);""#, "console.log(42);"),
        ("Execute this JS `console.log(42);`", "console.log(42);"),
        (
            "Run this JavaScript:\n```js\nconsole.log(42);\n```",
            "console.log(42);",
        ),
    ] {
        assert_eq!(
            extract_javascript_program(prompt).as_deref(),
            Some(expected),
            "{prompt}"
        );
    }
    assert_eq!(
        extract_javascript_program("Explain JavaScript: console.log(42);"),
        None
    );
}
