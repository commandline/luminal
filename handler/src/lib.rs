#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;

mod error;

use futures::future::{self, Future};
use hyper::StatusCode;
use hyper::server::{Request, Response, Service};

use error::*;

type ServiceFuture = Box<Future<Item = Response, Error = hyper::Error>>;

pub trait Handler {
    fn handle(&self, req: Request) -> Result<Response>;
}

pub struct HandlerService<H>
where
    H: Handler,
{
    handler: H,
}

impl<H> Service for HandlerService<H>
where
    H: Handler,
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ServiceFuture;

    fn call(&self, req: Request) -> Self::Future {
        match self.handler.handle(req) {
            Ok(response) => Box::new(future::ok(response)),
            Err(error) => Box::new(future::ok(
                Response::new()
                    .with_status(StatusCode::InternalServerError)
                    .with_body(format!("{}", error)),
            )),
        }
    }
}

impl<H> HandlerService<H>
where
    H: Handler,
{
    pub fn new(handler: H) -> Self {
        HandlerService { handler }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
