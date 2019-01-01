# Language Description

Rado is a declarative language that expresses logical systems. For the most
part, there is no state, input/output, or side effects in Rado. Later
declarations can modify earlier ones, however, all declarations are processed
fully before any evaluation can occur. The logical system described by a Rado
specification can be queried using a suitable interpreter.

## Overview

In Rado, the most basic concepts are *items* and *locations*. In a randomizer
game, the available items are shuffled and placed in the various locations. To
Rado, an item is little more than a piece of data with some properties, and a
location is a place where an item can go.

Not all locations are equal, of course. Some require that the player have
already collected a certain item, have reached a particular event trigger, or
any number of other conditions. In Rado, these are expressed as *requirements*.

In order to facilitate organizing locations, Rado has the concept of *regions*,
which correspond to areas in the game that a player can visit. Regions can have
*links* between each other, defining how the player can traverse the game and
the requirements to do so.

In order to allow for code reuse, Rado allows items to have *tags* indicating
common properties, as well as associated *values* such as numerical parameters.
These can be referenced in requirements or in *functions* used to compute values
(particularly requirements).

Values are generally numeric (arbitrary rational numbers) or boolean, but Rado
also supports strings, user-defined enumerations, and lists. Items are also a
type of their own.

Names in Rado follow a rudimentary scoping system. Regions introduce scopes
implicitly, and a scope can be explicitly declared with a *module*.

Finally, to support customization, Rado supports *configs*, which are values
describing various input parameters to the randomization process, such as the
game mode or a player's known techniques.

## Scoping

Scoping in Rado is quite simple. Each region introduces a new scope. When a name
is used, it can refer to any name declared in the same or an enclosing scope
(even if it is declared after the point of use). Names in other scopes can be
referred to with a dot syntax `Outer.Inner`; dot syntax is also used to refer to
values on items.

Conditional blocks also introduce scopes, however, they are anonymous so they
cannot be referred to from outside.

Almost all names are declared in the innermost scope in which they appear. Name
shadowing is not permitted; a declaration cannot use the same name as something
else in the same or an enclosing scope. This means that absolute paths are not
required.

## General Syntax

A Rado specification is composed of several files (usual extension `.rado`) put
together. Each file is a series of *statements*, which always start with a
keyword indicating the type of statement. A statement can span multiple lines,
and multiple statements can appear on the same line. Semicolons are permitted
between statements, but are not necessary; the starting keyword is enough to
disambiguate. They are encouraged when multiple statements appear on a single
line. Many statements include a *block*, which is a series of statements inside
a pair of braces `{ }`.

Rado's expression syntax supports basic arithmetic operators (`+`, `-`, `*`,
`/`, and `%`), comparison operators (`<`, `<=`, `>`, `>=`, `==`, and `!=`), and
logical operators (`and`, `or`, `not`), as well as function calls (`f()`). For
conditional evaluation, `if ... then ... else ...` can be used, or a `match`
expression can be used to perform a rudimentary switch-case operation on an
enumeration (`match val { Foo => ..., Bar => ... }`). Expressions can be grouped
in parentheses to disambiguate `( ... )` precedence, which is mostly like in `C`
except that it is an error to try to associate `and` and `or` without nesting
one operator or the other in parentheses.

Comments are in C++ style: `//` for line comments, `/* */` for block comments.
Unlike in C and C++, however, block comments can be nested.

Identifiers are supported per Unicode syntax. All keywords, including built-in
functions, are reserved and cannot be used as an identifier anywhere in the
program. Keywords are written in `lowercase`, and while no style of identifiers is
enforced, `UpperCamelCase` is recommended to avoid clashes with keywords and so
that human-readable names can be generated for declarations automatically.

Numeric literals are written as integers; only decimal literals are currently
supported. `_` may be used as a digit separator. `true` and `false` are the
boolean literals, and string literals are written between quotes `"..."`. Basic
common escape sequences are supported; an error is emitted for any unknown
escape sequence so that more can be added later. String literals are currently
not usable as expressions, but only in human-readable names in declarations.
They are UTF-8 encoded.

## Declarations

A *declaration* is a statement which creates something and gives it a name. Most
declarations have of the general form `decl Name "Human Name" ...`.

*  `decl` is a keyword which indicates the type of declaration. It is mandatory
   except in certain kinds of blocks which implicitly provide a type for
   declarations inside them.
