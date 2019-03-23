Proof still needed that the whole override/conditional system works out. In
particular:

 1. Prove commutativity of overrides that don't conflict or supersede.
 1. Prove that configuration variables can have values provided in a way which
    never leads to paradox or ambiguity.

Future work in the language itself:

 1. Create a fuller type system allowing for easy factoring out of common
    patterns like inventories and current/max items.
 1. Investigate restrictions on randomized variables to allow computation of
    things like "what are my legal placements?"
 1. Experiment with the language and ergonomics and see where improvements are
    needed, or if tweaks are needed for the more complex rule systems.
 1. Build out a standard library.
 1. The current logic is incapable of ensuring that, say, all items are
    accessible. Create a way to do this (probably multiple victory conditions?)

Next steps:

 1. Scrap the start of semantic work
 1. Rewrite the parser (replace the bespoke lexer with lalrpop's built-in)
 1. Rewrite the samples, using the parser to check that they match the intent
 1. Implement the language
