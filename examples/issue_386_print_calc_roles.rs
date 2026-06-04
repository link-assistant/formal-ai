//! Issue #386: print the calculator-domain role surfaces.
//!
//! Throwaway diagnostic used while converting `has_calculation_signal` from a
//! hardcoded signal array to role-driven lexicon queries. Prints the exact
//! surfaces each role yields so the rebuilt matcher can be checked for
//! faithfulness against the former array. Run with:
//!
//! ```sh
//! cargo run -p formal-ai --example issue_386_print_calc_roles
//! ```
use formal_ai::seed;

fn main() {
    let lexicon = seed::lexicon();
    for role in [
        seed::ROLE_MATH_FUNCTION_NAME,
        seed::ROLE_CALCULATION_DOMAIN_TERM,
        seed::ROLE_QUANTITY_CONVERSION_CUE,
    ] {
        println!("== {role} ==");
        for surface in lexicon.words_for_role(role) {
            let ascii = surface.is_ascii();
            println!("  {surface:?}  ascii={ascii}");
        }
    }
}