*  `Name` is an identifier indicating the name of the thing being declared, and
   is mandatory.
*  `"Human Name"` is a string literal providing a human-readable version of the
   thing being declared. Only some types of declarations allow them, and they
   are always optional. If one is not provided, the compiler will construct a
   human-readable name by adding spaces in between words of an `UpperCamelCase`
   name.
*  `...` is the rest of the declaration. The syntax varies depending on the kind
   of declaration and may be disallowed, optional, or mandatory.

Some kinds of declarations can be modified or overridden in conditional blocks,
as described below. Restrictions are noted on each kind of declaration are noted
in the section for that declaration.

Tags, values, and alias statements also declare names, but tags implicitly live
in the global namespace rather than being explicitly declared, values have their
own namespace, and aliases only give new names to already-existing things.

### Region Declaration

> Syntax: `region` *identifier* (*string-literal*)? *block*

A region declaration declares a region. Generally, the logic engine considers a
region to be a place where the player is able to be, assuming that the player is
actually able to make it there. Additionally, items can be located in regions,
so that the player can pick them up.

Regions double as namespaces; regions introduce new scopes and declarations can
be placed in them. There is no actual semantic meaning to the nesting property
of regions. They are not separate concepts because there is currently no need to
actually distinguish between them.

Note that a region's accessibility is declared by its links, so a region with no
links will be inaccessible (unless the player starts there) and can be used just
to introduce a scope without affecting the logic.

A region declaration cannot be deleted if it contains non-deleteable
declarations.

### Link Declaration

> Syntax: `link` (*identifier* (*string-literal*))? (`with` | `to` | `from`) list(*name*) (*block*)?

A link declaration declares a link between two regions. The logic uses links to
work out how the player can move around in the game. A link declares a
connection with the named regions based on the second word: `to` is a one-way
link to those regions, `from` is a one-way link from them, and `with` is a
two-way link.

On a modifying declaration, the `with`, `to`, or `from` and list can be omitted.
If it is not, the direction must match, and the list is optionally a modifier
list.

**Links are planned to be significantly changed and will not be implemented
anything like described.**

### Item Declaration

> Syntax: `item` *identifier* (*string-literal*)? (*block*)?

An item declaration introduces a new item into the logic. To the logic engine,
an item is something that the player can acquire, possibly in multiples. The
logic assumes that items are randomized among their locations.

#### Multi-Item Declaration

> Syntax: `items` list(*identifier*) *block*

A multi-item declaration is a shortcut for declaring multiple items. Inside its
block, only item declarations and nested multi-item declarations are allowed.
Item declarations must omit the leading `item` keyword.

The list of names in the declaration is a list of tags which are declared on
every item inside. When multi-item declarations are nested, all the tags from
all the nested declarations are declared on each item.

Multi-item declarations can be nested; the effect is to declare all the tags on
the contents.

Items declarations can be prefixed with `modify` or `override`, in which case
they behave as if every declaration in them is as well. In modifying
multi-item declarations, the tag list must be a modifier list.

### Location Declaration

> Syntax: `location` *identifier* (*string-literal*)? (*block*)?

A location declaration introduces a new location into the logic, which serves as
a place for an item to be located.

#### Multi-Location Declaration

> Syntax: `locations` *block*

A multi-location declaration is a block where many locations can be declared,
omitting the leading `location` keyword, similar to a multi-item declaration. It
can be prefixed with `modify` or `override` to make each declaration contained
into a modifying or overriding declaration.

### Function Declaration

> Syntax: `fn` *identifier* (*string-literal*)? (`(` list(*identifier* (`:` *type*)?) `)`)? (`->` *type*) = *expression*

A function declaration introduces a new function which can be used in
expressions. A function can have an argument list, or it can be omitted.
Likewise, the argument and return types can be omitted; if they are, then they
are inferred.

A function can be used anywhere an expression is legal. If it has no arguments,
then it is called automatically without needing a call expression `()`.

Functions can be overridden, but not modified or deleted.

### Enum Declaration

> Syntax: `enum` *identifier* (*string-literal*)? *block*

An enum declaration introduces a new enumeration type. Each statement in the
block must consist only of an identifier and possibly a human-readable name;
each one is the name of a value of the enumeration. The declaration declares
both the type name and the names of each value in the surrounding scope.

