# Language Description

**This is currently undergoing major revision**

Rado is a declarative language that expresses logical systems. For the most
part, there is no state, input/output, or side effects in Rado itself. The
language does support delcarations that override or modify earlier declarations,
however, in order to facilitate composability and dynamic configuration.

Once a schema is fully loaded and configurations selected, resolving all of the
overrides and modifications, the resulting Rado program represents a logical
system that can be queried programmatically to determine things like whether a
goal is achievable (and how) or what options a player has next. These logical
systems do have a concept of state, which is vital to their interpretation.

## Overview

In Rado, the most basic concepts are *nodes* and *actions*. A node represents
some location (possibly abstract) in the game being described, and an action is
some activity that a player can perform, such as traveling to a different node
or picking up an item.

In a typical randomizer game, the available items are shuffled and placed in
the various locations, and then the player plays through them. To keep track of
this, *items* and *flags* can be defined in order to store the state of the
game.

Not all locations are equal, of course. Some require that the player have
already collected a certain item, have reached a particular event trigger, or
any number of other conditions. In Rado, these are expressed as *requirements*
on the actions. A typical setup would be to have a node with an action to pick
up an item, and have every link to that node require the various prerequisites.

*Regions* are one of the two main kinds of scoping in Rado (the others are modules,
which will be covered later), and can play a direct role in structure. Regions
are generally used to represent areas of the game, and some properties are
inherited inherited by all nodes within the region or its children. Regions
double as general-purpose namespaces.

An action is usually deliberately done by a player, but in some cases they will
happen automatically. These *triggers* can be used, for instance, to describe a
state change that happens when a player enters or leaves a region.

Regions, nodes, actions, and a few other entities like items can have *tags*,
which are simply indicators that make it easy to refer to many similar entities.
They can also have *values*, which store additional data on them. Rado supports
definition of *functions* as well, which can be used to compose and reuse
calculations, requirements, and actions. Note that while functions in Rado are
stateless, they can be used to compose actions which may have effects on
the game state when performed by the player.

The type system is simple, featuring mostly built-in types, but with support for
user-defined enumerations. The other types are mostly primitives, as well as
lists and functions.

*Random parameters* are a similar concept, but represent features chosen
randomly by the randomizer and therefore are a potentially a part of the game
logic. **Random parameters are not yet in the design.**

In order to specify conditionals, such as difficulty settings, some flags are
*configurable*. These flags are not specified at run-time, but instead at
compile time. When all the values of configurable flags are specified (by some
external source), a Rado *schema* is compiled into a single Rado *program*, and
can be queried for information. A valid assignment of random parameters is
called a *model*.

For reusable components that can be mixed in, such as to declare reusable
combinations of flags and code, *templates* can be used. They are somewhat like
macros but fully scoped.

Finally, Rado supports *modules*, which are self-contained parts of a Rado
schema, like libraries in other languages. The can be used for reusable code
like libraries, but also to fuse two schemas together into a larger one, such
as with a combo randomizer. As with conditionals, code can override the contents
of a module it loads, such as to patch the two bases of a combo randomizer
together.

## File Structure

