//! The interpreter of the instruction set. Rust cannot evaluate the seed's
//! JavaScript `code`, so each operation is implemented here, keyed by its
//! primitive id, with the JavaScript semantics of that code. An id the
//! interpreter does not know behaves as a JavaScript throw (`None`): the
//! program is still enumerated and rendered, it just cannot be run here.
//!
//! Operations that need the `node` environment (the file system, a shell,
//! the process) only run when the caller grants it; the solver never does,
//! matching the web worker, where `require` is absent and such code throws.
#![allow(clippy::float_cmp, clippy::cast_precision_loss)]

use std::cmp::Ordering;

use super::catalog::{Catalog, Prim, Step};
use super::text::{Example, js_trim};
use super::value::{Value, js_join, js_less, same_value_zero};

/// The parameter of a program: `undefined` (inference failed), `null` (the
/// program takes none) or a number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Param {
    /// No parameter could be inferred.
    Undefined,
    /// The program takes no parameter.
    Null,
    /// The parameter's value.
    Number(f64),
}

impl Param {
    /// The parameter as the JavaScript text `String(parameter)` substitutes
    /// into `{k}`.
    ///
    /// Mirrors `String(parameter)` in `metaRender` (js/worker/formal_ai_worker_meta_synthesis.js).
    #[must_use]
    pub fn to_js(self) -> String {
        match self {
            Self::Undefined => String::from("undefined"),
            Self::Null => String::from("null"),
            Self::Number(number) => super::value::js_number(number),
        }
    }

    /// The numeric value the substituted literal evaluates to in arithmetic.
    #[must_use]
    pub const fn numeric(self) -> f64 {
        match self {
            Self::Undefined => f64::NAN,
            Self::Null => 0.0,
            Self::Number(number) => number,
        }
    }
}

/// Where a program runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Runtime {
    /// The web worker: no file system, no shell, no process.
    #[default]
    Worker,
    /// Node: the file system and a shell are available.
    Node,
}

fn text_list(items: Vec<String>) -> Value {
    Value::List(items.into_iter().map(Value::Text).collect())
}

fn numbers(input: &Value) -> Option<Vec<f64>> {
    input
        .as_list()
        .map(|items| items.iter().map(Value::to_js_number).collect())
}

