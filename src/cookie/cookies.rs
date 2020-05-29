use std::time::Duration;

/// Cookie Version 0
#[derive(Debug)]
pub struct Cookie {
    name: String,
    value: String,
    domain: Option<String>,
    path: Option<String>,
    secure: bool,
    expires: Option<Duration>,
}