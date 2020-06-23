use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use bytes::BytesMut;

pub use http1::conn::{HttpConfig, HttpConnector};
pub use http1::parse::{RequestParser, ResponseParser};

use crate::error::Result;

mod http1;
mod http2;

#[derive(Debug)]
pub enum ParserResult<T> {
    Complete(T),
    Partial,
}

/// the HttpClient inner type
/// usr can implement Connector and use it by HttpClient::from_connector()
pub trait Connector: Read + Write {
    /// connect to socket addr
    fn create_connection(&mut self, socket_addr: &SocketAddr) -> Result<TcpStream>;
    /// connect to socket addr
    fn connect_to(&mut self, addr: &SocketAddr) -> Result<()>;
}

pub trait HttpParser {
    type To;

    fn parse(&mut self, buf: &mut BytesMut) -> Result<ParserResult<Self::To>>;

    fn encode(from: Self::To) -> Result<BytesMut>;
}


