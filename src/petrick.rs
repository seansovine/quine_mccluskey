// Implements Petrick's method to get a minimal sum-of-products
// from a prime implicant chart for a boolean function.
//
// The implementation was inspired by this discussion:
//   https://math.stackexchange.com/a/4992057/198658

use super::{Minterm, PrimeImplicateChart};

use rayon::prelude::*;
use std::cmp::Ordering;

// --------------------------------------------
// Bit vector type for use in Petrick's method.

/// Bit vector representing a set of essential prime implicants, for
/// use in applying Petrick's method to a prime implicant chart.
///
/// Bit i will be 1 iff the set represented contains the ith implicant.
#[derive(Clone, Copy, Default)]
struct BitVec {
    bits: u64,
}

impl std::fmt::Debug for BitVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:#b})", self.bits)
    }
}

impl PartialEq for BitVec {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl BitVec {
    pub fn set_bit(&mut self, i: usize) {
        assert!(i < 64);
        self.bits |= 1u64 << i;
    }

    #[allow(unused)]
    pub fn reset_bit(&mut self, i: usize) {
        assert!(i < 64);
        self.bits &= !(1u64 << i);
    }

    pub fn get_bit(&self, i: usize) -> bool {
        assert!(i < 64);
        (self.bits & (1u64 << i)) != 0
    }

    pub fn count_ones(&self) -> u32 {
        self.bits.count_ones()
    }

    pub fn merge(&mut self, other: &BitVec) {
        self.bits |= other.bits;
    }

    pub fn nonzero_indices(&self) -> Vec<usize> {
        let mut nonzero_indices = vec![];
        for i in 0..64 {
            if self.get_bit(i) {
                nonzero_indices.push(i);
            }
        }
        nonzero_indices
    }

    pub fn is_subset(&self, other: &BitVec) -> bool {
        self.bits & other.bits == self.bits
    }
}

/// Represents a sequence of bit vectors w/ the same # of 1's.
///
/// `start_offset` gives the starting offset of bit vectors with
/// n 1's in a list of bit vectors sorted by # of 1's.
struct OnesGroup {
    n_ones: u32,
    start_offset: usize,
}

impl std::fmt::Debug for OnesGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}: {})", self.n_ones, self.start_offset)
    }
}

impl BitVec {
    pub fn bitvecs_from_chart_col(prime_impl_chart: &PrimeImplicateChart, col: usize) -> Vec<Self> {
        let rows = &prime_impl_chart.rows;
        if rows.is_empty() {
            return Default::default();
        }
        assert!(rows.first().unwrap().len() > col);
        let mut bit_vecs = vec![];

        for (i, row) in rows.iter().enumerate() {
            if row[col] {
                let mut bit_vec = BitVec::default();
                bit_vec.set_bit(i);
                bit_vecs.push(bit_vec);
            }
        }
        bit_vecs
    }

    /// Sort bit vectors first by number of bits, then in dictionary order.
    pub fn bitsort(bitvecs: &mut [BitVec]) -> Vec<OnesGroup> {
        bitvecs.sort_by(|a, b| {
            // First sort by # of 1-bits.
            if (a.count_ones()) < (b.count_ones()) {
                return Ordering::Less;
            }
            if (a.count_ones()) > (b.count_ones()) {
                return Ordering::Greater;
            }
            // Same # of 1-bits.
            a.bits.cmp(&b.bits)
        });

        // (# ones, starting position of bitvecs with this # ones)
        let mut ones_group_start: Vec<OnesGroup> = vec![];
        for (i, bv) in bitvecs.iter().enumerate() {
            let bv_ones = bv.count_ones();
            if ones_group_start.is_empty() || ones_group_start.last().unwrap().n_ones != bv_ones {
                ones_group_start.push(OnesGroup {
                    n_ones: bv_ones,
                    start_offset: i,
                });
            }
        }
        ones_group_start
    }
}

// ----------------------------------------
// Functions implementing Petrick's method.

const DEV_DEBUG: bool = false;

