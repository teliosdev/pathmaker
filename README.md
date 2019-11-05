# Pathmaker

Generalized routing for any HTTP library.  Not too fancy.

## Usage

To use this library, add the following to your `Cargo.toml`:

```toml
pathmaker = "0.1.0"
```

If you want to use it with a specific HTTP library, e.g. `hyper`, enable that
feature:

```toml
hyper = "0.12"
pathmaker = { version = "0.1", features = ["hyper"] }
```

Then, in your code, construct a router:

```rust
extern crate hyper;
extern crate pathmaker;
extern crate failure;
extern crate futures;

use hyper::{Request, Response, Method, Body};
use hyper::header::CONTENT_LENGTH;
use pathmaker::hyper::Router;
use failure::Error;

fn handler(_: Request<Body>, _: Vec<String>) -> Box<dyn Future<Item = Response<Body>, Error = Error>> {
    let body = "Hello, world!";
    Box::new(futures::future::result(
        Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .body(Body::from(body))
            .map_err(Error::from)
    ))
}

fn router() -> Router {
    let build = Router::build()
    build.get("/foo", handler);
    build.finish()
}

fn main() {
    let address = "0.0.0.0:8080".parse().unwrap();
    let server = Server::bind(&addr).serve(router()).map_err(|e| {
        eprintln!("error: {:?}", e)
    });
    hyper::rt::run(server)
}
```

## Query Parameters

Support for query parameters is allowed by using `{}` in the path:

```rust
// ...
fn router() {
    let build = Router::build();
    build.get("/foo", handler)
        .get("/hello/{}", hello_handler);
    build.finish()
}
// ...
```

Then, in the handler, you can access the first element of the second argument
in order to get the result:

```rust
//...
fn hello_handler(_: Request<Body>, params: Vec<String>) -> Box<dyn Future<Item = Response<Body>, Error = Error>> {
    let body = format!("Hello, {}!", params[0]);
    Box::new(futures::future::result(
        Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .body(Body::from(body))
            .map_err(Error::from)
    ))
}
// ...
```

Query parameters can be filtered down by format:

- `{}`, `{:string}` (the default): anything that isn't a `/` character is 
  matched.
- `{:int}`: a positive or negative number.
- `{:uint}`: just a number, no sign allowed.
- `{:uuid}`: a UUID, in 8-4-4-4-12 format.

More can be added if requested.

## Route Evaluation

Routes are evaluated from top to bottom.  The first route that matches is used.