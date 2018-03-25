//! This create wraps `hyper::server::Service` with a slightly more convenient interface.
//!
//! The request is wrapped with a new enum that helps provide additional information along with the
//! request and uses a more liberal error type to allow users to map their own error to an actual
//! http response.
extern crate futures;
extern crate hyper;
extern crate typemap;

use futures::future::{self, Future};
use hyper::server::{Request, Response, Service};
use typemap::TypeMap;

// A convenience alias.
pub type LuminalFuture = Box<Future<Item = Response, Error = hyper::Error>>;

/// Wraps a `hyper::Request` so that luminal can add additional information alongside the request.
pub enum HttpRequest {
    Raw(Request),
    Context { request: Request, context: TypeMap },
}

/// Trait for handling a request, returning either a success `Response` or an error `Response`.
pub trait Handler {
    fn handle(&self, req: HttpRequest) -> Result<LuminalFuture, Response>;
}

/// An impl of `hyper::Service` that consumes an impl of `Handler`.
pub struct HandlerService<H: Handler> {
    handler: H,
}

impl<H: Handler> HandlerService<H> {
    pub fn new(handler: H) -> Self {
        HandlerService { handler }
    }
}

impl<H: Handler> Service for HandlerService<H> {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = LuminalFuture;

    /// Dispatches to the owned `Handler`, marshalling success or error into the response.
    fn call(&self, request: Request) -> Self::Future {
        let http_request = HttpRequest::Raw(request);
        match self.handler.handle(http_request) {
            Ok(response) => response,
            Err(error) => Box::new(future::ok(error)),
        }
    }
}

/// Accepts a function or closure that takes an `HttpRequest` and returns a compatible `Result`.
pub fn handler_fn<F>(func: F) -> HandlerFn<F>
where
    F: Fn(HttpRequest) -> Result<LuminalFuture, Response>,
{
    HandlerFn { func }
}

/// Holds a function to dispatch to via its impl of `Handler<E>`.
pub struct HandlerFn<F>
where
    F: Fn(HttpRequest) -> Result<LuminalFuture, Response>,
{
    func: F,
}

impl<F> Handler for HandlerFn<F>
where
    F: Fn(HttpRequest) -> Result<LuminalFuture, Response>,
{
    fn handle(&self, req: HttpRequest) -> Result<LuminalFuture, Response> {
        (self.func)(req)
    }
}

#[cfg(test)]
mod tests {
    extern crate tokio_core;

    use futures::Stream;
    use hyper::Method;
    use hyper::{Body, StatusCode};

    use self::tokio_core::reactor::Core;

    use super::*;

    enum TestHandler {
        Success(String),
        Failure(String),
    }

    impl Handler for TestHandler {
        fn handle(&self, _request: HttpRequest) -> Result<Response, Response> {
            match *self {
                TestHandler::Success(ref body) => {
                    let body: String = body.clone();
                    Ok(Response::new().with_status(StatusCode::Ok).with_body(body))
                }
                TestHandler::Failure(ref error) => {
                    let body: String = error.clone();
                    Err(Response::new()
                        .with_status(StatusCode::InternalServerError)
                        .with_body(body))
                }
            }
        }
    }

    fn test_fn(_req: HttpRequest) -> Result<Response, Response> {
        Ok(Response::new()
            .with_status(StatusCode::Ok)
            .with_body(String::from("test")))
    }

    #[test]
    fn test_success() {
        let handler = TestHandler::Success(String::from("Success"));
        let service = HandlerService::new(handler);

        assert_call(&service, Method::Get, "/foo", (&StatusCode::Ok, "Success"));
    }

    #[test]
    fn test_failure() {
        let handler = TestHandler::Failure(String::from("Error"));
        let service = HandlerService::new(handler);

        assert_call(
            &service,
            Method::Get,
            "/foo",
            (&StatusCode::InternalServerError, "Error"),
        );
    }

    #[test]
    fn test_handler_fn() {
        let handler = handler_fn(test_fn);
        let service = HandlerService::new(handler);

        assert_call(&service, Method::Get, "/foo", (&StatusCode::Ok, "test"));
    }

    fn assert_call<H>(
        service: &HandlerService<H>,
        method: Method,
        uri: &str,
        expected: (&StatusCode, &str),
    ) where
        H: Handler,
    {
        let uri = uri.parse()
            .expect("Should have been able to convert to uri");
        let req: Request<Body> = Request::new(method, uri);

        let work = service.call(req);

        let mut core = Core::new().expect("Should have been able to create core");

        let response = core.run(work)
            .expect("Should have been able to run router call");

        assert_eq!(
            *expected.0,
            response.status(),
            "Should have received {} status.",
            expected.0
        );

        let body = core.run(response.body().concat2())
            .expect("Should have been able to resolve body concat");
        let body: &[u8] = &body.to_vec();

        assert_eq!(
            expected.1.as_bytes(),
            body,
            "Should have received correct body content"
        );
    }
}
