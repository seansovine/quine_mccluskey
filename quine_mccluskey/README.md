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
