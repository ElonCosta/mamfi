use crate::HELP_TEXT;
use std::{
    fmt::{Debug, Display},
    io,
    process::{ExitCode, Termination},
};

pub enum AppError {
    AliasNotFound(String),
    IoError(io::Error),
    NoFileSet,
    FileNotFound(String),
    NotEnoughArgs(&'static str, usize),
    TooManyArgs(&'static str, usize),
    UnknownOption(String),
    NoArgs,
    InvalidReplace(String),
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::NotFound => AppError::FileNotFound("".into()),
            _ => AppError::IoError(value),
        }
    }
}

impl Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(core::format_args!(
            "{}",
            match self {
                AppError::AliasNotFound(alias) => format!("Alias '{alias}' not found"),
                AppError::IoError(error) => format!("IoError {error:?}"),
                AppError::NotEnoughArgs(func, args) =>
                    format!("Not enough args for {func} ({args})"),
                AppError::TooManyArgs(func, args) => format!("Too many args for {func} ({args})"),
                AppError::UnknownOption(option) =>
                    format!("Unkown option {option}, use --help for more information"),
                AppError::NoFileSet =>
                    "Default file not set, use --set to define a default file".to_string(),
                AppError::NoArgs => "Missing operands, use --help for more information".to_string(),
                AppError::FileNotFound(file) => format!("File {file} not found"),
                AppError::InvalidReplace(first) => format!("Trying to replace {first} with itself"),
            }
        ))
    }
}

impl Display for Success {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileReplaced(target, aliased_file) => {
                writeln!(f, "Replaced {target} with {aliased_file}")
            }
            Self::FileLinked(target, aliased_file) => {
                writeln!(f, "Linked {target} to {aliased_file}")
            }
            Self::DataUpdated(UpdateType::Added(alias, file)) => {
                writeln!(f, "{file} aliased as {alias}")
            }
            Self::DataUpdated(UpdateType::SetDefaultFile(file)) => {
                writeln!(f, "{file} set as default")
            }
            Self::DataUpdated(UpdateType::SetDefaultOptions) => {
                writeln!(f, "Options set as default")
            }
            Self::HelpDisplayed => writeln!(f, "{HELP_TEXT}"),
            _ => write!(f, ""),
        }
    }
}

pub enum Success {
    FileReplaced(String, String),
    FileLinked(String, String),
    DataUpdated(UpdateType),
    DataSaved,
    HelpDisplayed,
    Silent,
}

pub enum UpdateType {
    Added(String, String),
    SetDefaultFile(String),
    SetDefaultOptions,
}

impl Termination for Success {
    fn report(self) -> std::process::ExitCode {
        print!("{}", self);
        ExitCode::SUCCESS
    }
}

pub type Result<T> = core::result::Result<T, AppError>;
