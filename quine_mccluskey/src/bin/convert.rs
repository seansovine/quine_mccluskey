use std::error::Error;

use clap::{Arg, Command};
use logic_minimization::check::sop_string_to_init;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Convert sum-of-products to INIT string.")
        .arg(
            Arg::new("sum-of-products")
                .short('s')
                .long("sop")
                .required(true)
                .help("Sum-of-products expression string."),
        )
        .get_matches();

    let sop_string = matches.get_one::<String>("sum-of-products").unwrap();
    let init_string = sop_string_to_init(sop_string);
    println!("INIT value: 16'h{init_string}");

    Ok(())
}

// For example: crr --bin convert -- -s '(A & !F) | (B & !C & D)'
