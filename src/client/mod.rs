use std::collections::BTreeMap;
use std::fmt::Write as fmtWrite;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::Lines;
use std::time::Duration;

use bytes::{Buf, BufMut, BytesMut};
use regex::Regex;
use url::Url;

use crate::error::{InvalidHttpHeader, InvalidUrl, IoError, Result};
use crate::error::Error;
use crate::method::Method;
use crate::request::Request;
use crate::response;
use crate::response::Response;
use crate::version::Version;

#[derive(Debug)]
pub struct Client {
    buf: BytesMut,
    timeout: Option<Duration>,
    url: Url,
}

impl Client {
    /// create client only parse request
    /// ```
    /// use request_rs::produce::*;
    /// use url::quirks::host;
    /// use url::Url;
    ///
    /// let mut req = Request::builder()
    ///             .method(Method::GET)
    ///             .uri(Url::parse("https://www.example.com").expect("invalid url"))
    ///             .header("User-Agent", "request-rs")
    ///             .header("Host", host)
    ///             .version(Version::HTTP_11)
    ///             .body(Vec::new());
    /// let mut client = Client::new(&req).expect("failed");
    /// let response = client.send().expect("request failed");
    /// assert_eq!(response.status(),StatusCode(200));
    /// ```
    pub fn new(req: &Request<Vec<u8>>) -> Result<Self> {
        Self::ready(req, None)
    }

    /// create client with timeout
    /// ```
    /// use request_rs::produce::*;
    /// use url::quirks::host;
    /// use url::Url;
    /// use std::time::Duration;
    ///
    /// let mut req = Request::builder()
    ///             .method(Method::GET)
    ///             .uri(Url::parse("https://www.example.com"))
    ///             .header("User-Agent", "request-rs")
    ///             .header("Host", host)
    ///             .version(Version::HTTP_11)
    ///             .body(Vec::new());
    /// let mut client = Client::with_timeout(&req,Duration::from_secs(30)).expect("failed");
    /// let response = client.send().expect("request failed");
    /// assert_eq!(response.status(),StatusCode(200));
    /// ```
    pub fn with_timeout(req: &Request<Vec<u8>>, timeout: Duration) -> Result<Self> {
        Self::ready(req, Some(timeout))
    }

    /// helper function for `new` `with_timeout`
    fn ready(req: &Request<Vec<u8>>, timeout: Option<Duration>) -> Result<Self> {
        let url = req.uri().ok_or(Error::from(InvalidUrl::new("missing url")))?.clone();
        let mut client = Self {
            buf: BytesMut::new(),
            timeout,
            url,
        };
        client.ready_start_line(req);
        client.ready_headers(req)?;
        client.end_of_headers();
        client.ready_body(req);
        Ok(client)
    }

    /// ready for request headers
    fn ready_headers<T>(&mut self, req: &Request<T>) -> Result<()> {
        for (name, value) in req.headers() {
            self.buf.write_fmt(format_args!("{}: {}\r\n", name.as_str(), value.to_str()?)).expect("failed write data to buffer");
        }
        Ok(())
    }

    /// ready request body
    fn ready_body(&mut self, req: &Request<Vec<u8>>) {
        let body = req.body();
        for data in body.iter() {
            self.buf.put_u8(*data);
        }
    }
    /// write \r\n
    fn end_of_headers(&mut self) {
        self.buf.write_str("\r\n").expect("failed write data to buffer");
    }

    /// ready for request start line
    fn ready_start_line<T>(&mut self, req: &Request<T>) {
        // [Method Path Version]
        self.buf.write_fmt(format_args!("{} {} {}\r\n", req.method().as_str(), self.url.path(), req.version().as_str())).unwrap();
    }

    /// get socket address from given url
    fn socket_addr(&self) -> Result<SocketAddr> {
        let scheme = self.url.scheme();
        // get request host and port
        let addr = self.url.socket_addrs(|| match scheme {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        })?.into_iter()
            .next()
            .ok_or(Error::from(InvalidUrl::new("invalid url")))?;
        Ok(addr)
    }

