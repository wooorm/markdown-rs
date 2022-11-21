use alloc::string::String;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parser error: {0}")]
    Parser(String),
    #[error("Compiler error: {0}")]
    Compiler(String),
    #[cfg(feature = "serde")]
    #[error("Serialization error: {0}")]
    Serialization(serde_json::Error),
}
