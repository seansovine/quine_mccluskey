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
    let minterms = init_to_minterms("BDBDBDBDBDBDBDBD")?;

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
    let minimal_sop_terms = petrick::get_minimal_sop_terms(prime_impl_chart, prime_impls);

    println!(
        "A minimal equivalent expression:\n  {}",
        string_for_sop_minterms(&minimal_sop_terms, true)
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

## Testing

The Quine-McCluskey algorithm takes a boolean function as input and produces an equivalent function
with the fewest number of terms possible. This means that a correct implementation has to do two
things:

1. Always produce as output a function that is equivalent to its input.

2. Produce an output that has the fewest terms among all such functions

Here equivalent means that the two functions give the same outputs for the same inputs.
We have a utility program for testing each of these two conditions using randomly-generated
input functions. These can be useful to make sure no problems are introduced when changes are made
to the code, and they help give confidence in its results.

### Round-trip testing

The program `test-round-trip.rs` generates a sequence of random boolean functions in the form of
hex init strings. Then it simplifes each of these functions to produce a sum-of-products, and
converts the simplified sum-of-products
back to the init string format. It's fairly straightforward to convert a sum-of-products
representation to an equivalent init string; you might call this process "un-minimization". Then
the simplifed function will be equivalent to the original input function exactly when the back-converted
init string is the same as the original init string. The program checks each generated example for
this equivalence.

### Comparison with a well-established library

The SymPy Python library has a module that simplifes boolean functions using the Quine-McCluskey
algorithm. SymPy is a well-established library, so it can be regarded as a reliable
source of truth to compare our results to.
We provide a program `sympy-compare.rs` to compare the results of this implementation to that of
SymPy. This program randomly generates a sequence of boolean functions, which are then simplified
with both SymPy and with the our implementation. There can be multiple equivalent minimal
functions for a given input, but we can confirm the minimality of our results by comparing the
number of terms they contain to the number of terms in the results from Sympy.