Enums cannot be modified or overridden.

### Config Declaration

> Syntax: `config` *identifier* (*string-literal*)? `:` *type* (`default` *expression*)?

A config declaration introduces a new configuration option for the logic. A type
must be explicitly specified. Optionally, a default value can be included; the
default must be a constant.

Enums cannot be modified, overridden, or deleted.

### Config-Enum Declaration

> Syntax: `config` *identifier* (*string-literal*)? `:` `enum` (`default` *expression*)

A config-enum declaration is a hybrid declaration that declares both an enum
type and a config with the same name. The config's type is that of the enum.
Since enums are types and configs are values, this does not cause ambiguity. It
is a shorthand for declaring the config and enum separately, but also allows
them to share a name which is not otherwise possible.

Config-enums cannot be modified, overridden, or deleted.

### Multi-Config Declaration

> Syntax: `configs` `:` *type* *block*

A multi-config declaration declares many configs in one block, all of the same
type. The block must contain only config declarations without the leading
`config` keyword. Config-enum declarations may not be declared in this fashion;
declare the enum separately.

### Configset Declaration

> Syntax: `configset` *identifier* (*string-literal*) *block*

A configset declaration declares a set of config values with a specific name,
which can be used to make sets of defaults which can be selected without having
to pick each individual option.

The block consists of comma-separated assignments of the form *name* `=>`
*expression*. The name must name a config, and the expression must be a
constant. Selecting the configset, subject to later overrides, sets the values
of all the configs as it specifies.

The block can also contain entries that are simply the name of another
configset. In this case, the configset is treated as if it contains the values
in the other configset as well, as modified by any explicit assignments. A
configset cannot contain multiple overlapping configsets, nor can it contain
itself directly or indirectly.

Configsets cannot be modified, overridden, or deleted.

### Random Declarations

> Syntax: `random` *identifier* (*string-literal*)? `=` `[` list(*expression*) `]`

A random declaration declares a randomized parameter that isn't an item or
location, but still needs to be accounted for in logic. The expression list must
all be the same type and must be distinct constants.

Random declarations can be modified and overridden. In a modifying declaration,
the expression list is optionally a modifier list. Random declarations cannot be
deleted.

## Conditional Blocks

> Syntax: `if` *expression* *block* (`else` *block*)?

A conditional block makes it so that its contents take effect conditionally. The
expression must be one that depends only on constants and the values of configs
and has boolean type. When the specification is evaluated, the declarations
within are ignored if the expression is false and evaluated if it is true.

Within a conditional block, declarations can have four forms: new, overriding,
modifying, and deleting. Overriding and modifying declarations must be prefixed
with the keyword `override` and `modify`, respectively, to avoid the possibility
of accidentally colliding names. Deleting declarations start with `override -`.
Overriding, modifying, and deleting declarations must refer to a previous
declaration.

Regions have a special exception; modifying declarations of regions do not
require the `modify` keyword if they only contain declarations and not property
statements (this is because the declarations inside would be interpreted the
same way whether they are modifying or new). This restriction still enforces
that nothing can be inadvertently modified.

A conditional block can contain property statements; these modify or override
statements on the surrounding region as if the conditional block is a modifying
declaration of the region, even though it has no `modify` keyword.

If two different conditional blocks override or modify a declaration in
ways that conflict, there exists some configuration such that both blocks can be
simultaneously active, and one is not contained within the other, then this is
an error.

### New Declarations

A new declaration inside a conditional block has no special syntax and works
exactly like a declaration outside a conditional. Declarations made inside
conditional blocks, except for tags, are not visible outside the conditional
block. As with other declarations, new declarations cannot shadow names declared
in a parent scope.

### Overriding Declarations

An overridding declaration is one that replaces a previous declaration
wholesale. It is prefixed with the word `override`. When an override declaration
is applied, the previous declaration is ignored.

### Modifying Declarations

A modifying declaration is one that modifies an existing declaration. A
modifying declaration must be prefixed with the keyword `modify` followed by the
name of the thing being modified (with no human-readable names used) and then
the rest of the declaration. Regions are an exception, as described above.

