# luminal
A minimalist, opt-in web framework that runs on hyper.

## Why

Iron is officially no longer maintained. The last feature that seemed to be the
final nail in the coffin was adapting the venerable web framework to the latest
version of hyper that use tokio and futures under the hood.

None of the other web frameworks seems to have quite the same approach as Iron.
hyper can actually be used directly to create stacks of middleware similar to
Iron's approach. hyper doesn't provide much convenience outside of its
`Service` trait.

luminal is an attempt to layer just enough functionality on top of hyper to
take off the rough edges. The design philosophy is for each feature to be
entirely optional. Use as much or as little as you want, falling back to hyper
where you want low level access.

Another important goal of luminal is to make testing as easy as possible. Iron
had a nice test package that made it easy to build up requests and examine
responses. There wasn't a need to run an actual server to test application
code. luminal will strive to support this level of testability.

This repository is a multiple crate project. Each crate can be depended on
separately, supporting the opt in nature of luminal.

## Help Wanted

luminal is brand new. See the TODO list for areas to help. Want a feature? Open
an issue on this repository.

In the meantime, the main driver for luminal's development is the chatelaine
project, a JWT/JWK server that started uses Iron but now needs a comparable
replacement. Outside of feature requests, additions to luminal are likely to
follow chatelaine's needs.

## TODO

Each one of these is likely to be a crate in this project, with its own
respective TODO list.

* [x] Add a router.
* [ ] Add service utilities, an opt in simpler interface for handlers.
* [ ] Add middleware utilities, for wrapping services.
* [ ] Add request parameter parsing.
* [ ] Add body parsing.
* [ ] Add test utilities.
