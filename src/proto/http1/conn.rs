use std::borrow::{Borrow, BorrowMut};
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

use net2::{TcpBuilder, TcpStreamExt};

use crate::body::{Body, BodyKind};
use crate::body_kind;
use crate::error::Result;
use crate::proto::Connector;

/// the tcp configuration for http client
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// if is None use default time
    pub connect_timeout: Option<Duration>,
    ///
    pub happy_eyeballs_timeout: Option<Duration>,
    /// if is None use default time
    pub keep_alive_timeout: Option<Duration>,
    /// if is None use System given ip address(127.0.0.1)
    pub local_address: Option<IpAddr>,
    /// not delay
    pub nodelay: bool,
    /// tcp connector will reuse ip address and port if `reuse_address` is true
    pub reuse_address: bool,
    /// tcp send buffer size default size if is None
    pub send_buffer_size: Option<usize>,
    /// tcp received buffer size default size if is None
    pub recv_buffer_size: Option<usize>,
    ///
    pub ttl: u32,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            connect_timeout: None,
            happy_eyeballs_timeout: Some(Duration::from_millis(300)),
            keep_alive_timeout: None,
            local_address: None,
            nodelay: false,
            reuse_address: false,
            send_buffer_size: None,
            recv_buffer_size: None,
            ttl: 64,
        }
    }
}

/// Simplified `hyper::HttpConnector`
#[derive(Debug)]
pub struct HttpConnector {
    config: HttpConfig,
    stream: Option<TcpStream>,
}

impl HttpConnector {
    /// Construct a new HttpConnector.
    pub fn new() -> Self {
        Self {
            config: HttpConfig::default(),
            stream: None,
        }
    }

    /// open tcp stream
    pub fn open_stream(addr: &SocketAddr) -> Result<Self> {
        let mut connector = Self::new();
        connector.connect_to(addr)?;
        Ok(connector)
    }

    /// open tcp stream
    pub fn open_stream_with_config(addr: &SocketAddr, config: HttpConfig) -> Result<Self> {
        let mut connector = Self::with_http_config(config);
        let stream = connector.create_connection(addr)?;
        connector.stream = Some(stream);
        Ok(connector)
    }

    /// Read the exact number of bytes required to fill `buf`.
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.read_exact(buf)?;
            return Ok(());
        }
        panic!("no connection opened, please open connection first")
    }

    /// read all
    pub fn read_all(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        if let Some(ref mut stream) = self.stream {
            let size = stream.read_to_end(buf)?;
            return Ok(size);
        }
        panic!("no connection opened, please open connection first")
    }


    /// write all
    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.write_all(buf)?;
            return Ok(());
        }
        panic!("no connection opened, please open connection first")
    }
    /// write body
    pub fn write_body(&mut self, body: Body) -> Result<()> {
        body_kind!(body.kind(),
            text => {
                self.write_all(text.as_bytes())?
            },
            binary => {
                self.write_all(binary.as_ref())?
            },
            empty => {

            }
        );
        Ok(())
    }

    /// Construct a new HttpConnector use given http config
    pub fn with_http_config(config: HttpConfig) -> Self {
        Self {
            config,
            stream: None,
        }
    }

    /// Set that all sockets have `SO_KEEPALIVE` set with the supplied duration.
    ///
    /// If `None`, the option will not be set.
    ///
    /// Default is `None`.
    #[inline]
    pub fn set_keepalive(&mut self, dur: Option<Duration>) {
        self.config_mut().keep_alive_timeout = dur;
    }

    ///
    #[inline]
    pub fn set_ttl(&mut self, ttl: u32) {
        self.config_mut().ttl = ttl;
    }

    /// Set that all sockets have `SO_NODELAY` set to the supplied value `nodelay`.
    ///
    /// Default is `false`.
    #[inline]
    pub fn set_nodelay(&mut self, nodelay: bool) {
        self.config_mut().nodelay = nodelay;
    }

    /// Set that all sockets are bound to the configured address before connection.
    ///
    /// If `None`, the sockets will not be bound.
    ///
    /// Default is `None`.
    #[inline]
    pub fn set_local_address(&mut self, addr: Option<IpAddr>) {
        self.config_mut().local_address = addr;
    }

    /// Sets the value of the SO_SNDBUF option on the socket.
    #[inline]
    pub fn set_send_buffer_size(&mut self, size: Option<usize>) {
        self.config_mut().send_buffer_size = size;
    }

    /// Sets the value of the SO_RCVBUF option on the socket.
    #[inline]
    pub fn set_recv_buffer_size(&mut self, size: Option<usize>) {
        self.config_mut().recv_buffer_size = size;
    }

    /// Set the connect timeout.
    ///
    /// If a domain resolves to multiple IP addresses, the timeout will be
    /// evenly divided across them.
    ///
    /// Default is `None`.
    #[inline]
    pub fn set_connect_timeout(&mut self, dur: Option<Duration>) {
        self.config_mut().connect_timeout = dur;
    }

    /// Set timeout for [RFC 6555 (Happy Eyeballs)][RFC 6555] algorithm.
    ///
    /// If hostname resolves to both IPv4 and IPv6 addresses and connection
    /// cannot be established using preferred address family before timeout
    /// elapses, then connector will in parallel attempt connection using other
    /// address family.
    ///
    /// If `None`, parallel connection attempts are disabled.
    ///
    /// Default is 300 milliseconds.
    ///
    /// [RFC 6555]: https://tools.ietf.org/html/rfc6555
    #[inline]
    pub fn set_happy_eyeballs_timeout(&mut self, dur: Option<Duration>) {
        self.config_mut().happy_eyeballs_timeout = dur;
    }
    /// private
    fn config_mut(&mut self) -> &mut HttpConfig {
        self.config.borrow_mut()
    }
    /// private
    fn config(&self) -> &HttpConfig {
        self.config.borrow()
    }
}


impl Connector for HttpConnector {
    fn create_connection(&mut self, socket_addr: &SocketAddr) -> Result<TcpStream> {
        let config = self.config();
        // use net2 crate to build Tcp Stream
        let tcp_builder = match socket_addr {
            SocketAddr::V4(_) => TcpBuilder::new_v4(),
            SocketAddr::V6(_) => TcpBuilder::new_v6(),
        }?;
        //  Set value for the `SO_REUSEADDR` option on this socket
        if config.reuse_address {
            tcp_builder.reuse_address(true)?;
        }
        // ttl
        tcp_builder.ttl(config.ttl)?;
        if let Some(ref local) = config.local_address {
            // let system chose port
            tcp_builder.bind(SocketAddr::new(local.clone(), 0))?;
        }
        let stream = tcp_builder.connect(socket_addr)?;
        stream.set_write_timeout(config.connect_timeout.clone())?;
        stream.set_read_timeout(config.connect_timeout.clone())?;
        stream.set_nodelay(config.nodelay)?;
        stream.set_keepalive(config.keep_alive_timeout.clone())?;
        Ok(stream)
    }

    fn connect_to(&mut self, addr: &SocketAddr) -> Result<()> {
        let stream = self.create_connection(addr)?;
        self.stream = Some(stream);
        Ok(())
    }
}

impl Read for HttpConnector {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref mut stream) = self.stream {
            return stream.read(buf);
        }
        panic!("read failed! no connection opened, please open connection first")
    }
}

impl Write for HttpConnector {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(ref mut stream) = self.stream {
            return stream.write(buf);
        }
        panic!("write failed! no connection opened, please open connection first")
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            return stream.flush();
        }
        panic!("flush failed! no connection opened, please open connection first")
    }
}
