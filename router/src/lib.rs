//! Router for mapping `hyper::Method` and a request path to something that will response.
//!
//! luminal's router uses a simplified radix tree for speedy lookups. `cargo +nightly bench` to see
//! relative performance across some contrived examples.
//!
//! The actual router implementation depends on the features used to build the crate. By default,
//! the `Router` implementation works with `hyper::server::Service`. Using the "handler" feature
//! switches to a `Router` that is aware of the luminal-handler create.
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
#[cfg(feature = "handler")]
extern crate luminal_handler;

mod error;
mod route;
mod tree;
#[cfg(feature = "handler")]
mod handler;
#[cfg(not(feature = "handler"))]
mod service;

use futures::future::Future;
use hyper::server::Response;
#[cfg(not(feature = "handler"))]
use hyper::server::{Request, Service};

#[cfg(feature = "handler")]
pub use handler::{FnRouteBuilder, HandlerRouteBuilder, Router};
#[cfg(not(feature = "handler"))]
pub use service::{FnRouteBuilder, Router, ServiceRouteBuilder};

pub use error::Error as LuminalError;
pub use error::ErrorKind as LuminalErrorKind;

/// Convenience, especially for `hyper::service::service_fn`.
pub type LuminalFuture = Box<Future<Item = Response, Error = hyper::Error>>;

#[cfg(not(feature = "handler"))]
type LuminalService =
    Service<Request = Request, Response = Response, Error = hyper::Error, Future = ServiceFuture>;
