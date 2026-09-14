//! Does `observed_directory` splice a workspace out of an unrelated path?
//!
//! Run: `cargo +1.98.1 run --example issue_1075_observed_directory`

fn main() {
    // Reproduces the shape by hand, mirroring the function's own logic, so the
    // splice can be seen without a server round-trip.
    for (planned, token) in [
        ("alpha.txt", "/tmp/session-31/alpha.txt"), // the intended case
        ("ls", "/usr/bin/tools"),                   // a mid-segment splice
    ] {
        let spliced = token
            .strip_suffix(planned)
            .map(|root| root.trim_end_matches('/'))
            .filter(|root| !root.is_empty() && std::path::Path::new(root).is_absolute());
        println!("planned={planned:<12} token={token:<24} -> {spliced:?}");
    }
}
