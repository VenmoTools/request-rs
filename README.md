# request-rs
A simple synchronous HTTP client library

This crate is base on [http crate](https://github.com/hyperium/http), thanks for the author of the crate.

# Usage

## Build Request 
```rust

use request_rs::produce::*;
use url::Url;

use request_rs::headers::HeaderMap;
pub fn simple_get(){
    let resp = HttpClient::request(Method::GET,"http://www.example.com/", None, None).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}

pub fn simple_get_with_header(){
    let mut header = HeaderMap::new();
    header.append("Accept","text/html".parse().unwrap());
    
    let resp = HttpClient::request(Method::GET,"http://www.example.com/", None, Some(header)).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}

pub fn build_request(){
    let mut req = Request::builder()
                .method(Method::GET)
                .uri(Url::parse("https://www.baidu.com").expect("invalid url"))
                .header("User-Agent", "request-rs")
                .header("Host", host)
                .version(Version::HTTP_11)
                .body(());
    let client = HttpClient::http();
    let response = client.send(req).expect("request failed");
    assert_eq!(response.status(),StatusCode::from_u16(200).unwrap());
}
```


## Simple Get Request
 ```rust
use request_rs::produce::*;
use url::Url;
use request_rs::headers::HeaderMap;
pub fn simple_get(){
    let resp = HttpClient::get("http://www.example.com/", None, None).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}

pub fn simple_get_with_header(){
    let mut header = HeaderMap::new();
    header.append("Accept","text/html".parse().unwrap());
    let resp = HttpClient::get("http://www.example.com/", None, Some(header)).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}

pub fn simple_get_with_param(){
    let mut header = HeaderMap::new();
    header.append("Accept","text/html".parse().unwrap());
    let resp = HttpClient::get("http://www.example.com/?admin=yes&show=yes",None, Some(header)).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}
```

## Simple Post Request
```rust
use request_rs::produce::*;
use url::Url;
use request_rs::headers::HeaderMap;

pub fn simple_post_with_data(){
    let body = Body::from_str("username=admin&password123");
    let mut header = HeaderMap::new();
    header.append("Accept","text/html".parse().unwrap());
    let resp = HttpClient::post("http://www.example.com/",Some(body), Some(header)).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}

pub fn simple_post_with_file(){ 
    let body = Body::from_file("example.file");
    let mut header = HeaderMap::new();
    header.append("Accept","text/html".parse().unwrap());
    let resp = HttpClient::post("http://www.example.com/",Some(body), Some(header)).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}

pub fn simple_get_with_macro_header(){
    let header = http_header! {
        "Accept" => "text/html",
        "Host" => "www.example.com",
    };
    let resp = HttpClient::request(Method::GET,"http://www.example.com/", None, Some(header)).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
}

pub fn simple_post_with_empty(){ 
    let body = Body::empty();
    let mut header = HeaderMap::new();
    header.append("Accept","text/html".parse().unwrap());
    let resp = HttpClient::post("http://www.example.com/",Some(body), Some(header)).expect("failed");
    assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
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
5. cookie jar
6. authorization support
