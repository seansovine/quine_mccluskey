//! Example of initial expression with "don't care" terms.
//!
//! This is the example from Wikipedia, but note that our binary strings
//! map to the variable letter names in reverse order from theirs.

use logic_minimization::{
    Minterm, create_prime_implicant_chart,
    format::{FormattedExpr, display_sort_minterms, string_for_sop_minterms},
    get_prime_implicants, petrick,
};

fn main() {
    let minterms: Vec<Minterm> = vec![
        "0010".into(), // ~A & B & ~C & ~D
        "0001".into(), // A & ~B & ~C & ~D
        "0101".into(), // A & ~B & C & ~D
        "1101".into(), // A & ~B & C & D
        "0011".into(), // A & B & ~C & ~D
        "1111".into(), // A & B & C & D
        // Don't care terms.
        Minterm::dont_care("0111"), // A & B & C & ~D
        Minterm::dont_care("1001"), // A & ~B & ~C & D
    ];

    let FormattedExpr {
        minterm_expr: minterm_string,
        dont_cares: dont_care_string,
    } = string_for_sop_minterms(&minterms, false, Some("\n"));

    println!("Initial expression:\n  {}", minterm_string);
    println!("\nDon't cares:\n {}", dont_care_string);

    let mut prime_impls: Vec<Minterm> = get_prime_implicants(&minterms).into_iter().collect();

    display_sort_minterms(&mut prime_impls);
    println!(
        "\nEquivalent expression from prime implicants:\n  {}",
        string_for_sop_minterms(&prime_impls, false, Some("\n")).sop_string()
    );

    let prime_impl_chart = create_prime_implicant_chart(&prime_impls, &minterms);
    let (mut minimal_sop_terms, _) = petrick::get_minimal_sop_terms(prime_impl_chart, prime_impls);

    display_sort_minterms(&mut minimal_sop_terms);
    println!(
        "\nA minimal equivalent expression ({} terms):\n  {}",
        minimal_sop_terms.len(),
        string_for_sop_minterms(&minimal_sop_terms, true, Some("\n")).sop_string()
    );
}
