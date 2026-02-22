//! Apply the Quine-McCluskey algorithm to minimize a logical expression.

use std::error::Error;

use clap::{Arg, ArgAction, Command};

use logic_minimization::{
    convert::{init_to_minterms, sop_to_minterms},
    format::{FormattedExpr, display_sort_minterms, string_for_sop_minterms},
    *,
};

const DEBUG: bool = false;
const SEPARATOR: &str = "\n";

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Quine-McCluskey")
        .arg(
            Arg::new("init")
                .short('i')
                .long("init")
                .help("Optional init string; up to 16 hex chars."),
        )
        .arg(
            Arg::new("sop")
                .short('s')
                .long("sop")
                .help("Sum-of-products string of expression to minimize."),
        )
        .arg(
            Arg::new("greedy")
                .short('g')
                .long("greedy")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let use_greedy = matches.get_flag("greedy");
    let mut minterms;

    if let Some(init) = matches.get_one::<String>("init") {
        // Convert hex init string to minterms for simplification.
        minterms = init_to_minterms(init)?;
    } else if let Some(sop_string) = matches.get_one::<String>("sop") {
        minterms = sop_to_minterms(sop_string);
    } else {
        println!("No input provided. Please use --help to see input options.");
        return Ok(());
    }

    let FormattedExpr {
        minterm_expr: minterm_string,
        dont_cares: dont_care_string,
    } = string_for_sop_minterms(&minterms, false, Some("\n"));
    display_sort_minterms(&mut minterms);
    println!(
        "Initial expression: ({} terms)\n  {}",
        minterms.len(),
        minterm_string
    );
    println!("Don't cares:\n  {}", dont_care_string);

    let mut prime_impls: Vec<Minterm> = get_prime_implicants(&minterms).into_iter().collect();

    display_sort_minterms(&mut prime_impls);
    println!(
        "\nEquivalent expression from prime implicants ({} terms):\n  {}",
        prime_impls.len(),
        string_for_sop_minterms(&prime_impls, false, Some(SEPARATOR)).sop_string()
    );

    let prime_impl_chart = create_prime_implicant_chart(&prime_impls, &minterms);

    if DEBUG {
        println!("\nPrime implicant chart:\n{prime_impl_chart:?}");
    }

    let mut minimal_sops = if use_greedy {
        greedy_min_sop::get_minimal_sops(prime_impl_chart, prime_impls)
    } else {
        petrick::get_minimal_sop_terms(prime_impl_chart, prime_impls)
    };

    display_sort_minterms(&mut minimal_sops);
    println!(
        "\nA minimal equivalent expression: ({} terms)\n  {}",
        minimal_sops.len(),
        string_for_sop_minterms(&minimal_sops, true, Some(SEPARATOR)).sop_string()
    );

    Ok(())
}
