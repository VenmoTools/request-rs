pub use client::HttpClient;

mod client;

// A basic Http request will take the following steps
// for example we request http://www.example.com:8080/ with GET method
// The first step is to resolve the hostname
// query the IP address of the hostname (`www.example.com` will resolve to 202.43.78.3 )
// get the port of the requesting host(HTTP usually use 80, HTTPS usually use 443)
// then client connect 202.43.78.3:8080 and send a GET Request to host
// once completed, the client will receive response data from the host
// finally client close the connection.

mod tests {
    use crate::produce::*;

    #[test]
    pub fn test_request() {
        let req = Request::builder()
            .version(Version::HTTP_11)
            .method(Method::GET)
            .uri(Url::parse("http://cn.bing.com/").expect("failed url"))
            .header("Host", "cn.bing.com")
            .body(Body::from_str("username=admin&password=123")).expect("build failed");
        let mut client = HttpClient::http();
        let resp = client.send(req).expect("request failed");
        assert_eq!(StatusCode::from_u16(200).expect(""), resp.status());
    }


    #[test]
    fn test_convenient_request() {
        let resp = HttpClient::get("http://www.baidu.com", None, None).expect("failed");
        assert_eq!(StatusCode::from_u16(200).expect(""), resp.status());
    }
}