// Implement Quine-McCluskey.

#![allow(unused)]

pub mod petrick;

use std::collections::HashSet;

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

const EQN_VARS: &[char] = &['A', 'B', 'C', 'D', 'E', 'F'];

fn string_for_minterm(minterm: &Minterm) -> String {
    let mut term_string = String::new();
    for (i, c) in minterm.values.iter().rev().enumerate() {
        if *c == b'x' {
            continue;
        }
        let var = match c {
            b'x' => continue,
            b'0' => format!("!{}", EQN_VARS[i]),
            b'1' => format!("{}", EQN_VARS[i]),
            _ => unreachable!(),
        };
        if term_string.is_empty() {
            term_string = var.to_string();
        } else {
            term_string = format!("{term_string} & {var}");
        }
    }

    if term_string.is_empty() {
        "True".into()
    } else {
        term_string
    }
}

/// Get a string representation for the SOP with minterm set `minterms`.
pub fn string_for_sop_minterms(minterms: &[Minterm], omit_trivial: bool) -> String {
    if minterms.is_empty() {
        return "False".into();
    }

    let mut expr_string = String::new();
    for (i, minterm) in minterms.iter().enumerate() {
        let term_string = string_for_minterm(minterm);
        if term_string == "True" && omit_trivial {
            continue;
        }
        if expr_string.is_empty() {
            expr_string = format!("({term_string})");
        } else {
            expr_string = format!("{expr_string} | ({term_string})");
        }
    }

    if expr_string.is_empty() {
        "True".into()
    } else {
        expr_string
    }
}

// Compute prime implicants from implicants.

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

    if was_merged.iter().filter(|w| **w).count() == 0 {
        prime_implicants
    } else {
        get_prime_implicants(&prime_implicants.into_iter().collect::<Vec<Minterm>>())
    }
}

/// Minterms can be merged if they differ in exactly one variable,
/// with neither minterm having a "don't care" in that position.
///
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

// Prime implicate chart.

pub struct PrimeImplicateChart {
    rows: Vec<Vec<bool>>,
}

impl std::fmt::Debug for PrimeImplicateChart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            let row_bytes: Vec<_> = row.iter().map(|v| if *v { b'1' } else { b'0' }).collect();
            writeln!(f, "{}", String::from_utf8_lossy(&row_bytes))?;
        }

        Ok(())
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