/// One seed primitive applied to a value, by id.
///
/// Mirrors running a primitive's compiled `code` (`metaCompile` in
/// js/worker/formal_ai_worker_meta_synthesis.js).
#[must_use]
pub fn apply_primitive(
    id: &str,
    input: &Value,
    parameter: Param,
    runtime: Runtime,
) -> Option<Value> {
    let k = parameter.numeric();
    match id {
        "split_words" => Some(text_list(
            js_trim(input.as_str()?)
                .split(' ')
                .filter(|word| !word.is_empty())
                .map(str::to_owned)
                .collect(),
        )),
        "join_words" => Some(Value::Text(js_join(input.as_list()?, " "))),
        "split_lines" => Some(text_list(
            input.as_str()?.split('\n').map(str::to_owned).collect(),
        )),
        "join_lines" | "print_lines" => {
            let text = js_join(input.as_list()?, "\n");
            if id == "print_lines" && runtime == Runtime::Node {
                println!("{text}");
            }
            Some(Value::Text(text))
        }
        "split_characters" => Some(text_list(
            input.as_str()?.chars().map(String::from).collect(),
        )),
        "join_characters" => Some(Value::Text(js_join(input.as_list()?, ""))),
        "reverse_text" => Some(Value::Text(input.as_str()?.chars().rev().collect())),
        "upper_case" => Some(Value::Text(input.as_str()?.to_uppercase())),
        "lower_case" => Some(Value::Text(input.as_str()?.to_lowercase())),
        "capitalize_first" => {
            let text = input.as_str()?;
            let mut characters = text.chars();
            let Some(first) = characters.next() else {
                return Some(Value::Text(String::new()));
            };
            // `charAt(0)` is one UTF-16 unit: a character outside the basic
            // plane is half a surrogate pair, which upper-casing leaves alone.
            if u32::from(first) > 0xFFFF {
                return Some(Value::Text(text.to_owned()));
            }
            Some(Value::Text(
                [
                    first.to_uppercase().collect::<String>(),
                    characters.collect(),
                ]
                .concat(),
            ))
        }
        "trim_text" => Some(Value::Text(js_trim(input.as_str()?).to_owned())),
        "text_length" => Some(Value::Number(input.as_str()?.chars().count() as f64)),
        "reverse_list" => {
            let mut items = input.as_list()?.to_vec();
            items.reverse();
            Some(Value::List(items))
        }
        "sort_list" => {
            let mut items = input.as_list()?.to_vec();
            items.sort_by(|a, b| {
                if js_less(a, b) {
                    Ordering::Less
                } else if js_less(b, a) {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });
            Some(Value::List(items))
        }
        "unique_items" => {
            let mut out: Vec<Value> = Vec::new();
            for item in input.as_list()? {
                if !out.iter().any(|kept| same_value_zero(kept, item)) {
                    out.push(item.clone());
                }
            }
            Some(Value::List(out))
        }
        "count_items" => match input {
            Value::List(items) => Some(Value::Number(items.len() as f64)),
            Value::Text(text) | Value::Path(text) => {
                Some(Value::Number(text.encode_utf16().count() as f64))
            }
            _ => None,
        },
        "sum_numbers" => Some(Value::Number(numbers(input)?.iter().sum())),
        "product_numbers" => Some(Value::Number(numbers(input)?.iter().product())),
        "maximum_number" => Some(Value::Number(numbers(input)?.into_iter().fold(
            f64::NEG_INFINITY,
            |best, value| {
                if value.is_nan() || best.is_nan() {
                    f64::NAN
                } else {
                    best.max(value)
                }
            },
        ))),
        "minimum_number" => Some(Value::Number(numbers(input)?.into_iter().fold(
            f64::INFINITY,
            |best, value| {
                if value.is_nan() || best.is_nan() {
                    f64::NAN
                } else {
                    best.min(value)
                }
            },
        ))),
        "multiply_by" => Some(Value::Number(input.to_js_number() * k)),
        "add_constant" => match input {
            Value::Number(number) => Some(Value::Number(number + k)),
            _ => None,
        },
        "square_number" => {
            let number = input.to_js_number();
            Some(Value::Number(number * number))
        }
        "absolute_number" => Some(Value::Number(input.to_js_number().abs())),
        "keep_even" | "keep_odd" => {
            let even = id == "keep_even";
            let kept = input
                .as_list()?
                .iter()
                .filter(|item| (item.to_js_number() % 2.0 == 0.0) == even)
                .cloned()
                .collect();
            Some(Value::List(kept))
        }
        "list_files" | "read_file" | "run_command" | "fail_when_any" => {
            node_primitive(id, input, runtime)
        }
        _ => None,
    }
}

/// The primitives that need the `node` environment.
fn node_primitive(id: &str, input: &Value, runtime: Runtime) -> Option<Value> {
    if runtime != Runtime::Node {
        return None;
    }
    match id {
        "list_files" => {
            let folder = input.as_str()?;
            let mut names: Vec<String> = std::fs::read_dir(folder)
                .ok()?
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            Some(Value::List(
                names
                    .iter()
                    .map(|name| Value::Path(path_join(folder, name)))
                    .collect(),
            ))
        }
        "read_file" => {
            let bytes = std::fs::read(input.as_str()?).ok()?;
            Some(Value::Text(String::from_utf8_lossy(&bytes).into_owned()))
        }
        "run_command" => {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(input.as_str()?)
                .output()
                .ok()?;
            output
                .status
                .success()
                .then(|| Value::Text(String::from_utf8_lossy(&output.stdout).into_owned()))
        }
        "fail_when_any" => Some(Value::Number(input.as_list()?.len() as f64)),
        _ => None,
    }
}

/// `path.join(folder, name)` with POSIX normalisation.
///
/// Mirrors `require('path').join` in the `list_files` code of
/// data/seed/meta-reasoning.lino.
#[must_use]
pub fn path_join(folder: &str, name: &str) -> String {
    let joined = [folder, "/", name].concat();
    let absolute = joined.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for segment in joined.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|last| *last != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..");
                }
            }
            other => parts.push(other),
        }
    }
    let body = parts.join("/");
    if absolute {
        ["/", &body].concat()
    } else if body.is_empty() {
        String::from(".")
    } else {
        body
    }
}

