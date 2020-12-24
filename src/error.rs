#[derive(Debug)]
pub struct Error {
    message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }
}

impl From<yaml_rust::ScanError> for Error {
    fn from(err: yaml_rust::ScanError) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }
}
