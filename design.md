## Underlying Philosophy

The first questions that Peri aims to tackle are of the form "Can the player
accomplish this goal, given certain parameters?" In the simplest case, the goals
are of the form "retrieve the item at a given location". The most important
other goal is "complete the game's win condition", since ultimately, completing
the game is the goal. There are other potential goals, however, such as "check
item at this location" or "trigger this event".

The parameters can take a few forms. There are configuration inputs, which in
randomizers may simply be a matter of personal preference, but may also imply
significant changes to the way that the game works. There can be questions about
what sort of techniques the player is willing or able to use. And there can also
be questions about the goals accomplished so far. Usually they are positive, but
they can sometimes be negative, forcing a player to take a gamble in completing
something at the possible cost of locking something else out. The most obvious
case is one of consumables that can only be used in one place.

There are advanced questions, like "What can be done next?" and "What is
required to complete this goal?" Ultimately, all these questions reduce to the
earlier questions and complicated algorithms. The question of whether a seed is
completeable, for instance, is a graph traversal starting from the beginning of
the game and seeing whether goals can be accomplished eventually allowing for
completion of the endgame goal.

## Concrete Structure

We can basically represent the entire game as a very abstract graph of goals and
their consequences, allowing us to dynamically calculate whatever we want, but
this doesn't represent how people think about games. Instead, we want concrete
primitives that people can work off of.

The simplest randomizer one can imagine only models items and locations. In
simple logics, a location has an item. A location has a number of prerequisite
items; if you have all of the items required to access it, then you have the
ability to visit the location and retrieve the item there.

We need to be able to compose pieces easily. For instance, some maneuvers are
common and always have the same requirements, and we want to be able to pull
those out into reusable functions. This not only helps code reuse, but also can
help explain when an item is required for multiple purposes.

Locations also need composition, as generally there are many steps to getting to
a particular location, each possibly having their own requirements. Composing
them makes it easier to understand the exact nature of the requirements, and is
important for making more complex randomizers like entrance randomizers, which
require a detailed mapping of points in the game and the requirements to
navigate between them.

A major piece from the logic side is configurations. Randomizers often have many
inputs that are user-configurable, to tweak difficulty or provide for
interesting new modes. Peri should be able to represent these. There are also
various options that the user may wish to set representing the techniques that
they know, and even if a randomizer does not take account for them, it is useful
for a tracker to be able to understand when an item is accessible but not by the
placement logic.

Finally, it is important to account for the fact that a requirement may itself
be randomized. The language must have some way of expressing this.

For more advanced usage, there are also things like being able to check whether
or not an item is accessible, complex relationships of consumables, and many
other features. From the randomization side, we may want to support other things
like subsets of items which are locked out from access, or situations where a
player may be forced to make a gamble and possibly reset their progress if it
does not pay off, as they risk locking themselves into a place from which they
cannot complete the game. This is most likely to occur in games whose original
design is to require you to use an item immediately after acquiring it to escape
the immediate area; without permitting this, you make the vanilla placement
unacceptable.

Placement restrictions must also be encodeable, limiting where items can be or
possibly even requiring them to be in certain locations. While not strictly
required for trackers, in practice the ability to do so may make it easier to
handle some cases more elegantly, and be more extensible to different cases.
Probably, a tracker would be more interested in understanding different classes
of locations (a type of metadata, perhaps?).

## Specific Requirements

Below follows a list of requirements that the language and engine should be able
to meet. Requirements that I feel are required for a minimum viable product
(largely, those required for a version version of ALttP and Super Metroid
support) will be marked with (\*) at the beginning; the other requirements need
not be implemented or even fully specced early on, so long as we can design with
the possibility of later adding them in mind. Examples will be added for most
requirements.

### Basic logic

1. (\*) It should be possible to describe a list of locations, a list of items,
   and the requirements to acquire the items at each location.
1. (\*) It should be possible to write logical expressions (AND and OR) for the
   requirements to acquire an item. *Example: ALttP requires the bow and the
   hammer to defeat Helmasaur and acquire his item.*
1. (\*) It should be possible to require multiples of a certain item, rather
   than just one. *Example: ALttP requires the Master Sword, which is
   equivalent to two progressive swords, to acquire the items on the Bombos and
   Ether tablets.*
1. (\*) It should be possible to write functions expressing requirements to
   reuse across different parts of the code. *Example: Super Metroid requires
   jumping while in Morph Ball form in various places; this requirement is met
   by having Bombs, Power Bombs, or Spring Ball.*
1. (\*) It must be possible to perform basic arithmetic, not just boolean
   expressions. *Example: ALttP requires certain a minimum amount of magic for
   certain actions; the available magic is a function that requires multiplying
   the number of bottles by a factor based on magic reduction level.*
1. (\*) It should be possible to describe a randomized requirement and a list of
   items that it can require. *Example: ALttP requires a random medallion to
   enter Misery Mire.*
1. It should be possible for a randomized requirement to take on other types,
   such as booleans, integers, and enumerations.
1. (\*) It must be possible for a negative requirement to exist. *Example: In
   Metroid Prime, triggering the floaty jump bug requires that the player not
   have the Gravity Suit.*
1. (\*) It should be possible for every location, item, and randomized
   requirement to be given both a human-readable name and one or more
   identifiers which can be easily referred to.
1. (\*) It should be possible to factor out access requirements common to a
   group of locations. This may simply be by expressing locations as a nested
   set of regions. *Example: Super Metroid requires that the player pass through
   the lava at the entrance to Lower Norfair to enter and access any of its
   items.*
