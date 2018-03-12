use std::collections::BTreeMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::Split;

use error::*;

/// Route mapping as a radix tree.
pub struct RouteTree<T> {
    root: PathNode<T>,
}

impl<T> RouteTree<T> {
    /// Create a new route mapping with an index node with no handler.
    pub fn empty_root() -> Self {
        RouteTree {
            root: PathNode::new("/", "", None),
        }
    }

    /// Add the specified handler at the given route.
    ///
    /// This method will update the internal tree used to store searchable routes. It will append
    /// any unknown path components in the route and assign the handler to the new, full route.
    pub fn add(&mut self, route: &str, handler: T) -> Result<&mut Self> {
        let path = route.trim_right_matches('/');
        let tokens: Vec<&str> = path.split('/').collect();
        if tokens[0] != "" {
            bail!("Paths must start with a slash (/)")
        }

        // updating the root route handler is a special case that doesn't require any trie
        // traversal
        if tokens.len() == 1 {
            self.root.handler = Some(handler);
            return Ok(self);
        }

        // limit the borrow of self needed to update the internal tree so that this method can
        // return a reference to this struct to support fluent calling
        {
            // a single element stack to track the new last already existing component in the route
            let mut last_existing = vec![];
            last_existing.push(&mut self.root);

            let mut path = String::new();

            // unlike dispatch, adding a route needs a mutable reference to the last match in the
            // existing tree; implementing a mutable iterator is non-trivial and isn't warranted
            // here since adding routes is likely to not be anywhere as performance sensitive as
            // dispatching
            let mut created = tokens
                .iter()
                // start with the first non-root component of the route
                .skip(1)
                .fold(Vec::new(), |mut created, token| {
                    path.push_str(&format!("/{}", token));
                    let last = last_existing.pop().expect("Should always have a last component");
                    if token.starts_with(':') {
                        // this is a guard because if it was an if..else then the borrow from
                        // last.params would live for the expression, both branches, not only the
                        // one where the dereferenced option contains Some
                        if last.params.deref_mut().is_none() {
                            last_existing.push(last);
                            created.push(PathNode::new(&path, "*", None));
                            return created;
                        }
                        let next = last.params.deref_mut().as_mut().unwrap();
                        last_existing.push(next);
                    // follow the existing components as far as possible
                    } else if last.next.contains_key(*token) {
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
                        created.push(PathNode::new(&path, token, None));
                    }
                    created
                });

            RouteTree::wire_handler(&mut last_existing, &mut created, route, handler)?;
        }

        Ok(self)
    }

    /// Find the handler for the specific route.
    ///
    /// Traverses the routing tree to find the matching handler. The outer `Option` is `None` if
    /// thethe requested path is not found at all. The inner `Option` reference will be None if the
    /// route is found but no handler is assigned. The handler will not be assigned in the
    /// requested path is only a partial match for a longer path added to the route tree.
    pub fn dispatch<'a>(&'a self, request_path: &str) -> Option<(&'a str, &'a Option<T>)> {
        let path = request_path.trim_left_matches('/');
        if path == "" {
            return Some((&self.root.path, &self.root.handler));
        }
        let mut tokens = path.split('/');
        let iter = self.iter(&mut tokens);
        if let Some(found) = iter.last() {
            Some((&found.path, &found.handler))
        } else {
            None
        }
    }

    // Consume the handler, assigning it to the terminal component of the routing path, adding any
    // new routing path components into the existing trie as needed
    fn wire_handler(
        last_existing: &mut Vec<&mut PathNode<T>>,
        created: &mut Vec<PathNode<T>>,
        route: &str,
        handler: T,
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
                let node = created.pop();
                if let Some(node) = node {
                    if let Some(last) = created.last_mut() {
                        if node.segment == "*" {
                            last.params = Box::new(Some(node));
                        } else {
                            last.next.insert(node.segment.clone(), node);
                        }
                    } else if let Some(last) = last_existing.pop() {
                        if node.segment == "*" {
                            last.params = Box::new(Some(node));
                        } else {
                            last.next.insert(node.segment.clone(), node);
                        }
                    } else {
                        bail!("Could not fully wire up route {}", route);
                    }
                } else {
                    bail!("Created stack ran dry too soon!");
                }
            }
        }

        Ok(())
    }

    fn iter<'a, 'b>(&'a self, tokens: &'b mut Split<'b, char>) -> Iter<'a, 'b, T> {
        Iter {
            tokens,
            previous: &self.root,
        }
    }
}

