mod build;
mod route;

pub use self::build::Build;
pub use self::route::Route;
use regex::RegexSet;
use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone)]
/// The main router.  This contains a set of routes that can be taken, as well
/// as a default hnadler for when the request matches none of those routes.
/// We make no requirements on the `Method` type here; this is the type used
/// by the HTTP library to represent the HTTP method (e.g. `GET`, `POST`, etc.).
/// When routes are created, they're created with this Method type, and when we
/// do a lookup, we make sure the route matches the method.
///
/// We also make no restrictions on the Handler; all that's returned upon
/// lookup is an immutable reference to the handler, if one exists.
pub struct Router<Method, Handler> {
    routes: Vec<Route<Method, Handler>>,
    set: RegexSet,
    default: Option<Handler>,
}

impl<M: Eq, H> Router<M, H> {
    /// Create a builder for the router.  See type [`Build`] for more
    /// information.
    pub fn build() -> Build<M, H> {
        Build::default()
    }

    /// This performs the actual lookup.  We take a reference to the method, and
    /// a reference to the path, and return the handler and the url parameters,
    /// if they exist.  Note that the path **must** be URL decoded, and *only*
    /// contain the path - it **must not** contain any query parameters.
    pub fn lookup<'s, 'p>(&'s self, method: &'_ M, path: &'p str) -> Option<(&'s H, Vec<&'p str>)> {
        self.set
            // First, we attempt to lookup any of the routes that match.  We
            // use our regex set to narrow down the routes easily...
            .matches(path)
            // Which returns an iterator of indexes...
            .iter()
            // So we'll have to lookup the routes in our array.
            .flat_map(|i| self.routes.get(i))
            // We then verify that the route has the corresponding method...
            .filter(|route| method == &route.method)
            // Then, we use the route's internal pattern to do the lookup.
            // This serves two purposes: 1. collect the url parameters; and 2.
            // verify that the route actually matched.
            .flat_map(|route| {
                route.pattern.captures(path).map(|caps| {
                    let caps = caps
                        .iter()
                        .skip(1)
                        .map(|m| m.unwrap().as_str())
                        .collect::<Vec<_>>();
                    (&route.handler, caps)
                })
            })
            // Grab the first route that matched.
            .next()
            // If no routes matched, we'll return the default, if it exists.
            .or_else(|| self.default.as_ref().map(|h| (h, vec![])))
    }

    /// Sets the default of the router.  This is similar to
    /// [`Build::set_default`].
    pub fn set_default(&mut self, default: H) {
        self.default = Some(default);
    }
}

impl<M: Debug, H: Debug> Debug for Router<M, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        struct InnerRoutes<'r, M, H>(&'r Router<M, H>);

        impl<M: Debug, H: Debug> Debug for InnerRoutes<'_, M, H> {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.debug_map()
                    .entries(
                        self.0
                            .routes
                            .iter()
                            .map(|route| (&route.path, &route.handler)),
                    )
                    .finish()
            }
        }

        f.debug_tuple("Router").field(&InnerRoutes(self)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_routes() {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        enum Method {
            Get,
            #[allow(dead_code)]
            Post,
        };
        let mut build = Router::build();
        build
            .add(Route::new("/some/path", Method::Get, 1))
            .add(Route::new("/some/{:uint}", Method::Get, 2))
            .add(Route::new("/some/{:int}", Method::Get, 3))
            .add(Route::new("/some/{:uuid}", Method::Get, 4))
            .add(Route::new("/some/{:string}", Method::Get, 5));
        let router = build.finish();

        assert_eq!(
            router.lookup(&Method::Get, "/some/path"),
            Some((&1, vec![]))
        );
        assert_eq!(
            router.lookup(&Method::Get, "/some/4"),
            Some((&2, vec!["4"]))
        );
        assert_eq!(
            router.lookup(&Method::Get, "/some/-4"),
            Some((&3, vec!["-4"]))
        );
        assert_eq!(
            router.lookup(&Method::Get, "/some/00000000-0000-0000-0000-000000000000"),
            Some((&4, vec!["00000000-0000-0000-0000-000000000000"]))
        );
        assert_eq!(
            router.lookup(&Method::Get, "/some/other"),
            Some((&5, vec!["other"]))
        );
        assert_eq!(router.lookup(&Method::Get, "/soap"), None);
    }
}
