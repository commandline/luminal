//! Modeled after `url::form_urlencoded`, parsing a route map and a requested path into a `Parse`
//! instance.
//!
//! Rather than worry about optimizing uses downstream, this create ends with an iterable struct
//! that provides access to the underlying data.
use std::iter::Zip;
use std::str::Split;

/// Accepts a route map and a request path, returns an iterator over the route parameters and their
/// raw values.
//TODO use Cow<str>
pub fn parse<'a>(route: &'a str, path: &'a str) -> Parse<'a> {
    Parse {
        source: route.split('/').zip(path.split('/')),
    }
}

/// A convenience that allows the caller to provide a `From<Parse<'a>>` implementation for their
/// own types.
///
/// This doesn't do anything other than pass the `Parse` instance back to a caller's type. However
/// some bench marks suggest that this approach with manual field assignment is faster than using
/// collect on the `Parse`. This may be the beginning of some more high level features like some
/// macros to help implement `From<Parse<'a>>` on your own traits.
pub fn from<'a, T>(route: &'a str, path: &'a str) -> T
where
    T: From<Parse<'a>>,
{
    T::from(parse(route, path))
}

pub struct Parse<'a> {
    source: Zip<Split<'a, char>, Split<'a, char>>,
}

impl<'a> Iterator for Parse<'a> {
    type Item = (&'a str, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((key, value)) = self.source.next() {
                if key.starts_with(':') {
                    return Some((key, value));
                }
            } else {
                break;
            }
        }
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_one() {
        let params = parse("/user/:user_id", "/user/123").collect::<HashMap<&str, &str>>();
        assert_eq!(Some(&"123"), params.get(":user_id"), "{:?}", params);
    }

    #[test]
    fn test_many() {
        let params = parse("/company/:comp_id/user/:user_id", "/company/123/user/456")
            .collect::<HashMap<&str, &str>>();
        assert_eq!(Some(&"123"), params.get(":comp_id"), "{:?}", params);
        assert_eq!(Some(&"456"), params.get(":user_id"), "{:?}", params);
    }

    #[test]
    fn test_into() {
        let test: TestStruct = into(
            "/company/:company/dept/:dept/user/:user",
            "/company/123/dept/456/user/789",
        );
        assert_eq!("123", test.company);
        assert_eq!("456", test.dept);
        assert_eq!("789", test.user);
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
                println!("{}:{}", key, value);
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
}
