use std::error;
use std::fmt;
use std::net::AddrParseError;
use std::result;
use std::string::FromUtf8Error;

use crate::header;
use crate::header::ToStrError;
use crate::method;
use crate::status;

macro_rules! from_error {
    ($from:ty,$to:expr) => {
        impl From<$from> for Error{
            fn from(err: $from) -> Self {
                Error{
                    inner: $to(err)
                }
            }
        }
    };
}

macro_rules! impl_error {
    ($err_ty:ty) => {
        impl std::error::Error for $err_ty{}
    };
}

/// A generic "error" for HTTP connections
///
/// This error type is less specific than the error returned from other
/// functions in this crate, but all other errors can be converted to this
/// error. Consumers of this crate can typically consume and work with this form
/// of error for conversions with the `?` operator.
#[derive(Clone)]
pub struct Error {
    inner: ErrorKind,
}

/// A `Result` typedef to use with the `http::Error` type
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct InvalidUrl {
    msg: String,
}

impl InvalidUrl {
    pub fn new(msg: &str) -> Self {
        Self { msg: msg.to_string() }
    }
}

impl fmt::Display for InvalidUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.msg.as_str())
    }
}


#[derive(Debug, Clone, Copy)]
pub struct IoError {
    repr: std::io::ErrorKind,
}

impl IoError {
    pub fn as_str(&self) -> &'static str {
        use std::io::ErrorKind::*;
        match self.repr {
            NotFound => "entity not found",
            PermissionDenied => "permission denied",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            ConnectionAborted => "connection aborted",
            NotConnected => "not connected",
            AddrInUse => "address in use",
            AddrNotAvailable => "address not available",
            BrokenPipe => "broken pipe",
            AlreadyExists => "entity already exists",
            WouldBlock => "operation would block",
            InvalidInput => "invalid input parameter",
            InvalidData => "invalid data",
            TimedOut => "timed out",
            WriteZero => "write zero",
            Interrupted => "operation interrupted",
            Other => "other os error",
            UnexpectedEof => "unexpected end of file",
            _ => "invalid error"
        }
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl IoError {
    pub fn from_kind(repr: std::io::ErrorKind) -> Self {
        Self {
            repr
        }
    }
}

#[derive(Debug, Clone)]
pub struct InvalidHttpVersion {
    msg: String,
}

impl InvalidHttpVersion {
    pub fn new(msg: &str) -> Self {
        Self { msg: msg.to_string() }
    }
}

impl fmt::Display for InvalidHttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.msg.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct InvalidHttpHeader {
    msg: String,
}

impl InvalidHttpHeader {
    pub fn new(msg: &str) -> Self {
        Self { msg: msg.to_string() }
    }
}

impl fmt::Display for InvalidHttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.msg.as_str())
    }
}


#[derive(Clone)]
enum ErrorKind {
    StatusCode(status::InvalidStatusCode),
    Method(method::InvalidMethod),
    Uri(url::ParseError),
    SocketParseError(AddrParseError),
    InvalidUrl(InvalidUrl),
    InvalidHttpVersion(InvalidHttpVersion),
    HeaderName(header::InvalidHeaderName),
    HeaderValue(header::InvalidHeaderValue),
    ToStrError(ToStrError),
    IoError(IoError),
    FromUtf8Error(FromUtf8Error),
    InvalidHttpHeader(InvalidHttpHeader),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("http::Error")
            // Skip the noise of the ErrorKind enum
            .field(&self.get_ref())
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.get_ref(), f)
    }
}

impl Error {
    /// Return true if the underlying error has the same type as T.
    pub fn is<T: error::Error + 'static>(&self) -> bool {
        self.get_ref().is::<T>()
    }

    /// Return a reference to the lower level, inner error.
    pub fn get_ref(&self) -> &(dyn error::Error + 'static) {
        use self::ErrorKind::*;
        match self.inner {
            StatusCode(ref e) => e,
            Method(ref e) => e,
            Uri(ref e) => e,
            HeaderName(ref e) => e,
            HeaderValue(ref e) => e,
            InvalidUrl(ref e) => e,
            ToStrError(ref e) => e,
            SocketParseError(ref e) => e,
            IoError(ref e) => e,
            FromUtf8Error(ref e) => e,
            InvalidHttpVersion(ref e) => e,
            InvalidHttpHeader(ref e) => e,
        }
    }
}

impl error::Error for Error {
    // Return any available cause from the inner error. Note the inner error is
    // not itself the cause.
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.get_ref().source()
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            inner: ErrorKind::IoError(IoError::from_kind(err.kind()))
        }
    }
}

impl_error!(InvalidHttpVersion);
impl_error!(InvalidUrl);
impl_error!(IoError);
impl_error!(InvalidHttpHeader);

from_error!(InvalidHttpHeader,ErrorKind::InvalidHttpHeader);
from_error!(IoError,ErrorKind::IoError);
from_error!(InvalidUrl,ErrorKind::InvalidUrl);
from_error!(InvalidHttpVersion,ErrorKind::InvalidHttpVersion);
from_error!(ToStrError,ErrorKind::ToStrError);
from_error!(status::InvalidStatusCode,ErrorKind::StatusCode);
from_error!(method::InvalidMethod,ErrorKind::Method);
from_error!(url::ParseError,ErrorKind::Uri);
from_error!(header::InvalidHeaderName,ErrorKind::HeaderName);
from_error!(header::InvalidHeaderValue,ErrorKind::HeaderValue);
from_error!(AddrParseError,ErrorKind::SocketParseError);
from_error!(FromUtf8Error,ErrorKind::FromUtf8Error);

impl From<std::convert::Infallible> for Error {
    fn from(err: std::convert::Infallible) -> Error {
        match err {}
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_error_is_invalid_status_code() {
        if let Err(e) = status::StatusCode::from_u16(6666) {
            let err: Error = e.into();
            let ie = err.get_ref();
            assert!(!ie.is::<header::InvalidHeaderValue>());
            assert!(ie.is::<status::InvalidStatusCode>());
            ie.downcast_ref::<status::InvalidStatusCode>().unwrap();

            assert!(!err.is::<header::InvalidHeaderValue>());
            assert!(err.is::<status::InvalidStatusCode>());
        } else {
            panic!("Bad status allowed!");
        }
    }
}