// An internal struct used for fast traversal during dispatching.
struct Iter<'a, 'b, T: 'a> {
    // Working directly with the `Split` is more efficient than collecting it into a `Vec`, based
    // on benchmarking both approaches.
    tokens: &'b mut Split<'b, char>,
    previous: &'a PathNode<T>,
}

// An impl that uses references to traversal the routing tree as fast and as cheaply as possible.
impl<'a, 'b, T> Iterator for Iter<'a, 'b, T> {
    type Item = &'a PathNode<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.tokens.next() {
            if let Some(next) = self.previous.next.get(token) {
                self.previous = next;
                Some(next)
            } else if let Some(next) = self.previous.params.deref().as_ref() {
                self.previous = next;
                Some(next)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Node in the internal routing tree.
//
// Since the radix tree doesn't need to split the path components, use a hash map as an efficient
// to connect the nodes. The params field handles path parameter links, allowing one handler for
// routes ending with a parameter and more routes to be added with additional path parameters
// beyond this node.
#[derive(Debug, PartialEq)]
struct PathNode<T> {
    // The path as originally mapped, will include the names of any path parameters.
    path: String,
    // The specific segment within the original path for this node, will be "*" for a path
    // parameter as a convenience.
    segment: String,
    // Edges will be any path segments after this one.
    next: BTreeMap<String, PathNode<T>>,
    // A node representing a handler for a path parameter may also have connected edges to further
    // nodes.
    params: Box<Option<PathNode<T>>>,
    // An optional handler.
    handler: Option<T>,
}

impl<T> PathNode<T> {
    fn new(path: &str, segment: &str, handler: Option<T>) -> PathNode<T> {
        PathNode {
            path: path.to_owned(),
            segment: segment.to_owned(),
            next: BTreeMap::new(),
            params: Box::new(None),
            handler,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test a path parameter
    #[test]
    pub fn test_path_param() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/:foo", String::from("Foo"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo/123", "Foo");
    }

    // Test a path parameter in the middle of a route
    #[test]
    pub fn test_path_param_middle() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/:foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo/123/bar", "Bar");
    }

    // Test multiple path parameters
    #[test]
    pub fn test_multiple_path_param() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/:foo/bar/:bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo/123/bar/456", "Bar");
    }

    // Test adding a single, multiple component path
    #[test]
    pub fn test_add() {
        let mut expected = PathNode::new("/", "", None);
        let mut foo = PathNode::new("/foo", "foo", None);
        let bar = PathNode::new("/foo/bar", "bar", Some(String::from("Bar")));
        foo.next.insert(String::from("bar"), bar);
        expected.next.insert(String::from("foo"), foo);
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_eq!(expected, route.root);
    }

    // Test adding a two routes that have a common ancestor
    #[test]
    pub fn test_add_two() {
        let foo = sub_route2("foo", "bar", "baz");
        let mut expected = PathNode::new("/", "", None);
        expected.next.insert(String::from("foo"), foo);
        let mut route = RouteTree::empty_root();
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
        let mut expected = PathNode::new("/", "", None);
        let mut foo = PathNode::new("/foo", "foo", Some(String::from("Foo")));
        foo.next.insert(
            String::from("bar"),
            PathNode::new("/foo/bar", "bar", Some(String::from("Bar"))),
        );
        expected.next.insert(String::from("foo"), foo);
        let mut route = RouteTree::empty_root();
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
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo/bar", "Bar");
    }

