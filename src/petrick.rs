// Implement Petrick's method to get a minimal sum-of-products
// from a prime implicants chart.
//
// The discussion here was helpful in understanding how to implement this:
//   https://math.stackexchange.com/a/4992057/198658

use std::{
    cmp::Ordering,
    fmt::Write,
    time::{Duration, Instant},
};

use super::{Minterm, PrimeImplicateChart};

// --------------------------------------------
// Bit vector type for use in Petrick's method.

/// Bit vector representing a set of essential prime implicants.
/// For use in applying Petrick's method to a prime implicant chart.
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

    #[allow(unused)]
    pub fn is_subset(&self, other: &BitVec) -> bool {
        self.bits & other.bits == self.bits
    }
}

/// Represents sequence of bit vectors w/ the same # of 1's.
struct OnesGroup {
    n_ones: u32,
    start_offset: usize,
}

impl std::fmt::Debug for OnesGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}", self.n_ones, self.start_offset)
    }
}

impl BitVec {
    pub fn bitvecs_from_chart_col(
        prime_impl_chart: &PrimeImplicateChart,
        col: usize,
        time: &mut PetrickTimeInfo,
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

    pub fn bitsort(bitvecs: &mut [BitVec]) -> Vec<OnesGroup> {
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

/// Get a minimal set of prime implicants for an equivalent expression.
pub fn get_minimal_sop_terms(
    mut prime_impl_chart: PrimeImplicateChart,
    mut prime_impls: Vec<Minterm>,
) -> (Vec<Minterm>, PetrickTimeInfo) {
    if prime_impl_chart.rows.is_empty() || prime_impl_chart.rows.first().unwrap().is_empty() {
        // Ok to panic here because this condition indicates programmer error.
        panic!("Prime implicant chart has either no rows or no columns.");
    }
    // This version currently supports at most 6-variables, but could be extended.
    assert!(prime_impls.len() == prime_impl_chart.rows.len());
    assert!(prime_impl_chart.rows.first().unwrap().len() <= 64);

    let mut time = PetrickTimeInfo::default();

    // Remove essential prime implicants from chart.
    let (mut min_expr_terms, remaining_cols) =
        remove_essential_prime_impls(&mut prime_impl_chart, &mut prime_impls, Some(&mut time));
    if remaining_cols.is_empty() {
        // Indicates all prime impls were essential, so we're done.
        return (min_expr_terms, time);
    }

    // Simplify remaining terms with boolean logic rules.
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
fn pairwise_and(
    current_bitvecs: &[BitVec],
    next_col_bitvecs: &[BitVec],
    time: &mut PetrickTimeInfo,
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

    // Sort and deduplicate.
    let _ = BitVec::bitsort(&mut merged_bitvecs);
    merged_bitvecs.dedup();
    time.pairwise_and += start.elapsed();
    merged_bitvecs
}

const DEV_DEBUG: bool = false;

/// Remove bitvecs that are subsumed by others in the set.
/// As a side effect, sorts reduced `bitvecs`.
///
/// Precondition: `bitvecs` has been sorted and deduplicated.
fn remove_redundant(bitvecs: &mut Vec<BitVec>, time: &mut PetrickTimeInfo) {
    if bitvecs.is_empty() {
        return;
    }
    let start = Instant::now();
    let ones_group_start = BitVec::bitsort(bitvecs);
    let mut to_remove = vec![false; bitvecs.len()];

    if DEV_DEBUG {
        println!("{ones_group_start:?} - {}", bitvecs.len());
    }

    // Find redundant bitvecs.
    let start_inner = Instant::now();
    for i in 0..ones_group_start.last().unwrap().start_offset {
        // If we removed bitvec i, then we'd have removed its supersets also.
        if to_remove[i] {
            continue;
        }
        let bitvec_i = &bitvecs[i];
        let ogs_n = ones_group_start
            .iter()
            .position(|OnesGroup { n_ones, .. }| *n_ones == bitvec_i.count_ones())
            .unwrap();
        for j in ones_group_start[ogs_n + 1].start_offset..bitvecs.len() {
            if !to_remove[j] && (bitvecs[i].bits & bitvecs[j].bits == bitvecs[i].bits) {
                to_remove[j] = true;
            }
        }
    }
    time.remove_redundant_first_loop += start_inner.elapsed();

    // Remove redundant bitvecs.
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
pub fn remove_essential_prime_impls(
    prime_impl_chart: &mut PrimeImplicateChart,
    prime_impls: &mut Vec<Minterm>,
    time: Option<&mut PetrickTimeInfo>,
) -> (Vec<Minterm>, Vec<usize>) {
    assert!(prime_impls.len() == prime_impl_chart.rows.len());
    let start = Instant::now();
    let num_cols = prime_impl_chart.rows.first().unwrap().len();

    // Records how many rows cover each column, singling out the case
    // where a column is covered by only a single essential row.
    let mut remove_cols: Vec<RowCount> = vec![RowCount::None; num_cols];

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

    // Records if each prime implicant is essential.
    let mut is_essential = vec![false; prime_impls.len()];
    // Columns that are covered by a prime impilcant.
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

    if let Some(time) = time {
        time.remove_essential_prime_impls += start.elapsed();
    }
    (ess_prime_impls, remaining_cols)
}

// -----------------------------
// Timing data for optimization.

#[derive(Default)]
pub struct PetrickTimeInfo {
    pub remove_essential_prime_impls: Duration,
    pub bitvecs_from_chart_cols: Duration,

    pub remove_redundant: Duration,
    pub remove_redundant_first_loop: Duration,

    pub first_loop: Duration,
    pub second_loop: Duration,

    pub pairwise_and_calls: u64,
    pub pairwise_and: Duration,
}

impl PetrickTimeInfo {
    pub fn get_report(&self) -> String {
        let mut message = String::new();
        writeln!(message, "Petrick run time:").unwrap();

        writeln!(
            message,
            "-- remove_essential_prime_impls: {:>5} ms",
            self.remove_essential_prime_impls.as_millis()
        )
        .unwrap();

        writeln!(
            message,
            "-- bitvecs_from_chart_cols:      {:>5} ms",
            self.bitvecs_from_chart_cols.as_millis()
        )
        .unwrap();

        writeln!(message).unwrap();
        writeln!(
            message,
            "-- remove_redundant:             {:>5} ms",
            self.remove_redundant.as_millis()
        )
        .unwrap();

        writeln!(
            message,
            "-- remove_redundant first loop:  {:>5} ms",
            self.remove_redundant_first_loop.as_millis()
        )
        .unwrap();

        writeln!(message).unwrap();
        writeln!(
            message,
            "-- first loop:                   {:>5} ms",
            self.first_loop.as_millis()
        )
        .unwrap();

        writeln!(
            message,
            "-- second loop:                  {:>5} ms",
            self.second_loop.as_millis()
        )
        .unwrap();

        writeln!(message).unwrap();
        writeln!(
            message,
            "-- pairwise_and calls:           {:>5} ",
            self.pairwise_and_calls
        )
        .unwrap();

        write!(
            message,
            "-- pairwise_and:                 {:>5} ms",
            self.pairwise_and.as_millis()
        )
        .unwrap();

        message
    }
}
