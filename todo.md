These are the areas of work left in the redesign, in no particular order.

1.  Develop a formal model of conditional, template, and override evaluation,
    and prove it sound.
1.  Determine a way to handle undecideability of whether two config blocks can
    be simultaneously active.
1.  Determine if interactions between configurable flags via conditionals and
    overrides can be soundly added to the model without too much trouble.
1.  Settle on a syntax for overrides (taking into account the major revised
    program structure, greater applicability of overrides, and simplified
    properties other than tags).
1.  Update samples.
1.  Determine how items and capacities, etc. can work, including parameters on
    them (like total amount healed). Also address numeric types, and in
    particular whether there should be an `int` type (probably) and whether
    types with infinities are needed (probably yes, also?)
1.  Implement randomized parameters.
1.  Define the effect of division by 0 or other runtime errros.

Ideas:

*   For 2, conditionals can be tagged; new property `exclusive T, not F`
    expresses that it will never be the case that a `T`'s true block and a `F`'s
    false block will be active at the same time. Interaction with override model
    might be tricky?
*   For 2 and 4, tag modifiers: `#+[ ... ]` for adding tags and `#-[ ... ]` for
    removing them on modifying declarations, and `#not [ ... ]` for negating
    conditional tags.
*   For 6, can they reasonably be implemented on top of flags (rename to
    variables?)?
*   For 7, remember to include visibility statement.
*   6 and 7 may need to be done together, to ensure that randomized item
    locations are quite easy to do.
*   Standard library of templates.
*   `x / 0 = 0` might be viable? Runtime errors seem awful.
