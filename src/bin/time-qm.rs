#![allow(unused)]

use clap::{Arg, Command};
use std::{error::Error, time::Instant};

use logic_minimization::{
    format::{binary_strings_from_init_hex, display_sort_minterms, string_for_sop_minterms},
    *,
};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Quine-McCluskey")
        .arg(
            Arg::new("init")
                .required(true)
                .short('i')
                .long("init")
                .help("Optional init string; up to 16 hex chars."),
        )
        .get_matches();

    let init = matches.get_one::<String>("init").unwrap();

    // Convert hex init string to minterms for simplification.
    let start_time = Instant::now();
    let term_strings = binary_strings_from_init_hex(init)?;
    let minterms = term_strings
        .iter()
        .map(|s| (&**s).into())
        .collect::<Vec<_>>();
    let elapsed = start_time.elapsed().as_millis();
    println!(
        "(*) {elapsed:>4} ms - Converted init string to set of {} minterms.",
        minterms.len()
    );

    let start_time = Instant::now();
    let prime_impls: Vec<Minterm> = get_prime_implicants(&minterms).into_iter().collect();
    let elapsed = start_time.elapsed().as_millis();
    println!(
        "(*) {elapsed:>4} ms - Generated prime {} implicants.",
        prime_impls.len()
    );

    let start_time = Instant::now();
    let prime_impl_chart = create_prime_implicant_chart(&prime_impls, &minterms);
    let elapsed = start_time.elapsed().as_millis();
    println!("(*) {elapsed:>4} ms - Created prime implicant chart.");

    let start_time = Instant::now();
    let (mut minimal_sops, time) = petrick::get_minimal_sops(prime_impl_chart, prime_impls);
    let elapsed = start_time.elapsed().as_millis();
    println!("(*) {elapsed:>4} ms - Simplified using Petrick's method.");

    println!("\n{}", time.format_me());

    display_sort_minterms(&mut minimal_sops);
    println!(
        "\nA minimal equivalent expression: ({} terms)\n  {}",
        minimal_sops.len(),
        string_for_sop_minterms(&minimal_sops, true, Some("\n"))
    );

    Ok(())
}
