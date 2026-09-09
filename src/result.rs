use std::{fmt::Debug, io};

pub enum AppError {
    AliasNotFound(String),
    IoError(io::Error),
    NoFileSet,
    InexistentFile(String),
    NotEnoughArgs(String, usize),
    UnknownOption(String),
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::IoError(value)
    }
}

impl Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppError::AliasNotFound(alias) => format!("Alias '{}' not found", alias),
                AppError::IoError(error) => format!("IoError {:?}", error),
                AppError::NoFileSet =>
                    "Default file not set, use --set to define a default file".into(),
                AppError::NotEnoughArgs(func, args) =>
                    format!("Not enough args for {func} ({args})"),
                AppError::UnknownOption(option) =>
                    format!("Unkown option {}, use --help for more information", option),
                AppError::InexistentFile(file) => format!("File {} doesn't exist", file),
            }
        )
    }
}

pub type Result<T> = core::result::Result<T, AppError>;
