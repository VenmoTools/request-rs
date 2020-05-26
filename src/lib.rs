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

pub mod produce {
    pub use url::{ParseError, Url};

    pub use crate::client::Client;
    pub use crate::error::{Error, Result};
    pub use crate::extensions::Extensions;
    pub use crate::method::Method;
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

mod tests {
    use crate::produce::*;

    #[test]
    pub fn response() {
        let req = Request::new(Vec::new());
        let mut client = Client::new(&req).expect("error");
        let _resp = client.send().expect("");
    }

    #[test]
    pub fn parse_url() {
        let url = Url::parse("http://www.baidu.com").expect("invalid url");
        let scheme = url.scheme();
        let addr = url.socket_addrs(|| match scheme {
            "http" => Some(80),
            "https" => Some(8080),
            _ => None,
        }).expect("invalid url").into_iter()
            .next().expect("invalid url");
        println!("{}", addr.ip());
        println!("{}", addr.port());
    }
}