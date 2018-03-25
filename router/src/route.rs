pub struct Route<T> {
    pub route_path: String,
    pub target: T,
}

impl<T> Route<T> {
    pub fn new(route_path: &str, target: T) -> Self {
        Route {
            route_path: route_path.to_owned(),
            target,
        }
    }
}
