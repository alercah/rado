# Formalisms

This page describes formal models for Rado, where formalisms are required to
explain parts of the language or implementation, or to prove properties of them.

## Dependency Loops

Some entities *E* have *governing expressions*:

  * The conditional expressions of conditional blocks.
  * The definitions of functions.
  * The expression to which a configuration variable is fixed or defaulted.
  * The arguments to an instance.

An entity *E* is *contained within* another entity *F* if *F* declares a scope
and *E* is contained, directly or indirectly, within that scope.

An entity *E* is *subject to* another entity *F* if either of the following are
true:

   * *E* is contained within *F*.
   * The declaration of *E* is lexically situated inside the declaration of some
     entity subject to *F*

A conditional block *B* *modifies* an entity *E* if it contains an override, not
contained within some nested conditional block, which does either of the following:

  * replaces, modifies, or deletes *E*, including in a template or instance
  * replaces or deletes an entity to which *E* is subject

An entity *E* *depends on* another entity *F* if any of the following are true:

  * *E* has a governing expression containing a reference to *F*
  * *E* has a governing expression containing a reference to an entity which
    depends on *F*
  * *E* is subject to an entity which depends on *F*
  * *E* depends on an entity modified by *F*

It is an error if an entity, other than a function, depends on itself.

It is also an error if a template contains an instantiation of itself.

## Trigger Commutativity

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
    containing a reference to a variable that the first action can set), then those
    actions must be ordered with respect to each other.

If these requirements are not met, then the program is in error. If they are
met, then the execution of triggers will happen following some refinement of the
order on those triggers.
