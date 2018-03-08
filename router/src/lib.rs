#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;

mod error;
mod types;

use futures::future::{self, Future};
use hyper::{Method, StatusCode};
use hyper::server::{Request, Response, Service};

use std::collections::HashMap;

use error::*;
use self::types::RouteTree;

/// Convenience, especially for `hyper::service::service_fn`.
pub type ServiceFuture = Box<Future<Item = Response, Error = hyper::Error>>;

type LuminalService =
    Service<Request = Request, Response = Response, Error = hyper::Error, Future = ServiceFuture>;

/// Router for Hyper.
#[derive(Default)]
pub struct Router {
    pub routes: HashMap<Method, RouteTree<Box<LuminalService>>>,
}

impl Service for Router {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ServiceFuture;

    fn call(&self, req: Request) -> Self::Future {
        let handler = self.dispatch(req.method(), req.path());
        if let Some(&Some(ref handler)) = handler {
            handler.call(req)
        } else {
            let mut response = Response::new();
            response.set_status(StatusCode::NotFound);
            Box::new(future::ok(response))
        }
    }
}

impl Router {
    /// Add a handler for `Method::Get` at the specified route.
    pub fn get<
        H: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = ServiceFuture,
        >
            + 'static,
    >(
        self,
        route: &str,
        handler: H,
    ) -> Result<Self> {
        self.add(Method::Get, route, handler)
    }

    /// Add a handler for `Method::Post` at the specified route.
    pub fn post<
        H: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = ServiceFuture,
        >
            + 'static,
    >(
        self,
        route: &str,
        handler: H,
    ) -> Result<Self> {
        self.add(Method::Post, route, handler)
    }

    /// Add a handler at the specific route path for the given `Method`.
    pub fn add<
        H: Service<
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
        handler: H,
    ) -> Result<Self> {
        {
            let routing = self.routes
                .entry(method)
                .or_insert_with(RouteTree::empty_root);
            routing.add(route, Box::new(handler))?;
        }
        Ok(self)
    }

    pub fn dispatch<'a>(
        &'a self,
        method: &Method,
        route_path: &str,
    ) -> Option<&'a Option<Box<LuminalService>>> {
        if let Some(routing) = self.routes.get(method) {
            routing.dispatch(route_path)
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
    use hyper::server;
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
            .get("/foo/bar", server::service_fn(get_bar_handler))
            .expect("Should have been able to add route")
            .get("/foo/baz", StringHandler::new("Baz"))
            .expect("Should have been able to add route")
            .post("/foo/bar", StringHandler::new("Post bar"))
            .expect("Should have been able to add route");

        assert_call(&router, Method::Get, "/foo/bar", "Get bar");
        assert_call(&router, Method::Post, "/foo/bar", "Post bar");
        assert_call(&router, Method::Get, "/foo/baz", "Baz");
    }

    #[test]
    fn test_not_found() {
        let router = Router::new();

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
