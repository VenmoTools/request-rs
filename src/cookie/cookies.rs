use std::process::exit;
use std::time;
use std::time::Duration;

/// Cookie Version 0
#[derive(Debug)]
pub struct Cookie {
    version: Option<usize>,
    name: String,
    value: String,
    domain: Option<String>,
    path: Option<String>,
    secure: bool,
    expires: Option<Duration>,
}

impl Cookie {
    pub fn is_expired(&self) -> bool {
        let now = time::SystemTime::now();
        let now = now.elapsed().unwrap();
        if let Some(expires) = self.expires {
            expires <= now
        } else {
            true
        }
    }
}

