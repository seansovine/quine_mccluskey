//! Test program for logical equivalence checking code.

use std::error::Error;

use logic_minimization::{check::sop_string_to_init, qm_simplify_init};

const TEST_SOP_STRING: &str = "(A & !F) | (B & !C & D)";

fn main() -> Result<(), Box<dyn Error>> {
    let sop_string = TEST_SOP_STRING;
    println!("Converting sum-of-products to INIT: {sop_string}");
    let init_string = sop_string_to_init(sop_string);
    println!("INIT string: 16'h{init_string}");

    // Now convert back to sum-of-products string.
    let (sop_string, _time) = qm_simplify_init(&init_string)?;
    println!("Back to sum-of-products: {sop_string}");

    Ok(())
}
