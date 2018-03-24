//! Router for mapping `hyper::Method` and a request path to a `hyper::Service`.
//!
//! luminal's router uses a simplified radix tree for speedy lookups. `cargo +nightly bench` to see
//! relative performance across some contrived examples.
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;

mod error;
mod route;
mod tree;

use futures::future::{self, Future};
use hyper::{Method, StatusCode};
use hyper::server::{self, Request, Response, Service};

use std::collections::HashMap;

use error::*;
use self::tree::RouteTree;
use self::route::Route;

/// Convenience, especially for `hyper::service::service_fn`.
pub type ServiceFuture = Box<Future<Item = Response, Error = hyper::Error>>;

type LuminalService =
    Service<Request = Request, Response = Response, Error = hyper::Error, Future = ServiceFuture>;

/// Router for Hyper.
#[derive(Default)]
pub struct Router {
    pub routes: HashMap<Method, RouteTree<Route>>,
}

impl Service for Router {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ServiceFuture;

    fn call(&self, req: Request) -> Self::Future {
        let route = self.dispatch(req.method(), req.path());
        if let Some(&Some(ref route)) = route {
            route.service.call(req)
        } else {
            let mut response = Response::new();
            response.set_status(StatusCode::NotFound);
            Box::new(future::ok(response))
        }
    }
}

pub trait IntoService: Send + Sync + 'static {
    fn into_service(
        self,
    ) -> Box<
        Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = Box<Future<Item = Response, Error = hyper::Error>>,
        >,
    >;
}

impl<F> IntoService for F
where
    F: Send + Sync + 'static + Fn(Request) -> Box<Future<Item = Response, Error = hyper::Error>>,
{
    fn into_service(
        self,
    ) -> Box<
        Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = Box<Future<Item = Response, Error = hyper::Error>>,
        >,
    > {
        Box::new(server::service_fn(self))
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            ..Default::default()
        }
    }

    /// Add a service for `Method::Get` at the specified route.
    pub fn get_svc<
        S: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = ServiceFuture,
        >
            + 'static,
    >(
        self,
        route: &str,
        service: S,
    ) -> Result<Self> {
        self.add(Method::Get, route, service)
    }

    /// Add a `Service` for `Method::Get` at the specific route by coercing `I` into a `Service`.
    pub fn get<I>(self, route: &str, source: I) -> Result<Self>
    where
        I: Send + Sync + 'static + IntoService,
    {
        self.add(Method::Get, route, source.into_service())
    }

    /// Add a handler for `Method::Post` at the specified route.
    pub fn post_svc<
        S: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = ServiceFuture,
        >
            + 'static,
    >(
        self,
        route: &str,
        service: S,
    ) -> Result<Self> {
        self.add(Method::Post, route, service)
    }

    /// Add a handler at the specific route path for the given `Method`.
    pub fn add<
        S: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = ServiceFuture,
        >
            + 'static,
    >(
        mut self,
        method: Method,
        route: &str,
        service: S,
    ) -> Result<Self> {
        {
            let routing = self.routes
                .entry(method)
                .or_insert_with(RouteTree::empty_root);
            routing.add(route, Route::new(route, service))?;
        }
        Ok(self)
    }

    pub fn dispatch<'a>(&'a self, method: &Method, route_path: &str) -> Option<&'a Option<Route>> {
        if let Some(routing) = self.routes.get(method) {
            if let Some(route) = routing.dispatch(route_path) {
                Some(route)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tokio_core;

    use hyper::Body;
    use hyper::header::ContentLength;
    use futures::Stream;

    use self::tokio_core::reactor::Core;

    use super::*;

    struct StringHandler(String);

    impl Service for StringHandler {
        type Request = Request;
        type Response = Response;
        type Error = hyper::Error;
        type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
        fn call(&self, _req: Request) -> Self::Future {
            Box::new(future::ok(
                Response::new()
                    .with_header(ContentLength(self.0.len() as u64))
                    .with_body(self.0.clone()),
            ))
        }
    }

    impl StringHandler {
        fn new(msg: &str) -> Self {
            StringHandler(msg.to_owned())
        }
    }

    fn get_bar_handler(_req: Request) -> ServiceFuture {
        let msg = String::from("Get bar");
        Box::new(future::ok(
            Response::new()
                .with_header(ContentLength(msg.len() as u64))
                .with_body(msg),
        ))
    }

    #[test]
    fn test_router() {
        let router = Router::new()
            .get("/foo/bar", get_bar_handler)
            .expect("Should have been able to add route")
            .get_svc("/foo/baz", StringHandler::new("Baz"))
            .expect("Should have been able to add route")
            .post_svc("/foo/bar", StringHandler::new("Post bar"))
            .expect("Should have been able to add route");

        assert_call(&router, Method::Get, "/foo/bar", "Get bar");
        assert_call(&router, Method::Post, "/foo/bar", "Post bar");
        assert_call(&router, Method::Get, "/foo/baz", "Baz");
    }

    #[test]
    fn test_not_found() {
        let router = Router {
            ..Default::default()
        };

        let uri = "/foo"
            .parse()
            .expect("Should have been able to convert to uri");
        let req: Request<Body> = Request::new(Method::Get, uri);

        let work = router.call(req);

        let mut core = Core::new().expect("Should have been able to create core");

        let response = core.run(work)
            .expect("Should have been able to run router call");

        assert_eq!(
            StatusCode::NotFound,
            response.status(),
            "Should have received not found status."
        );
    }

    fn assert_call(router: &Router, method: Method, uri: &str, expected: &str) {
        let uri = uri.parse()
            .expect("Should have been able to convert to uri");
        let req: Request<Body> = Request::new(method, uri);

        let work = router.call(req);

        let mut core = Core::new().expect("Should have been able to create core");

        let response = core.run(work)
            .expect("Should have been able to run router call");

        assert_eq!(
            StatusCode::Ok,
            response.status(),
            "Should have received Ok status."
        );

        let body = core.run(response.body().concat2())
            .expect("Should have been able to resolve body concat");
        let body: &[u8] = &body.to_vec();

        assert_eq!(
            expected.as_bytes(),
            body,
            "Should have received correct body content"
        );
    }
}
