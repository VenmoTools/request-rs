use crate::cookie::cookies::Cookie;

mod cookies;
mod cookie_jar;

pub trait CookieJar {
    fn cookie(&self, name: &str) -> Cookie;
}