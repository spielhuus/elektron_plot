#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Can not parse file.")]
    ParseError,
    #[error("Pin not found for {0}")]
    PinNotFound(u32),
    #[error("can not find symbol {0}.")]
    SymbolNotFound(String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("Can not find Theme item: {0}{1}")]
    Theme(String, String),
    #[error("Spice model not found: {0}")]
    SpiceModelNotFound(String),
    #[error("Unknown circuit element {0}")]
    UnknownCircuitElement(String),
    #[error("No pins found in {0} for unit {1}")]
    NoPinsFound(String, u32),
    #[error("Property \"{0}\" not found in \"{1}\"")]
    PropertyNotFound(String, String),
    #[error("Library \"{0}\" not found in schema")]
    LinraryNotFound(String),
    #[error("Unknown image type \"{0}\", must be jpg, png or pdf.")]
    UnknownImageType(String),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<cairo::Error> for Error {
    fn from(err: cairo::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<cairo::IoError> for Error {
    fn from(err: cairo::IoError) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<elektron_sexp::Error> for Error {
    fn from(err: elektron_sexp::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
