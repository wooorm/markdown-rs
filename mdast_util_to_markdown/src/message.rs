use core::{error::Error, fmt::Display};

use alloc::string::{String, ToString};

#[derive(Debug, PartialEq)]
pub struct Message {
    pub reason: String,
}

impl Error for Message {}

impl Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        Message {
            reason: value.to_string(),
        }
    }
}
