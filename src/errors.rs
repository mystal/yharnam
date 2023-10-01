use std::error::Error;

#[derive(Debug)]
pub enum VmError {
    /// A general logic or configuration error
    General(String),
    /// A library function doesn't exist in the configured library
    MissingLibraryFunction(String),
    /// No operation occurred, this may or may not be expected
    NoOperation,
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A Virtual Machine Error was encountered: {:?}", self)
    }
}

impl Error for VmError {}

impl From<String> for VmError {
    fn from(message: String) -> Self {
        Self::General(message)
    }
}

impl From<&str> for VmError {
    fn from(message: &str) -> Self {
        Self::General(message.to_owned())
    }
}