    /// send request
    pub fn send(&mut self) -> Result<Response<Vec<String>>> {
        let addr = self.socket_addr()?;
        // connect to target host use timeout steam if possible
        let mut steam = if let Some(time) = self.timeout {
            TcpStream::connect_timeout(&addr, time)
        } else {
            TcpStream::connect(&addr)
        }?;
        // first, send request data
        Self::send_data(&mut steam, self.buf.bytes())?;
        // then, recv data from host
        let data = Self::recv_data(&mut steam)?;
        //todo: how to process the response when body is binary data?
        let resp_data = String::from_utf8(data)?;
        // analysis each line
        let mut iter = resp_data.lines();
        // make response
        Self::make_response(&mut iter)
    }

    /// make response from received data
    fn make_response(iter: &mut Lines) -> Result<Response<Vec<String>>> {
        // make response
        let response = Response::builder();
        // parse start line
        let mut response = Self::parse_start_line(response, iter)?;
        // parse headers return true if end of \r\n else iter.next() is None
        let end_of_header = Self::parse_header(&mut response, iter)?;
        // not received data completely
        if !end_of_header {
            return Err(Error::from(IoError::from_kind(ErrorKind::UnexpectedEof)));
        }
        // do not considered \r\n cause already removed in parse_header function
        let body = iter.map(|str| str.to_owned()).collect::<Vec<_>>();
        // other data is body
        response.body(body)
    }

