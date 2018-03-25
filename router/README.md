# luminal_router

A router for hyper.

## Why

There are a couple of attempts to build routers for hyper. Most are not under
active development. The one that is, hyper-router, is primitive, relying on
regex matches.

luminal_router uses an internal radix tree for efficient dispatch. The included
benchmarks demonstrate that performance is a linear function of the matching
path. The standard build doesn't introduce any additional traits or types, only
aliases, so any `Service` implementation or function compatible with
`hyper::server::service_fn` will work with luminal_router or bog standard
hyper.

## Help Wanted

The radix tree implementation seems reasonable and no doubt could stand bench
marking and improvement, especially as it picks up the capability to support
path parameters which will directly affect the look up time based on using path
components as the edges in the underlying tree.

## TODO

* [x] Support path parameters
* [ ] Convert message errors to explicit types.
* [x] Add benchmarks.
* [x] Add iterator to RouteTree that consumes path tokens, yields None on first miss
* [ ] Add examples to docs
* [x] Add examples to example crate
* [ ] For the handler feature, add the mapped route into Request Extensions
  * Experiment with a trait on http::Request for type binding
* [ ] Re-visit the associated types that get bound
  * Can these truly be made to with with a generic IntoFuture?
  * Some advanced response handling may need much greater flexibility than binding only to Response for Future::Item.
