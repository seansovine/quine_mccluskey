use logic_minimization::*;

fn main() {
  let minterms: Vec<Minterm> = vec![
    "000".into(), // A'B'C'
    "100".into(), // A'B'C
    "010".into(), // A'BC'
    "101".into(), // AB'C
    "011".into(), // ABC'
    "111".into(), // ABC
  ];

  //   let minterms: Vec<Minterm> = vec![
  //       "0100".into(), // m4
  //       "1000".into(), // m8
  //       "1001".into(), // (m9)
  //       "1010".into(), // m10
  //       "1011".into(), // m11
  //       "1100".into(), // m12
  //       "1110".into(), // (m14)
  //       "1111".into(), // m15
  //   ];

  //   let minterms: Vec<Minterm> = vec![
  //     "100000".into(), //
  //     "000000".into(), //
  //     "000010".into(), //
  //     "000011".into(), //
  //   ];
  println!("Initial expression:\n  {}", string_for_minterms(&minterms));

  // TODO: Update to handle "don't cares" in the input minterm set.

  let prime_impls: Vec<Minterm> = get_prime_implicants(&minterms).into_iter().collect();
  println!(
    "Equivalent expression from prime implicants:\n  {}",
    string_for_minterms(&prime_impls)
  );

  let prime_impl_chart = create_prime_implicant_chart(&prime_impls, &minterms);
  let minimal_sops = petrick::get_minimal_sops(prime_impl_chart, prime_impls);
  println!(
    "A minimal equivalent expression:\n  {}",
    string_for_minterms(&minimal_sops)
  );
}
