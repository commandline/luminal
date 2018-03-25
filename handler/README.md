# luminal-handler

A crate to provide a trait to implement and a function to call to lift
non-future aware request handling into hyper.

## Why

`hyper::server::Service` isn't a super forgiving API. It exposes the plumbing of futures pretty directly and makes error handling unclear. It is hoped that this create provides an easier API without sacrificing much, if any performance. In particular, the trait `IntoResponse` is introduced to help caller's use their own error kinds, layering in what is needed to convert those errors into valid `hyper::server::Response` instances.

## TODO

* [x] Add handler_fn to match hyper's service_fn
* [x] Improve error handling
  * [x] Support status code and response body for error
* [x] Add tests
* [x] Figure out how to pass additional information with requests cleanly
* [ ] Add examples to docs
* [x] Add examples to example crate
* [ ] Add macros to make working with responses easier
* [ ] Remove the Result with a Future and a Response
  * The idea was better suited when the Result held two Responses
  * Streaming/async response writing needs the Future
  * Make a bail or error macro to build a Future with a client error response
