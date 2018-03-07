#![feature(test)]
extern crate futures;
extern crate hyper;
extern crate test;

extern crate luminal_router;

use futures::future;
use hyper::header::ContentLength;
use hyper::Method;
use hyper::server::{self, Request, Response};
use test::Bencher;

use luminal_router::{Router, ServiceFuture};

fn noop_handler(_req: Request) -> ServiceFuture {
    let msg = String::from("No op");
    Box::new(future::ok(
        Response::new()
            .with_header(ContentLength(msg.len() as u64))
            .with_body(msg),
    ))
}

#[bench]
fn bench_empty(bencher: &mut Bencher) {
    let router = Router::new();

    bencher.iter(|| router.dispatch(&Method::Get, "/"));
}

#[bench]
fn bench_broad(bencher: &mut Bencher) {
    let router = permute_map(1000, 1);

    bencher.iter(|| router.dispatch(&Method::Get, "/0"));
}

#[bench]
fn bench_shallow(bencher: &mut Bencher) {
    let router = permute_map(5, 2);

    let mut to_find = String::from("/");
    for x in 0..2 {
        to_find += &format!("{}/", x);
    }

    bencher.iter(|| router.dispatch(&Method::Get, &to_find));
}

#[bench]
fn bench_deep(bencher: &mut Bencher) {
    let router = permute_map(5, 10);

    let mut to_find = String::from("/");
    for x in 0..10 {
        to_find += &format!("{}/", x);
    }

    bencher.iter(|| router.dispatch(&Method::Get, &to_find));
}

#[bench]
fn bench_deeper(bencher: &mut Bencher) {
    let router = permute_map(5, 100);

    let mut to_find = String::from("/");
    for x in 0..100 {
        to_find += &format!("{}/", x);
    }

    bencher.iter(|| router.dispatch(&Method::Get, &to_find));
}

#[bench]
fn immediate_miss_deep(bencher: &mut Bencher) {
    let router = permute_map(5, 100);

    let mut to_find = String::from("/a/");
    for x in 0..99 {
        to_find += &format!("{}/", x);
    }

    bencher.iter(|| router.dispatch(&Method::Get, &to_find));
}

#[bench]
fn iter_deeper(bencher: &mut Bencher) {
    let router = permute_map(5, 100);

    let mut to_find = String::from("/");
    for x in 0..100 {
        to_find += &format!("{}/", x);
    }

    let tokens: Vec<&str> = to_find.trim_left_matches('/').split('/').collect();

    let routes = router
        .routes
        .get(&Method::Get)
        .expect("Should have been able to get route tree");
    bencher.iter(|| {
        let iter = routes
            .iter(&tokens)
            .expect("Should have been able to get iter");
        iter.last()
    });
}

fn permute_map(breadth: usize, depth: usize) -> Router {
    let mut router = Router::new();
    let mut path_prefix = String::from("/");
    for d in 0..depth {
        for path in 0..breadth {
            router = router
                .get(
                    &format!("{}{}", path_prefix, path),
                    server::service_fn(noop_handler),
                )
                .expect("Failed to add route");
        }
        path_prefix += &format!("{}/", d);
    }
    router
}
