# History

Rado was conceived of by myself (alercah) shortly after the SM/ALttPR combo
randomizer release An earlier design of Rado treated links as first-class
entities, and didn't have a concept of actions. After a surge of interest,
largely from the SM/ALttPR combo randomizer community, and especially @gjgfuj,
it was redesigned around the concept of actions, which made the language more
complex but also resolved a lot of the outstanding design issues.

Earlier versions of Rado had a syntactic sugar of multi-item and multi-location
declarations which allowed declaring multiple items and/or locations with common
properties quickly. As it became clear that significant revision was needed to
the core language, and more cpomplex semantics, it became clear that syntactic
sugar was a problem for later.

The new version includes a lot more in the way of tools for code reuse and
modularization, in particular, the ability to build on top of an existing
project by modifying it. It has a lot of new (to me, anyway) ideas, including
the first-stage compilation model that focuses on a highly declarative style
with overrides, designed to have powerful flexibility and expressiveness without
creating ambiguity or confusion. The focus on commutativity is particularly
important. This component of the langauge, if it proves successful, may be
valuable in other contexts such as configuration languages.

Rado was formerly called Peri; the name was changed after it had developed
negative conntations with something in my personal life (unrelated to the
language).