Rado schemas are written in one or more files (usual extension `.rado`), and
one file represents the base file. It specifies which other files to load (which
can in turn specify additional files, etc.) in [module and region
declarations](#module-region-declarations).

A module can have one or more files in it; these files are all compiled together
as part of a single schema (so, for instance, they can have mutual references
between them). Modules, on the other hand, cannot see any modules that they did
not load directly. This ensures that there are no namespace collisions, etc.
Modules can also be declared inline within a single file (mostly for testing
purposes).

The recommended content type for Rado programs is, for the time being,
`application/prs.rado`.

## Scoping

Scoping in Rado is generally quite simple lexical scope. Paths use `.` as a
separator as in `Outer.Inner`; there is no distinction between accessing
properties on entities and namespace scoping. The first name in a path must
always name an entity (never a member/property of the entity), with a few
exceptions, and is looked up in successive scopes outward from where it is used.

Lookup can never travel out of a module or into a conditional block; names
declared in modules are not visible in inner modules, and names declared in
conditional blocks are only visible inside that block. Note that when overriding
a module, this does not apply, since an override is outside the module.

Almost all names are declared in the innermost scope in which they appear. Name
shadowing is not permitted; a declaration cannot use the same name as something
else in the same or an enclosing scope. This is true even for modules; a program
can't import a module and declare somthing of the same name as part of the
module in an outer scope. This is because overrides can do name lookup in the
context of the module, but with the ability to escape it.

## General Syntax

A Rado file is a series of *statements*, which always start with a
keyword indicating the type of statement. A statement can span multiple lines,
and multiple statements can appear on the same line. Semicolons are permitted
between statements, but are not necessary; the starting keyword is enough to
disambiguate. They are encouraged when multiple statements appear on a single
line. Many statements include a *block*, which is a series of statements inside
a pair of braces `{ }`.

Rado's [expression](#expressions) syntax supports basic arithmetic operators
(`+`, `-`, `*`, `/`, and `%`), comparison operators (`<`, `<=`, `>`, `>=`, `==`,
and `!=`), and logical operators (`and`, `or`, `not`), as well as function calls
(`f()`). For conditional evaluation, `if ... then ... else ...` can be used, or
a `match` expression can be used to perform a rudimentary switch-case operation
on an enumeration (`match val { Foo => ..., Bar => ... }`). Expressions can be
grouped in parentheses to disambiguate `( ... )` precedence, which is mostly
like in `C` except that it is an error to try to associate `and` and `or`
without nesting one operator or the other in parentheses.

Comments are in C++ style: `//` for line comments, `/* */` for block comments.
Unlike in C and C++, however, block comments can be nested.

Identifiers are supported per Unicode syntax. All keywords, including built-in
functions, are reserved and cannot be used as an identifier anywhere in the
program. Keywords are written in `lowercase`, and while no style of identifiers is
enforced, `UpperCamelCase` is recommended except for actions and functions
returning actions, for which `lower_camel_case` is recommended.

Numeric literals are written as integers; only decimal literals are currently
supported. `_` may be used as a digit separator. `true` and `false` are the
boolean literals, and string literals are written between quotes `"..."`. Basic
common escape sequences are supported; an error is emitted for any unknown
escape sequence so that more can be added later. String literals are currently
not usable as expressions, but only in human-readable names in declarations.
They are UTF-8 encoded.

## Declarations

A *declaration* is a statement which creates an entity and gives it a name. Most
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
   name. This can be used to make spoiler output more readable, for instance.
*  `...` is the rest of the declaration. The syntax varies depending on the kind
   of declaration and may be disallowed, optional, or mandatory.

The name and human name together are represented by the nonterminal *decl-name*
in the syntax described below.

Optionally, a declaration can be prefixed with a list of *tags*: names
which serve to mark sets of entities. To do so, a comma-separated list of paths
must appear inside `#[ ... ]` immediately in front of the declaration. Unlike
most names, tags are always scoped to an entire module and declared implicitly.
Thus, a multi-segment path such as `Parent.Tag` will only be required to declare
a tag in another module. Tags occupy the same namespace as other names, and thus
their names must be unique within a module. All entities with the same tag must
have the same type.

Some kinds of declarations can be [modified, replaced, fixed, or
deleted](#overrides) in conditional blocks or modules. The specific effects of
overriding any given kind of declaration are described below. Unless
specifically mentioned, a kind of declaration cannot be fixed.

Enum declarations also declare the names of their enumerator values in addition
to their type.

Modules, regions, and nodes all introduce scopes. In addition, the
gobal scope at the root of the program is considered to be a module scope.
Modules can only appear in other modules, and nodes cannot contain other scopes.
Otherwise, any declaration can appear in any scope. Templates can also create
scopes, and contain definitions permitted in the underlying declaration kind.
The scope cannot be referred to from the outside, however, until it is
instantiated and a copy made.

### Module & Region Declarations

> Syntax: *tags* (`module` | `region`) *identifier* (*string-literal*?) (*block* | *string-literal*)

Modules and regions are declared the same way. They either contain a block with
the statements in the module or region, or a filename denoting the file
containing the code for that scope. The filename is always looked up relative to
the directory containing the declaration, or the current working directory if
the statement doesn't originate in a file (for instance, because it's added
programmatically). There is no semantic difference between a module or region
defined inline versus in another file.

Modules and regions can both be modified; this is part of the core of the
override system. Deleting a module or region deletes every declaration inside
it.

### Node Declaration

> Syntax: *tags* `node` *decl-name* (*block*)?

There is very little to say about a node declaration other than that it declares
a node.

### Item Declaration

> Syntax: *tags* `item` *decl-name* (*block*)?

An item declaration introduces a kind of state into the logic system,
representing, more or less, how many of some item or trigger the player has hit.
The optional block in the declaration can be used to declare properties of an
item. Otherwise, by default, the player can accumulate any number of a given
item (though cannot go below 0 or have a fractional number).

Note that the term "item" is a bit misleading and narrow. An item declaration
can be used to represent a wide variety of player state properties, not just
in-game items. For instance, items can represent story progression or other
event triggers, or special states that the player might acquire. In many cases,
however, a flag may be more appropriate. The primary difference between the two
is that a flag must always have some value, while an item is something that a
player can acquire potentially in multiples. They support different
functionality as a result.

Item declarations can be overridden. Items can be fixed to a valid quantity
value; the player can never otherwise gain or lose any of that item. It is an
error for an action to do so, even if the action is never evaluated.

**Item declarations need some work for storing dynamic values and may get rolled
into flags?**

### Flag Declaration

> Syntax: *tags* `flag` *decl-name* (`:` *type*)? (*block*)?

A flag declaration is the other primary kind of state in the logic system. It
must be of scalar type; if none is specified, then `bool` is default. It must
always have a value, but the value can change over time. If not specified, the
default value is `false` for `bool` flags, or `0` for `num` flags, but a default
must be specified for enum flags.

Flag declarations can be overridden. Flags can be fixed; a fixed flag's value
cannot be changed through any means and it is an error for an action to attempt
nt items to share a common capacity. The optional block can
contain property statements.  The default maximum quantity is 1.

### Function Declaration

> Syntax: *tags* `fn` *decl-name* (`(` list(*identifier* (`:` *type*)?) `)`)? (`->` *type*) = *expression*

A function declaration introduces a new function which can be used in
expressions. A function can have an argument list, or it can be omitted.
Likewise, the argument and return types can be omitted; if they are, then they
are inferred.

A function can be used anywhere an expression is legal. If it has no arguments,
then it is called automatically without needing a call expression `()`, so
functions with no arguments can be used like constants.

Functions can be replaced or deleted, but not modified. A replaced function must
have the same signature as the one it replaces.

### Action Declaration

> Syntax: *tags* `action` *decl-name* (*do-call* | *block*)

An action declares an action that the player can perform whenever they are at
the current node or, if declared on a region or module, any node in that scope
(including in its children). The actual content of the action is indicated by a
do call. It must either appear directly in the declaration or, if a block is
used instead (in order to specify properties), inside the block as if it were a
statement.

To declare an action that can be called by other actions, but cannot be
performed on its own, do not use the action declaration. Instead, declare a
function with no arguments that returns an action.

All actions declared with an `action` statement must be named so that they can
be overridden. The *do-expression* defining the action's effect can be replaced
as if it were a statement, but it cannot be deleted.

### Trigger Declarations

> Syntax: `trigger` *decl-name* (`action` | `enter` | `exit`) (*do-call* | *sub-call* | *block*)

A trigger declaration declares an action that is performed automatically
whenever a certain event occurs.

1.  `enter` and `exit` triggers trigger when a player enters or leaves the
    scope, respectively, via a `link` action. The `link` action is considered a
    single event which triggers both `enter` and `exit` triggers simultaneously.
1.  `action` triggers trigger when a player performs an action at a node in the
    scope. This triggers before any triggers based on the content of the action.

As with an action, the action to actually be performed can either be specified
directly or inside the block. Currently, action cannot include a `link` action
either directly or through a called action.

A trigger is performed immediately before at the triggering point in action
execution, interrupting the rest of the action. If a trigger fails, then the
entire action fails; this behaviour can be avoided by using a `sub` action.
Note, however, that any state changes that already happened will not be undone.

In order for the outcome of triggers not to depend on the order of declaration,
it must be possible to evaluate them in a consistent fashion. In order to do so,
any two triggers on the same event may be *ordered* based on the first of the
following rules to apply. For the purposes of the following rules, the content
of an action is evaluated by considering the entire action, expanded through all
calls, conditionals, and other evaluations, but unevaluated operands of
conditionals whose conditions are constant are not considered.

1.  If one or both of the triggers contains an ordering statement
    referring to the other, and they con't contradict, then they are ordered as
    specified.
1.  If one consists only of requirements, and the other does not, then the
    former comes before the latter.
1.  An `enter` trigger comes before an `exit` trigger.

For any possible triggering event, the applicable triggers must meet the
following requirements:

1.  The ordering of applicable triggers must not contain any cycles.
1.  If one action can potentially alter an aspect of state and the other either
    can fail or has behaviour potentially dependent on that state (such as by
    containing a reference to a flag that the first action can set), then those
    actions must be ordered with respect to each other.

If these requirements are not met, then the program is in error. If they are
met, then the execution of triggers will happen following some refinement of the
order on those triggers.

Triggers can be suppressed in an inner scope by noinherit statements, and
ignored on actions by ignore statements. They can be overridden. Triggers must
be named so that they can be suppressed, ignored, or overridden.

### Enum Declaration

> Syntax: `enum` *decl-name* *block*

An enum declaration introduces a new enumeration type. Each statement in the
block must consist only of *decl-name*; each one is the name of a value of the
enumeration. This is an exception to the general syntax of declarations. The
declaration declares both the type name and the names of each value in the
surrounding scope.

Enums cannot be replaced, but can be deleted or modified. When modifying an enum
declaration, new values can be declared and old values deleted. Deleting an
enumerator also deletes all its values. Deleted enum values can still be
referred to in `match` arms; they simply cannot be matched.

### Template Declaration

> Syntax: `template` *decl-name* (*param-list*)? `:` *keyword* *block*

A template declares a reusable series of declarations that can be mixed in to
other blocks. The keyword in the declaration is the keyword that introduces
another kind of declaration, one of the following: `region`, `item`, or `node`.

The block can contain any statement that could occur in the named kind of
declaration. The statements have no meaning in the template itself; they are
given meaning when the template is instantiated.

Optionally, a parameter list can be included with the same syntax as in function
declarations. The parameters are bound within the block based on the arguments
provided in the instantiation.

A template declaration can be overridden, but its kind cannot be changed.

### Instance Declaration

> Syntax: `instance` *decl-name* `:` *name* (`(` list(*expression*) `)`)?

An instance takes a template, along with values for its parameters if it has
any (the semantics are like a function call). However, rather than evaluating to
a value, the instantiation makes copies of all the statements in the template
and places copies in the current scope. The template's declaration kind must
match the current scope, except that a `region` template may be instantiated in
a module.

While the statements in the template are included in the surrounding scope
semantically, including for the purpose of inheritance of actions and
requirements, any declarations are declared in a scope created by the instance.
Thus, if a template declares an action called `Teleport`, and then it is
instantiated in an instantiation named `Doodad`, then it must be referred to as
`Doodad.Teleport`.

Because instantiating a template makes copies of its declarations, instantiating
a template multiple times will declare each entity contained in the template
multiple times. The contents of the instantiations are not shared.

The names within the template retain their original, lexical scope, and are not
reinterpreted in the context of the instantiation. Thus, parameters are the only
way to make a template that expands differently in multiple contexts.

A declaration inside an instance can be overridden from outside it, or by
overriding the declaration on the template. Such overrides are subject to the
same rules about about conflicts that apply to overrides generally.

A template cannot be instantiated recursively.

## Conditional Blocks

> Syntax: *tags*? `if` *expression* *block* (`else` *block*)?

A conditional block makes it so that its contents take effect conditionally. The
expression must be a constant expression of type `bool`. When the schema is
evaluated, the declarations within the main block are evaluated if and only if
the condition is true. If an else block is present, it is evaluated if and only
if the condition is false.

When evaluated, the declarations in the conditional block are semantically
introduced into the surrounding scope. This does not allow name lookup to
penetrate into the conditional block; it is treated as its own anonymous scope
for name lookup.

The tag list for a conditional block may optionally prefix any tag with `not`;
this inverts the tag's meaning. The tag as declared applies to the `if` block,
and the negation applies to the `else` block. Thus, `#[Tag] if A { } else { }`
has the same meaning as `#[not Tag] if not A { } else { }`.

Conditional blocks cannot be overridden.

### Exclusive Statement

> Syntax: `exclusive` (`not`? *ident*)*-list*

Conditional blocks containing conflicting overrides are not permitted to be
simultaneously active (see below). In order to reduce the possibility of errors,
the compiler may give errors when it detects that two conditional blocks have a
potential for conflict, even if it doesn't directly observe them being active
together.

It's generally not viable for the compiler to attempt to reason through every
possibile configuration to determine if two blocks can be active at the same
time or not, so instead, if two blocks cannot be active at the same time, this
must be explicitly declared. The exclusive statement declares that, of the tags
(or their negations) in the list, only one of them will be active at a time. The
compiler will allow conflicts between the conditionals so tagged, and will error
if it ever does detect them simultaneously active (regardless of whether or not
they conflict).

The only exception to the principle of needing explicit declarations of
exclusivity is that a tag and its negation are automatically exclusive.

It's an error for a single block to have multiple exclusive tags declared on it.

Currently, exclusive statements must occur at top level of a module and apply to
the tags declared in that module, or its children. They may not themselves be
conditional. They may not be overridden.

## Overrides

An override is a statement which modifies a previous statement, used inside
conditionals to change behaviour based on configuration, or in modules to change
behaviour of inner modules.

There is, in general, no fixed evaluation order, and all conflicts between
conditional blocks that could possibly depend on their order of evaluation are
forbidden. If the compiler does not catch that the schema contains a potential
conflict, it will error when trying to evaluate the schema in a manner that does
conflict.

Overrides come in three forms:

1.  Replacing, which provide an alternate definition for the statement. Any
    declarations in the original definition which are not in the new one are
    deleted.
1.  Modifying, which modifies some properties or contained entities, or declares
    new ones, without replacing it wholesale.
1.  Deleting, which delete a statement. When a declaration is deleted, it is not
    wholly removed; instead, it is replaced by a placeholder declaration to
    maintain the name binding. Usually it is an error to refer to a deleted
    declaration in any way, although there are a few exceptions. Deleting a
    declaration also deletes all its contents, if applicable.

For clarity and to reduce mistakes, overrides can only be used in contexts where
the original statement could not just be edited by hand. These are where the 

**The syntax and further semantics of overrides will be decided later.**

## Properties

Property statements are used to give properties to declared entities. They are
applied to an entity by placing them in the block in its declaration; if it has
no block, then it cannot have any properties. Each property only applies to
some kinds of declarations as specified below.

### Noinherit Statement

> Can appear in: modules, regions, nodes

> Syntax: `noinherit` *name*

A noinherit statement suppresses inheritance of an action or trigger from a
parent scope. The argument names the entity to suppress; it does not apply
within the current scope. Suppressed triggers can still be referred to for
ordering purposes.

### Ignore Statement

> Can appear in: modules, regions, nodes, actions

> Syntax: `ignore` *name*

An ignore statement causes the action on which it occurs, or all action
declarations in the scope on which it occurs, to ignore the named trigger.

### Ordering Statement

> Can appear in: triggers

> (`before` | `after`) *name*

An ordering statement specifies that the trigger on which it appears comes
before or after another named trigger, which must trigger on the same event, and
is used to clarify the order of triggers. Order statements cannot be
contradictory. 

### Value Statement

**I have no idea what the semantics of `val` should be right now, or even
whether they should actually exist.**

### Max Statement

> Can appear in: items

> Syntax: `max` *expression*

A max statement declares the maximum amount of an item that a player can
possess. Above this limit, more instances of the item cannot be acquired. The
expression must be a constant expression.

**Need to figure out how to handle shared/dynamic capacities (e.g. LttP bottles)
and pickup griggers (e.g. ammo expansions)**

The default, if no maximum is specified, is infinity.

### Default Statement

> Can appear in: items, flags

> Syncax: `default` *expression*

A default statement sets the starting value of a flag or starting quantity of an
item. For a `num` flag or an item, the starting value or quantiy is 0 if no
default statement is used. For a `bool` flag, it is `false`. Enum flags must
have a default specified.

### Fix Statement

> Can appear in: items, flags

> Syntax: `fix` *expression*

A fix statement fixes the value of a flag, or the quantity of an item, to a
constant expression, thus effectively turning the flag or item into a constant.
Its primary purpose is to be used in overrides to replace the flag or item with
a constant value without having to update every use to remove it. It is an error
to create a set action that corresponds to a fixed flag.

### Configurable Statement

> Can appear in: flags

> Syntax: `configurable`

A configurable statement declares that a user or client of the Rado schema can
provide a value for use in evaluating the schema into a program. A configurable
flag can also be fixed, in which case the configurability is ignored. Either
way, like a fix flag, its value cannot be set using a set action.

### Disallow Statement

> Can appear in: configurable enum-typed flags

> Syntax: `disallow` *name*

A disallow statement must name one of the enum values for the flag's type; the
flag cannot take on that value. The flag must then be set to one of the
remaining values. It is an error to disallow all values or a fixed flag's value.

Disallowance is not presently taken into account when determining if an
expression is constant, and thus matching on a flag wth disallowed values will
still consider the corresponding arms to be potentially evaluated.

### Start Statement

> Can appear in: nodes

> Syntax: `start`

A start statement declares a node to be the starting node of the player. Exactly
one node in an entire program must contain a start statement.

## Actions

Actions are the most complex part of Rado, and capture all of the dynamic
actions that a player can perform in the game represented by the program.

An action is a primitive operation understood by the logic engine, or a sequence of
other actions. Prerequisites for actions are expressed in the form of a
*requirement*. A requirement is actually just an action and can be placed
anywhere in a sequence of actions; its effect is to halt the execution of the
entire action if its condition is not met.

### Blocks

> Syntax: (`do` | `sub`) *block*

A block sequences the actions stated in its block, and performs them
sequentially. If one of the actions fails, then the block stops executing, then
the outcome depends on the kind of block. A `do` block fails if one of the inner
actions fails, while a `sub` block will succeed and execution will continue
after the block.

### Calls

> Syntax: (`do` | `sub`) *expression*

A call evaluates the provided expression and then executes the result. As with
blocks, a `do` call fails if the called action does, but `sub` calls never fail.

### Requirement

> Syntax: `require` *expression*

A requirement evaluates the specified expression, which must have type `bool`.
If it is `false`, then the action fails. Otherwise, it continues executing.

### Flag Set

> Syntax: `set` *name* `=` *expression*

A set action sets the value of the specified flag to the specified value. The
flag must not be fixed or configurable, and the expression must match the
value's type.

### Gain Item

> Syntax: `gain` (*num-literal*) *name*

A gain action causes the player to gain or lose a specified item. If a quantity
(which must be an integer) is specified, then then player gains or loses that
quantity instead of only one.

**This is likely to be revised along with other item stuff.**

### Link

> Syntax: `link` *name*

A link action causes the player to move to the specified node.

### Victory

> Syntax: `victory`

A victory action indicates that the player wins the game. It is the objective
representing completion of the game.

### Failure

> Syntax: `failure`

A failure action represents the player failing their objective and losing the
game. No progress can occur past this point. Note that in some games, death does
not necessarily reset progress to a previous save point; `failure` is
inappropriate for these.

### Panic

A call to the builtin `panic()` can be used as an action. It causes a panic.

## Types

Rado has the following types:

* `int`: arbitrary-precision integers
* `num`: arbitrary-precision rational numbers
* `item`: an item
* `bool`: a boolean
* `action`: an action
* `node`: a node
* `inventory`: an inventory
* `fn (A1, A2, ...) -> T`: a function
* lists: `[T]` is a list of `T`s
* enums: for any declared enum `E`, `E` is the type of that enum
* references: `&T` is a reference to a variable (flag or item) of type `T`; `T`
  must be a scalar type or `item`.

Most of these types are quite straightforward. There are no function types
without arguments, as in `fn () -> T`, because functions are stateless. Instead,
functions with no argument just return `T`.

`item` coerces to `bool`; the value is equal to "Does the player have one or
more of the item?" References coerce to the referred type.

`bool`, `num`, and enum types are collectively called *scalar types*.

## Expressions

Expressions are fairly straightforward in Rado. The following are supported, in
order of precedence:

1.  Parenthesized expressions
1.  Literals and values (`foo`, `3`, etc.)
    1.  Value access (`i.Val`)
1.  Referencing (`&flg`)
1.  Action expression (`do A` or `sub { ... }`)
1.  Explicit list creation (`[a, b, c]`)
1.  Function calls (`fn(...)`)
1.  Boolean negation (`not`)
1.  Multiplication, division, integer division, and remainder (`*`, `/`, `//`, and `%`)
1.  Addition and subtraction for numbers (`+` and `-`)
1.  Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`)
1.  Boolean conjunction and disjunction (`and` and `or`)
1.  `if A then B else C` and `match E { V => R, V => R, ... }`

Because arithmetic is infinitely precise, assocativity of most arithmetic binary
operations doesn't matter. In order to reduce errors and avoid having to decide
associativity otherwise, `and` and `or` do not associate with each other; one
must be parenthesized. Similarly `%` does not associate with `*`, `/`, or `//`.

Division of `int`s with `/` returns a `num`. Integer division with `//` returns
the result rounded down (not towards 0 for negative numbers). Integer remainder
with `%` returns the remainder so that `p == (p // q) * q + (p % q)`; it will
always have absolute value less than `q`'s and it will share `q`'s sign (if it
is nonzero). Any attempt to divide by 0 will ccause a panic.

If a declared name is encountered as a value, it represents the value of that
entity, unless a reference is taken. If a tag name is encountered, it represents
a list of all entities or values that have that tag. References cannot be taken
to tags.

Actions are values that can be passed around, but are not executed during
expression evaluation. They're only executed during action evaluation.

If a function has a single argument that is a list `[T]`, then it can also be
called with any number of `T` arguments, and a list is implicitly created.
Function arguments are passed by value; in particular, if a variable is passed
into a function then the value will not change once the function starts
executing, even if the function returns an action which updates the value.

`else` branches are mandatory on `if` expressions unless the type is `action`,
in which case the default is `do {}` i.e. the empty action.

`match` expressions are used on enums only right now; each arm must be either
one or more enumerator values separated by `|`, or `_` to mean "anything". `_`
must come last and must be present if not all enum values are covered (this can
make overriding enums to add new elements difficult!). The comma between arms is
currently mandatory; it is optional on the last arm and encouraged unless the
`}` is on the same line.

### Constant expressions

A constant expression is one whose value can be computed at compile-time,
without circularity. These include literals, the values of configurable flags,
fixed flags or items, and expressions computed from them. Unevaluated operands
may be non-constant without making the expression fail to be constant.

### Built-in functions

* `min([num]) -> num` and `max([num]) -> num` take a list of numeric expressions
  and return the least or greatest value, respectively.
* `count([item]) -> num` returns the total count of items `i` possessed by the
  player at evaluation time.
* `sum([num]) -> num` returns the sum of a list.
* `any([bool]) -> bool` returns `true` if any elements of the list are true.
* `all([bool])` -> bool` returns `true` if all elements of the list are true.
* `map(fn(T) -> U, [T]) -> [U]` maps over a list.
* `capacity(item) -> int` returns how many more of the specified item the player
  can obtain before they are full. Returns 2¹²⁸-1 (the maximum value of an
  unsigned 128-bit integer) if there is no limit.
*  `panic(...)`, see below.

*TODO: Figure out how to handle infinite capacity more elegantly.*

### Panicking

A panic is an error in expression evaluation that cannot be recovered from. They
can occur either because of expressions that cannot be evaluated, such as
division by 0, or by an explicit call to `panic()`.

The built-in function `panic()` is magical. It can be called with no arguments,
or with a message (a string literal). The message is a rudimentary format
string; `%` in the message is used to format an argument into the string. `%%`
escapes a literal `%`. The number of formatting `%` in the message must match
the number of additional arguments. There are no additional specifiers provided.

When a panic occurs, the compilation or evaluation (depending on when it occurs)
aborts with the error message specified, if any. No meaningful results can come
out of a panic, so they represent truly dire situations.
