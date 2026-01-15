// Implement Quine-McCluskey.

pub mod convert;
pub mod format;
pub mod greedy_min_sop;
pub mod petrick;
pub mod test;

use std::{collections::HashSet, error::Error};

use crate::{
    convert::binary_strings_from_init_hex,
    format::{display_sort_minterms, string_for_sop_minterms},
    petrick::PetrickTimeInfo,
};

// ------------------------
// Top-level API functions.

pub fn qm_simplify(minterms: &[Minterm]) -> (String, usize, PetrickTimeInfo) {
    let prime_impls: Vec<Minterm> = get_prime_implicants(minterms).into_iter().collect();
    let prime_impl_chart = create_prime_implicant_chart(&prime_impls, minterms);
    let (mut minimal_sops, time) = petrick::get_minimal_sop_terms(prime_impl_chart, prime_impls);
    display_sort_minterms(&mut minimal_sops);
    let message = string_for_sop_minterms(&minimal_sops, true, Some(" "));
    (message, minimal_sops.len(), time)
}

pub fn qm_simplify_init(
    init_str: &str,
) -> Result<(String, usize, PetrickTimeInfo), Box<dyn Error>> {
    let term_strings = binary_strings_from_init_hex(init_str)?;
    let minterms = term_strings
        .iter()
        .map(|s| (&**s).into())
        .collect::<Vec<_>>();
    Ok(qm_simplify(&minterms))
}

pub fn qm_simplify_greedy(minterms: &[Minterm]) -> (String, usize) {
    let prime_impls: Vec<Minterm> = get_prime_implicants(minterms).into_iter().collect();
    let prime_impl_chart = create_prime_implicant_chart(&prime_impls, minterms);
    let mut minimal_sops = greedy_min_sop::get_minimal_sops(prime_impl_chart, prime_impls);
    display_sort_minterms(&mut minimal_sops);
    (
        string_for_sop_minterms(&minimal_sops, true, Some(" ")),
        minimal_sops.len(),
    )
}

pub fn qm_simplify_init_greedy(init_str: &str) -> Result<(String, usize), Box<dyn Error>> {
    let term_strings = binary_strings_from_init_hex(init_str)?;
    let minterms = term_strings
        .iter()
        .map(|s| (&**s).into())
        .collect::<Vec<_>>();
    Ok(qm_simplify_greedy(&minterms))
}

// ------------------
// Minterm structure.

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct Minterm {
    values: Vec<u8>,
}

impl Minterm {
    pub fn merge(&self, other: &Minterm, first_diff: usize) -> Minterm {
        let mut outterm = other.clone();
        outterm.values[first_diff] = b'x';
        outterm
    }
}

impl std::fmt::Debug for Minterm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Minterm: {}", std::str::from_utf8(&self.values).unwrap())
    }
}

impl From<&str> for Minterm {
    fn from(values: &str) -> Self {
        Minterm {
            values: values.into(),
        }
    }
}

// -----------------------------------------
// Compute prime implicants from implicants.

const DEV_DEBUG: bool = false;

pub fn get_prime_implicants(minterms: &[Minterm]) -> HashSet<Minterm> {
    let mut prime_implicants = HashSet::<Minterm>::new();
    let mut was_merged = vec![false; minterms.len()];

    for i in 0..minterms.len() {
        for j in i + 1..minterms.len() {
            let minterm_i = &minterms[i];
            let minterm_j = &minterms[j];
            if let Some(n) = can_merge(minterm_i, minterm_j) {
                prime_implicants.insert(minterm_i.merge(minterm_j, n));
                was_merged[i] = true;
                was_merged[j] = true;
            }
        }
    }
    for (i, was) in was_merged.iter().enumerate() {
        if !was {
            prime_implicants.insert(minterms[i].clone());
        }
    }

    if DEV_DEBUG {
        // Print intermediate results for debugging.
        let mut current_terms = prime_implicants.iter().cloned().collect::<Vec<Minterm>>();
        display_sort_minterms(&mut current_terms);
        println!(
            "\nAfter merge operation:\n  {}",
            string_for_sop_minterms(&current_terms, false, Some("\n"))
        );
    }

    if was_merged.iter().filter(|w| **w).count() == 0 {
        prime_implicants
    } else {
        get_prime_implicants(&prime_implicants.into_iter().collect::<Vec<Minterm>>())
    }
}