In a modifying declaration's syntax, most lists can be replaced with modifier
lists (function parameter lists and any lists inside an expression cannot). A
modifier list is wrapped as `+[ ... ]`, and each element is optionally prefixed
with '-'. Elements without '+' are added, elements without are removed. (The
leading '+' is required even when only removing elements to avoid parse
ambiguities). Expressions' values are computed as constants before determining
whether they match or not.

Note that when a negative numeric constants are tokenized together, and
therefore could get confusing: `+[-1]` is a list adding `-1`, but `+[- 1]` is a
list adding 1. As a result, both are disallowed and grouping is required: either
`+[(-1)]` or `+[-(1)]`.

If a list can be a modifier list but the modifier syntax is not used, then it
replaces the previous list entirely.

Property statements inside modifying declarations usually behave similarly, with
those accepting lists allowing both modifying and overriding lists, and other
kinds always overriding the original statement. Exceptions are specifically
noted.

### Deleting Declarations

A deleting declaration has the syntax `override` `-` *name*. It deletes the
declared thing. References to it (such as in requirements) remain valid, but
the logic assumes the player cannot interact with them at all (items cannot be
acquired, locations cannot contain items, regions cannot be entered, etc.).

## Property Statements

Property statements are used to give properties to declared items. They can only
appear inside declarations that admit properties in their blocks, 

### Requirement Statement

> Can appear in: regions, locations, links

> Syntax: `requires` *expression*

A requirement statement sets requirements for the player to navigate the game:
visit a region, travel along a link, or access a location. The expression must
be boolean-typed. If none is present, then there are no requirements, equivalent
to `requires true`.

### Visibility Statement

> Can appear in: locations

> Syntax: `visible` *expression*

A visibility statement expresses the requirements for a location to be visible;
that is, for the player to be able to determine what the item is without being
able to pick it up. Regardless of the visibility statement, a location is always
assumed to be visible if it is accessible. If no visibility statement is
present, the location is assumed to not otherwise be visible, equivalent to
`visible false`.

### Unlock Statement

> Can appear in: regions, links.

> Sytnax: `unlock` *name*

An unlock statement expresses a one-time spending requirement for a region or
link. Once the named consumable item is spent, the unlock requirement is
permanently met.

### Tag Statement

> Can appear in: items

> Syntax: `tag` list(*identifier*)

A tag statement specifies that an item has one or more tags. The tags are
implicitly declared globally, and so all tags with the same name are the same
tag. Tag names must be unique within the program as a result. Tags can also be
added to items by way of multi-item declarations.

### Alias Statement

> Can appear in: items, regions, locations, links

> Syntax: `alias` list(*identifier*)

An alias statement specifies additional names for something. They are declared
in the surrounding scope.

Alias statements cannot be overridden, but can be modified by adding additional
names. Such added alias names are only visible in the conditional block in which
they appear.

### Provides Statement

> Can appear in: items

> Syntax: `provides` list(*name*)

A provides statement declares that each item in the list is provided by the item
containing the statement. For all purposes, when computing whether the player
posesses one of the named items, the containing item is counted as if it were
one of them.

### Progressive Statement

> Can appear in: items

> Syntax: `progressive` list(*name*)

A progressive statements declares an item to be provide other items
progressively. The first item listed is provided when the player has one of the
containing item; the second is provided when the player has two, and so on.
Progression is not cumulative; two of the containing item do not provide the
first item.

Note that the progressive statement is an exception to the general rule that
lists are really sets and that order does not matter.

Progressive statements cannot be modified, but can be overridden.

### Value Statement

> Can appear in: items

> Syntax: `val` *identifier* (`:` *type*) = *expression*

A value statement sets a named value on an item. It can be referred to similarly
to a compound name, by writing `Item.Value`. All declarations of the values with
the same name must have the same type, but values live in their own namespace.

A value statement looks sort of like a declaration, but semantically it does not
actually behave as one, because it does not really declare a name. It is more
like a setting an value in a key-value mapping.

### Max Statement

> Can appear in: items

> Syntax: `max` *expression*

A max statement declares the maximum amount of an item that a player can
possess. Above this limit, more instances of the item cannot be acquired.

### Consumable Statement

> Can appear in: items

> Syntax: `consumable`

A consumable statement declares an item to be consumable. It cannot be
referenced in most expressions (either directly or via one of its tags), but can
be referred to in unlock, grant, and availability statements. An item can never
be removed from the player unless it is marked consumable.

