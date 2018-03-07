# luminal_router

A router for hyper.

## Why

There are a couple of attempts to build routers for hyper. Most are not under
active development. The one that is, hyper-router, is primitive, relying on
regex matches.

luminal_router uses an internal radix tree for efficient dispatch. The included
benchmarks demonstrate that performance is a linear function of the matching
path. It doesn't introduce any additional traits or types, only aliases, so any
`Service` implementation or function compatible with
`hyper::server::service_fn` will work with luminal_router or bog standard
hyper.

## Help Wanted

The radix tree implementation seems reasonable and no doubt could stand bench
marking and improvement, especially as it picks up the capability to support
path parameters which will directly affect the look up time based on using path
components as the edges in the underlying tree.

## TODO

* [ ] Support path parameters
* [ ] Convert message errors to explicit types.
* [x] Add benchmarks.
* [x] Add iterator to RouteTree that consumes path tokens, yields None on first miss
  * [ ] Use iterator to find last existing in add fn
  * [ ] Use iterator to find handler in dispatch so misses can short circuit faster
