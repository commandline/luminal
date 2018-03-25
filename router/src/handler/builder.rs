//! Builders to add implementations of `Handler` and functions for specific methods and routes.
use http;
use hyper::{Body, Method};
use hyper::server::Response;
use luminal_handler::{self, Handler};

use std::collections::HashMap;

use error::*;
use LuminalFuture;
use super::Router;

/// Fluent builder, takes ownership of a `Router` while adding routes.
///
/// Call `build` to move ownership of the route back out.
pub struct HandlerRouteBuilder {
    pub router: Router,
}

impl HandlerRouteBuilder {
    /// Create a new instance with a `Router` with empty routes.
    pub fn new() -> HandlerRouteBuilder {
        HandlerRouteBuilder {
            router: Router {
                routes: HashMap::new(),
            },
        }
    }

    /// Add a service for `Method::Get` at the specified route.
    pub fn get<H: Handler + 'static>(mut self, route: &str, handler: H) -> Result<Self> {
        {
            self.router.add(Method::Get, route, handler)?;
        }
        Ok(self)
    }

    /// Add a `Handler` for `Method::Post` at the specified route.
    pub fn post<H: Handler + 'static>(mut self, route: &str, handler: H) -> Result<Self> {
        {
            self.router.add(Method::Post, route, handler)?;
        }
        Ok(self)
    }

    /// Return a new `FnRouteBuilder` that now owns the router being contructed.
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

    /// Add a `Handler` for `Method::Post` at the specified route.
    pub fn get<F>(mut self, route: &str, function: F) -> Result<Self>
    where
        F: Fn(http::Request<Body>) -> ::std::result::Result<LuminalFuture, Response> + 'static,
    {
        {
            self.router
                .add(Method::Get, route, luminal_handler::handler_fn(function))?;
        }
        Ok(self)
    }

    /// Add a `Handler` for `Method::Post` at the specified route.
    pub fn post<F>(mut self, route: &str, function: F) -> Result<Self>
    where
        F: Fn(http::Request<Body>) -> ::std::result::Result<LuminalFuture, Response> + 'static,
    {
        {
            self.router
                .add(Method::Post, route, luminal_handler::handler_fn(function))?;
        }
        Ok(self)
    }

    /// Return a new `FnRouteBuilder` that now owns the router being contructed.
    pub fn handler_builder(self) -> HandlerRouteBuilder {
        HandlerRouteBuilder {
            router: self.router,
        }
    }

    pub fn build(self) -> Router {
        self.router
    }
}
