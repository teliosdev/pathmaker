use failure::{Compat, Error};
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::borrow::Cow;
use futures::prelude::*;

type HandlerFuture = Box<dyn Future<Item = Response<Body>, Error = failure::Error> + Send + 'static>;

/// The handler that's stored as a part of every route in the router.  Since
/// we're dealing with Hyper, it must return a future; we use the `Box<Fn>`
/// type in order to keep flexibility.
///
/// The [`Route`] and [`Build`] types automatically box the closure as a part
/// of its shortcut methods.
pub type Handler = Box<dyn Fn(Request<Body>, Vec<String>) -> HandlerFuture + Send + 'static>;

/// A single route, tied to Hyper's types, and our [`Handler`].  We add some
/// shortcut methods onto this type in order to make building routes for hyper
/// easier.
pub type Route = super::router::Route<Method, Handler>;

/// The router type, tied to Hyper's types, and our [`Handler`].  This
/// implements [`hyper::serivce::Service`] by default, and if no default handler
/// is given, it returns an empty 404 response.
pub type Router = super::router::Router<Method, Handler>;

/// A builder for building routes, tied to Hyper's types and our [`Handler`].
/// We add some shortcut methods onto this type in order to make building
/// routes for hyper easier.
pub type Build = super::router::Build<Method, Handler>;

macro_rules! route {
    (
        $(#$meta:tt)*
        $name:ident => $method:expr
    ) => {
        $(#$meta)*
        pub fn $name<P, F>(path: P, handler: F) -> Self
        where
            P: Into<Cow<'static, str>>,
            F: Fn(Request<Body>, Vec<String>) -> HandlerFuture + Send + 'static
        {
            Self::new(path, $method, Box::new(handler))
        }
    };
}

impl Route {
    route!(options => Method::OPTIONS);
    route!(get => Method::GET);
    route!(post => Method::POST);
    route!(put => Method::PUT);
    route!(delete => Method::DELETE);
    route!(head => Method::HEAD);
    route!(trace => Method::TRACE);
    route!(connect => Method::CONNECT);
    route!(patch => Method::PATCH);
}

macro_rules! build {
    (
        $(#$meta:tt)*
        $name:ident
    ) => {
        $(#$meta)*
        pub fn $name<P, F>(&mut self, path: P, handler: F) -> &mut Self
        where
            P: Into<Cow<'static, str>>,
            F: Fn(Request<Body>, Vec<String>) -> HandlerFuture + Send + 'static
        {
            self.add(Route::$name(path, handler))
        }
    }
}

impl Build {
    build!(options);
    build!(get);
    build!(post);
    build!(put);
    build!(delete);
    build!(head);
    build!(trace);
    build!(connect);
    build!(patch);

    pub fn default_fn<F>(&mut self, default: F) -> &mut Self
        where F: Fn(Request<Body>, Vec<String>) -> HandlerFuture + Send + 'static
    {
        self.with_default(Box::new(default))
    }
}

impl Service for Router {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<dyn Future<Item = Response<Body>, Error = Compat<Error>> + Send + 'static>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let path = req.uri().path();
        if let Some((handler, params)) = self.lookup(req.method(), path) {
            let params = params.into_iter().map(str::to_string).collect();
            Box::new(handler(req, params).map_err(Error::compat))
        } else {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .map_err(Error::from)
                .map_err(Error::compat);
            Box::new(futures::future::result(response))
        }
    }
}
