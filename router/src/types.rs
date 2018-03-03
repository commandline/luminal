use std::collections::HashMap;

use error::*;

const PATH_LIMIT: usize = 25;

/// Route mapping as a radix trie.
pub struct Route {
    root: PathComp,
}

impl Route {
    /// Create a new route mapping with an index component and no handler.
    pub fn new() -> Self {
        Route {
            root: PathComp::new("", None),
        }
    }

    /// Add the specified handler at the given route.
    pub fn add(&mut self, route: &str, handler: String) -> Result<()> {
        let tokens = path_to_tokens(route)?;
        if tokens.len() == 1 {
            self.root.handler = Some(handler)
        } else {
            let mut exists = vec![];
            exists.push(&mut self.root);
            let tokens_len = tokens.len();
            let mut created = tokens.iter().enumerate().fold(
                Vec::new(),
                |mut created, (idx, token)| {
                    if idx != 0 {
                        let last = exists.pop().expect("Should always have a last component");
                        if last.next.contains_key(*token) {
                            let next = last.next.get_mut(*token);
                            if let Some(next) = next {
                                if idx + 1 == tokens_len {
                                    next.handler = Some(handler.clone());
                                }
                                exists.push(next);
                            }
                        } else {
                            exists.push(last);
                            if idx + 1 == tokens_len {
                                created.push(PathComp::new(token, Some(handler.clone())));
                            } else {
                                created.push(PathComp::new(token, None));
                            }
                        }
                    }
                    created
                },
            );
            while !created.is_empty() {
                let comp = created.pop();
                if let Some(comp) = comp {
                    if let Some(last) = created.last_mut() {
                        last.next.insert(comp.path.clone(), comp);
                    } else if let Some(last) = exists.pop() {
                        last.next.insert(comp.path.clone(), comp);
                    } else {
                        bail!("Could not fully wire up route {}", route);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn dispatch<'a>(&'a self, request_path: &str) -> Result<&'a Option<String>> {
        // TODO re-factor into iterative
        fn go<'a>(
            idx: usize,
            tokens: &[&str],
            current: &'a HashMap<String, PathComp>,
        ) -> Result<&'a Option<String>> {
            if let Some(current) = current.get(tokens[idx]) {
                if idx + 1 == tokens.len() {
                    Ok(&current.handler)
                } else {
                    go(idx + 1, tokens, &current.next)
                }
            } else {
                bail!("Path not found!")
            }
        }

        let tokens = path_to_tokens(request_path)?;
        go(1, &tokens, &self.root.next)
    }
}

fn path_to_tokens(path: &str) -> Result<Vec<&str>> {
    let path = path.trim_right_matches('/');
    let tokens: Vec<&str> = path.split("/").collect();
    // a short term protection until re-factoring the recursive code to be iterative
    if tokens.len() > PATH_LIMIT {
        bail!("Currently cannot work with a path with more than 25 components")
    }
    if tokens[0] != "" {
        bail!("Paths must start with a slash (/)")
    }
    Ok(tokens)
}

#[derive(Debug, PartialEq)]
pub struct PathComp {
    path: String,
    next: HashMap<String, PathComp>,
    handler: Option<String>,
}

impl PathComp {
    fn new(path: &str, handler: Option<String>) -> PathComp {
        PathComp {
            path: path.to_owned(),
            next: HashMap::new(),
            handler,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_add() {
        let mut expected = PathComp::new("", None);
        let mut foo = PathComp::new("foo", None);
        let bar = PathComp::new("bar", Some(String::from("Bar")));
        foo.next.insert(String::from("bar"), bar);
        expected.next.insert(String::from("foo"), foo);
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_eq!(expected, route.root);
    }

    #[test]
    pub fn test_add_two() {
        let foo = sub_route2("foo", "bar", "baz");
        let mut expected = PathComp::new("", None);
        expected.next.insert(String::from("foo"), foo);
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("BAR"))
            .expect("Should have added route without error");
        route
            .add("/foo/baz", String::from("BAZ"))
            .expect("Should have added route without error");
        assert_eq!(expected, route.root);
    }

    #[test]
    pub fn test_add_extend() {
        let mut expected = PathComp::new("", None);
        let mut foo = PathComp::new("foo", Some(String::from("Foo")));
        foo.next.insert(
            String::from("bar"),
            PathComp::new("bar", Some(String::from("Bar"))),
        );
        expected.next.insert(String::from("foo"), foo);
        let mut route = Route::new();
        route
            .add("/foo/bar/", String::from("Bar"))
            .expect("Should have added route without error");
        route
            .add("/foo/", String::from("Foo"))
            .expect("Should have added route without error");
        assert_eq!(expected, route.root);
    }

    #[test]
    pub fn test_dispatch() {
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        let found = route.dispatch("/foo/bar");
        if let Ok(found) = found {
            assert_eq!(
                Some(String::from("Bar")),
                *found,
                "Could not find handler, {:?}",
                route.root
            );
        } else {
            panic!("Error searching {:?}", found.unwrap_err());
        }
    }

    #[test]
    pub fn test_partial() {
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        let found = route.dispatch("/foo");
        if let Ok(found) = found {
            assert_eq!(
                None, *found,
                "Should not have found handler, {:?}",
                route.root
            );
        } else {
            panic!("Error searching {:?}", found.unwrap_err());
        }
    }

    fn sub_route2(parent: &str, first: &str, second: &str) -> PathComp {
        let mut comp = PathComp::new(parent, None);
        comp.next.insert(
            first.to_owned(),
            PathComp::new(first, Some(String::from(first.to_uppercase()))),
        );
        comp.next.insert(
            second.to_owned(),
            PathComp::new(second, Some(String::from(second.to_uppercase()))),
        );
        comp
    }
}
