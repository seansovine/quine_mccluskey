#![allow(unused)]

use std::error::Error;

use logic_minimization::*;

fn main() -> Result<(), Box<dyn Error>> {
    const TEST_INIT_STR_1: &str = "0000F0F0000000FF";
    // Should simplify to (B & !C & !D & !E & !F).
    const TEST_INIT_STR_2: &str = "000000000000000C";
    // Should zero-pad to same as previous.
    const TEST_INIT_STR_3: &str = "C";

    // Convert hex init string to minterms for simplification.
    let term_strings = binary_strings_from_init_hex(TEST_INIT_STR_3)?;
    let minterms: Vec<Minterm> = term_strings.iter().map(|s| (&**s).into()).collect();

    println!(
        "Initial expression:\n    {}",
        string_for_sop_minterms(&minterms, false, Some("\n  "))
    );

    let prime_impls: Vec<Minterm> = get_prime_implicants(&minterms).into_iter().collect();
    println!(
        "Equivalent expression from prime implicants:\n  {}",
        string_for_sop_minterms(&prime_impls, false, Some(" "))
    );

    let prime_impl_chart = create_prime_implicant_chart(&prime_impls, &minterms);
    let minimal_sops = petrick::get_minimal_sops(prime_impl_chart, prime_impls);
    println!(
        "A minimal equivalent expression:\n  {}",
        string_for_sop_minterms(&minimal_sops, true, Some(" "))
    );

    Ok(())
}

// A few examples for testing.

fn test_case_a() -> Vec<Minterm> {
    let minterms: Vec<Minterm> = vec![
        "01x".into(), // !C & B
        "11x".into(), //  C & B
        "x01".into(), // !B & A
        "x00".into(), // !B & !A
        "1xx".into(), //  C
    ];
    minterms
}

fn test_case_b() -> Vec<Minterm> {
    let minterms: Vec<Minterm> = vec![
        "000".into(), // A'B'C'
        "100".into(), // A'B'C
        "010".into(), // A'BC'
        "101".into(), // AB'C
        "011".into(), // ABC'
        "111".into(), // ABC
    ];
    minterms
}

fn test_case_c() -> Vec<Minterm> {
    let minterms: Vec<Minterm> = vec![
        "0100".into(), // m4
        "1000".into(), // m8
        "1001".into(), // (m9)
        "1010".into(), // m10
        "1011".into(), // m11
        "1100".into(), // m12
        "1110".into(), // (m14)
        "1111".into(), // m15
    ];
    minterms
}

fn test_case_d() -> Vec<Minterm> {
    let minterms: Vec<Minterm> = vec![
        "100000".into(), //
        "000000".into(), //
        "000010".into(), //
        "000011".into(), //
    ];
    minterms
}
