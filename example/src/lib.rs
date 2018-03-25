#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate luminal_handler;
extern crate luminal_router;

use futures::{Future, Stream};
use hyper::{Response, StatusCode};
use hyper::server::Http;
use luminal_router::{FnRouteBuilder, Router};
use luminal_handler::{HttpRequest, LuminalFuture};

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

fn get_echo(req: HttpRequest) -> ::std::result::Result<LuminalFuture, Response> {
    if let HttpRequest::Raw(request) = req {
        if let Some(query) = request.query() {
            let query = query.to_owned();
            Ok(Box::new(futures::future::ok(
                Response::new().with_body(query.to_owned()),
            )))
        } else {
            Ok(Box::new(futures::future::ok(
                Response::new().with_body("Empty body"),
            )))
        }
    } else {
        Err(Response::new().with_status(StatusCode::InternalServerError))
    }
}

fn post_echo(req: HttpRequest) -> ::std::result::Result<LuminalFuture, Response> {
    if let HttpRequest::Raw(request) = req {
        Ok(Box::new(
            request
                .body()
                .concat2()
                .map(|b| Response::new().with_body(b)),
        ))
    } else {
        Err(Response::new().with_status(StatusCode::InternalServerError))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
