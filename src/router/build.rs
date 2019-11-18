use super::{Route, Router};
use regex::RegexSet;

#[derive(Debug, Clone)]
/// The builder for the router.  This collects all of the routes that the router
/// will have, and then builds the cache to quickly perform lookups for that
/// router.
pub struct Build<M, H> {
    routes: Vec<Route<M, H>>,
    default: Option<H>,
}

impl<M, H> Build<M, H> {
    /// Adds the given route to the builder.
    pub fn add(&mut self, route: Route<M, H>) -> &mut Self {
        self.routes.push(route);
        self
    }

    /// Sets the default of the builder.  If no other route matches the given
    /// path, the default is instead returned.  Because there was no route
    /// to match, there will obviously be no url parameters in that match,
    /// either.
    pub fn with_default(&mut self, default: H) -> &mut Self {
        self.default = Some(default);
        self
    }
}

impl<M: Eq, H> Build<M, H> {
    /// Completes the build, returning the router.
    pub fn finish(self) -> Router<M, H> {
        let set = RegexSet::new(self.routes.iter().map(|route| route.pattern.as_str())).unwrap();
        Router {
            routes: self.routes,
            set,
            default: self.default,
        }
    }
}

impl<M, H> Default for Build<M, H> {
    fn default() -> Self {
        Build {
            routes: vec![],
            default: None,
        }
    }
}
