//! Ask Formal AI to change itself, and get back a reviewable pull request (issue
//! #558, `R558-07`).
//!
//! Prints the canonical change request — a user's natural-language request turned into
//! a reviewable pull request: a derived requirement, a proposed test, and an ordered
//! patch plan against a target module grounded in the owned manifest. Nothing is
//! applied; the change merges only through the same human-gated loop the self-healing
//! slices use — a green benchmark gate *and* an explicit human approval.
//!
//! Like the whole-repository source-links projection, the document depends on the
//! whole source tree (the target module's manifest content id changes with every
//! edit), so it is a *workspace-only* artifact — asserted live in the tests, never
//! committed byte-for-byte. Usage: `cargo run --example request_change`. The one-line
//! summary and the accept/decline demonstration print to stderr; the full reviewable
//! pull request (Links Notation) prints to stdout.

use formal_ai::self_improvement::BenchmarkGateReport;
use formal_ai::{canonical_change_request, ChangeRejected, HumanApproval};

fn main() {
    let request = canonical_change_request();

    eprintln!("{}", request.summary());

    // Review demonstration: a green gate plus a granted approval merges the change; a
    // withheld approval refuses it — the same human gate as the learning ledger.
    let green = BenchmarkGateReport::issue_362_from_counts(4, 0);
    match request.review(&green, &HumanApproval::granted("maintainer")) {
        Ok(accepted) => eprintln!("accepted: {}", accepted.summary()),
        Err(reason) => eprintln!("rejected: {}", reason.slug()),
    }
    let declined = request.review(&green, &HumanApproval::declined("maintainer"));
    debug_assert_eq!(declined.err(), Some(ChangeRejected::HumanDeclined));

    println!("{}", request.links_notation());
}