Consumable statements cannot be modified, added, or removed from an item.

The restrictions on consumable items may be relaxed in the future.

**Consumables are planned to be changed to be a distinct type, rather than a
subtype of item. They will consequently work very differently from how they are
described here.**

### Availability Statement

> Can appear in: regions

> Syntax: `avail` list(`not`? *name* (`*` (*integer* | `infinity`))?)

An availability statement declares that an item is available in a region for
pickup. While in the region, the player can acquire the item. It can be used for
event triggers or for non-randomized items. An item name can be prefixed with
`not` to indicate that the player can discard/lose the item rather than acquire
it. It can have an integer or `infinity` on the end indicating how many are
available; it defaults to 1.

Availability statements cannot be modified, and must be explicitly overridden
with `override`; this is to avoid confusion about the effect of modifying a
quantity.

**Availability statements will likely be modified or removed to suit a new
location model.**

### Grant Statement

> Can appear in: regions, links

> Syntax: `grants` list(`not`? *name*)

A grants statement declares that entering a region or travelling along a link
grants or removes a specified item or items. Unlike an availability statement,
this is not optional, even if the player does not want it.

**Grant statements will likely be modified or removed to suit a new location
model.**

### Start With Statement

> Can appear in: regions

> Syntax: `start` `with` list(*name*)

A start with statement indicates that a player starts with the items listed.

### Start In Statement

> Can appear in: global region

> Syntax: `start` `in` *name*

A start in statement declares the starting location of the player for the
logic's purposes.

## Types

Rado has the following types:

* `num`: arbitrary-precision rational numbers
* `item`: a declared item or tag
* `bool`: a boolean
* `fn (A1, A2, ...) -> T`: a function
* lists: `[T]` is a list of `T`s
* enums: for any declared enum `E`, `E` is the type of that enum

Most of these types are quite straightforward, except for `Item`. `Item`
represents an item or tag, and refers to the player's possessions at the time
the expression is evaluated. It may refer to multiple copies of the same item or
to multiple different items. `Item` coerces to `Bool`, and functions accepting
and returning `Bool` or `Item` coerce similarly. The coercion means "Does the
player have any of this item?".

There are no function types without arguments as in `fn () -> T`; because
functions are stateless, this is equivalent to a `T`.

`num`, `item`, and `bool` are keywords and can't be redeclared.

## Expressions

Expressions are fairly straightforward in Rado. The following are supported, in
order of precedence:

1.  Parenthesized expressions
1.  Literals and values (`foo`, `3`, etc.)
    1.  Value access (`i.Val`)
1.  Explicit list creation (`[a, b, c]`)
1.  Function calls (`fn(...)`)
1.  Boolean negation (`not`)
1.  Multiplication, division, and modulus (`\*`, `/`, and `%`)
1.  Addition and subtraction for numbers (`+` and `-`)
1.  Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`)
1.  Boolean conjunction and disjunction (`and` and `or`)
1.  `if A then B else C` and `match E { V => R, V => R, ... }`

Because arithmetic is infinitely precise, assocativity of most arithmetic binary
operations doesn't matter. In order to reduce errors and avoid having to decide
associativity otherwise, `and` and `or` do not associate with each other; one
must be parenthesized. Similarly `%` does not associate with `\*` or `/`.

If a function has a single argument that is a list `[T]`, then it can also be
called with any number of `T` arguments, and a list is implicitly created.

Value access is written `i.V`; it evaluates to a list of all values `V` on items
`i` that the player possesses. If any of the items that `i` could possibly refer
to (that is, `i` if it is a single item, or all items tagged with `i` if it is a
tag) don't have a value `V`, it's an error. Syntactically, value access is
indistinguisable from a named access.

`match` expressions are used on enums only right now; each arm must be either an
enumerator value or `_` to mean "anything". `_` must come last and must be
present if not all enum values are covered (this can make overriding enums to
add new elements difficult!). The comma between arms is currently mandatory; it
is optional on the last arm and encouraged unless the `}` is on the same line.

### Built-in functions

The following functions are built-in; their names are keywords and cannot be
redeclared:

* `min(...)` and `max(...)` take a list of numeric expressions and return the
  least or greatest value, respectively.
* `count(i)` returns the total count of items `i` possessed by the player at
  evaluation time.
* `sum(list)` returns the sum of a list.

