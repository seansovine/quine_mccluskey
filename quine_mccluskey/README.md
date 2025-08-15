# Basic Rust Quine-McCluskey Implementation.

This is a basic Rust implementation of the [Quine-McCluskey](https://en.wikipedia.org/wiki/Petrick%27s_method)
algorithm for simplifying sum-of-products logic expressions
in at most 6 variables. This initial implementation doesn't support
having "don't care" values for variables in the initial set of
product terms, but I will add that feature in the near future.

__Example:__

```text
Initial expression:
  (!A & !B & !C) | (!A & !B & C) | (!A & B & !C) | (A & !B & C) | (A & B & !C) | (A & B & C)
Equivalent expression from prime implicants:
  (B & !C) | (!B & C) | (!A & !B) | (A & C) | (!A & !C) | (A & B)
A minimal equivalent expression:
  (B & !C) | (!A & !B) | (A & C)
```
