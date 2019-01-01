# Rado

Rado is a description language to express randomizer logic chains in, and a
library (written most likely in Rust) to load and evaluate item placements for
item randomizers in games. It's inteded primarily for use in games like
metroidvanias and adventure games, where items unlock progression to other
items requiring possibly-complex logic for which items can be used to get
where, rather than in, say, an RPG with randomized character abilities.

The immediate goal is to write a unified description of randomizer logics so
that they can easily be shared across many tools. Probably the most immediate
application would be to make it easier to allow trackers to support different
logis. The other application is to allow randomizers to be able to outsource the
"Is this arrangement of items valid?" portion of the randomization algorithms.
That said, there are no immediate plans to provide randomization algorithms
directly in the Rado libraries.

The language is named after graph theorist Richard Rado, and in particular the
[Rado graph](https://en.wikipedia.org/wiki/Rado_graph) which is the graph you
almost always get when you try to generate a random infinite graph. It also
sounds quite close to "rando".
