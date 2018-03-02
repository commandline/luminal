pub struct Route {
    root: PathComp,
}

impl Route {
    pub fn add(&self, route: &str, handler: String) {}

    fn go(idx: usize, tokens: &[&str], current: &mut PathComp) {
        if idx >= tokens.len() {
            return;
        }
        let replace_idx = {
            let PathComp::Path { ref next, .. } = *current;
            next.iter().position(|comp| {
                let PathComp::Path { ref path, .. } = *comp;
                path == tokens[idx]
            })
        };
        if let Some(replace_idx) = replace_idx {
            let PathComp::Path { ref mut next, .. } = *current;
            let mut to_replace = PathComp::Path {
                path: tokens[idx].to_owned(),
                next: vec![],
            };
            Self::go(idx + 1, tokens, &mut to_replace);
            next[replace_idx] = to_replace;
        } else {
            let PathComp::Path { ref mut next, .. } = *current;
            let mut to_add = PathComp::Path {
                path: tokens[idx].to_owned(),
                next: vec![],
            };
            Self::go(idx + 1, tokens, &mut to_add);
            next.push(to_add);
        }
    }

    pub fn dispatch(&self, request_path: &str) -> String {
        String::from("")
    }
}

#[derive(Debug, PartialEq)]
pub enum PathComp {
    Path { path: String, next: Vec<PathComp> },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_go() {
        let mut root = PathComp::Path {
            path: "".to_owned(),
            next: vec![],
        };
        let path = vec!["foo", "bar"];
        Route::go(0, &path, &mut root);
        assert_eq!(
            PathComp::Path {
                path: "".to_owned(),
                next: vec![
                    PathComp::Path {
                        path: "foo".to_owned(),
                        next: vec![],
                    },
                ],
            },
            root
        );
    }
}
