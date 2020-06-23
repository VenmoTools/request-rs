// #![doc(html_root_url = "")]

//! A simple sync HTTP Client library, Support HTTP/1.0 and HTTP/1.1
//! This crate is base on [http](https://github.com/hyperium/http), thanks for
//! the author of the crate.

#![deny(warnings, missing_docs, missing_debug_implementations)]


#[macro_use]
extern crate log;

pub use url;

#[macro_use]
mod convert;
mod header;
pub mod status;
pub mod version;
pub mod method;
pub mod request;
pub mod response;
pub mod error;
mod extensions;
mod client;
mod byte_str;
mod cookie;
pub mod macros;
mod proto;
mod body;

/// http configuration
pub mod config {
    /// for http 1.*
    pub mod h1 {
        pub use crate::proto::HttpConfig;
    }
}

/// http headers
pub mod headers {
    pub use crate::header::{ACCEPT, ACCEPT_CHARSET, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCEPT_RANGES,
                            ACCESS_CONTROL_ALLOW_CREDENTIALS,
                            ACCESS_CONTROL_ALLOW_HEADERS,
                            ACCESS_CONTROL_ALLOW_METHODS,
                            ACCESS_CONTROL_ALLOW_ORIGIN,
                            ACCESS_CONTROL_EXPOSE_HEADERS,
                            ACCESS_CONTROL_MAX_AGE,
                            ACCESS_CONTROL_REQUEST_HEADERS,
                            ACCESS_CONTROL_REQUEST_METHOD,
                            AGE,
                            ALLOW,
                            ALT_SVC,
                            AUTHORIZATION,
                            CACHE_CONTROL,
                            CONNECTION,
                            CONTENT_DISPOSITION,
                            CONTENT_ENCODING,
                            CONTENT_LANGUAGE,
                            CONTENT_LENGTH,
                            CONTENT_LOCATION,
                            CONTENT_RANGE,
                            CONTENT_SECURITY_POLICY,
                            CONTENT_SECURITY_POLICY_REPORT_ONLY,
                            CONTENT_TYPE,
                            COOKIE,
                            DATE,
                            DNT,
                            Entry,
                            ETAG,
                            EXPECT,
                            EXPIRES,
                            FORWARDED,
                            FROM,
                            HeaderMap,
                            HeaderName,
                            HeaderValue,
                            HOST,
                            IF_MATCH,
                            IF_MODIFIED_SINCE,
                            IF_NONE_MATCH,
                            IF_RANGE,
                            IF_UNMODIFIED_SINCE,
                            LAST_MODIFIED,
                            LINK,
                            LOCATION,
                            MAX_FORWARDS,
                            ORIGIN,
                            PRAGMA,
                            PROXY_AUTHENTICATE,
                            PROXY_AUTHORIZATION,
                            PUBLIC_KEY_PINS,
                            PUBLIC_KEY_PINS_REPORT_ONLY,
                            RANGE,
                            REFERER,
                            REFERRER_POLICY,
                            REFRESH,
                            RETRY_AFTER,
                            SEC_WEBSOCKET_ACCEPT,
                            SEC_WEBSOCKET_EXTENSIONS,
                            SEC_WEBSOCKET_KEY,
                            SEC_WEBSOCKET_PROTOCOL,
                            SEC_WEBSOCKET_VERSION,
                            SERVER,
                            SET_COOKIE,
                            STRICT_TRANSPORT_SECURITY,
                            TE,
                            TRAILER,
                            TRANSFER_ENCODING,
                            UPGRADE,
                            UPGRADE_INSECURE_REQUESTS,
                            USER_AGENT,
                            VARY,
                            VIA,
                            WARNING,
                            WWW_AUTHENTICATE,
                            X_CONTENT_TYPE_OPTIONS,
                            X_DNS_PREFETCH_CONTROL,
                            X_FRAME_OPTIONS,
                            X_XSS_PROTECTION, };
}

/// simple
pub mod produce {
    pub use url::{ParseError, Url};

    pub use crate::body::{Body, BodyKind};
    pub use crate::client::HttpClient;
    pub use crate::error::{Error, Result};
    pub use crate::extensions::Extensions;
    pub use crate::method::Method;
    pub use crate::proto::Connector;
    pub use crate::request::{Builder, Request};
    pub use crate::response::Response;
    pub use crate::status::StatusCode;
    pub use crate::version::Version;
}


fn _assert_types() {
    use produce::*;
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<Request<()>>();
    assert_send::<Response<()>>();

    assert_sync::<Request<()>>();
    assert_sync::<Response<()>>();
}

mod test {
    #[test]
    pub fn header_macros() {
        use crate::http_header;
        let header = http_header! {
            "Content-Type" => "text/html",
            "Host" => "cn.bing.com",
        };
        assert_eq!("text/html", header["Content-Type"]);
        assert_eq!("cn.bing.com", header["Host"]);
    }
}