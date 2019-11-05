#![feature(never_type)]
#![feature(test)]

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