1. (\*) It should be possible to support configuration parameters with various
   types and values (at the least, boolean, integer, enumeration) and use values
   of these parameters in requirements.
1. (\*) It should be possible to define a single configuration item which
   applies its values to a number of other items. *Example: An author defining a
   logic with a number of glitches wishes to define "all glitches" and "no
   glitches" mode which enable and disable each glitch configuration
   separately.*

### Items

1. It should be possible to mark an item as consumable, where it can be used in
   multiple places but only once. It must be easy to use in the context of
   complex expressions. *Exampe: Small keys in ALttP are one-use only.*
1. It should be possible to track the maximum and current values of an item, and
   express when the player can or cannot refill it. *I don't have an example off
   the top of my head, but I believe that the Metroid Prime 2 logic uses this
   for energy in the Dark World. It may also be relevant in Super Metroid
   romhacks or door randomizers where many missile doors in a row are required
   and the ability or inability to refill in between matters.*
1. It should be possible to mark certain items as already possessed at the start
   of the game, conditional on configuration. *Example: ALttP requires that
   Zelda be rescued to access most items, but Open Mode starts with Zelda
   rescued.*

### Locations

1. It should be possible to describe locations that do not have any items, and
   requirements to move from one location to another. *Example: ALttP Entrance
   Randomizer.*
1. It should be possible to describe as randomized the way locations are
   connected. If regions are supported, then this must be able to remove the
   parenting effects of regions if desired. *Example: ALttP Entrance Randomizer
   may move an entrance located in Dark World Death Mountain, which has many
   access requirements, to the Light World, which has none.*
1. It should be possible to provide an alternate set of requirements in order to
   learn what is at a location without being able to collect it. *Example: In
   ALttP, a player can check the item at the Lumberjack Cave with no
   requirements, even though defeating Agahnim and the Pegasus Boots are
   required to collect it.*

### Placements

1. (\*) It should be possible to require a placement ensure that all items are
   accessible.
1. (\*) It should be possible to divide the items into subsets which have
   restrictions on their placement. This includes one-item subsets which have a
   fixed location. *Example: In ALttP, keys, maps, and compasses are restricted
   to the dungeon in which they occur. In Super Metroid, each boss always gives
   its own completion event regardless of randomization.*
1. It should be possible to permit some subsets of items or locations to be
   inaccessible. *Example: In ALttP randomizer, keys and only keys are
   permitted to be inaccessible.*
1. It should be possible to permit placements, for at least some subset of items
   or locations, where the player may risk having to reset if they do not have
   the required items. *Example: In Super Metroid, if such placements are
   permitted, a player may have to fight Draygon in order to receive an item
   which will allow them to leave the area, possibly forcing them to reset if
   they do not find it and get stuck. If not permitted, the player would never
   have to fight Draygon without first knowing that they will be able to leave.*
1. (\*) It should be possible to define a game-winning condition.

### Composability

1. (\*) It should be possible to divide a single logic modules across multiple
   files.
1. It should be possible to have later definitions override or update
   earlier ones, possibly conditional on configuration. The overrides apply even
   in earlier definitions. *Example: A developer wants to account for a new
   glitch in the logic. They do so by writing a file which adds a configuration
   option for the glitch and updates several requirement functions to refer to
   it. The new definitions replace the previous ones for all purposes, and all
   logic is based only on the additional definitions.*
1. (\*) It should be possible to make definitions, overrides, or updates
   conditional on configuration. *Example: A developer wants to write the logic
   for a configuration that radically changes the game without needing to put
   conditionals in every requirement function; instead, they write the logic as
   a series of overrides and make it conditional on the configuration.*

### Functionality

1. (\*) Peri should provide a library which can be used to parse and work with
   logic files.
1. (\*) The library should allow specifying the files composing a module, and
   the order in which they are loaded.
1. The library should allow dynamically adding additional files or updated
   definitions to an already-loaded module.
1. (\*) The library should allow querying which locations are accessible given a
   set of current items and configuration. *Example: A tracker wishes to know
   which locations can be obtained, given the current items and configuration.*
1. The library should provide queries about what is required to reach a
   location. *Example: A tracker wishes to display what items remain to reach a
   location.*
1. The library should provide queries about visibility in addition to
   accessibility.
1. (\*) The library should allow queries about whether a placement obeys all the
   provided restrictions, given a configuration. *Example: A naive randomizer
   that shuffles items completely randomly wishes to query whether the result is
   valid.*
1. (\*) The library should allow queries that query possible locations to place
   an item, given the configuration, existing placements and the remaining items
   which are assumed to be accessible. *Example: A more intelligent randomizer
   wants to know where it can legally place an item.*

### Nice-to-Haves

The following are nice-to-haves, but it is unlikely they would ever truly be
necessary for the language:
1. It would be nice to be able to provide human-readable names for functions
   used in evaluations, for functionality like producing a human-readable list
   of missing requirements for an item.
1. Supporting multiple items at a single location would avoid the need to name
   every spot in places where there are many available, at some cost to
   expressivity and specificness.
1. It would be nice to support some form of simplification or coalescing in
   order to be able to do things like automatically calculate the requirements
   to complete a dungeon.

### Non-requirements

There are also a number of non-requirements that need not be implemented. Here
are some that have been thought of, with reasoning:
1. A separate description of an event as a concept distinct from an item is not
   required. They can simply be represented as items; possibly non-randomized
   ones.
