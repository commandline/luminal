#![feature(test)]
extern crate futures;
extern crate hyper;
extern crate test;
extern crate time;

extern crate luminal_router;

use futures::future;
use hyper::header::ContentLength;
use hyper::Method;
use hyper::server::{Request, Response};
use test::Bencher;
use time::PreciseTime;

use luminal_router::{Router, ServiceFuture};

fn noop_handler(req: Request) -> ServiceFuture {
    // consume the request
    ::std::mem::forget(req);
    let msg = String::from("No op");
    Box::new(future::ok(
        Response::new()
            .with_header(ContentLength(msg.len() as u64))
            .with_body(msg),
    ))
}

#[bench]
fn bench_empty(bencher: &mut Bencher) {
    let router = Router {
        ..Default::default()
    };

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
fn bench_deep_path(bencher: &mut Bencher) {
    let router = permute_map_path(5, 10);

    let mut to_find = String::from("/");
    for x in 0..10 {
        if x % 3 == 0 {
            to_find += &format!(":{}/", x);
        } else {
            to_find += &format!("{}/", x);
        }
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
fn bench_deeper_path(bencher: &mut Bencher) {
    let router = permute_map_path(5, 100);

    let mut to_find = String::from("/");
    for x in 0..10 {
        if x % 3 == 0 {
            to_find += &format!(":{}/", x);
        } else {
            to_find += &format!("{}/", x);
        }
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
fn dispatch_ms(_: &mut Bencher) {
    let router = permute_map(5, 100);

    let mut to_find = String::from("/");
    for x in 0..100 {
        to_find += &format!("{}/", x);
    }

    let n = 1_000_000;
    let start = PreciseTime::now();
    for _ in 0..n {
        router.dispatch(&Method::Get, &to_find);
    }
    let end = PreciseTime::now();
    let runtime = start.to(end).num_milliseconds() as f64;
    println!("Took {:.2} MS to run.", runtime);
    println!("{:.2} dispatches per MS", f64::from(n) / runtime);
}

fn permute_map(breadth: usize, depth: usize) -> Router {
    let mut router = Router {
        ..Default::default()
    };
    let mut path_prefix = String::from("/");
    for d in 0..depth {
        for path in 0..breadth {
            router = router
                .get(&format!("{}{}", path_prefix, path), noop_handler)
                .expect("Failed to add route");
        }
        path_prefix += &format!("{}/", d);
    }
    router
}

fn permute_map_path(breadth: usize, depth: usize) -> Router {
    let mut router = Router {
        ..Default::default()
    };
    let mut path_prefix = String::from("/");
    for d in 0..depth {
        for path in 0..breadth {
            if path % 3 == 0 {
                router = router
                    .get(&format!("{}:{}", path_prefix, path), noop_handler)
                    .expect("Failed to add route");
            } else {
                router = router
                    .get(&format!("{}{}", path_prefix, path), noop_handler)
                    .expect("Failed to add route");
            }
        }
        path_prefix += &format!("{}/", d);
    }
    router
}
