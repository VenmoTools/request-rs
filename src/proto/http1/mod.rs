use crate::header::HeaderValue;

pub(crate) mod conn;
pub(crate) mod parse;


pub fn connection_keep_alive(value: &HeaderValue) -> bool {
    connection_has(value, "keep-alive")
}

pub fn connection_close(value: &HeaderValue) -> bool {
    connection_has(value, "close")
}

fn connection_has(value: &HeaderValue, needle: &str) -> bool {
    if let Ok(s) = value.to_str() {
        for val in s.split(',') {
            if val.trim().eq_ignore_ascii_case(needle) {
                return true;
            }
        }
    }
    false
}

mod parser_test {
    use bytes::BytesMut;
    use url::Url;

    use crate::body::{Body, BodyKind};
    use crate::error::Result;
    use crate::header::{AGE, CONNECTION, CONTENT_LENGTH, CONTENT_TYPE, DATE, SERVER};
    use crate::method::Method;
    use crate::proto::{HttpParser, ParserResult, RequestParser, ResponseParser};
    use crate::request::Request;
    use crate::status::StatusCode;

    #[test]
    pub fn test_request() -> Result<()> {
        // with username
        let body = Body::from_str("username=123");
        let req = Request::builder()
            .uri(Url::parse("http://www.baidu.com").unwrap())
            .method(Method::GET)
            .header("Host", "www.baidu.com")
            .header("User-Agent", "request-rs")
            .body(body)?;
        let resp = RequestParser::encode(req)?;
        let result = String::from_utf8_lossy(resp.as_ref());
        assert_eq!("GET / HTTP/1.1\r\nhost: www.baidu.com\r\nuser-agent: request-rs\r\n\r\nusername=123", result);

        let req = Request::builder()
            .uri(Url::parse("http://www.baidu.com/ath/aa?query=image&pass=yes").unwrap())
            .header("Host", "www.baidu.com")
            .header("User-Agent", "request-rs")
            .body(Body::empty())?;
        let resp = RequestParser::encode(req)?;
        let result = String::from_utf8_lossy(resp.as_ref());
        assert_eq!("GET /ath/aa?query=image&pass=yes HTTP/1.1\r\nhost: www.baidu.com\r\nuser-agent: request-rs\r\n\r\n", result);
        Ok(())
    }

    #[test]
    fn test_response() -> Result<()> {
        let resp = r#"HTTP/1.1 404 Not Found
Date: Fri, 29 May 2020 05:42:19 GMT
Content-Type: text/html
Content-Length: 150
Connection: keep-alive
Server: Tengine/2.1.1
Age: 1
Ws-S2h-Acc-Level: 1
X-Via: 1.1 PSfjfzsx3av123:4 (Cdn Cache Server V2.0), 1.1 PS-CKG-01rzq56:7 (Cdn Cache Server V2.0), 1.1 PS-000-01bMW67:2 (Cdn Cache Server V2.0)
X-Ws-Request-Id: 5ed0a0bb_PS-000-01bMW67_10521-64568
X-Cache-Webcdn: WS
Access-Control-Allow-Origin: *

<html>
<head><title>404 Not Found</title></head>
<body>
<center><h1>404 Not Found</h1></center>
<hr><center>openresty</center>
</body>
</html>"#;
        let mut buf = BytesMut::from(resp);
        let mut parser = ResponseParser::new();
        let response = parser.parse(&mut buf)?;
        if let ParserResult::Complete(data) = response {
            assert_eq!(StatusCode::from_u16(404)?, data.status());
            let headers = data.headers();
            assert_eq!("Fri, 29 May 2020 05:42:19 GMT", headers.get(DATE).unwrap());
            assert_eq!("text/html", headers.get(CONTENT_TYPE).unwrap());
            assert_eq!("150", headers.get(CONTENT_LENGTH).unwrap());
            assert_eq!("keep-alive", headers.get(CONNECTION).unwrap());
            assert_eq!("Tengine/2.1.1", headers.get(SERVER).unwrap());
            assert_eq!("1", headers.get(AGE).unwrap());
            assert_eq!("1", headers.get("Ws-S2h-Acc-Level").unwrap());
            assert_eq!("1.1 PSfjfzsx3av123:4 (Cdn Cache Server V2.0), 1.1 PS-CKG-01rzq56:7 (Cdn Cache Server V2.0), 1.1 PS-000-01bMW67:2 (Cdn Cache Server V2.0)", headers.get("X-Via").unwrap());
            assert_eq!("5ed0a0bb_PS-000-01bMW67_10521-64568", headers.get("X-Ws-Request-Id").unwrap());
            assert_eq!("WS", headers.get("X-Cache-Webcdn").unwrap());
            assert_eq!("*", headers.get("Access-Control-Allow-Origin").unwrap());
            let body = data.body();
            if let BodyKind::Text(text) = body.kind() {
                assert_eq!(r#"<html>
<head><title>404 Not Found</title></head>
<body>
<center><h1>404 Not Found</h1></center>
<hr><center>openresty</center>
</body>
</html>"#, text);
            }
        } else {
            panic!("parse error")
        }
        Ok(())
    }
}