/// Minterms can be merged if they differ in exactly one variable,
/// with neither minterm having a "don't care" in that position.
fn can_merge(minterm_1: &Minterm, minterm_2: &Minterm) -> Option<usize> {
    assert!(minterm_1.values.len() == minterm_2.values.len());
    let mut first_diff = None;
    for (i, (val_1, val_2)) in minterm_1
        .values
        .iter()
        .zip(minterm_2.values.iter())
        .enumerate()
    {
        match (val_1, val_2) {
            (b'x', b'1' | b'0') => return None,
            (b'1' | b'0', b'x') => return None,
            (b'1', b'0') | (b'0', b'1') => {
                if first_diff.is_none() {
                    first_diff = Some(i);
                } else {
                    // More than one mismatch, so incompatible.
                    return None;
                }
            }
            _ => {}
        }
    }
    first_diff
}

// ---------------------------
// Prime implicate chart type.

pub struct PrimeImplicateChart {
    rows: Vec<Vec<bool>>,
}

impl std::fmt::Debug for PrimeImplicateChart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rows.is_empty() {
            return Ok(());
        }
        // Draw chart.
        let mut num_rows = vec![0_usize; self.rows.first().unwrap().len()];
        for (i, row) in self.rows.iter().enumerate() {
            write!(f, "{i:2}: ")?;
            for (col, present) in row.iter().enumerate() {
                let char = if *present {
                    num_rows[col] += 1;
                    '1'
                } else {
                    '0'
                };
                write!(f, "{char}")?;
                if col != row.len() - 1 {
                    write!(f, " | ")?;
                } else {
                    writeln!(f)?;
                }
            }
        }
        // Write bottom row with for each col itscol #; or * / R for
        // essential prime implicate columns / columns with no support.
        write!(f, "---")?;
        for (i, num) in num_rows.iter().enumerate() {
            if *num == 1 {
                write!(f, " E")?; // essential
            } else if *num == 0 {
                write!(f, " U")?; // unsupported
            } else {
                write!(f, " M")?; // multiple support
            }
            if i < num_rows.len() - 1 {
                write!(f, " |")?;
            }
        }
        Ok(())
    }
}

impl PrimeImplicateChart {
    #[allow(unused)]
    fn count_in_row(&self, row: usize) -> usize {
        self.rows[row].iter().filter(|s| **s).count()
    }
}

pub fn create_prime_implicant_chart(
    prime_impls: &[Minterm],
    minterms: &[Minterm],
) -> PrimeImplicateChart {
    let mut prime_impl_chart = vec![vec![false; minterms.len()]; prime_impls.len()];
    for (i, row) in prime_impl_chart.iter_mut().enumerate() {
        set_matches(&prime_impls[i], minterms, row);
    }

    PrimeImplicateChart {
        rows: prime_impl_chart,
    }
}

fn check_match(minterm_1: &Minterm, minterm_2: &Minterm) -> bool {
    assert!(minterm_1.values.len() == minterm_2.values.len());
    for i in 0..minterm_1.values.len() {
        if minterm_1.values[i] == b'x' {
            continue;
        }
        if minterm_1.values[i] != minterm_2.values[i] {
            return false;
        }
    }
    true
}

fn set_matches(patt_term: &Minterm, minterms: &[Minterm], matches: &mut [bool]) {
    assert!(minterms.len() == matches.len());
    for (i, minterm) in minterms.iter().enumerate() {
        matches[i] = check_match(patt_term, minterm);
    }
}
