mod build;
mod route;

pub use self::build::Build;
pub use self::route::Route;
use regex::RegexSet;
use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone)]
pub struct Router<Method, Handler> {
    routes: Vec<Route<Method, Handler>>,
    set: RegexSet,
    default: Option<Handler>,
}

impl<M: Eq, H> Router<M, H> {
    pub fn build() -> Build<M, H> {
        Build::default()
    }

    pub fn lookup<'s, 'p>(&'s self, method: &'_ M, path: &'p str) -> Option<(&'s H, Vec<&'p str>)> {
        self.set
            .matches(path)
            .iter()
            .flat_map(|i| self.routes.get(i))
            .filter(|route| method == &route.method)
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
            //            .map(|r| (&r.handler, vec![]))
            .next()
            .or_else(|| self.default.as_ref().map(|h| (h, vec![])))
    }

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
