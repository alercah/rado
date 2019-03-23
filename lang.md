# Language Description

Rado is a declarative language that expresses logical systems. For the most
part, there is no state, input/output, or side effects in Rado itself. The
language does support delcarations that override or modify earlier declarations,
however, in order to facilitate composability and dynamic configuration.

Once a schema is fully loaded and configurations selected, resolving all of the
overrides and modifications, the resulting Rado program represents a logical
system that can be queried programmatically to determine things like whether a
goal is achievable (and how) or what options a player has next. These logical
systems do have a concept of state, which is vital to their interpretation.

In a few parts of this document, a more exact formal model is needed to explain
behaviour precissely. Such formalisms are found in [formalisms.md].

## Overview

In Rado, the most basic concepts are *nodes* and *actions*. A node represents
some location (possibly abstract) in the game being described, and an action is
some activity that a player can perform, such as traveling to a different node
or picking up an item.

In a typical randomizer game, the available items are shuffled and placed in
the various locations, and then the player plays through them. To keep track of
this, *variables* can be defined in order to store the state of the game.

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

Many entities can have *tags*, which are simply indicators that make it easy to
refer to many similar entities. They can also have *values*, which store
additional data on them. Rado supports definition of *functions* as well, which
can be used to compose and reuse calculations, requirements, and actions. Note
that while functions in Rado are stateless, they can be used to compose actions
which may have effects on the game state when performed by the player.

The type system is simple, featuring mostly built-in types, but with support for
user-defined enumerations. The other types are mostly primitives, as well as
lists and functions.

In order to specify conditionals, such as difficulty settings, some variables are
*configurable*. These varirables are not specified at run-time, but instead at
compile time. When all the values of configurable variables are specified (by some
external source), a Rado *schema* is compiled into a single Rado *program*, and
can be queried for information. *Randomized variables* represent the parts of
the program that are set or modified by the client working with the program; a
valid assignment of some or all random variables is called a *model*, with an
assignment of all of them called a *complete model*.

For reusable components that can be mixed in, such as to declare reusable
combinations of variables and code, *templates* can be used. They are somewhat
like macros but fully scoped.

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

Scoping in Rado is mostly based on simple nesting, mostly-lexical, scopes. Paths
use `.` as a separator as in `Outer.Inner`. In most cases, lookup for the
leading path segment simply proceeds outwards until a matching name is found,
but never passes outside the module in which the name occurs.

Overrides, which allow a declaration to be changed from a different location,
normally do lookup from the location of the original declaration. The keyword
`outer` can be used at the start of a path in overrides in order cause the rest
of the path to be looked up in the lexical context of the override. The `outer`
keyword applies only to the outermost override, and it is an error if it is used
elsewhere. There is no facility for escaping to the context of any other
override when multiple are nested.

Almost all names are declared in the innermost scope in which they appear. Name
shadowing is generally not permitted; a declaration cannot use the same name as
something else in the same or an enclosing scope. The exception is where lookup
would never notice the difference: names in one module can shadow names in
another, and 

Declarations made in conditional blocks are declared in the same scope as the ,
but cannot be referred to if the conditional block is not active.

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
or use a tag in another module. Tags occupy the same namespace as other names,
and thus their names must be unique within a module. All entities with the same
tag must have the same kind and, where applicable, type. The tag name can in
some circumstances be used to refer to the collection of all entities with a
tag.

Some kinds of declarations can be [modified, replaced, or deleted](#overrides).
The specific effects of overriding any given kind of declaration are described
below.

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

> Syntax: *tags* (`module` | `region`) *identifier* (*string-literal*?) (*block* | `:` *string-literal*)

Modules and regions are declared the same way. They either contain a block with
the statements in the module or region, or a filename denoting the file
containing the code for that scope. The filename is always looked up relative to
the directory containing the declaration, or the current working directory if
the statement doesn't originate in a file (for instance, because it's added
programmatically). There is no semantic difference between a module or region
defined inline versus in another file.

If a module is referred to 

Modules and regions can both be modified; this is part of the core of the
override system. Deleting a module or region deletes every declaration inside
it.

### Node Declaration

> Syntax: *tags* `node` *decl-name* (*block*)?

There is very little to say about a node declaration other than that it declares
a node.

### Variable Declaration

> Syntax: *tags* `var` *decl-name* `:` *type* (*block*)?

A variable holds state in the logic system, and are used for a wide variety of
purposes. The optional block can contain property statements.

Variables come in three kinds, depending on where they are set: dynamic, the
default, which are modifiable by actions and represent changes to state coming
from gameplay; configuration, which are pre-specified when compiling a schema
into a program; and randomizable, which are specified by the client working with
the program to produce models.

