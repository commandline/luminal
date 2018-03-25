//! Router for mapping `hyper::Method` and a request path to a `luminal_handler::Handler`.
//!
//! Use the feature, "handler", to build this alternative router that is able to work with the
//! luminal-handler create directly.
use futures::future;
use hyper::{self, Method, StatusCode};
use hyper::server::{Request, Response, Service};
use luminal_handler::{Handler, HttpRequest};

use std::collections::HashMap;

mod builder;

use ServiceFuture;
use error::*;
use tree::RouteTree;
use route::Route;
pub use self::builder::{FnRouteBuilder, HandlerRouteBuilder};

/// Router for luminal-handler.
#[derive(Default)]
pub struct Router {
    routes: HashMap<Method, RouteTree<Route<Box<Handler>>>>,
}

impl Service for Router {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ServiceFuture;

    fn call(&self, req: Request) -> Self::Future {
        let route = self.dispatch(req.method(), req.path());
        if let Some(&Some(ref route)) = route {
            match route.target.handle(HttpRequest::Raw(req)) {
                Ok(response) => Box::new(future::ok(response)),
                Err(error) => Box::new(future::ok(
                    Response::new()
                        .with_status(error.status())
                        .with_body(error.body()),
                )),
            }
        } else {
            let mut response = Response::new();
            response.set_status(StatusCode::NotFound);
            Box::new(future::ok(response))
        }
    }
}

impl Router {
    /// Add a handler at the specific route path for the given `Method`.
    pub fn add<H: Handler + 'static>(
        &mut self,
        method: Method,
        route: &str,
        handler: H,
    ) -> Result<()> {
        {
            let routing = self.routes
                .entry(method)
                .or_insert_with(RouteTree::empty_root);
            routing.add(route, Route::new(route, Box::new(handler)))?;
        }
        Ok(())
    }

    pub fn dispatch<'a>(
        &'a self,
        method: &Method,
        route_path: &str,
    ) -> Option<&'a Option<Route<Box<Handler>>>> {
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

    use hyper::{self, Body};
    use hyper::header::ContentLength;
    use futures::Stream;

    use self::tokio_core::reactor::Core;

    use super::*;

    struct StringHandler(String);

    impl Service for StringHandler {
        type Request = Request;
        type Response = Response;
        type Error = hyper::Error;
        type Future = ServiceFuture;
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

    impl Handler for StringHandler {
        fn handle(&self, _req: HttpRequest) -> ::std::result::Result<Response, Response> {
            Ok(Response::new()
                .with_header(ContentLength(self.0.len() as u64))
                .with_body(self.0.clone()))
        }
    }

    fn get_bar_handler(_req: HttpRequest) -> ::std::result::Result<Response, Response> {
        let msg = String::from("Get bar");
        Ok(Response::new()
            .with_header(ContentLength(msg.len() as u64))
            .with_body(msg))
    }

    #[test]
    fn test_router() {
        let router = FnRouteBuilder::new()
            .get("/foo/bar", get_bar_handler)
            .expect("Should have been able to add route")
            .handler_builder()
            .get("/foo/baz", StringHandler::new("Baz"))
            .expect("Should have been able to add route")
            .post("/foo/bar", StringHandler::new("Post bar"))
            .expect("Should have been able to add route")
            .build();

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
