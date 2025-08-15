# Basic Rust Quine-McCluskey Implementation.

This is a basic Rust implementation of the [Quine-McCluskey](https://en.wikipedia.org/wiki/Petrick%27s_method)
algorithm for simplifying sum-of-products logic expressions.
This version supports expressions in at most 6 variables, but the
implementation could be adapted to handle more variables.

__Example 1:__

```text
Initial expression:
  (!A & !B & !C) | (!A & !B & C) | (!A & B & !C) | (A & !B & C) | (A & B & !C) | (A & B & C)
Equivalent expression from prime implicants:
  (B & !C) | (!B & C) | (!A & !B) | (A & C) | (!A & !C) | (A & B)
A minimal equivalent expression:
  (B & !C) | (!A & !B) | (A & C)
```

__Example 2:__

```text
Initial expression:
  (B & !C) | (B & C) | (A & !B) | (!A & !B) | (C)
Equivalent expression from prime implicants:
  (C) | (True)
A minimal equivalent expression:
  (C)
```

## Usage example:

This example of the API usage is from `bin/qm.rs`:

```rust
use logic_minimization::*;

fn main() {
  let minterms: Vec<Minterm> = vec![
    "01x".into(), // !C & B
    "11x".into(), //  C & B
    "x01".into(), // !B & A
    "x00".into(), // !B & !A
    "1xx".into(), //  C
  ];
  // Function to simplify: (!C & B) | (C & B) | (!B & A) | (!B & !A) | C.

  println!(
    "Initial expression:\n  {}",
    string_for_sop_minterms(&minterms, false)
  );

  let prime_impls: Vec<Minterm> = get_prime_implicants(&minterms).into_iter().collect();
  println!(
    "Equivalent expression from prime implicants:\n  {}",
    string_for_sop_minterms(&prime_impls, false)
  );

  let prime_impl_chart = create_prime_implicant_chart(&prime_impls, &minterms);
  let minimal_sops = petrick::get_minimal_sops(prime_impl_chart, prime_impls);
  println!(
    "A minimal equivalent expression:\n  {}",
    string_for_sop_minterms(&minimal_sops, true)
  );
}
```
