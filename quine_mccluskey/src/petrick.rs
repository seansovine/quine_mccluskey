// Implement Petrick's method to get min-SOPs.

use super::{Minterm, PrimeImplicateChart};
use std::cmp::Ordering;

// Bit vector data structure for simplifying prime implicant table.

/// Bit vector representing a set of essential prime implicants.
///
/// **Note:** For now we support up to 64 bits; this could be extended.
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
}

impl BitVec {
  pub fn bitvecs_from_chart_col(prime_impl_chart: &PrimeImplicateChart, col: usize) -> Vec<Self> {
    let mut bit_vecs = vec![];
    let rows = &prime_impl_chart.rows;

    if rows.is_empty() {
      return bit_vecs;
    }
    assert!(rows.first().unwrap().len() > col);

    for (i, row) in rows.iter().enumerate() {
      if row[col] {
        let mut bit_vec = BitVec::default();
        bit_vec.set_bit(i);
        bit_vecs.push(bit_vec);
      }
    }
    bit_vecs
  }

  pub fn bitsort(bitvecs: &mut [BitVec]) {
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
  }
}

// Functions to perform Petrick's method to simplify prime implicants table.

/// Get a minimal set of prime implicants for an equivalent expression.
///
pub fn get_minimal_sops(
  mut prime_impl_chart: PrimeImplicateChart,
  mut prime_impls: Vec<Minterm>,
) -> Vec<Minterm> {
  if prime_impl_chart.rows.is_empty() || prime_impl_chart.rows.first().unwrap().is_empty() {
    // Ok to panic here because this condition indicates programmer error.
    panic!("Prime implicant chart has either no rows or no columns.");
  }
  // This version currently supports at most 6-variable eqns, but could be extended.
  assert!(prime_impls.len() == prime_impl_chart.rows.len());
  assert!(prime_impl_chart.rows.first().unwrap().len() <= 64);

  // Remove essential prime implicants from chart.

  let (mut min_expr_terms, remaining_cols) =
    remove_essential_prime_impls(&mut prime_impl_chart, &mut prime_impls);
  if remaining_cols.is_empty() {
    // Indicates all prime impls were essential, so we're done.
    return min_expr_terms;
  }

  // Simplify remaining terms with boolean logic rules.

  let first_remaining_col = *remaining_cols.first().unwrap();
  let mut current_bitvecs: Vec<BitVec> = vec![BitVec::default()];

  for rem_col_i in remaining_cols.iter().copied() {
    let next_col_bitvecs = BitVec::bitvecs_from_chart_col(&prime_impl_chart, rem_col_i);
    if !next_col_bitvecs.is_empty() {
      current_bitvecs = pairwise_and(&current_bitvecs, &next_col_bitvecs);
    }
    remove_redundant(&mut current_bitvecs);
  }

  BitVec::bitsort(&mut current_bitvecs);
  let chosen_min_bitvec = current_bitvecs.first().unwrap();
  for i in chosen_min_bitvec.nonzero_indices() {
    min_expr_terms.push(prime_impls.get(i).unwrap().clone());
  }

  min_expr_terms
}

/// Computes the logical 'and' between
///
fn pairwise_and(current_bitvecs: &[BitVec], next_col_bitvecs: &[BitVec]) -> Vec<BitVec> {
  let mut merged_bitvecs = vec![];
  for c_bitvec in current_bitvecs {
    for n_bitvec in next_col_bitvecs {
      let mut new_bitvec = *c_bitvec;
      new_bitvec.merge(n_bitvec);
      merged_bitvecs.push(new_bitvec);
    }
  }
  merged_bitvecs.dedup();

  merged_bitvecs
}

/// Remove bitvecs that are subsumed by others in the set.
/// As a side efecct, sorts reduced `bitvecs`.
///
fn remove_redundant(bitvecs: &mut Vec<BitVec>) {
  // Bit vecs to remove at end.
  let mut to_remove = vec![false; bitvecs.len()];
  // Sort so that vecs with fewer bits are first.
  BitVec::bitsort(bitvecs);

  // Find redundant bitvecs.
  for i in 0..bitvecs.len() - 1 {
    for j in i + 1..bitvecs.len() {
      if !to_remove[j] && (bitvecs[i].bits & bitvecs[j].bits == bitvecs[i].bits) {
        to_remove[j] = true;
      }
    }
  }
  // Remove redundancies.
  for i in (0..bitvecs.len()).rev() {
    if to_remove[i] {
      bitvecs.remove(i);
    }
  }
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
) -> (Vec<Minterm>, Vec<usize>) {
  assert!(prime_impls.len() == prime_impl_chart.rows.len());

  // Indexed the same as prime_impls.
  let mut ess_prime_impls_i = vec![false; prime_impls.len()];

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

  (ess_prime_impls, remaining_cols)
}
