This doc has some extra musings about the future of links, since I know the
first design is not suitable for all the design goals.

Rather than the current design of links, which represent a connection, instead
the actual entity represents a way to leave a region, such as a door out. A link
may have a destination, which is often itself another link. A link with no destination
is usable only as a destination. Alternatively, if it simply connects to another
region, that indicates, effectively, an anonymous destination-only link. Links
can be tagged; the same tag cannot be used for both items and links.

Links can be grouped in "multi-port" (name likely to be changed) links, which
are expected to be randomized together. For instance, in ALTTPR, entrance
randomizer can be configured so that multi-entrance caves are randomized keeping
both entrances corresponding. In these cases, these would represent multi-port
links. Since this is only a part of randomization logic, we may not need
first-class support; it may be doable using tags. This could be worked out later
when we add better randomization logic support.

A traversal is a new type of entity which exists in a region and represents the
requirements to get from point A to point B. 

Longer-term idea: come up with an idea of a 'node' which subsumes the above.