/// The comparison of a filter test `(measure, threshold) => measure OP threshold`.
fn compare(test: &str, measure: f64, threshold: f64) -> Option<bool> {
    let body = test.rsplit("=>").next().unwrap_or(test);
    for (operator, result) in [
        (">=", measure >= threshold),
        ("<=", measure <= threshold),
        ("===", measure == threshold),
        ("!==", measure != threshold),
        ("==", measure == threshold),
        ("!=", measure != threshold),
        (">", measure > threshold),
        ("<", measure < threshold),
    ] {
        if body.contains(operator) {
            return Some(result);
        }
    }
    None
}

/// A filter's test applied to one measured element.
///
/// Mirrors running a filter's compiled `test` (`metaRun` in
/// js/worker/formal_ai_worker_meta_synthesis.js).
#[must_use]
pub fn filter_test(test: &str, measure: &Value, threshold: Param) -> Option<bool> {
    compare(test, measure.to_js_number(), threshold.numeric())
}

/// A program operation (a primitive or a composed measure) applied to a value.
///
/// Mirrors `metaCompile(primitive, parameter)(value)` in
/// js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn apply_prim(
    catalog: &Catalog,
    prim: &Prim,
    input: &Value,
    parameter: Param,
    runtime: Runtime,
) -> Option<Value> {
    if !prim.measure {
        return apply_primitive(&prim.id, input, parameter, runtime);
    }
    let mut value = input.clone();
    for index in &prim.chain {
        let part = catalog.prims.get(*index)?;
        value = apply_primitive(&part.id, &value, Param::Null, runtime)?;
    }
    Some(value)
}

fn rewrite(catalog: &Catalog, prim: &Prim, input: &Value, runtime: Runtime) -> Option<Value> {
    if runtime != Runtime::Node {
        return None;
    }
    let path = input.as_str()?;
    let text = std::fs::read(path).ok()?;
    let original = Value::Text(String::from_utf8_lossy(&text).into_owned());
    let changed = apply_prim(catalog, prim, &original, Param::Null, runtime)?;
    std::fs::write(path, changed.as_str()?).ok()?;
    Some(input.clone())
}

/// Run a program on one input; a parametric step takes `parameter`.
///
/// Mirrors `metaRun` in js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn run_program(
    catalog: &Catalog,
    steps: &[Step],
    input: &Value,
    parameter: Param,
    runtime: Runtime,
) -> Option<Value> {
    let mut value = input.clone();
    for step in steps {
        let prim = catalog.prim(*step);
        let own = if prim.infer.is_empty() {
            Param::Null
        } else {
            parameter
        };
        value = if step.rewrite {
            if step.mapped {
                Value::List(
                    value
                        .as_list()?
                        .iter()
                        .map(|item| rewrite(catalog, prim, item, runtime))
                        .collect::<Option<Vec<_>>>()?,
                )
            } else {
                rewrite(catalog, prim, &value, runtime)?
            }
        } else if let Some(filter) = step.filter {
            let test = &catalog.filters[usize::from(filter)].test;
            let mut kept = Vec::new();
            for item in value.as_list()? {
                let measured = apply_prim(catalog, prim, item, Param::Null, runtime)?;
                if filter_test(test, &measured, parameter)? {
                    kept.push(item.clone());
                }
            }
            Value::List(kept)
        } else if step.mapped {
            Value::List(
                value
                    .as_list()?
                    .iter()
                    .map(|item| apply_prim(catalog, prim, item, own, runtime))
                    .collect::<Option<Vec<_>>>()?,
            )
        } else {
            apply_prim(catalog, prim, &value, own, runtime)?
        };
    }
    Some(value)
}

