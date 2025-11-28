# Rust Quine-McCluskey

This is a basic Rust implementation of the [Quine-McCluskey](https://en.wikipedia.org/wiki/Petrick%27s_method)
algorithm for simplifying sum-of-products logic expressions.
This version supports expressions in at most 6 variables, but
could be adapted to handle more variables. There are several other
crates available with good implementations of Quine-McCluskey, but
implementing it from scratch was fun and a good learning experience.

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
    // Function to simplify: (!C & B) | (C & B) | (!B & A) | (!B & !A) | C.
    let minterms: Vec<Minterm> = vec![
        "01x".into(), // !C &  B
        "11x".into(), //  C &  B
        "x01".into(), // !B &  A
        "x00".into(), // !B & !A
        "1xx".into(), //  C
    ];
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

## Greedy search for faster results

By default the second stage of the algorithm -- choosing a minimal set of prime implicants -- is
performed using Petrick's method. This method is produces an exact result but is unfortunately
quite slow sometimes. In fact, the problem being solved is an instance of the Set Covering
optimization problem which is known to be NP-hard.

However, it is also known that the basic greedy algorithm generally does quite a good job of
approximating solutions to the optimal set covering. We have implemented it here, and test
results bear this out; it produces expressions that are usually at most one or two terms
longer than the optimal expression produced by Petrick's method. To use this feature you can
pass the `-g, --greedy` flag to the `qm` program.
