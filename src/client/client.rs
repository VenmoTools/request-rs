use std::io::ErrorKind;

use bytes::BytesMut;
use url::Url;

use crate::body::Body;
use crate::error::{Error, InvalidUrl, IoError, Result};
use crate::header::HeaderMap;
use crate::method::Method;
use crate::proto::{Connector, HttpConfig, HttpConnector, HttpParser, ParserResult, RequestParser, ResponseParser};
use crate::request::Request;
use crate::response::Response;
use crate::version::Version;

/// the struct of http client
#[derive(Debug)]
pub struct HttpClient<C: Connector> {
    connector: C,
}

impl<C: Connector> HttpClient<C> {
    ///
    ///
    /// ```
    /// use request_rs::produce::*;
    /// use url::quirks::host;
    /// use url::Url;
    /// use hyper::client::HttpConnector;
    ///
    /// let req = Request::builder()
    ///             .method(Method::GET)
    ///             .uri(Url::parse("https://www.example.com").expect("invalid url"))
    ///             .header("User-Agent", "request-rs")
    ///             .header("Host", host)
    ///             .version(Version::HTTP_11)
    ///             .body(Vec::new());
    /// let mut client = Client::from_connector(HttpConnector::new());
    /// let response =client.send(req).expect("request failed");
    /// assert_eq!(response.status(),StatusCode::from_u16(200).unwrap());
    /// ```
    pub fn from_connector(connector: C) -> Self {
        Self {
            connector,
        }
    }
}


impl HttpClient<HttpConnector> {
    /// do http request
    /// ```
    /// use request_rs::produce::*;
    /// use url::quirks::host;
    /// use url::Url;
    ///
    /// fn main(){
    ///     let req = Request::builder()
    ///                 .method(Method::GET)
    ///                 .uri(Url::parse("https://www.example.com").expect("invalid url"))
    ///                 .header("User-Agent", "request-rs")
    ///                 .header("Host", host)
    ///                 .version(Version::HTTP_11)
    ///                 .body(Body::empty()).unwrap();
    ///     let mut client = HttpClient::http();
    ///     let response =client.send(req).expect("request failed");
    ///     assert_eq!(response.status(),StatusCode::from_u16(200).unwrap());
    /// }
    /// ```
    pub fn send(&mut self, req: Request<Body>) -> Result<Response<Body>> {
        let url = req.uri().ok_or(Error::from(InvalidUrl::new("missing url")))?.clone();
        let sock_addr = RequestParser::socket_addr(&url)?;

        let req_buf = RequestParser::encode(req)?;
        self.connector.connect_to(&sock_addr)?;

        // send request
        self.connector.write_all(req_buf.as_ref())?;

        // response
        let mut data = Vec::new();
        self.connector.read_all(&mut data)?;

        let mut buf = BytesMut::from(data.as_slice());
        let mut parser = ResponseParser::new();
        let resp = match parser.parse(&mut buf)? {
            ParserResult::Complete(resp) => resp,
            ParserResult::Partial => {
                return Err(Error::from(IoError::from_kind(ErrorKind::UnexpectedEof)));
            }
        };
        Ok(resp)
    }

    /// send request
    /// use http connector
    /// ```
    /// use request_rs::produce::*;
    ///
    /// fn main(){
    ///     let mut client = HttpClient::http();
    ///     let resp = client.send_request("http://www.example.com",Method::GET,None,None).unwrap();
    ///     assert_eq!(resp,StatusCode::from_u16(200).unwrap())
    /// }
    /// ```
    pub fn send_request(&mut self, url: &str, method: Method, headers: Option<HeaderMap>, body: Option<Body>) -> Result<Response<Body>> {
        let url = Url::parse(url)?;
        let host = url.domain().ok_or(Error::from(InvalidUrl::new("invalid url")))?;
        let mut req = Request::builder()
            .method(method)
            .version(Version::HTTP_11);
        if let Some(header) = headers {
            req = req.replace_header_map(header);
        } else {
            req = req.header("User-Agent", "request-rs");
        }
        let body = match body {
            Some(body) => body,
            None => Body::empty()
        };

        let req = req.header("Host", host)
            .uri(url)
            .header("Content-Length", body.body_length())
            .body(body)?;
        self.send(req)
    }

