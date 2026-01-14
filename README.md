# Rust Quine-McCluskey

This is a basic Rust implementation of the
[Quine-McCluskey](https://en.wikipedia.org/wiki/Petrick%27s_method)
algorithm for simplifying sum-of-products logic expressions.
This version supports expressions in at most 6 variables, but
could be adapted to handle more variables. There are several other
crates available with good implementations of Quine-McCluskey, but
implementing it from scratch was a good learning project.

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
  True
```

## Usage example:

An example of the API usage:

```rust
use logic_minimization::*;

fn main() {
    // Function to simplify: (~C & B) | (C & B) | (~B & A) | (~B & ~A) | C.
    let minterms: Vec<Minterm> = vec![
        "01x".into(), // ~C &  B
        "11x".into(), //  C &  B
        "x01".into(), // ~B &  A
        "x00".into(), // ~B & ~A
        "1xx".into(), //  C
    ];

    // Or from sum-of-products string:
    let minterms = sop_to_minterms("(~C & B) | (C & B) | (~B & A) | (~B & ~A) | (C)");

    // Or from hex init string:
    let term_strings = binary_strings_from_init_hex("BDBDBDBDBDBDBDBD")?;
    let minterms = term_strings.iter().map(|s| (&**s).into()).collect()

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

Or the `src/bin/qm.rs` program will take as input either a sum-of-products string or a hex init
string representing a sum-of-products expression:

```shell
target/release/qm -s '(~C & B) | (C & B) | (~B & A) | (~B & ~A) | C'
```

## Greedy search for faster results

By default the second stage of the algorithm -- choosing a minimal set of prime implicants -- is
performed using Petrick's method. This method produces an exact result but is unfortunately
quite slow sometimes. In fact, the problem being solved is an instance of the Set Covering
optimization problem which is known to be NP-hard.

However, it is also known that the basic greedy algorithm typically produces a good
approximate covering solution. We have implemented the greedy approximation here, and test
results bear this out: It produces expressions that are usually at most one or two terms
longer than the optimal expression produced by Petrick's method.

To use this feature you can pass the `-g, --greedy` flag to the `qm` program.

## Correctness

In boolean logic two functions are either equivalent or they are not.
But the Quine-McCluskey method is proven to produce equivalent expressions of the
shortest possible length. So a correct implementation must both:

1. Always produce expressions that are functionally equivalent to its input.

2. Produce outputs that have the fewest terms among all equivalent expressions.

We have two corresponding utitities for testing our implementation.

### Round-trip testing

The program `test-round-trip.rs` generates a sequence of random boolean functions using random hex
init strings. Then it simplifes each of these functions and converts the simplified function
back to the init string format. It's fairly straightforward to convert a minimal sum-of-products
representation back to an init string; you might call this "un-minimization". Then
the simplifed function will be equivalent to the original exactly when the back-converted init
string is the same as the original input string. The program checks each generated example for
this equivalence.

### Comparison with a well-established library

The SymPy Python library has a module that simplifes boolean functions in disjoint normal form
using Quine-McCluskey. SymPy is a well-established library, so its results have been well tested.
We provide a program `sympy-compare.rs` to compare the results of this implementation to that of
SymPy. This program randomly generates a sequence of boolean functions that are simplified with
SymPy and with the implementation in this library.
There can be multiple equivalent minimal functions for a given input, so we only compare the
number of terms in each result. This comparison supports the second, minimality part of our
correctness claim.
