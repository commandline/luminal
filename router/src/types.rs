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
            root: PathComp::Path {
                path: "".to_owned(),
                next: vec![],
                handler: None,
            },
        }
    }

    /// Add the specified handler at the given route.
    pub fn add(&mut self, route: &str, handler: String) -> Result<()> {
        // TODO re-factor into iterative
        fn go(idx: usize, tokens: &[&str], current: &mut PathComp, handler: String) {
            let replace_idx = Route::pos_comp(tokens[idx], current);
            let to_update = if idx + 1 == tokens.len() {
                PathComp::path(tokens[idx], Some(handler))
            } else {
                let mut to_update = PathComp::path(tokens[idx], None);
                go(idx + 1, tokens, &mut to_update, handler);
                to_update
            };
            if let Some(replace_idx) = replace_idx {
                let PathComp::Path { ref mut next, .. } = *current;
                next[replace_idx] = to_update;
            } else {
                let PathComp::Path { ref mut next, .. } = *current;
                next.push(to_update);
            }
        }

        // replace the root of the radix trie
        if "/" == route {
            self.root = PathComp::path("", Some(handler));
            return Ok(());
        }
        if !route.starts_with("/") {
            bail!("Route to add must start with a slash (/)")
        }
        let tokens: Vec<&str> = route.split("/").collect();
        // a short term protection until re-factoring the recursive code to be iterative
        if tokens.len() > PATH_LIMIT {
            bail!("Currently cannot add a path with more than 25 components")
        }
        // since the root node replacement is handled above, start with the next level and path
        // component
        go(1, &tokens, &mut self.root, handler);
        Ok(())
    }

    pub fn dispatch(&self, request_path: &str) -> Result<Option<String>> {
        // TODO re-factor into iterative
        fn go(idx: usize, tokens: &[&str], current: &PathComp) -> Result<Option<String>> {
            let PathComp::Path {
                ref path,
                ref handler,
                ..
            } = *current;
            if path == tokens[idx] {
                if idx + 1 == tokens.len() {
                    Ok(handler.clone())
                } else if let Some(current) = Route::find_comp(tokens[idx + 1], current) {
                    go(idx + 1, tokens, current)
                } else {
                    bail!("Handler not found at path!")
                }
            } else {
                bail!("Path not found!")
            }
        }
        let tokens: Vec<&str> = request_path.split("/").collect();
        // a short term protection until re-factoring the recursive code to be iterative
        if tokens.len() > PATH_LIMIT {
            bail!("Currently cannot search for a path with more than 25 components")
        }
        go(0, &tokens, &self.root)
    }

    fn pos_comp(to_find: &str, to_search: &mut PathComp) -> Option<usize> {
        let PathComp::Path { ref next, .. } = *to_search;
        next.iter().position(|comp| {
            let PathComp::Path { ref path, .. } = *comp;
            path == to_find
        })
    }

    fn find_comp<'a>(to_find: &str, to_search: &'a PathComp) -> Option<&'a PathComp> {
        let PathComp::Path { ref next, .. } = *to_search;
        next.iter().find(|comp| {
            let &PathComp::Path { ref path, .. } = *comp;
            path == to_find
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum PathComp {
    Path {
        path: String,
        next: Vec<PathComp>,
        handler: Option<String>,
    },
}

impl PathComp {
    fn path(path: &str, handler: Option<String>) -> PathComp {
        PathComp::Path {
            path: path.to_owned(),
            next: vec![],
            handler,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_add() {
        let mut route = Route::new();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_eq!(
            PathComp::Path {
                path: "".to_owned(),
                handler: None,
                next: vec![
                    PathComp::Path {
                        path: "foo".to_owned(),
                        handler: None,
                        next: vec![
                            PathComp::Path {
                                path: "bar".to_owned(),
                                handler: Some(String::from("Bar")),
                                next: vec![],
                            },
                        ],
                    },
                ],
            },
            route.root
        );
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
                found,
                "Could not find handler, {:?}",
                route.root
            );
        } else {
            panic!("Error searching {:?}", found.unwrap_err());
        }
    }
}
