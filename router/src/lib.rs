#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;

mod error;
mod types;

use futures::future::{self, Future};
use hyper::{Method, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Request, Response, Service};

use std::collections::HashMap;

use error::*;
use self::types::Route;

/// Router for Hyper.
pub struct Router {
    routes: HashMap<Method, Route<String>>,
}

impl Service for Router {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let handler = self.dispatch(req.method(), req.path());
        if let Ok(&Some(ref handler)) = handler {
            Box::new(future::ok(
                Response::new()
                    .with_header(ContentLength(handler.len() as u64))
                    .with_body(handler.to_owned()),
            ))
        } else {
            let mut response = Response::new();
            response.set_status(StatusCode::InternalServerError);
            Box::new(future::ok(response))
        }
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    /// Add a handler for `Method::Get` at the specified route.
    pub fn get(&mut self, route: &str, handler: String) -> Result<&mut Self> {
        self.add(Method::Get, route, handler)
    }

    /// Add a handler for `Method::Post` at the specified route.
    pub fn post(&mut self, route: &str, handler: String) -> Result<&mut Self> {
        self.add(Method::Post, route, handler)
    }

    /// Add a handler at the specific route path for the given `Method`.
    pub fn add(&mut self, method: Method, route: &str, handler: String) -> Result<&mut Self> {
        {
            let routing = self.routes.entry(method).or_insert(Route::new());
            routing.add(route, handler)?;
        }
        Ok(self)
    }

    fn dispatch<'a>(&'a self, method: &Method, route_path: &str) -> Result<&'a Option<String>> {
        if let Some(routing) = self.routes.get(method) {
            routing.dispatch(route_path)
        } else {
            bail!("No routes for {}", method)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tokio_core;

    use hyper::Body;
    use futures::Stream;

    use self::tokio_core::reactor::Core;

    use super::*;

    #[test]
    fn test_router() {
        let mut router = Router::new();

        router
            .get("/foo/bar", String::from("Get bar"))
            .expect("Should have been able to add route")
            .get("/foo/baz", String::from("Baz"))
            .expect("Should have been able to add route")
            .post("/foo/bar", String::from("Post bar"))
            .expect("Should have been able to add route");

        assert_call(&router, Method::Get, "/foo/bar", "Get bar");
        assert_call(&router, Method::Post, "/foo/bar", "Post bar");
        assert_call(&router, Method::Get, "/foo/baz", "Baz");
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
