use std::fmt::Display;

#[derive(Debug)]
pub struct PressError {
    message: String,
}

impl PressError {
    pub fn new(message: &str) -> PressError {
        PressError {
            message: message.to_string(),
        }
    }
}

impl Display for PressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.message))
    }
}

impl std::error::Error for PressError {}

pub type PressResult<T> = std::result::Result<T, PressError>;

impl From<std::io::Error> for PressError {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }
}

impl From<toml::de::Error> for PressError {
    fn from(err: toml::de::Error) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }
}

impl From<yaml_rust::ScanError> for PressError {
    fn from(err: yaml_rust::ScanError) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }
}