    /// recv data from tcp steam
    fn recv_data(client: &mut TcpStream) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        let size = client.read_to_end(&mut vec)?;
        Self::warn_if_zero(size);
        Ok(vec)
    }

    /// check send/received data size if size is zero then show warning
    fn warn_if_zero(size: usize) {
        if size == 0 {
            warn!("no data send/received!");
        }
    }

    /// write data to tcp steam
    fn send_data(client: &mut TcpStream, data: &[u8]) -> Result<()> {
        let size = client.write(data)?;
        Self::warn_if_zero(size);
        Ok(())
    }

    /// parse http start line
    fn parse_start_line(resp: response::Builder, iter: &mut Lines) -> Result<response::Builder> {
        // parse start line  pattern -> [Version StatusCode Reason]
        let regx = Regex::new(r"(?P<version>[\w/.]{8}) (?P<status>[\d]+) (?P<reason>[\w ]+)").expect("invalid regex");
        let start_line = iter.next().ok_or(std::io::Error::from(ErrorKind::UnexpectedEof))?;
        let cap = regx.captures(start_line).ok_or(std::io::Error::from(ErrorKind::InvalidData))?;
        Ok(resp.version(cap.name("version").ok_or(std::io::Error::from(ErrorKind::InvalidData))?.as_str().parse::<Version>()?)
            .status(cap.name("status").ok_or(std::io::Error::from(ErrorKind::InvalidData))?.as_str()))
    }

    /// parse http headers split as \r\n
    fn parse_header(resp: &mut response::Builder, iter: &mut Lines) -> Result<bool> {
        // parse header
        let header_regx = Regex::new(r"(?P<key>[\w\-]+): ?(?P<value>.+)").expect("invalid regex");
        let mut end_of_header = false;
        loop {
            if let Some(content) = iter.next() {
                if content == "" {
                    end_of_header = true;
                    break;
                }
                let caps = header_regx.captures(content)
                    .ok_or(Error::from(InvalidHttpHeader::new(format!("invalid header: {}", content).as_str())))?;
                // get key
                let key = caps.name("key")
                    .ok_or(
                        Error::from(
                            InvalidHttpHeader::new(format!("invalid header: {}", content).as_str())
                        )
                    )?.as_str();
                // get value
                let value = caps.name("value")
                    .ok_or(
                        Error::from(InvalidHttpHeader::new(
                            format!("invalid header: {}", content).as_str())
                        )
                    )?.as_str();
                resp.add_header(key, value)?;
            } else {
                break;
            }
        }
        Ok(end_of_header)
    }

    /// do http request
    /// ```
    /// use request_rs::produce::*;
    ///
    /// ```
    pub fn request(method: Method, url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        let host = url.domain().ok_or(Error::from(InvalidUrl::new("invalid url")))?;
        let mut req = Request::builder()
            .method(method)
            .header("User-Agent", "request-rs")
            .header("Host", host)
            .version(Version::HTTP_11);
        if let Some(headers) = headers {
            for (key, value) in headers.iter() {
                req.add_header(key.to_owned(), value.to_owned())?;
            }
        }
        let req = req.uri(url)
            .header("Content-Length", body.len())
            .body(body)?;
        let mut client = Self::with_timeout(&req, Duration::from_secs(15))?;
        client.send()
    }

    /// do http get request
    ///
    /// ```
    /// use request_rs::produce::*;
    /// use url::Url;
    /// use std::collections::BTreeMap;
    /// pub fn simple_get(){
    ///     let resp = Client::get(Url::parse("http://www.example.com/").expect("failed"), Vec::new(), None).expect("failed");
    ///     assert_eq!(StatusCode(200), resp.status());
    /// }
    ///
    /// pub fn simple_get_with_header(){
    ///     let mut header = BTreeMap::new();
    ///     header.insert("Accept","text/html");
    ///     let resp = Client::get(Url::parse("http://www.example.com/").expect("failed"), Vec::new(), Some(header)).expect("failed");
    ///     assert_eq!(StatusCode(200), resp.status());
    /// }
    ///
    /// pub fn simple_get_with_param(){
    ///     let mut header = BTreeMap::new();
    ///     header.insert("Accept","text/html");
    ///     let resp = Client::get(Url::parse("http://www.example.com/?admin=yes&show=yes").expect("failed"),Vec::new(), Some(header)).expect("failed");
    ///     assert_eq!(StatusCode(200), resp.status());
    /// }
    /// ```
    pub fn get(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::GET, url, body, headers)
    }
    /// do http post request
    /// ```
    /// use request_rs::produce::*;
    /// use url::Url;
    /// use std::collections::BTreeMap;
    /// pub fn simple_post_with_data(){
    ///     let body = Vec::from("username=admin&password123".as_bytes());
    ///     let mut header = BTreeMap::new();
    ///     header.insert("Accept","text/html");
    ///     let resp = Client::post(Url::parse("http://www.example.com/").expect("failed"),body, Some(header)).expect("failed");
    ///     assert_eq!(StatusCode(200), resp.status());
    /// }
    /// ```
    pub fn post(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::POST, url, body, headers)
    }

    /// do http put request
    pub fn put(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::PUT, url, body, headers)
    }

    /// do http delete request
    pub fn delete(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::DELETE, url, body, headers)
    }
    /// do http head request
    pub fn head(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::HEAD, url, body, headers)
    }

    /// do http patch request
    pub fn patch(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::PATCH, url, body, headers)
    }

    /// do http connect request
    pub fn connect(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::CONNECT, url, body, headers)
    }

    /// do http options request
    pub fn options(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::OPTIONS, url, body, headers)
    }

    /// do http trace request
    pub fn trace(url: Url, body: Vec<u8>, headers: Option<BTreeMap<&str, &str>>) -> Result<Response<Vec<String>>> {
        Self::request(Method::TRACE, url, body, headers)
    }
}


mod tests {
    use std::collections::BTreeMap;

    use regex::Regex;
    use url::Url;

    use crate::client::Client;
    use crate::produce::*;

