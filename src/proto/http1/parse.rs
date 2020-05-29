use std::fmt::Write;
use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};
use url::Url;

use crate::body::{Body, BodyKind};
use crate::body_kind;
use crate::error::{InvalidUrl, Result};
use crate::error::Error;
use crate::header::{CONNECTION, HeaderMap, HeaderName, HeaderValue, InvalidHeaderName};
use crate::proto::{HttpParser, ParserResult};
use crate::proto::http1::{connection_close, connection_keep_alive};
use crate::request::Request;
use crate::response::Response;
use crate::status::StatusCode;
use crate::version::Version;

const MAX_HEADERS: usize = 100;

pub struct ResponseParser {
    keep_alive: bool,
}

impl ResponseParser {
    pub fn new() -> Self {
        Self { keep_alive: false }
    }
}

impl HttpParser for ResponseParser {
    type To = Response<Body>;

    fn parse(&mut self, buf: &mut BytesMut) -> Result<ParserResult<Self::To>> {
        let mut headers_indices = [HeaderIndices::default(); MAX_HEADERS];

        let (len, status_code, version, header_len) = {
            let mut header = [httparse::EMPTY_HEADER; MAX_HEADERS];
            let mut resp = httparse::Response::new(&mut header);

            match resp.parse(buf.as_ref())? {
                httparse::Status::Complete(len) => {
                    let status_code = StatusCode::from_u16(resp.code.unwrap())?;
                    let version = if resp.version.unwrap_or(1) == 1 {
                        Version::HTTP_11
                    } else {
                        Version::HTTP_10
                    };
                    let header_len = resp.headers.len();

                    record_header_indices(buf.as_ref(), &mut header, &mut headers_indices)?;
                    (len, status_code, version, header_len)
                }
                httparse::Status::Partial => {
                    return Ok(ParserResult::Partial);
                }
            }
        };
        // immutable header buffer
        let headers_buf = buf.split_to(len).freeze();

        let mut header_map = HeaderMap::new();

        header_map.reserve(header_len);
        let mut keep_alive = version == Version::HTTP_11;

        for header in &headers_indices[..header_len] {
            let name = HeaderName::from_bytes(&headers_buf[header.name.start..header.name.end])?;
            // Unsafe: httparse already validated header value
            let value = unsafe { HeaderValue::from_maybe_shared_unchecked(headers_buf.slice(header.value.start..header.value.end)) };
            // need keep alive?
            if let CONNECTION = name {
                if keep_alive {
                    keep_alive = !connection_close(&value);
                } else {
                    keep_alive = connection_keep_alive(&value);
                }
            }
            header_map.append(name, value);
        }
        self.keep_alive = keep_alive;

        let body = BytesMut::from(&buf[header_len..]);
        let parsed_rep = Response::builder()
            .version(version)
            .status(status_code)
            .set_header_map(header_map)
            .body(Body::new(BodyKind::Binary(body)))?;

        Ok(ParserResult::Complete(parsed_rep))
    }

    fn encode(_from: Self::To) -> Result<BytesMut> {
        unreachable!("response don't need encode for http client");
    }
}

pub struct RequestParser;

impl RequestParser {
    /// get socket address from given url
    pub fn socket_addr(url: &Url) -> Result<SocketAddr> {
        let scheme = url.scheme();
        // get request host and port
        let addr = url.socket_addrs(|| match scheme {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        })?.into_iter()
            .next()
            .ok_or(Error::from(InvalidUrl::new("invalid url")))?;
        Ok(addr)
    }

    /// helper function for `new` `with_timeout`
    fn ready(req: &Request<Body>) -> Result<BytesMut> {
        let url = req.uri().ok_or(Error::from(InvalidUrl::new("missing url")))?.clone();
        let mut buf = BytesMut::new();
        Self::ready_start_line(&mut buf, req, &url);
        Self::ready_headers(&mut buf, req)?;
        Self::end_of_headers(&mut buf);
        Self::ready_body(&mut buf, req)?;
        Ok(buf)
    }

    /// write \r\n
    fn end_of_headers(buf: &mut BytesMut) {
        buf.write_str("\r\n").expect("failed write data to buffer");
    }

    /// ready for request start line
    fn ready_start_line(buf: &mut BytesMut, req: &Request<Body>, url: &Url) {
        if let Some(query) = url.query() {
            buf.write_fmt(format_args!("{} {}?{} {}\r\n", req.method().as_str(), url.path(), query, req.version().as_str())).unwrap();
        } else {
            // [Method Path Version]
            buf.write_fmt(format_args!("{} {} {}\r\n", req.method().as_str(), url.path(), req.version().as_str())).unwrap();
        }
    }


    fn ready_body(buf: &mut BytesMut, req: &Request<Body>) -> Result<()> {
        body_kind!(req.body().kind(),
            text => {
                buf.write_str(text.as_str()).unwrap();
            },
            binary => {
                buf.put(binary.as_ref());
            },
            empty => {

            }
        );
        Ok(())
    }

    /// ready for request headers
    fn ready_headers(buf: &mut BytesMut, req: &Request<Body>) -> Result<()> {
        for (name, value) in req.headers() {
            buf.write_fmt(format_args!("{}: {}\r\n", name.as_str(), value.to_str()?)).expect("failed write data to buffer");
        }
        Ok(())
    }
}

impl HttpParser for RequestParser {
    type To = Request<Body>;

    fn parse(&mut self, _buf: &mut BytesMut) -> Result<ParserResult<Self::To>> {
        unreachable!("request don't need parse for http client");
    }

    fn encode(from: Self::To) -> Result<BytesMut> {
        let req = Self::ready(&from)?;

        Ok(req)
    }
}

#[derive(Clone, Copy, Default)]
struct HeaderIndices {
    name: Range,
    value: Range,
}

#[derive(Clone, Copy, Default)]
struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

fn record_header_indices(
    bytes: &[u8],
    headers: &[httparse::Header<'_>],
    indices: &mut [HeaderIndices],
) -> Result<()> {
    let bytes_ptr = bytes.as_ptr() as usize;

    for (header, indices) in headers.iter().zip(indices.iter_mut()) {
        if header.name.len() >= (1 << 16) {
            debug!("header name larger than 64kb: {:?}", header.name);
            return Err(Error::from(InvalidHeaderName::new()));
        }
        let name_start = header.name.as_ptr() as usize - bytes_ptr;
        let name_end = name_start + header.name.len();
        indices.name = Range::new(name_start, name_end);

        let value_start = header.value.as_ptr() as usize - bytes_ptr;
        let value_end = value_start + header.value.len();
        indices.value = Range::new(value_start, value_end);
    }

    Ok(())
}
