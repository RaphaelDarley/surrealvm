use thiserror::Error;

pub type SVMResult<T> = Result<T, SVMError>;

#[derive(Debug, Error)]
pub enum SVMError {
    #[error("{0}")]
    Thrown(String),
    // #[error("IO Error: {0}")]
    // Io(#[from] std::io::Error),
    // #[error("{0}")]
    // Other(Box<dyn std::error::Error>),
}

#[macro_export]
macro_rules! throw {
    ($err:expr) => {{
        use crate::error::SVMError;
        return Err(SVMError::Thrown($err.into()).into());
    }};
}

impl From<String> for SVMError {
    fn from(value: String) -> Self {
        SVMError::Thrown(value)
    }
}
impl From<&str> for SVMError {
    fn from(value: &str) -> Self {
        SVMError::Thrown(value.to_owned())
    }
}
// impl From<homedir::GetHomeError> for SVMError {
//     fn from(value: homedir::GetHomeError) -> Self {
//         SVMError::Other(Box::new(value))
//     }
// }
