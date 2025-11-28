//! Use the simple greedy algorithm for set covering to construct
//! an approximately-minimal sum-of-products from the prime
//! implicant chart.
//!
//! Provides a much faster alternative to Petrick's method.

use crate::{Minterm, PrimeImplicateChart, petrick::remove_essential_prime_impls};

const EXTRA_DEBUG: bool = false;

pub fn get_minimal_sops(
    mut prime_impl_chart: PrimeImplicateChart,
    mut prime_impls: Vec<Minterm>,
) -> Vec<Minterm> {
    if prime_impl_chart.rows.is_empty() || prime_impl_chart.rows.first().unwrap().is_empty() {
        // Ok to panic here because this condition indicates programmer error.
        panic!("Prime implicant chart has either no rows or no columns.");
    }
    assert!(prime_impls.len() == prime_impl_chart.rows.len());
    assert!(prime_impl_chart.rows.first().unwrap().len() <= 64);

    if EXTRA_DEBUG {
        println!("PI chart before removing essential:\n{prime_impl_chart:?}");
    }

    // Remove essential prime implicants from chart.
    let (mut min_expr_terms, remaining_cols) =
        remove_essential_prime_impls(&mut prime_impl_chart, &mut prime_impls, None);
    if remaining_cols.is_empty() {
        // Indicates all prime impls were essential, so we're done.
        return min_expr_terms;
    }
    assert!(!prime_impl_chart.rows.is_empty());

    if EXTRA_DEBUG {
        println!("PI chart after removing essential:\n{prime_impl_chart:?}");
        println!("Number of essential PIs: {}", min_expr_terms.len());
    }

    let mut covered = vec![true; prime_impl_chart.rows.first().unwrap().len()];
    remaining_cols
        .iter()
        .for_each(|index| covered[*index] = false);
    let mut selected_rows = vec![false; prime_impl_chart.rows.len()];

    // Keep selecting next best until the cover is complete.
    while has_uncovered(&covered) {
        let unselected_inds = selected_rows
            .iter()
            .enumerate()
            .filter_map(|(i, row_was_selected)| if !*row_was_selected { Some(i) } else { None })
            .collect::<Vec<_>>();
        assert!(!unselected_inds.is_empty());

        // TODO: This shouldn't be necessary! Find the problem.
        if unselected_inds.is_empty() {
            println!("{covered:?}");
            break;
        }

        let first = *unselected_inds.first().unwrap();
        let mut max_count = count_additional_covered(&covered, &prime_impl_chart.rows[first]);
        let mut max_row = first;

        // Find unselected row with max coverage of uncovered cols.
        for next in unselected_inds.iter().skip(1) {
            let next_count = count_additional_covered(&covered, &prime_impl_chart.rows[*next]);
            if next_count > max_count {
                max_count = next_count;
                max_row = *next;
            }
        }
        selected_rows[max_row] = true;
        for (i, col_in_row) in prime_impl_chart.rows[max_row].iter().enumerate() {
            if *col_in_row {
                covered[i] = true;
            }
        }
    }

    for (row, selected) in selected_rows.iter().enumerate() {
        if *selected {
            min_expr_terms.push(prime_impls.get(row).unwrap().clone());
        }
    }
    min_expr_terms
}

// Count number of uncovered elements that would be covered
// by adding the candidate set to selected.
fn count_additional_covered(covered: &[bool], candidate: &[bool]) -> usize {
    covered
        .iter()
        .zip(candidate.iter())
        .filter(|(curr_cov, cand_cov)| !**curr_cov && **cand_cov)
        .count()
}

fn has_uncovered(covered: &[bool]) -> bool {
    covered.iter().any(|is_covered| !*is_covered)
}
