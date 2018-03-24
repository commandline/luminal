use hyper;
use hyper::server::{Request, Response, Service};

use {LuminalService, ServiceFuture};

pub struct Route {
    pub route_path: String,
    pub service: Box<LuminalService>,
}

impl Route {
    pub fn new<
        H: Service<
            Request = Request,
            Response = Response,
            Error = hyper::Error,
            Future = ServiceFuture,
        >
            + 'static,
    >(
        route_path: &str,
        service: H,
    ) -> Self {
        Route {
            route_path: route_path.to_owned(),
            service: Box::new(service),
        }
    }
}
