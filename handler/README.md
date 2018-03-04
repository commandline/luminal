# luminal-handler

A crate to provide a trait to implement and a function to call to lift
non-future aware request handling into hyper.

## TODO

* [ ] Add handler_fn to match hyper's service_fn
* [ ] Improve error handling
  * [ ] Support translating error to hyper::Error
  * [ ] Support status code and response body for error
* [ ] Add tests