/// One parameter inference rule of the seed applied to an example pair:
/// a ratio (`output / input`) or a difference (`output - input`).
///
/// Mirrors running a primitive's compiled `infer` in `metaInferParameter`
/// (js/worker/formal_ai_worker_meta_synthesis.js).
fn infer_rule(rule: &str, input: &Value, output: &Value) -> Param {
    // The rule's result expression is the text after its last arrow; its
    // operator decides ratio or difference.
    let expression = rule.rsplit("=>").next().unwrap_or_default();
    if expression.contains('/') {
        if matches!(input, Value::Number(number) if *number == 0.0) {
            return Param::Null;
        }
        return Param::Number(output.to_js_number() / input.to_js_number());
    }
    if expression.contains('-') {
        return Param::Number(output.to_js_number() - input.to_js_number());
    }
    Param::Undefined
}

/// Infer the parameter of a program's parametric last step: an arithmetic
/// parameter from the first example, a filter threshold from all of them.
///
/// Mirrors `metaInferParameter` in js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn infer_parameter(catalog: &Catalog, steps: &[Step], examples: &[Example]) -> Param {
    let (Some(last), Some(example)) = (steps.last(), examples.first()) else {
        return Param::Undefined;
    };
    let before_steps = &steps[..steps.len() - 1];
    if before_steps.iter().any(|step| catalog.is_parametric(*step)) {
        return Param::Undefined;
    }
    if last.filter.is_some() {
        return infer_threshold(catalog, steps, examples);
    }
    let prim = catalog.prim(*last);
    if prim.infer.is_empty() {
        return Param::Null;
    }
    let Some(before) = run_program(
        catalog,
        before_steps,
        &example.input,
        Param::Null,
        Runtime::Worker,
    ) else {
        return Param::Undefined;
    };
    let pair = if last.mapped {
        (
            before.as_list().and_then(<[Value]>::first),
            example.output.as_list().and_then(<[Value]>::first),
        )
    } else {
        (Some(&before), Some(&example.output))
    };
    let (Some(input), Some(output)) = pair else {
        return Param::Undefined;
    };
    match infer_rule(&prim.infer, input, output) {
        Param::Number(number) if number.is_finite() => Param::Number(number),
        _ => Param::Undefined,
    }
}

/// The threshold separating the kept elements from the dropped ones across
/// every example, or `Undefined` when the output is not a filtering of the
/// input or no threshold separates them.
///
/// Mirrors `metaInferThreshold` in js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn infer_threshold(catalog: &Catalog, steps: &[Step], examples: &[Example]) -> Param {
    let Some(last) = steps.last() else {
        return Param::Undefined;
    };
    let Some(filter) = last.filter else {
        return Param::Undefined;
    };
    let prim = catalog.prim(*last);
    let before_steps = &steps[..steps.len() - 1];
    let mut kept: Vec<f64> = Vec::new();
    let mut dropped: Vec<f64> = Vec::new();
    for example in examples {
        let Some(before) = run_program(
            catalog,
            before_steps,
            &example.input,
            Param::Null,
            Runtime::Worker,
        ) else {
            return Param::Undefined;
        };
        let (Some(items), Some(expected)) = (before.as_list(), example.output.as_list()) else {
            return Param::Undefined;
        };
        let mut cursor = 0;
        for item in items {
            let Some(measured) = apply_prim(catalog, prim, item, Param::Null, Runtime::Worker)
            else {
                return Param::Undefined;
            };
            let measure = measured.to_js_number();
            if cursor < expected.len() && item.to_json() == expected[cursor].to_json() {
                kept.push(measure);
                cursor += 1;
            } else {
                dropped.push(measure);
            }
        }
        if cursor != expected.len() {
            return Param::Undefined;
        }
    }
    if kept.is_empty() || dropped.is_empty() {
        return Param::Undefined;
    }
    let test = &catalog.filters[usize::from(filter)].test;
    let mut candidates: Vec<f64> = dropped.iter().chain(kept.iter()).copied().collect();
    candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    for candidate in candidates {
        let threshold = Param::Number(candidate);
        let keeps_all = kept
            .iter()
            .all(|value| compare(test, *value, threshold.numeric()) == Some(true));
        let drops_all = dropped
            .iter()
            .all(|value| compare(test, *value, threshold.numeric()) == Some(false));
        if keeps_all && drops_all {
            return threshold;
        }
    }
    Param::Undefined
}
