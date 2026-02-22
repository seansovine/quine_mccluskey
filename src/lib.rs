// Implement Quine-McCluskey.

pub mod convert;
pub mod format;
pub mod greedy_min_sop;
pub mod petrick;
pub mod test;

use std::{collections::HashSet, error::Error};

use crate::{
    convert::binary_strings_from_init_hex,
    format::{FormattedExpr, display_sort_minterms, string_for_sop_minterms},
};

// ---------------------------
// Higher-level API functions.

pub fn qm_simplify(minterms: &[Minterm]) -> (String, usize) {
    let prime_impls: Vec<Minterm> = get_prime_implicants(minterms).into_iter().collect();
    let prime_impl_chart = create_prime_implicant_chart(&prime_impls, minterms);
    let mut minimal_sops = petrick::get_minimal_sop_terms(prime_impl_chart, prime_impls);
    display_sort_minterms(&mut minimal_sops);
    let FormattedExpr {
        minterm_expr: sop_string,
        ..
    } = string_for_sop_minterms(&minimal_sops, true, Some(" "));
    (sop_string, minimal_sops.len())
}

pub fn qm_simplify_init(init_str: &str) -> Result<(String, usize), Box<dyn Error>> {
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
        string_for_sop_minterms(&minimal_sops, true, Some(" "))
            .sop_string()
            .into(),
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

#[derive(Hash, Clone, PartialEq, Eq, Default)]
pub struct Minterm {
    values: Vec<u8>,
    dont_care: bool,
}

impl Minterm {
    /// Result will have `dont_care == false`.
    pub fn merge(&self, other: &Minterm, first_diff: usize) -> Minterm {
        let mut out_term = other.clone();
        out_term.values[first_diff] = b'x';
        out_term.dont_care = false;
        out_term
    }

    pub fn dont_care(values: &str) -> Self {
        Minterm {
            values: values.into(),
            dont_care: true,
        }
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
            dont_care: false,
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
            string_for_sop_minterms(&current_terms, false, Some("\n")).sop_string()
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
        // Write bottom row with a symbol describing each column's support.
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
    // Filter out don't care minterms; they aren't needed from here on.
    let minterms: Vec<Minterm> = minterms
        .iter()
        .filter_map(|minterm| {
            if minterm.dont_care {
                None
            } else {
                Some(minterm.clone())
            }
        })
        .collect();

    let mut prime_impl_chart = vec![vec![false; minterms.len()]; prime_impls.len()];
    for (i, row) in prime_impl_chart.iter_mut().enumerate() {
        compute_row(&prime_impls[i], &minterms, row);
    }
    PrimeImplicateChart {
        rows: prime_impl_chart,
    }
}

/// Check if the minterm implies the prime implicant.
///
/// Note that by construction each prime implicant is the logical 'or' of one or
/// more terms in the initial expression. Our goal in QM is to find a minimal set
/// of prime implicants whose logical 'or' is equivalent to the initial expression.
/// This function checks if the logical 'or' that makes up `prime_implicant`
/// includes `minterm`.
fn check_includes(prime_implicant: &Minterm, minterm: &Minterm) -> bool {
    assert!(prime_implicant.values.len() == minterm.values.len());
    for i in 0..prime_implicant.values.len() {
        if prime_implicant.values[i] == b'x' {
            continue;
        }
        if prime_implicant.values[i] != minterm.values[i] {
            return false;
        }
    }
    true
}

/// Compute one row of the prime implicant chart.
fn compute_row(prime_implicants: &Minterm, minterms: &[Minterm], matches: &mut [bool]) {
    assert!(minterms.len() == matches.len());
    for (i, minterm) in minterms.iter().enumerate() {
        matches[i] = check_includes(prime_implicants, minterm);
    }
}
