//! Convert hex init string to sum-of-products string or vice-versa.

use std::error::Error;

use clap::{Arg, Command};
use logic_minimization::{
    Minterm,
    convert::{binary_strings_from_init_hex, sop_string_to_init, sop_to_minterms},
    format::{display_sort_minterms, string_for_sop_minterms},
};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Convert sum-of-products to INIT string.")
        .arg(
            Arg::new("sum-of-products")
                .short('s')
                .long("sop")
                .required(false)
                .help("Sum-of-products expression string to convert to hex init."),
        )
        .arg(
            Arg::new("hex-init")
                .short('i')
                .long("init")
                .required(false)
                .help("Hex init string to convert to sum-of-products."),
        )
        .arg(
            Arg::new("format-sop")
                .short('f')
                .long("format")
                .required(false)
                .help("Sum-of-products expression string to convert to sort and format."),
        )
        .get_matches();

    if let Some(sop_string) = matches.get_one::<String>("sum-of-products") {
        let init_string = sop_string_to_init(sop_string);
        println!("INIT value: 16'h{init_string}");
    }

    if let Some(init) = matches.get_one::<String>("hex-init") {
        let term_strings = binary_strings_from_init_hex(init)?;
        let minterms = term_strings
            .iter()
            .map(|s| (&**s).into())
            .collect::<Vec<Minterm>>();
        let sop_string = string_for_sop_minterms(&minterms, true, Some("\n"));
        println!(
            "SoP string for init: ({} terms)\n  {sop_string}",
            minterms.len()
        );
    }

    if let Some(sop_string) = matches.get_one::<String>("format-sop") {
        let mut minterms = sop_to_minterms(sop_string);
        display_sort_minterms(&mut minterms);
        let sop_string = string_for_sop_minterms(&minterms, true, Some("\n"));
        println!(
            "Formatted SoP string: ({} terms)\n  {sop_string}",
            minterms.len()
        );
    }

    Ok(())
}

// For example: target/release/convert -s '(A & !F) | (B & !C & D)'
