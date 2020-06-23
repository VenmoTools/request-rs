/// macro for http header
/// # Usage
/// ```
/// use request_rs::headers::HeaderMap;
/// use request_rs::method::Method;
/// use request_rs::status::StatusCode;
/// pub fn simple_get_with_header(){
///     let header = http_header! {
///         "Accept" => "text/html",
///         "Host" => "www.example.com",
///     };
///     let resp = HttpClient::request(Method::GET,"http://www.example.com/", None, Some(header)).expect("failed");
///     assert_eq!(StatusCode::from_u16(200).unwrap(), resp.status());
/// }
/// ```
#[macro_export]
macro_rules! http_header {

    (@item $($x:tt)*) => (());

    (@count $($key:expr),*) => (<[()]>::len(&[$(http_header!(@item $key)),*]));

    ($($key:expr => $value:expr),*$(,)*) => {
        {
            let len = http_header! (@count $($key),*);
            let mut __inner_map: crate::header::HeaderMap<crate::header::HeaderValue> = crate::header::HeaderMap::with_capacity(len);
            $(
                __inner_map.append($key,$value.parse().unwrap());
            )*
            __inner_map
        }
    };
}

