use super::{Route, Router};
use regex::RegexSet;

#[derive(Debug, Clone)]
pub struct Build<M, H> {
    routes: Vec<Route<M, H>>,
}

impl<M, H> Build<M, H> {
    pub fn add(&mut self, route: Route<M, H>) -> &mut Self {
        self.routes.push(route);
        self
    }
}

impl<M: Eq, H> Build<M, H> {
    pub fn finish(self) -> Router<M, H> {
        let set = RegexSet::new(self.routes.iter().map(|route| route.pattern.as_str())).unwrap();
        Router {
            routes: self.routes,
            set,
        }
    }
}

impl<M, H> Default for Build<M, H> {
    fn default() -> Self {
        Build { routes: vec![] }
    }
}
