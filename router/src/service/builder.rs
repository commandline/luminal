//! Builders to add implementations of `Service` and functions for specific methods and reoutes.
use hyper::{self, Method};
use hyper::server::{self, Request, Response, Service};

use std::collections::HashMap;

use LuminalFuture;
use error::*;
use super::Router;

/// Fluent builder, takes ownership of a `Router` while adding routes.
///
/// Call `build` to move ownership of the route back out.
pub struct ServiceRouteBuilder {
    pub router: Router,
}

impl ServiceRouteBuilder {
    /// Create a new instance with a `Router` with empty routes.
    pub fn new() -> ServiceRouteBuilder {
        ServiceRouteBuilder {
            router: Router {
                routes: HashMap::new(),
            },
        }
    }

    /// Add a service for `Method::Get` at the specified route.
    pub fn get<
        S: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = LuminalFuture,
        >
            + 'static,
    >(
        mut self,
        route: &str,
        service: S,
    ) -> Result<Self> {
        {
            self.router.add(Method::Get, route, Box::new(service))?;
        }
        Ok(self)
    }

    /// Add a service for `Method::Post` at the specified route.
    pub fn post<
        S: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = LuminalFuture,
        >
            + 'static,
    >(
        mut self,
        route: &str,
        service: S,
    ) -> Result<Self> {
        {
            self.router.add(Method::Post, route, Box::new(service))?;
        }
        Ok(self)
    }

    pub fn fn_builder(self) -> FnRouteBuilder {
        FnRouteBuilder {
            router: self.router,
        }
    }

    /// Call to gain/regain ownership of the `Router`.
    pub fn build(self) -> Router {
        self.router
    }
}

pub struct FnRouteBuilder {
    pub router: Router,
}

impl FnRouteBuilder {
    pub fn new() -> FnRouteBuilder {
        FnRouteBuilder {
            router: Router {
                routes: HashMap::new(),
            },
        }
    }

    /// Add a service for `Method::Get` at the specified route.
    pub fn get<F: Fn(Request) -> LuminalFuture + 'static>(
        mut self,
        route: &str,
        function: F,
    ) -> Result<Self> {
        {
            self.router
                .add(Method::Get, route, Box::new(server::service_fn(function)))?;
        }
        Ok(self)
    }

    pub fn service_builder(self) -> ServiceRouteBuilder {
        ServiceRouteBuilder {
            router: self.router,
        }
    }

    pub fn build(self) -> Router {
        self.router
    }
}
