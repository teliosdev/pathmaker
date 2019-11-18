//! # Pathmaker
//!
//! Generalized routing for any HTTP library.  Not too fancy.
//!
//! ## Usage
//!
//! To use this library, add the following to your `Cargo.toml`:
//!
//! ```toml
//! pathmaker = "0.1.0"
//! ```
//!
//! If you want to use it with a specific HTTP library, e.g. `hyper`, enable that
//! feature:
//!
//! ```toml
//! hyper = "0.12"
//! pathmaker = { version = "0.1", features = ["hyper"] }
//! ```
//!
//! Then, in your code, construct a router:
//!
//! ```rust
//! extern crate hyper;
//! extern crate pathmaker;
//! extern crate failure;
//! extern crate futures;
//!
//! use hyper::{Request, Response, Method, Body, Server};
//! use hyper::service::make_service_fn;
//! use hyper::header::CONTENT_LENGTH;
//! use pathmaker::hyper::Router;
//! use failure::Error;
//! use futures::prelude::*;
//!
//! fn router() -> Router {
//!     let mut build = Router::build();
//!     build.get("/foo", |_, _| {
//!         let body = "Hello, world!";
//!         Box::new(futures::future::result(Response::builder()
//!             .header(CONTENT_LENGTH, body.len() as u64)
//!             .body(Body::from(body))
//!             .map_err(Error::from)
//!         ))
//!     });
//!     build.finish()
//! }
//!
//! fn main() {
//!     let address = "0.0.0.0:8080".parse().unwrap();
//!     let server = Server::bind(&address)
//!         .serve(make_service_fn(|_| Ok::<_, hyper::Error>(router()))).map_err(|e| {
//!             eprintln!("error: {:?}", e);
//!         });
//!     // hyper::rt::run(server)
//! }
//! ```
//!
//! ## Query Parameters
//!
//! Support for query parameters is allowed by using `{}` in the path:
//!
//! ```rust
//! // ...
//! # use pathmaker::hyper::Router;
//! # use futures::prelude::*;
//! # use hyper::{Request, Response, Body};
//! # use hyper::header::CONTENT_LENGTH;
//! # use failure::Error;
//! # fn handler(_: Request<Body>, _: Vec<String>) -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send> {
//! #   let body = "Hello, world!";
//! #   Box::new(futures::future::result(Response::builder()
//! #       .header(CONTENT_LENGTH, body.len() as u64).body(Body::from(body))
//! #       .map_err(Error::from)))
//! # }
//! # fn hello_handler(a: Request<Body>, b: Vec<String>) -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send> { handler(a, b) }
//! fn router() -> Router {
//!     let mut build = Router::build();
//!     build.get("/foo", handler)
//!          .get("/hello/{}", hello_handler);
//!     build.finish()
//! }
//! // ...
//! ```
//!
//! Then, in the handler, you can access the first element of the second argument
//! in order to get the result:
//!
//! ```rust
//! # use hyper::{Request, Response, Body};
//! # use failure::Error;
//! # use futures::prelude::*;
//! # use hyper::header::CONTENT_LENGTH;
//! //...
//! fn hello_handler(_: Request<Body>, params: Vec<String>) -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send> {
//!     let body = format!("Hello, {}!", params[0]);
//!     Box::new(futures::future::result(
//!         Response::builder()
//!             .header(CONTENT_LENGTH, body.len() as u64)
//!             .body(Body::from(body))
//!             .map_err(Error::from)
//!     ))
//! }
//! // ...
//! ```
//!
//! Query parameters can be filtered down by format:
//!
//! - `{}`, `{:string}` (the default): anything that isn't a `/` character is
//!   matched.
//! - `{:int}`: a positive or negative number.
//! - `{:uint}`: just a number, no sign allowed.
//! - `{:uuid}`: a UUID, in 8-4-4-4-12 format.
//!
//! More can be added if requested.
//!
//! ## Route Evaluation
//!
//! Routes are evaluated from top to bottom.  The first route that matches is used.

#[macro_use]
extern crate failure;
#[cfg(feature = "test")]
extern crate test;

#[cfg(feature = "hyper")]
pub mod hyper;

pub mod router;
pub use self::router::*;

#[cfg_attr(feature = "test", bench)]
#[cfg(feature = "test")]
pub fn bench_mark(b: &mut test::Bencher) {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    enum Method {
        Get,
        Post,
    }
    let mut build = Router::build();
    build
        .add(Route::new("/", Method::Get, 1))
        .add(Route::new("/hello", Method::Get, 2))
        .add(Route::new("/hello/world", Method::Get, 3))
        .add(Route::new("/foo", Method::Get, 4))
        .add(Route::new("/foo/bar", Method::Get, 5))
        .add(Route::new("/foo/baz", Method::Get, 6))
        .add(Route::new("/foo/{}", Method::Get, 7));
    let route = build.finish();
    assert!(route.lookup(&Method::Get, "/foo/bar").is_some());
    assert_eq!(
        route.lookup(&Method::Get, "/foo/bar").unwrap(),
        (&5, vec![])
    );

    b.iter(|| route.lookup(&Method::Get, "/foo/bar"));
}

fn normalize_url<V: AsRef<str>>(string: V) -> String {
    let str = string.as_ref();
    let url = str.split_terminator("?").next().unwrap_or(str).as_bytes();

    percent_encoding::percent_decode(url)
        .decode_utf8_lossy()
        .into_owned()
}
