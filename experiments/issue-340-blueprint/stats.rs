//! Fetch JSON from a URL and report the mean and median of every number in it.
//!
//! Cargo.toml dependencies:
//!   reqwest = { version = "0.12", features = ["blocking", "json"] }
//!   serde_json = "1"

use std::env;
use std::error::Error;

use serde_json::Value;

/// Recursively collect every numeric value out of a decoded JSON document,
/// regardless of how deeply it is nested inside arrays or objects.
fn collect_numbers(value: &Value, numbers: &mut Vec<f64>) {
    match value {
        Value::Number(number) => {
            if let Some(as_float) = number.as_f64() {
                numbers.push(as_float);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_numbers(item, numbers)),
        Value::Object(map) => map.values().for_each(|item| collect_numbers(item, numbers)),
        _ => {}
    }
}

/// Arithmetic mean of the samples (the caller guarantees a non-empty slice).
fn mean(samples: &[f64]) -> f64 {
    samples.iter().sum::<f64>() / samples.len() as f64
}

/// Median of the samples; averages the two middle values when the count is even.
fn median(samples: &mut [f64]) -> f64 {
    samples.sort_by(|left, right| left.partial_cmp(right).expect("no NaN in input"));
    let middle = samples.len() / 2;
    if samples.len() % 2 == 0 {
        (samples[middle - 1] + samples[middle]) / 2.0
    } else {
        samples[middle]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Read the target URL from the first command-line argument.
    let url = env::args()
        .nth(1)
        .ok_or("usage: stats <url-returning-json>")?;

    // 2. Make the HTTP GET request and parse the JSON body. Both steps can fail,
    //    so `?` propagates any network or decoding error up to `main`.
    let document: Value = reqwest::blocking::get(&url)?.json()?;

    // 3. Gather every number, then guard against an empty data set.
    let mut numbers = Vec::new();
    collect_numbers(&document, &mut numbers);
    if numbers.is_empty() {
        return Err("the JSON response contained no numbers".into());
    }

    // 4. Compute and print the statistics.
    println!("count:  {}", numbers.len());
    println!("mean:   {:.4}", mean(&numbers));
    println!("median: {:.4}", median(&mut numbers));
    Ok(())
}
