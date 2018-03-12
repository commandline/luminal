#![feature(test)]
extern crate test;
extern crate url;

extern crate luminal_pathparam;

use url::form_urlencoded;

use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::Zip;
use std::str::Split;
use test::Bencher;

use luminal_pathparam::Parse;

#[bench]
fn bench_empty(bencher: &mut Bencher) {
    bencher.iter(|| luminal_pathparam::parse("", ""));
}

#[bench]
fn bench_some(bencher: &mut Bencher) {
    bencher.iter(|| {
        luminal_pathparam::parse("/company/:comp_id/dept/:dept_id", "/company/123/dept/456")
    });
}

#[bench]
fn bench_many(bencher: &mut Bencher) {
    bencher.iter(|| {
        luminal_pathparam::parse(
            "/company/:comp_id/dept/:dept_id/user/:user_id/company/:comp2_id/dept/:dept2_id/user/:user2_id",
            "/company/123/dept/456/user/789/company/123/dept/456/user/789",
        )
    });
}

#[bench]
fn bench_raw(bencher: &mut Bencher) {
    bencher.iter(|| {
        "/company/:comp_id/dept/:dept_id/user/:user_id/company/:comp2_id/dept/:dept2_id/user/:user2_id"
            .split('/')
            .zip("/company/123/dept/456/user/789/company/123/dept/456/user/789".split('/'))
            .filter_map(|(key, value)| {
                if key.starts_with(':') {
                    Some((key.trim_left_matches(':'), value))
                } else {
                    None
                }
            })
    });
}

#[bench]
fn test_into_some(bencher: &mut Bencher) {
    bencher.iter(|| {
        let _test: TestStruct = luminal_pathparam::into(
            "/company/:company/dept/:dept/user/:user",
            "/company/123/dept/456/user/789",
        );
    });
}

#[bench]
fn test_into_owned_some(bencher: &mut Bencher) {
    bencher.iter(|| {
        let _test: TestStructOwned = luminal_pathparam::into(
            "/company/:company/dept/:dept/user/:user",
            "/company/123/dept/456/user/789",
        );
    });
}

#[bench]
fn test_into_many(bencher: &mut Bencher) {
    bencher.iter(|| {
        let _test: TestStruct = luminal_pathparam::into(
            "/company/:company/dept/:dept/user/:user/company/:company2/dept/:dept2/user2/:user",
            "/company/123/dept/456/user/789/company2/123/dept2/456/user2/789",
        );
    });
}

#[bench]
fn test_form_many(bencher: &mut Bencher) {
    bencher.iter(|| {
        let _params: HashMap<Cow<str>, Cow<str>> = form_urlencoded::parse(
            "company=123&dept=456&user=789&company2=123&dept2=456&user2=789".as_bytes(),
        ).collect();
    });
}

#[bench]
fn test_into_owned_many(bencher: &mut Bencher) {
    bencher.iter(|| {
        let _test: TestStructOwned = luminal_pathparam::into(
            "/company/:company/dept/:dept/user/:user/company/:company2/dept/:dept2/user2/:user",
            "/company/123/dept/456/user/789/company2/123/dept2/456/user2/789",
        );
    });
}

struct TestStruct<'a> {
    company: &'a str,
    dept: &'a str,
    user: &'a str,
}

impl<'a> From<Parse<'a>> for TestStruct<'a> {
    fn from(parse: Parse<'a>) -> Self {
        let mut company = "";
        let mut dept = "";
        let mut user = "";
        for (key, value) in parse {
            if key == ":company" {
                company = value;
            }
            if key == ":dept" {
                dept = value;
            }
            if key == ":user" {
                user = value;
            }
        }

        Self {
            company,
            dept,
            user,
        }
    }
}

struct TestStructOwned {
    company: String,
    dept: String,
    user: String,
}

impl<'a> From<Parse<'a>> for TestStructOwned {
    fn from(parse: Parse<'a>) -> Self {
        let mut company = String::from("");
        let mut dept = String::from("");
        let mut user = String::from("");
        for (key, value) in parse {
            if key == ":company" {
                company.push_str(value);
            }
            if key == ":dept" {
                dept.push_str(value);
            }
            if key == ":user" {
                user.push_str(value);
            }
        }

        Self {
            company,
            dept,
            user,
        }
    }
}