    /// with http config
    /// ```
    /// use request_rs::config::h1::HttpConfig;
    /// fn main(){
    ///     use std::time::Duration;
    /// use request_rs::produce::{HttpClient, Method, StatusCode};
    ///     let config = HttpConfig {
    ///        connect_timeout: None,
    ///        happy_eyeballs_timeout: Some(Duration::from_millis(300)),
    ///        keep_alive_timeout: None,
    ///        local_address: None,
    ///        nodelay: false,
    ///        reuse_address: false,
    ///        send_buffer_size: None,
    ///        recv_buffer_size: None,
    ///        ttl: 64,
    ///    };
    ///    let mut client = HttpClient::with_config(config);
    ///    client.send_request("http://www.example.com",Method::GET,None,None).unwrap();
    ///    assert_eq!(response.status(),StatusCode::from_u16(200).unwrap());
    /// }
    /// ```
    pub fn with_config(config: HttpConfig) -> Self {
        Self::from_connector(HttpConnector::with_http_config(config))
    }
    /// use http connector
    /// ```
    /// use request_rs::produce::*;
    ///
    /// fn main(){
    ///     let mut client = HttpClient::http();
    ///     let resp = client.send_request("http://www.example.com",Method::GET,None,None).unwrap();
    ///     assert_eq!(resp,StatusCode::from_u16(200).unwrap())
    /// }
    /// ```
    pub fn http() -> Self {
        Self::from_connector(HttpConnector::new())
    }

    /// do http get request
    ///
    /// ```
    /// use request_rs::produce::*;
    /// use url::Url;
    /// use request_rs::headers::HeaderMap;
    /// pub fn simple_get(){
    ///     let resp = HttpClient::request(Method::GET,"http://www.example.com/", None, None).expect("failed");
    ///     assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
    /// }
    ///
    /// pub fn simple_get_with_header(){
    ///     let mut header = HeaderMap::new();
    ///     header.append("Accept","text/html".parse().unwrap());
    ///     let resp = HttpClient::request(Method::GET,"http://www.example.com/", None, Some(header)).expect("failed");
    ///     assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
    /// }
    /// ```
    pub fn request(method: Method, url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        let mut client = Self::http();
        client.send_request(url, method, headers, body)
    }

    /// do http get request
    ///
    /// ```
    /// use request_rs::produce::*;
    /// use url::Url;
    /// use request_rs::headers::HeaderMap;
    /// pub fn simple_get(){
    ///     let resp = HttpClient::get("http://www.example.com/", None, None).expect("failed");
    ///     assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
    /// }
    ///
    /// pub fn simple_get_with_header(){
    ///     let mut header = HeaderMap::new();
    ///     header.append("Accept","text/html".parse().unwrap());
    ///     let resp = HttpClient::get("http://www.example.com/", None, Some(header)).expect("failed");
    ///     assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
    /// }
    ///
    /// pub fn simple_get_with_param(){
    ///     let mut header = HeaderMap::new();
    ///     header.append("Accept","text/html".parse().unwrap());
    ///     let resp = HttpClient::get("http://www.example.com/?admin=yes&show=yes",None, Some(header)).expect("failed");
    ///     assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
    /// }
    /// ```
    pub fn get(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::GET, url, body, headers)
    }
    /// do http post request
    /// ```
    /// use request_rs::produce::*;
    /// use url::Url;
    /// use request_rs::headers::HeaderMap;
    /// pub fn simple_post_with_data(){
    ///     let body = Body::from_str("username=admin&password123");
    ///     let mut header = HeaderMap::new();
    ///     header.append("Accept","text/html".parse().unwrap());
    ///     let resp = HttpClient::post("http://www.example.com/",Some(body), Some(header)).expect("failed");
    ///     assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
    /// }
    /// ```
    pub fn post(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::POST, url, body, headers)
    }

    /// do http put request
    pub fn put(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::PUT, url, body, headers)
    }

    /// do http delete request
    pub fn delete(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::DELETE, url, body, headers)
    }
    /// do http head request
    pub fn head(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::HEAD, url, body, headers)
    }

    /// do http patch request
    pub fn patch(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::PATCH, url, body, headers)
    }

    /// do http connect request
    pub fn connect(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::CONNECT, url, body, headers)
    }

    /// do http options request
    pub fn options(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::OPTIONS, url, body, headers)
    }

    /// do http trace request
    pub fn trace(url: &str, body: Option<Body>, headers: Option<HeaderMap>) -> Result<Response<Body>> {
        Self::request(Method::TRACE, url, body, headers)
    }
}