/// Compute a minimal set of prime implicants from a prime implicant chart.
pub fn get_minimal_sop_terms(
    mut prime_impl_chart: PrimeImplicateChart,
    mut prime_impls: Vec<Minterm>,
) -> Vec<Minterm> {
    assert!(prime_impls.len() == prime_impl_chart.rows.len());
    if prime_impl_chart.rows.is_empty() || prime_impl_chart.rows.first().unwrap().is_empty() {
        panic!("Prime implicant chart has either no rows or no columns.");
    }
    // This version currently supports at most 6 variables, but could be extended.
    assert!(prime_impl_chart.rows.first().unwrap().len() <= 64);

    // Remove essential prime implicants from chart.
    let (mut min_expr_terms, remaining_cols) =
        remove_essential_prime_impls(&mut prime_impl_chart, &mut prime_impls);
    if remaining_cols.is_empty() {
        // Indicates all prime impls were essential, so we're done.
        return min_expr_terms;
    }

    let mut column_bitvecs = remaining_cols
        .into_iter()
        .map(|rem_col_i| BitVec::bitvecs_from_chart_col(&prime_impl_chart, rem_col_i))
        .filter(|vecs| !vecs.is_empty())
        .collect::<Vec<_>>();

    // Apply distributive property to column sets pairwise.
    while column_bitvecs.len() > 1 {
        let num_sets = column_bitvecs.len();
        column_bitvecs.par_chunks_mut(2).for_each(|chunk| {
            if chunk.len() == 1 {
                return;
            }
            let mut updated = pairwise_and(&chunk[0], &chunk[1]);
            if num_sets > 2 {
                remove_redundant(&mut updated);
            }
            chunk[0] = updated;
        });
        column_bitvecs = column_bitvecs
            .into_iter()
            .enumerate()
            .filter_map(|(i, vec)| if i.is_multiple_of(2) { Some(vec) } else { None })
            .collect();
    }
    let current_bitvecs = &mut column_bitvecs[0];

    let ones_group_start = BitVec::bitsort(current_bitvecs);
    if DEV_DEBUG {
        println!("Final expression set:");
        println!(" {:<7} - {ones_group_start:?}", current_bitvecs.len());
        println!(" # essential prime implicants: {}", min_expr_terms.len());
    }

    let chosen_min_bitvec = current_bitvecs.first().unwrap();
    for i in chosen_min_bitvec.nonzero_indices() {
        min_expr_terms.push(prime_impls.get(i).unwrap().clone());
    }

    min_expr_terms
}

/// Computes the logical 'and' to build up a set of prime implicants
/// covering all the columns of a prime implicant chart. This is applying
/// the logical distributive property using a bit vector representation.
///
/// **Note:** The actual bitwise operation performed on bit vectors is
/// the logical 'or', because a bit vector is interpreted as the 'and'
/// of the terms corresponding to its nonzero digits.
fn pairwise_and(current_bitvecs: &[BitVec], next_col_bitvecs: &[BitVec]) -> Vec<BitVec> {
    let mut merged_bitvecs = vec![];
    for c_bitvec in current_bitvecs {
        for n_bitvec in next_col_bitvecs {
            let mut new_bitvec = *c_bitvec;
            new_bitvec.merge(n_bitvec);
            merged_bitvecs.push(new_bitvec);
        }
    }
    let _ = BitVec::bitsort(&mut merged_bitvecs);
    merged_bitvecs.dedup();
    merged_bitvecs
}

