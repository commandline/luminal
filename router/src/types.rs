use std::collections::HashMap;

use error::*;

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
    ///
    /// This method will update the internal trie used to store searchable routes. It will append
    /// any unknown path components in the route and assign the handler to the new, full route.
    pub fn add(&mut self, route: &str, handler: String) -> Result<&mut Self> {
        let tokens = path_to_tokens(route)?;

        // updating the root route handler is a special case that doesn't require any trie
        // traversal
        if tokens.len() == 1 {
            self.root.handler = Some(handler);
            return Ok(self);
        }

        // limit the borrow of self needed to update the internal trie so that this method can
        // return a reference to this struct to support fluent calling
        {
            // a single element stack to track the new last already existing component in the route
            let mut last_existing = vec![];
            last_existing.push(&mut self.root);

            let mut created = tokens
                .iter()
                // start with the first non-root component of the route
                .skip(1)
                .fold(Vec::new(), |mut created, token| {
                    let last = last_existing.pop().expect("Should always have a last component");
                    // follow the existing components as far as possible
                    if last.next.contains_key(*token) {
                        let next = last.next.get_mut(*token);
                        if let Some(next) = next {
                            last_existing.push(next);
                        } else {
                            panic!("Could not update last component of the route!")
                        }
                    // preserve the last existing know component and build up a sequence of new
                    // components to wire together
                    } else {
                        last_existing.push(last);
                        created.push(PathComp::new(token, None));
                    }
                    created
                });

            Route::wire_handler(&mut last_existing, &mut created, route, handler)?;
        }

        Ok(self)
    }

    /// Find the handler for the specific route.
    ///
    /// Traverses the routing trie to find the matching handler, if any, returning `Err` if none is
    /// found.
    pub fn dispatch<'a>(&'a self, request_path: &str) -> Result<&'a Option<String>> {
        let tokens = path_to_tokens(request_path)?;
        let comp = tokens
            .iter()
            // start with the first non-root component of the route
            .skip(1)
            .fold(Some(&self.root), |comp, token| {
                if let Some(comp) = comp {
                    comp.next.get(*token)
                } else {
                    None
                }
            });
        if let Some(comp) = comp {
            Ok(&comp.handler)
        } else {
            bail!("Path not found!")
        }
    }

    // Consume the handler, assigning it to the terminal component of the routing path, adding any
    // new routing path components into the existing trie as needed
    fn wire_handler(
        last_existing: &mut Vec<&mut PathComp>,
        created: &mut Vec<PathComp>,
        route: &str,
        handler: String,
    ) -> Result<()> {
        // the route isn't new, only the handler is
        if created.is_empty() {
            if let Some(last) = last_existing.pop() {
                last.handler = Some(handler);
            }
        // the route is new in part or total and needs to be connected into the existing routing
        // trie
        } else {
            if let Some(mut last) = created.pop() {
                last.handler = Some(handler);
                created.push(last);
            }
            while !created.is_empty() {
                let comp = created.pop();
                if let Some(comp) = comp {
                    if let Some(last) = created.last_mut() {
                        last.next.insert(comp.path.clone(), comp);
                    } else if let Some(last) = last_existing.pop() {
                        last.next.insert(comp.path.clone(), comp);
                    } else {
                        bail!("Could not fully wire up route {}", route);
                    }
                }
            }
        }

        Ok(())
    }
}

// Not only splits an arbitrary string path, ensures that it is well formed for our purposes, that
// means they start with a slash and if then end with a slash, we trim that terminals slash
fn path_to_tokens(path: &str) -> Result<Vec<&str>> {
    let path = path.trim_right_matches('/');
    let tokens: Vec<&str> = path.split('/').collect();
    if tokens[0] != "" {
        bail!("Paths must start with a slash (/)")
    }
    Ok(tokens)
}

/// Node in the internal routing trie.
///
/// Since the radix trie doesn't need to split the path components, use a hash map as an efficient
/// to connect the nodes.
#[derive(Debug, PartialEq)]
struct PathComp {
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
mod tests {
    use super::*;

    // Test adding a single, multiple component path
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

    // Test adding a two routes that have a common ancestor
    #[test]
    pub fn test_add_two() {
        let foo = sub_route2("foo", "bar", "baz");
        let mut expected = PathComp::new("", None);
        expected.next.insert(String::from("foo"), foo);
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("BAR"))
            .expect("Should have added route without error")
            .add("/foo/baz", String::from("BAZ"))
            .expect("Should have added route without error");
        assert_eq!(expected, route.root);
    }

    // Test adding two routes, one that is an extension of another
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

    // Test that we can find an added path
    #[test]
    pub fn test_dispatch() {
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo/bar", "Bar");
    }

    // Test that we can find an added path with a more complex routing trie
    #[test]
    pub fn test_dispatch_two() {
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error")
            .add("/foo/baz", String::from("Baz"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo", "");
        assert_dispatch(&route, "/foo/bar", "Bar");
        assert_dispatch(&route, "/foo/baz", "Baz");
    }

    // Test that we can find an added path with a more complex routing trie
    #[test]
    pub fn test_dispatch_complex() {
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error")
            .add("/foo/baz", String::from("Baz"))
            .expect("Should have added route without error")
            .add("/foo/baz/qux", String::from("Qux"))
            .expect("Should have added route without error")
            .add("/qux/quux/quuux/quuuux/quuuuux", String::from("LongPath"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo", "");
        assert_dispatch(&route, "/foo/bar", "Bar");
        assert_dispatch(&route, "/foo/baz", "Baz");
        assert_dispatch(&route, "/foo/baz/qux", "Qux");
        assert_dispatch(&route, "/qux", "");
        assert_dispatch(&route, "/qux/quux/quuux/quuuux/quuuuux", "LongPath");
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

    fn assert_dispatch(route: &Route, route_path: &str, handler: &str) {
        let found = route.dispatch(route_path);
        if let Ok(found) = found {
            assert_eq!(
                if handler.is_empty() {
                    None
                } else {
                    Some(handler.to_owned())
                },
                *found,
                "Could not find handler, {:?}",
                route.root
            );
        } else {
            panic!("Error searching {:?}", found.unwrap_err());
        }
    }
}
