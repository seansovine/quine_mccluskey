# Set Game Solver

This program is a solver for the game Set.
If you aren't familiar with it, in Set there is a deck
of cards, and each card varies along four dimensions:
color, number, shape, and texture. For example, a card
could have 3 red solid diamonds on it.

Color of objects:

- red
- green
- blue

Number of objects:

- one
- two
- three

Shading of objects:

- filled
- outlined
- striped

Shape of objects:

- diamond
- oval
- squiggle

At the start of gameplay 12 cards are laid face-up on the table.
Players score points by forming _sets_ from the visible cards.
In Set, a _set_, is a collection of the cards that are either
all the same or all different in each dimensions.
So, for example, the cards in a set might all have the same shape,
all have different colors, all have different textures, and
all have the same number of objects.

## The algorithm:

The approach here is to first choose all candidate sets of 3 cards based on
a single attribute (color is arbitrarily chosen here), and then
to check if each candidate has all attributes the same or all different
in each other dimension, in which case it is considered a valid "set"
according to the game's definition.

The game has a relatively small search space, so the approach here works fine
as long as it's implemented with reasonable efficiency. My experience as a 
human player is that I find sets through a combination of scanning the table
until my mind notices a pattern intuitively, probably based on some heuristics,
and a more systematic approach as is used in this program.

## This program:

Randomly draws 15 cards (†) from the Set deck, and then finds all
_sets_ (by the game's definition) contained in those cards.

† `CARDS_ON_TABLE` is set to 15 in `find_set.cpp`, but can be any value between
0 and 3 \* 3 \* 3 \* 3. The normal Set game uses 12 cards, unless there is
no set in the current table cards, in which case 3 more cards are added
until there is a set.

**Example output:**

![set solver output](https://github.com/seansovine/math_fun/blob/public/set_game_solver/example_output.png?raw=true)
