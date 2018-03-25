# luminal
A minimalist, opt-in web framework that runs on [hyper](https://hyper.rs/).

## Why

Even though it is [early days for web development with
Rust](http://www.arewewebyet.org/), one of the early contenders,
[Iron](http://ironframework.io/), is officially no longer maintained. The last
feature that seemed to be the final nail in the coffin was adapting the
venerable web framework to the latest version of hyper that use
[tokio](https://tokio.rs/) and
[futures](https://github.com/rust-lang-nursery/futures-rs) under the hood.

None of the other web frameworks seems to have quite the same approach as Iron.
hyper can actually be used directly to create stacks of middleware similar to
Iron's approach. hyper doesn't provide much convenience outside of its
`Service` trait.

luminal is an attempt to layer just enough functionality on top of hyper to
take off the rough edges. The design philosophy is for each feature to be
entirely optional. Use as much or as little as you want, falling back to hyper
where you want low level access.

Why so minimal? For speed and scale. By creating each piece as an optional and
independent crate, they can each be designed, tested and bench marked to keep
any added cost as small as possible. There are several high level web
frameworks in Rust, where convenience and developer support are higher
priorities.  luminal is being written to support a low level infrastructure
project. While it may have some conveniences, it needs to result in web
applications that are fast and scale well, first and foremost.

Another important goal of luminal is to make testing as easy as possible. Iron
had a nice test package that made it easy to build up requests and examine
responses. There wasn't a need to run an actual server to test application
code. luminal will strive to support this level of testability.

This repository is a multiple crate project. Each crate can be depended on
separately, supporting the opt in nature of luminal.

## Help Wanted

luminal is brand new. See the TODO list for areas to help. Want a feature? Open
an issue on this repository.

In the meantime, the main driver for luminal's development is the [chatelaine
project](https://github.com/commandline/chatelaine/), a JWT/JWK server that
started uses Iron but now needs a comparable replacement. Outside of feature
requests, additions to luminal are likely to follow chatelaine's needs.

## TODO

Each one of these is likely to be a crate in this project, with its own
respective TODO list.

* [x] Add a router (see [luminal-router](router/)).
* [x] Add service utilities (see [luminal-handler](handler/)), an opt in
  simpler interface for handlers.
* [ ] Add middleware utilities, for wrapping handlers.
* [ ] Add request parameter parsing.
* [ ] Add body parsing.
* [ ] Look into syn, quote for deriving data handling
  * A user could derive this trait for their service/handler
  * Would consumer the request, offering a strongly typed view into the request data
  * Could this replace Request in the Handler trait?
* [ ] Add test utilities.