/// Remove bitvecs that are subsumed by others in the set.
/// As a side effect, sorts the reduced `bitvecs`.
///
/// Precondition: `bitvecs` has been sorted and deduplicated.
fn remove_redundant(bitvecs: &mut Vec<BitVec>) {
    if bitvecs.is_empty() {
        return;
    }

    let ones_group_start = BitVec::bitsort(bitvecs);
    if DEV_DEBUG {
        println!(
            "Merging on thread {}.\n {:<7} - {ones_group_start:?}",
            rayon::current_thread_index().unwrap_or_default(),
            bitvecs.len()
        );
    }

    // Find redundant bitvecs. This is the main bottleneck of the program.
    let mut to_remove = vec![false; bitvecs.len()];
    for i in 0..ones_group_start.last().unwrap().start_offset {
        // If we removed bitvec i, then we'd have removed its supersets also.
        if to_remove[i] {
            continue;
        }

        let bitvec_i = &bitvecs[i];
        let ones_group_i = ones_group_start
            .iter()
            .position(|OnesGroup { n_ones, .. }| *n_ones == bitvec_i.count_ones())
            .unwrap();

        // Start checks with next larger size sets.
        for j in ones_group_start[ones_group_i + 1].start_offset..bitvecs.len() {
            if !to_remove[j] && bitvec_i.is_subset(&bitvecs[j]) {
                to_remove[j] = true;
            }
        }
    }

    // Keep only non-redundant bitvecs.
    *bitvecs = bitvecs
        .iter()
        .enumerate()
        .filter_map(|(i, vec)| if !to_remove[i] { Some(vec) } else { None })
        .copied()
        .collect();
}

#[derive(Debug, Clone, Copy)]
enum RowCount {
    None,
    One(usize),
    Multi,
}

/// Removes essential prime implicants from list and chart and returns them as a vec,
/// along with a list of indices for columns that weren't eliminated in the process.
///
/// An prime implicant is essential when it is the only one covering one of the columns
/// in the prime implicant chart.
///
/// Modifies `prime_impls` and `prime_impl_chart`.
pub fn remove_essential_prime_impls(
    prime_impl_chart: &mut PrimeImplicateChart,
    prime_impls: &mut Vec<Minterm>,
) -> (Vec<Minterm>, Vec<usize>) {
    assert!(!prime_impls.is_empty());
    assert!(prime_impls.len() == prime_impl_chart.rows.len());
    let num_cols = prime_impl_chart.rows.first().unwrap().len();

    // Records how many rows cover each column, singling out the case
    // where a column is covered by only a single essential row.
    let mut remove_cols: Vec<RowCount> = vec![RowCount::None; num_cols];

    // Find essential prime implicants and corresponding chart columns.
    for (row_i, row) in prime_impl_chart.rows.iter().enumerate() {
        for (col_i, row_col_val) in row.iter().copied().enumerate() {
            if row_col_val {
                remove_cols[col_i] = match remove_cols[col_i] {
                    RowCount::None => RowCount::One(row_i),
                    RowCount::One(_) => RowCount::Multi,
                    RowCount::Multi => RowCount::Multi,
                };
            }
        }
    }

    // Records whether each prime implicant is essential.
    let mut is_essential = vec![false; prime_impls.len()];
    // Records whether each column is covered by a prime implicant.
    let mut covered_by_prime = vec![false; num_cols];

    // Mark essential prime implicants and columns they cover.
    for val in remove_cols.iter_mut() {
        if let RowCount::One(row_i) = val {
            is_essential[*row_i] = true;
            for (j, covered) in prime_impl_chart.rows[*row_i].iter().enumerate() {
                covered_by_prime[j] |= covered;
            }
        } else if matches!(val, RowCount::None) {
            panic!("An implicant chart column that was not covered by any row.");
        }
    }

    // Keep columns that aren't covered by a prime implicant.
    let mut remaining_cols = vec![];
    for (i, _) in remove_cols.iter().enumerate() {
        if !covered_by_prime[i] {
            remaining_cols.push(i);
        }
    }

    // Remove prime implicants from prime_impls and chart.
    let mut ess_prime_impls = vec![];
    for (i, val) in is_essential.iter().copied().enumerate().rev() {
        if val {
            prime_impl_chart.rows.remove(i);
            ess_prime_impls.push(prime_impls.remove(i));
        }
    }

    // Remove any columns from remaining that now have no row support.
    let mut col_rows = vec![0_usize; num_cols];
    for row in &prime_impl_chart.rows {
        for (i, row_has_col) in row.iter().enumerate() {
            if *row_has_col {
                col_rows[i] += 1;
            }
        }
    }
    for i in (0..remaining_cols.len()).rev() {
        if col_rows[remaining_cols[i]] == 0 {
            remaining_cols.remove(i);
        }
    }

    (ess_prime_impls, remaining_cols)
}
