#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate luminal_handler;
extern crate luminal_router;

use futures::{Future, Stream};
use hyper::{Body, Response};
use hyper::server::Http;
use luminal_router::{FnRouteBuilder, Router};
use luminal_handler::LuminalFuture;

mod error;

use error::*;

pub fn run() -> Result<()> {
    let addr = "127.0.0.1:3000"
        .parse()
        .chain_err(|| "Could not parse address for binding server socket!")?;
    let server = Http::new().bind(&addr, || Ok(routes().expect("Could not add all routes!")))?;
    server.run()?;
    Ok(())
}

fn routes() -> Result<Router> {
    Ok(FnRouteBuilder::new()
        .get("/echo", get_echo)?
        .post("/echo", post_echo)?
        .build())
}

fn get_echo(req: http::Request<Body>) -> ::std::result::Result<LuminalFuture, Response> {
    let (parts, ..) = req.into_parts();
    let query = parts.uri.query().unwrap_or_else(|| "No query string");
    Ok(Box::new(futures::future::ok(
        Response::new().with_body(query.to_owned()),
    )))
}

fn post_echo(req: http::Request<Body>) -> ::std::result::Result<LuminalFuture, Response> {
    let (.., body) = req.into_parts();
    Ok(Box::new(
        body.concat2().map(|b| Response::new().with_body(b)),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