Dynamic and configuration variables must be of one of the following types (with
the associated default values for variables without a default specified:

  * `int` (default 0)
  * `num` (default 0)
  * `bool` (default `false`)
  * tuple of any legal type(s) (each field follows its type's default)
  * list of any legal type(s) (default to empty list)
  * enum (no default)

Randomized variables can have any of the above types, plus:

  * `action`
  * `node`
  * function
  * reference

Variable declarations can be overridden.

### Function Declaration

> Syntax: *tags* `fn` *decl-name* (`(` list(*identifier* (`:` *type*)?) `)`)? (`->` *type*) = *expression*

A function declaration introduces a new function which can be used in
expressions. A function can have an argument list, or it can be omitted.
Likewise, the argument and return types can be omitted; if they are, then they
are inferred.

A function can be used anywhere an expression is legal. If it has no arguments,
then it is simply a constant and its body is evaluated when it is declared.

Functions can be replaced or deleted, but not modified. A replaced function must
have the same signature (type) as the one it replaces.

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

Triggers can be explicitly ordered relative to each other with `before` and
`after` statements. When order is not explicitly specified, then triggers that
only impose requirements are ordered first, and then `enter` triggers are
ordered before `exit` triggers. If the result is contradictory or ambiguous,
then it is an error; see the [formalisms](formalisms.md) for more.

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

Enums can be overridden. When modifying an enum declaration, new values can be
declared and old values deleted. Deleting an enumerator also deletes all its
values, and replacing it is the same as declaring new values and deleting those
that don't appear in the replacement. ones. Deleted enum values can still be
referred to in `match` arms; they simply cannot be matched.

### Template Declaration

> Syntax: `template` *decl-name* (*param-list*)? `:` *keyword* *block*

A template declares a reusable series of declarations that can be mixed in to
other blocks. The keyword in the declaration is the keyword that introduces
another kind of declaration, which must be either `region` or `node`, 

The block can contain any statement that could occur in the named kind of
declaration. The statements have no meaning in the template itself; they are
given meaning when the template is instantiated.

Optionally, a parameter list can be included with the same syntax as in function
declarations. The parameters are bound within the block based on the arguments
provided in the instantiation.

A template declaration can be overridden, but its kind cannot be changed. If it
is modified or replaced, this will affect every instance.

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
same rules about about conflicts that apply to overrides generally. The
instance declaration itself `

A template cannot be instantiated recursively, conditionally or unconditionally;
this is checked when the templates are defined.

## Conditional Blocks

> Syntax: *tags*? `if` *expression* *block* (`else` *block*)?

A conditional block makes it so that its contents take effect conditionally. The
expression must be a constant expression of type `bool`. When the schema is
evaluated, the declarations within the main block are evaluated if and only if
the condition is true. If an else block is present, it is evaluated if and only
if the condition is false.

The block itself is a modifying block on the enclosing scope, and is treated as
if the declarations appeared in a modifying override, with the exception that
the `outer` keyword is not applicable because it would do nothing (unless the
conditional block itself is in another override). However, entities newly
declared within a conditional block may only be referred to within conditional
other conditional blocks per the rules in the next section, unless they are
being deleted or replaced (which can happen unconditionally) from outside the
module or instance that contains them. Replacing a declaration in this fashion
makes it unconditional.

Overrides present in conditional blocks may not conflict except in accordance
with the rules in the next section.

Conditional blocks cannot be overridden.

### Relationships Between Conditionals

In general, Rado requires everything to resolve unamgiuously. This means that,
for conditionals, nothing may refer to anything in a conditional unless it is
guaranteed to be active, and two conditionals may not conflict unless they are
guaranteed not to simultaneously be active. Only detecting conflicts when they
occur would increase the possibility of mistakes going undetected, particularly
when there are a large variety of configuration options. On the other hand, it
is infeasible to have the compiler attempt to reason out every possible
permutation when compiling a schema.

A tag on a conditional block is therefore intended to represent some abstract
condition and allow for explicit specifications of how conditionals interact.
For instance, if there are to be no flying enemies in some configurations, a
`FlyingEnemies` tag could be created and applied to all conditional blocks that
work on the assumption that there are flying enemies. Tags for conditional
blocks (only) can be negated: thus the tag `not FlyingEnemies` indicates the
assumption that there are no flying enemies.

The tags present on the conditional block have inverted meaning to the `else`
block if one is present. Thus `#[FlyingEnemies] if A { ... } else { ... }` is
the same as `#[FlyingEnemies] if A { ...} #[not FlyingEnemies] if not A { ...
}`. When conditionals are nested, inner conditionals inherit all the tags of
outer ones.

The rules for tags are as follows:

  * Two conditional blocks may not both be active if they have mutually
    exclusive tags. This is true for `A` and `not A`, but see also the next
    section.
  * If one conditional block has all the tags that a second one does (and
    possibly more), then any time the first block is active, the second one must
    be as well. As an exception for ease of use, this rule does not apply when
    the second block has no tags at all.

The compiler will not detect violations of these rules until they occur in a
specific configuration, unless as single conditional block has multiple exlusive
tags. The rules do, however, allow the tags to be relied on, so that the
following are allowed:

  * Two conditional blocks may contain conflicting overrides if their tags are
    mutually exclusive, or if they are an `if`-`else` pair regardless of tags.
  * A name declared in one conditional block may be referred to from another if
    the second one has all of the tags of the first (plus possibly more), and
    the first one has at least one tag.

### Exclusive Statement

> Syntax: `exclusive` list(`not`? *tag*)

An exclusive statement names (positive or negative) conditional tags that must
be mutually exclusive. This makes it an error for the active blocks of the
program to collectively have two or more of the listed tags, and allows blocks
that have distinct tags from the list to contain distinct tags.

No `exclusive` statement is required for `A` and `not A`, which are always
exclusive.

### Dependency Loops

A conditional block's condition will usually depend on some configuration
variable, which may have its value fixed by another conditional block. It could
also depend on a function whose definition might be replaced. This could lead to
circular conflicts. To resolve this, dependency loops are forbidden.

A conditional block depends on a configuration variable if the variable appears,
directly or indirectly, anywhere in the conditional expression of the block or
any containing conditional block (even in an unevaluated context, because
overrides might make it be evaluated). A conditional block is considered to
modify a variable if it overrides any part of the variable in any way.

See the page on [formalisms](formalisms.md) for a precise specification.

## Overrides

An override is a statement which modifies a previous statement, used inside
conditionals to change behaviour based on configuration, or in modules to change
behaviour of inner modules.

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
the original statement could not just be edited by hand. These are:

1.  When the override is outside the module containing the original.
2.  When the override is in a conditional block, and the original is outside it.
3.  When the original statement is created by an instance of a template, and the
    override is outside the instance.
4.  Overrides can appear in modifying declarations.

Overriding statements cannot themselves be overridden.

### Replacements

> Syntax: `replace` (*declaration* | *property*)

A replacement statement is written in the same form as the original one was,
with the name in a declaration being permitted to be any path that refers to the
declaration to be replaced. The new statement completely replaces the original.
For properties, normally the effect is only to undelete a statement, but some
properties can be modified by way of replacement.

Name lookup in replacing declarations is done as if it were from the location of
the original declaration, unless the `outer` keyword is used.

### Modifications

> Syntax: `modify` *kind* *name* *block*

Modifying a statement is done by specifying the kind and name of the declaration
to modify (as in `modify module Utility { ... }`), and specifying the
modifications inside the block. Within the block, overrides can be specified for
anything in the original, and any non-overriding declarations or statements are
treated as if they appeared in the original context.

Names declared by a modifying declaration must obey shadowing rules and,
additionally, it is an error for them to be referred to from a context that
cannot refer to the scope containing the modifying declaration. For instance,
a name introduced by a modifying declaration in an outer module cannot be
referred to in except if the reference is also in that outer module (or a module
that further encloses it).

Names within the modifying declaration are looked up from the location of the
original declaration, unless the `outer` keyword is used.

### Deletions

> Syntax: `delete` (*kind* *name* | *property*)

Deleting a statement is done by specifying either the kind and name of the
declaration to be deleted (as in, `delete region Alpha`), or specifying a
property statement to delete (as in, `delete before EmergencyTeleport`). A
property statement can only be deleted from a modifying block on the declaration
containing it, and need not match the original exactly as long as it is
semantically equivalent (e.g. they may refer to the same declared entity by
different paths). It is an error if the deleted entity doesn't exist or is
mismatched with the delete statement, although it's not an error to delete it
multiple times.

### Conflicting Overrides

It is possible for multiple overrides to specify mutually-contradictory things
to be done. Specifically, two overrides on the same entity or property conflict
unless they either are both deletions or are both modifications and they do not
declare entities with the same name, modify the properties of the same entity,
or contain conflicting overrides. Overrides on parent entities that affect inner
entities conflict on individual inner entities in the same way.

In general, conflicting overrides are not permitted, with two exceptions. The
first is that mutually exclusive blocks may contain contradictions, since they
can't both be active. The second is where one override occurs in an outer
context and has more knowledge, so should be taken as authoritative.

Thus, an override supersedes another if it is in an outer module, outside an
instance containing the latter, or if neither of those situations apply and it
is more deeply nested in conditionals than the latter. In such a case the
superseding override takes precedence; modifications apply to the result of
applying the superseded override(s). It is an error if a modification tries to
modify a deleted entity this way.

Otherwise, evaluation of overrides is commutative, and it is an error if a
conflict occurs.

## Properties

Property statements are used to give properties to declared entities. They are
applied to an entity by placing them in the block in its declaration; if it has
no block, then it cannot have any properties. Each property only applies to
some kinds of declarations as specified below. Unless otherwise specified,
property statements can be deleted and replaced; for the most part, replacing a
statement is only useful to undelete it.

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

### Default Statement

> Can appear in: variables

> Syncax: `default` *expression*

A default statement sets the default value of a variable. It cannot
appear in randomized variables. If not present in a dynamic or configuration
value, the variable will either have a default based on its type or it will be
an error. The default value must be a constant expression.

Replacing a default statement changes the default value, as there can only be
one such value. A default statement cannot be deleted.

### Fix Statement

> Can appear in: variables

> Syntax: `fix` *expression*

A fix statement fixes the value of a variable, thus effectively turning it into
a constant. Its primary purpose is to be used in overrides to replace the
variable with a constant value without having to update every use to remove it.
The fixed value must be a constant expression.

Replacing a fix statement changes the fixed value, as there can only be one such
value. A variable cannot have both a fixed and default value.

### Config Statement

> Can appear in: variables

> Syntax: `config`

A config statement declares that a variable is a configuration variable,
specified by the user to create parameters in the schema. A configuration
variable can be fixed, in which case the configurability is ignored.

Config statements can neither be added nor removed by overrides.

### Random Statmeent

> Can appear in: variables

> Syntax: `random`

A random statement declares that a variable is a randomizable variable,
specified by the client working with a program to create a model.

Config statements can neither be added nor removed by overrides.

### Disallow Statement

> Can appear in: enum-typed configuration variables

> Syntax: `disallow` *name*

A disallow statement must name one of the enum values for the variable's type;
the variable cannot take on that value. The variable must then be set to one of
the remaining values. It is an error to disallow all values or a fixed
variable's value.

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

A set action sets a specified variable to the specified value. The variable must
be dynamic and unfixed.

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
* `bool`: a boolean
* `action`: an action
* `node`: a node
* `fn (A1, A2, ...) -> T`: a function
* `(T1, T2, T3, ...)`: a tuple
* lists: `[T]` is a list of `T`s
* enums: for any declared enum `E`, `E` is the type of that enum
* references: `&T` is a reference to a dynamic variable of type `T`; `T`
  must be a valid type for dynamic variables.
* `!`: the never (empty) type.

Most of these types are quite straightforward. There are no function types
without arguments, as in `fn () -> T`, because functions are stateless and this
would be meaningless. There are no 1-tuples since they are just regular types,
nor is there a unit type as it is never needed. The `!` type is empty and is the
return type of `panic()`; it indicates a function doesn't return or that a value
can't exist. `!` can be coerced to anything, though such a coercion will never
be executed.

References coerce to the referred type.

## Expressions

Expressions are fairly straightforward in Rado. The following are supported, in
order of precedence:

1.  Parenthesized expressions
1.  Literals and values (`foo`, `3`, etc.)
    1.  Value access (`i.1`)
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

Value access is used with numeric indices, 0-indexed, to access elements of
tuples. E.g. `a.0` is the first element of `a`.

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
without circularity. These include literals, the values of configuration
variables, fixed variables, and expressions computed from them. Unevaluated
operands must also be constant, because overrides could make them be evaluated.

### Built-in functions

The magical `panic()` function, described below, is always available (its name
is a keyword). Many functions in the standard library are built-in, or
implemented in terms of built-ins, and intended to act as the building blocks
for other functions. They act like normal functions otherwise.

### Panicking

A panic is an error in expression evaluation that cannot be recovered from. They
can occur either because of expressions that cannot be evaluated, such as
division by 0, or by an explicit call to the magical built-in `panic()`.

`panic()` can be called with no arguments, or with a message (a string literal)
as the first argument. The message is a rudimentary format string; `%` in the
message is used to format an argument into the string. `%%` escapes a literal
`%`. The number of formatting `%` in the message must match the number of
additional arguments. There are no additional specifiers provided. Panic's
return type is `!`.

When a panic occurs, the compilation or evaluation (depending on when it occurs)
aborts with the error message specified, if any. No meaningful results can come
out of a panic, so they represent truly dire situations.

## Standard Library

The standard library is a module with the name `std` and is predeclared
immediately inside each module's scope, as if by `module std:
"some/path/to/stdlib.rado"`. Declarations in `std` cannot be overridden.
