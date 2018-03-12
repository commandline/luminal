//! This create wraps `hyper::server::Service` with a slightly more convenient interface.
//!
//! The request is wrapped with a new enum that helps provide additional information along with the
//! request and uses a more liberal error type to allow users to map their own error to an actual
//! http response.
extern crate futures;
extern crate hyper;
extern crate typemap;

use futures::future::{self, Future};
use hyper::{Body, StatusCode};
use hyper::server::{Request, Response, Service};
use typemap::TypeMap;

use std::marker::PhantomData;

// A convenience alias.
type ServiceFuture = Box<Future<Item = Response, Error = hyper::Error>>;

/// Wraps a `hyper::Request` so that luminal can add additional information alongside the request.
pub enum HttpRequest {
    Raw(Request),
    Context { request: Request, context: TypeMap },
}

/// Trait to implement on errors so that a caller's error type can be converted into a response.
pub trait IntoResponse {
    fn status(&self) -> StatusCode {
        StatusCode::InternalServerError
    }

    fn body(&self) -> Body;
}

/// Trait for handling a request, returning a response or an error that can be converted into a
/// `hyper::Response`.
pub trait Handler<E: IntoResponse> {
    fn handle(&self, req: HttpRequest) -> Result<Response, E>;
}

/// An impl of `hyper::Service` that consumes an impl of `Handler`.
pub struct HandlerService<H, E>
where
    E: IntoResponse,
    H: Handler<E>,
{
    handler: H,
    _phantom: PhantomData<E>,
}

impl<H, E> Service for HandlerService<H, E>
where
    E: IntoResponse,
    H: Handler<E>,
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ServiceFuture;

    fn call(&self, request: Request) -> Self::Future {
        let http_request = HttpRequest::Raw(request);
        match self.handler.handle(http_request) {
            Ok(response) => Box::new(future::ok(response)),
            Err(error) => Box::new(future::ok(
                Response::new()
                    .with_status(error.status())
                    .with_body(error.body()),
            )),
        }
    }
}

impl<H, E> HandlerService<H, E>
where
    E: IntoResponse,
    H: Handler<E>,
{
    pub fn new(handler: H) -> Self {
        HandlerService {
            handler,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tokio_core;

    use futures::Stream;
    use hyper::Method;

    use self::tokio_core::reactor::Core;

    use super::*;

    #[derive(Clone)]
    struct TestError {
        status: StatusCode,
        body: String,
    }

    impl IntoResponse for TestError {
        fn status(&self) -> StatusCode {
            self.status
        }

        fn body(&self) -> Body {
            self.body.clone().into()
        }
    }

    enum TestHandler {
        Success(String),
        Failure(TestError),
    }

    impl Handler<TestError> for TestHandler {
        fn handle(&self, _request: HttpRequest) -> Result<Response, TestError> {
            match *self {
                TestHandler::Success(ref body) => {
                    let body: String = body.clone();
                    Ok(Response::new().with_status(StatusCode::Ok).with_body(body))
                }
                TestHandler::Failure(ref error) => Err(error.clone()),
            }
        }
    }

    #[test]
    fn test_success() {
        let handler = TestHandler::Success(String::from("Success"));
        let service = HandlerService::new(handler);

        assert_call(&service, Method::Get, "/foo", (&StatusCode::Ok, "Success"));
    }

    #[test]
    fn test_failure() {
        let handler = TestHandler::Failure(TestError {
            status: StatusCode::InternalServerError,
            body: String::from("Error"),
        });
        let service = HandlerService::new(handler);

        assert_call(
            &service,
            Method::Get,
            "/foo",
            (&StatusCode::InternalServerError, "Error"),
        );
    }

    fn assert_call<H, E>(
        service: &HandlerService<H, E>,
        method: Method,
        uri: &str,
        expected: (&StatusCode, &str),
    ) where
        E: IntoResponse,
        H: Handler<E>,
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
