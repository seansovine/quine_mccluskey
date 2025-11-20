// Implement Petrick's method to get min-SOPs.
//
// THe discussion here was helpful in understanding how to implement this:
//   https://math.stackexchange.com/a/4992057/198658

use super::{Minterm, PrimeImplicateChart};

use std::{
    cmp::Ordering,
    fmt::Write,
    time::{Duration, Instant},
};

// Bit vector data structure for simplifying prime implicant chart.

/// Bit vector representing a set of essential prime implicants.
/// For use in applying Petrick's method to a prime implicant chart.
///
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

impl BitVec {
    pub fn bitvecs_from_chart_col(
        prime_impl_chart: &PrimeImplicateChart,
        col: usize,
        time: &mut TimeInfo,
    ) -> Vec<Self> {
        let start = Instant::now();
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
        time.bitvecs_from_chart_cols += start.elapsed();

        bit_vecs
    }

    pub fn bitsort(bitvecs: &mut [BitVec]) -> Vec<(u32, usize)> {
        bitvecs.sort_by(|a, b| {
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
        let mut ones_group_start: Vec<(u32, usize)> = vec![];
        for (i, bv) in bitvecs.iter().enumerate() {
            let bv_ones = bv.count_ones();
            if ones_group_start.is_empty() || ones_group_start.last().unwrap().0 != bv_ones {
                ones_group_start.push((bv_ones, i));
            }
        }
        ones_group_start
    }
}

// Functions to perform Petrick's method to simplify prime implicants chart.

#[derive(Default)]
pub struct TimeInfo {
    pub remove_essential_prime_impls: Duration,
    pub bitvecs_from_chart_cols: Duration,

    pub remove_redundant: Duration,
    pub remove_redundant_first_loop: Duration,

    pub first_loop: Duration,
    pub second_loop: Duration,

    pub pairwise_and_calls: u64,
    pub pairwise_and: Duration,
}

impl TimeInfo {
    pub fn format_me(&self) -> String {
        let mut message = String::new();
        writeln!(message, "Petrick run time:");
        writeln!(
            message,
            "-- remove_essential_prime_impls: {:>5} ms",
            self.remove_essential_prime_impls.as_millis()
        );
        writeln!(
            message,
            "-- bitvecs_from_chart_cols:      {:>5} ms",
            self.bitvecs_from_chart_cols.as_millis()
        );
        writeln!(message);
        writeln!(
            message,
            "-- remove_redundant:             {:>5} ms",
            self.remove_redundant.as_millis()
        );
        writeln!(
            message,
            "-- remove_redundant first loop:  {:>5} ms",
            self.remove_redundant_first_loop.as_millis()
        );
        writeln!(message);
        writeln!(
            message,
            "-- first loop:                   {:>5} ms",
            self.first_loop.as_millis()
        );
        writeln!(
            message,
            "-- second loop:                  {:>5} ms",
            self.second_loop.as_millis()
        );
        writeln!(message);
        writeln!(
            message,
            "-- pairwise_and calls:           {:>5} ",
            self.pairwise_and_calls
        );
        write!(
            message,
            "-- pairwise_and:                 {:>5} ms",
            self.pairwise_and.as_millis()
        );
        message
    }
}

/// Get a minimal set of prime implicants for an equivalent expression.
///
pub fn get_minimal_sops(
    mut prime_impl_chart: PrimeImplicateChart,
    mut prime_impls: Vec<Minterm>,
) -> (Vec<Minterm>, TimeInfo) {
    if prime_impl_chart.rows.is_empty() || prime_impl_chart.rows.first().unwrap().is_empty() {
        // Ok to panic here because this condition indicates programmer error.
        panic!("Prime implicant chart has either no rows or no columns.");
    }
    // This version currently supports at most 6-variable eqns, but could be extended.
    assert!(prime_impls.len() == prime_impl_chart.rows.len());
    assert!(prime_impl_chart.rows.first().unwrap().len() <= 64);

    let mut time = TimeInfo::default();

    // Remove essential prime implicants from chart.
    let (mut min_expr_terms, remaining_cols) =
        remove_essential_prime_impls(&mut prime_impl_chart, &mut prime_impls, &mut time);
    if remaining_cols.is_empty() {
        // Indicates all prime impls were essential, so we're done.
        return (min_expr_terms, time);
    }

    // Simplify remaining terms with boolean logic rules.
    let first_remaining_col = *remaining_cols.first().unwrap();
    let mut current_bitvecs: Vec<BitVec> = vec![BitVec::default()];
    let col_bitvecs = remaining_cols
        .into_iter()
        .map(|rem_col_i| BitVec::bitvecs_from_chart_col(&prime_impl_chart, rem_col_i, &mut time))
        .filter(|vecs| !vecs.is_empty())
        .collect::<Vec<_>>();
    let start = Instant::now();
    for (i, next_col_bitvecs) in col_bitvecs.iter().enumerate() {
        time.pairwise_and_calls += 1;
        current_bitvecs = pairwise_and(&current_bitvecs, next_col_bitvecs, &mut time);
        if i < col_bitvecs.len() - 1 {
            remove_redundant(&mut current_bitvecs, &mut time);
        }
    }
    time.first_loop += start.elapsed();

    let start = Instant::now();
    let _ = BitVec::bitsort(&mut current_bitvecs);
    let chosen_min_bitvec = current_bitvecs.first().unwrap();
    for i in chosen_min_bitvec.nonzero_indices() {
        min_expr_terms.push(prime_impls.get(i).unwrap().clone());
    }
    time.second_loop += start.elapsed();

    (min_expr_terms, time)
}

/// Computes the logical 'and' to build up a set of prime implicants
/// covering all the columns of a prime implicant chart.
///
/// **Note:** The actual bitwise operation performed on bit vectors is
/// the logical 'or', because a bit vector is interpreted as the 'and'
/// of the terms corresponding to its nonzero digits.
///
fn pairwise_and(
    current_bitvecs: &[BitVec],
    next_col_bitvecs: &[BitVec],
    time: &mut TimeInfo,
) -> Vec<BitVec> {
    let start = Instant::now();
    let mut merged_bitvecs = vec![];
    for c_bitvec in current_bitvecs {
        for n_bitvec in next_col_bitvecs {
            let mut new_bitvec = *c_bitvec;
            new_bitvec.merge(n_bitvec);
            merged_bitvecs.push(new_bitvec);
        }
    }
    merged_bitvecs.dedup();
    time.pairwise_and += start.elapsed();

    merged_bitvecs
}

const DEV_DEBUG: bool = false;

/// Remove bitvecs that are subsumed by others in the set.
/// As a side effect, sorts reduced `bitvecs`.
///
fn remove_redundant(bitvecs: &mut Vec<BitVec>, time: &mut TimeInfo) {
    if bitvecs.is_empty() {
        return;
    }
    let start = Instant::now();
    // Sort so that vecs with fewer bits are first.
    let ones_group_start = BitVec::bitsort(bitvecs);
    bitvecs.dedup();
    let ones_group_start = BitVec::bitsort(bitvecs);
    if DEV_DEBUG {
        println!("{ones_group_start:?} - {}", bitvecs.len());
    }

    // Bit vecs to remove at end.
    let mut to_remove = vec![false; bitvecs.len()];
    let start_inner = Instant::now();
    // Find redundant bitvecs.
    for i in 0..ones_group_start.last().unwrap().1 {
        let bitvec_i = &bitvecs[i];
        let ogs_n = ones_group_start
            .iter()
            .position(|(ones, _)| *ones == bitvec_i.count_ones())
            .unwrap();
        for j in ones_group_start[ogs_n + 1].1..bitvecs.len() {
            if !to_remove[j] && (bitvecs[i].bits & bitvecs[j].bits == bitvecs[i].bits) {
                to_remove[j] = true;
            }
        }
    }
    time.remove_redundant_first_loop += start_inner.elapsed();
    // Remove redundancies.
    for i in (0..bitvecs.len()).rev() {
        if to_remove[i] {
            bitvecs.remove(i);
        }
    }
    time.remove_redundant += start.elapsed();
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
/// An essential prime implicants is one which is the only prime implicant covering
/// some column in the prime implicant chart.
///
/// Modifies `prime_impls` and `prime_impl_chart`.
///
fn remove_essential_prime_impls(
    prime_impl_chart: &mut PrimeImplicateChart,
    prime_impls: &mut Vec<Minterm>,
    time: &mut TimeInfo,
) -> (Vec<Minterm>, Vec<usize>) {
    assert!(prime_impls.len() == prime_impl_chart.rows.len());
    let start = Instant::now();

    // If only one row (prime implicant) covers the minterm of a column,
    // the entry for that column holds that row index; else it holds None.
    let mut remove_cols: Vec<RowCount> =
        vec![RowCount::None; prime_impl_chart.rows.first().unwrap().len()];

    // Find essential prime implicants and corresponding columns.
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

    // Indexed the same as prime_impls.
    let mut ess_prime_impls_i = vec![false; prime_impls.len()];
    // Mark essential prime implicants and get columns that are kept.
    let mut remaining_cols = vec![];
    for (i, val) in remove_cols.iter().enumerate() {
        if let RowCount::One(row_i) = val {
            ess_prime_impls_i[*row_i] = true;
        } else {
            remaining_cols.push(i);
        }
    }

    // Remove prime implicants and from prime_impls.
    let mut ess_prime_impls = vec![];
    for (i, val) in ess_prime_impls_i.iter().copied().enumerate().rev() {
        if val {
            prime_impl_chart.rows.remove(i);
            ess_prime_impls.push(prime_impls.remove(i));
        }
    }
    time.remove_essential_prime_impls += start.elapsed();

    (ess_prime_impls, remaining_cols)
}
