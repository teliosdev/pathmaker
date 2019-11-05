use failure::Error;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::borrow::Cow;

//Options,
//Get,
//Post,
//Put,
//Delete,
//Head,
//Trace,
//Connect,
//Patch,

pub type Handler = fn(
    Request<Body>,
    Vec<String>,
) -> Box<dyn hyper::rt::Future<Item = Response<Body>, Error = failure::Error>>;

pub type Route = super::router::Route<Method, Handler>;
pub type Router = super::router::Router<Method, Handler>;
pub type Build = super::router::Build<Method, Handler>;

macro_rules! route {
    ($name:ident, $method:expr) => {
        pub fn $name<P: Into<Cow<'static, str>>>(path: P, handler: Handler) -> Self {
            Self::new(path, $method, handler)
        }
    };
}

impl Route {
    route!(options, Method::OPTIONS);
    route!(get, Method::GET);
    route!(post, Method::POST);
    route!(put, Method::PUT);
    route!(delete, Method::DELETE);
    route!(head, Method::HEAD);
    route!(trace, Method::TRACE);
    route!(connect, Method::CONNECT);
    route!(patch, Method::PATCH);
}

macro_rules! build {
    ($($name:ident),+) => {
        $(
            pub fn $name<P: Into<Cow<'static, str>>>(&mut self, path: P, handler: Handler) -> &mut Self {
                self.add(Route::$name(path, handler))
            }
        )+
    }
}

impl Build {
    build!(options, get, post, put, delete, head, trace, connect, patch);
}

impl hyper::service::Service for Router {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = failure::Error;
    type Future = ();

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let path = req.uri().path();
        if let Some((handler, params)) = self.lookup(req.method(), path) {
            handler(req, params)
        } else {
            let mut response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .map_err(Error::from);
            Box::new(futures::future::result(response))
        }
    }
}