    // Test that we can find an added path with a more complex routing tree
    #[test]
    pub fn test_dispatch_two() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error")
            .add("/foo/baz", String::from("Baz"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo", "");
        assert_dispatch(&route, "/foo/bar", "Bar");
        assert_dispatch(&route, "/foo/baz", "Baz");
    }

    // Test that we can find an added path with a more complex routing tree
    #[test]
    pub fn test_dispatch_complex() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error")
            .add("/foo/:foo", String::from("Foo"))
            .expect("Should have added route without error")
            .add("/foo/:foo/bar", String::from("PathBar"))
            .expect("Should have added route without error")
            .add("/foo/:foo/baz", String::from("PathBaz"))
            .expect("Should have added route without error")
            .add("/foo/baz", String::from("Baz"))
            .expect("Should have added route without error")
            .add("/foo/baz/qux", String::from("Qux"))
            .expect("Should have added route without error")
            .add("/qux/quux/quuux/quuuux/quuuuux", String::from("LongPath"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo", "");
        assert_dispatch(&route, "/foo/123", "Foo");
        assert_dispatch(&route, "/foo/123/bar", "PathBar");
        assert_dispatch(&route, "/foo/123/baz", "PathBaz");
        assert_dispatch(&route, "/foo/bar", "Bar");
        assert_dispatch(&route, "/foo/baz", "Baz");
        assert_dispatch(&route, "/foo/baz/qux", "Qux");
        assert_dispatch(&route, "/qux", "");
        assert_dispatch(&route, "/qux/quux/quuux/quuuux/quuuuux", "LongPath");
    }

    // Test the expected outcome of a partial routing match.
    #[test]
    pub fn test_partial() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar", String::from("Bar"))
            .expect("Should have added route without error");
        assert_dispatch(&route, "/foo", "");
    }

    // Test iterating partially into the tree.
    #[test]
    pub fn test_iter_partial() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar/baz", String::from("Baz"))
            .expect("Should have been able to add route.");

        let mut tokens = "foo".split('/');

        let mut iter = route.iter(&mut tokens);
        assert_eq!(
            Some(&String::from("foo")),
            iter.next().map(|node| &node.segment)
        );
        assert_eq!(None, iter.next());
    }

    // Test iterating with a requested path that misses.
    #[test]
    pub fn test_iter_miss() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar/baz", String::from("Baz"))
            .expect("Should have been able to add route.");

        let mut tokens = "foo/baz".split('/');

        let mut iter = route.iter(&mut tokens);
        assert_eq!(
            Some(&String::from("foo")),
            iter.next().map(|node| &node.segment)
        );
        assert_eq!(None, iter.next());
    }

    // Test iterating a requested path that is in the tree.
    #[test]
    pub fn test_iter_hit() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/bar/baz", String::from("Baz"))
            .expect("Should have been able to add route.");

        let mut tokens = "foo/bar/baz".split('/');

        let mut iter = route.iter(&mut tokens);
        assert_eq!(
            Some(&String::from("foo")),
            iter.next().map(|node| &node.segment)
        );
        assert_eq!(
            Some(&String::from("bar")),
            iter.next().map(|node| &node.segment)
        );
        assert_eq!(
            Some(String::from("Baz")),
            iter.next().as_ref().and_then(|node| node.handler.clone())
        );
        assert_eq!(None, iter.next());
    }

    // Test iterating with a path parameter.
    #[test]
    pub fn test_iter_path() {
        let mut route = RouteTree::empty_root();
        route
            .add("/foo/:bar", String::from("Foo"))
            .expect("Should have been able to add route.");

        let mut tokens = "foo/123".split('/');

        let mut iter = route.iter(&mut tokens);
        assert!(
            iter.next().map(|node| node.params.is_some()).unwrap(),
            "First component should have had a params opt"
        );
        assert_eq!(
            Some(&Some(String::from("Foo"))),
            iter.next().map(|node| &node.handler)
        );
    }

    fn sub_route2(parent: &str, first: &str, second: &str) -> PathNode<String> {
        let mut node = PathNode::new(&format!("/{}", parent), parent, None);
        node.next.insert(
            first.to_owned(),
            PathNode::new(
                &format!("/{}/{}", parent, first),
                first,
                Some(String::from(first.to_uppercase())),
            ),
        );
        node.next.insert(
            second.to_owned(),
            PathNode::new(
                &format!("/{}/{}", parent, second),
                second,
                Some(String::from(second.to_uppercase())),
            ),
        );
        node
    }

    fn assert_dispatch(route: &RouteTree<String>, route_path: &str, handler: &str) {
        let found = route.dispatch(route_path);
        if let Some((_, found)) = found {
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
            panic!("Not found");
        }
    }
}
