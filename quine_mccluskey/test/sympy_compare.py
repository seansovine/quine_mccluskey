from sympy.logic.boolalg import to_dnf
from sympy.abc import A, B, C, D, E, F

# Expression obtained from: target/release/convert --init 6A40D19FCD51B0EC

result = to_dnf(
    (~A & B & ~C & ~D & ~E & ~F)
    | (A & B & ~C & ~D & ~E & ~F)
    | (A & ~B & C & ~D & ~E & ~F)
    | (~A & B & C & ~D & ~E & ~F)
    | (A & B & C & ~D & ~E & ~F)
    | (~A & ~B & C & D & ~E & ~F)
    | (A & ~B & C & D & ~E & ~F)
    | (A & B & C & D & ~E & ~F)
    | (~A & ~B & ~C & ~D & E & ~F)
    | (~A & ~B & C & ~D & E & ~F)
    | (~A & B & C & ~D & E & ~F)
    | (~A & ~B & ~C & D & E & ~F)
    | (~A & B & ~C & D & E & ~F)
    | (A & B & ~C & D & E & ~F)
    | (~A & B & C & D & E & ~F)
    | (A & B & C & D & E & ~F)
    | (~A & ~B & ~C & ~D & ~E & F)
    | (A & ~B & ~C & ~D & ~E & F)
    | (~A & B & ~C & ~D & ~E & F)
    | (A & B & ~C & ~D & ~E & F)
    | (~A & ~B & C & ~D & ~E & F)
    | (A & B & C & ~D & ~E & F)
    | (~A & ~B & ~C & D & ~E & F)
    | (~A & ~B & C & D & ~E & F)
    | (~A & B & C & D & ~E & F)
    | (A & B & C & D & ~E & F)
    | (~A & B & C & ~D & E & F)
    | (A & ~B & ~C & D & E & F)
    | (A & B & ~C & D & E & F)
    | (A & ~B & C & D & E & F)
    | (~A & B & C & D & E & F),
    simplify=True,
)

print(result)

# Formatted like qm output. Result of:
#  target/release/convert -f "$(python test/sympy_compare.py)"
"""
  (A & B & C & !E)
| (A & C & !E & !F)
| (A & B & D & E & !C)
| (A & D & E & F & !B)
| (!A & B & C & E)
| (!A & F & !B & !E)
| (!A & !B & !C & E & !F)
| (!A & !B & !D & E & !F)
| (B & C & D & F & !E)
| (B & D & E & !F)
| (B & !D & !E & !F)
| (C & D & !A & !B & !E)
| (!C & !D & !E & F)
"""

# target/release/convert -sop '(A & B & C & !E) | (A & C & !E & !F) | (A & B & D & E & !C) | (A & D & E & F & !B) | (!A & B & C & E) | (!A & F & !B & !E) | (!A & !B & !C & E & !F) | (!A & !B & !D & E & !F) | (B & C & D & F & !E) | (B & D & E & !F) | (B & !D & !E & !F) | (C & D & !A & !B & !E) | (!C & !D & !E & F)'
#
# Result: INIT value: 16'h6A40D19FCD51B0EC
