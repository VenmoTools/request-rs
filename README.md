# request-rs
A simple synchronous HTTP client library

This crate is base on [http crate](https://github.com/hyperium/http), thanks for the author of the crate.

# Usage

## Build Request 
```rust
use request_rs::produce::*;
use url::Url;

pub fn build_request(){
    let mut req = Request::builder()
                .method(Method::GET)
                .uri(Url::parse("https://www.baidu.com").expect("invalid url"))
                .header("User-Agent", "request-rs")
                .header("Host", host)
                .version(Version::HTTP_11)
                .body(Vec::new());
    let mut client = Client::new(&req).expect("failed");
    let response = client.send().expect("request failed");
    assert_eq!(response.status(),StatusCode(200));
}
```

## Build Request with timeout
```rust
use request_rs::produce::*;
use url::Url;
use std::time::Duration;

pub fn with_timeout(){
    let mut req = Request::builder()
                .method(Method::GET)
                .uri(Url::parse("https://www.baidu.com"))
                .header("User-Agent", "request-rs")
                .header("Host", host)
                .version(Version::HTTP_11)
                .body(Vec::new());
    let mut client = Client::with_timeout(&req,Duration::from_secs(30)).expect("failed");
    let response = client.send().expect("request failed");
    assert_eq!(response.status(),StatusCode(200));
}
```

## Simple Get Request
```rust
use request_rs::produce::*;
use url::Url;
use std::collections::BTreeMap;
pub fn simple_get(){
    let resp = Client::get(Url::parse("http://www.example.com/").expect("failed"), Vec::new(), None).expect("failed");
    assert_eq!(StatusCode(200), resp.status());
}

pub fn simple_get_with_header(){
    let mut header = BTreeMap::new();
    header.insert("Accept","text/html");
    let resp = Client::get(Url::parse("http://www.example.com/").expect("failed"), Vec::new(), Some(header)).expect("failed");
    assert_eq!(StatusCode(200), resp.status());
}

pub fn simple_get_with_param(){
    let mut header = BTreeMap::new();
    header.insert("Accept","text/html");
    let resp = Client::get(Url::parse("http://www.example.com/?admin=yes&show=yes").expect("failed"),Vec::new(), Some(header)).expect("failed");
    assert_eq!(StatusCode(200), resp.status());
}
```

## Simple Post Request
```rust
use request_rs::produce::*;
use url::Url;
use std::collections::BTreeMap;
pub fn simple_post_with_data(){
  let body = Vec::from("username=admin&password123".as_bytes());
  let mut header = BTreeMap::new();
  header.insert("Accept","text/html");
  let resp = Client::post(Url::parse("http://www.example.com/").expect("failed"),body, Some(header)).expect("failed");
  assert_eq!(StatusCode(200), resp.status());
}
```


# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

# todo
1. long connection
2. https support
3. More ergonomic APIs
4. json serialize/deserialize support