    #[test]
    pub fn test_parser() {
        let response = "HTTP/1.1 200 OK\r\nServer: nginx/1.12.2\r\nDate: Tue, 26 May 2020 00:22:59 GMT\r\nContent-Type: text/html; charset=UTF-8\r\nTransfer-Encoding: chunked\r\nConnection: keep-alive\r\nVary: Accept-Encoding\r\nX-Powered-By: PHP/7.1.14\r\nVary: Accept-Encoding, Cookie\r\nX-Pingback: http://www.selenium.org.cn/xmlrpc.php\r\nLink: <http://www.selenium.org.cn/wp-json/>; rel=\"https://api.w.org/\"\r\nLink: <http://www.selenium.org.cn/>; rel=shortlink\r\nX-Frame-Options: SAMEORIGIN\r\nContent-Encoding: gzip\r\n\r\n";
        let mut iter = response.lines();
        // test start line regex pattern
        let start_line_regx = Regex::new(r"(?P<version>[\w/.]{8}) (?P<status>[\d]+) (?P<reason>[\w ]+)").expect("invalid regex");
        let start_line = iter.next().expect("impossible");
        let cap = start_line_regx.captures(start_line).expect("can't find start line match pattern");
        assert_eq!("HTTP/1.1", cap.name("version").expect("version").as_str());
        assert_eq!("200", cap.name("status").expect("status").as_str());
        assert_eq!("OK", cap.name("reason").expect("reason").as_str());

        // test header regex pattern
        let header_regx = Regex::new(r"(?P<key>[\w\-]+): ?(?P<value>.+)").expect("invalid regex");
        let mut end_of_header = false;
        let mut headers = BTreeMap::new();
        loop {
            if let Some(content) = iter.next() {
                if content == "" {
                    end_of_header = true;
                    break;
                }
                let caps = header_regx.captures(content).expect("can't find header match pattern");
                let key = caps.name("key").expect("parse header key").as_str();
                let value = caps.name("value").expect("parse header value").as_str();
                headers.insert(key, value);
            } else {
                println!("impossible");
                break;
            }
        }
        assert!(end_of_header);
        let mut header_iter = headers.iter();
        assert_eq!(Some((&"Connection", &"keep-alive")), header_iter.next());
        assert_eq!(Some((&"Content-Encoding", &"gzip")), header_iter.next());
        assert_eq!(Some((&"Content-Type", &"text/html; charset=UTF-8")), header_iter.next());
        assert_eq!(Some((&"Date", &"Tue, 26 May 2020 00:22:59 GMT")), header_iter.next());
        assert_eq!(Some((&"Link", &"<http://www.selenium.org.cn/>; rel=shortlink")), header_iter.next());
        assert_eq!(Some((&"Server", &"nginx/1.12.2")), header_iter.next());
        assert_eq!(Some((&"Transfer-Encoding", &"chunked")), header_iter.next());
        // todo: process response header has multi key
        // assert_eq!(Some(("Vary","Accept-Encoding")),header_iter.next());
        // assert_eq!(Some(("Link","<http://www.selenium.org.cn/wp-json/>; rel=\"https://api.w.org/\"")),header_iter.next());
        assert_eq!(Some((&"Vary", &"Accept-Encoding, Cookie")), header_iter.next());
        assert_eq!(Some((&"X-Frame-Options", &"SAMEORIGIN")), header_iter.next());
        assert_eq!(Some((&"X-Pingback", &"http://www.selenium.org.cn/xmlrpc.php")), header_iter.next());
        assert_eq!(Some((&"X-Powered-By", &"PHP/7.1.14")), header_iter.next());
    }

    #[test]
    pub fn test_request() {
        let req = Request::builder()
            .version(Version::HTTP_11)
            .method(Method::GET)
            .uri(Url::parse("http://cn.bing.com/").expect("failed url"))
            .header("Host", "cn.bing.com")
            .body("username=admin&password=123".bytes().collect()).expect("build failed");
        let client = Client::new(&req).expect("bad reqeust");
        let except = "GET / HTTP/1.1\r\nhost: cn.bing.com\r\n\r\nusername=admin&password=123";
        let str = String::from_utf8(client.buf.to_vec()).expect("failed");
        assert_eq!(except, str.as_str());
    }

    #[test]
    fn test_convenient_request() {
        let resp = Client::get(Url::parse("http://www.cn.bing.com/").expect("failed"), Vec::new(), None).expect("failed");
        println!("{}", resp.status());
        println!("{:?}", resp.body());
    }
}