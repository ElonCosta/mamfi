#![feature(gen_blocks, try_blocks, string_into_chars)]
mod binary_data;
mod cli;
mod args;
mod data;
mod macros;
mod result;

use std::{
    env,
    ffi::OsStr,
    fmt::Debug,
    fs,
    io::{self, ErrorKind::NotFound},
    os::unix,
    path::{self, Path, PathBuf},
};

use crate::{
    args::{AddArgs, LoadArgs, SetArgs, SetKind},
    data::*,
    result::AppError,
    result::{AppError, Success, UpdateType},
};

const FILE_NAME: &str = ".mmf";
const HELP_TEXT: &str = include_str!("../texts/help.txt");

type AppResult = result::Result<Success>;
trait_alias! {
    PathRef: AsRef<Path> + Debug
}

    }
}


impl FileData {
    /// Adds file alias to mmf, if new_file is provided copies the content of {file} to new_file
    fn add_file_alias(
        &mut self,
        AddArgs {
            alias,
            file,
            new_file,
        }: AddArgs,
    ) -> AppResult {
        let added_file = new_file.unwrap_or(file);

        self.data.insert(alias.to_string(), added_file.to_string());

        let Some(new_file) = new_file else {
            return Ok(Success::FileAdded);
        };

        if !fs::exists(file)? {
            return Err(AppError::InexistentFile(file.into()));
        }

        fs::copy(file, new_file)?;

        Ok(Success::FileSaved)
    }

    fn load_file_alias(&mut self, LoadArgs { alias, file }: LoadArgs) -> AppResult {
        let Some(target_file) = file.or(self.set_file.as_ref()) else {
            return Err(AppError::NoFileSet);
        };

        if !fs::exists(target_file)? {
            return Err(AppError::InexistentFile(target_file.into()));
        }

        let Some(aliased_file) = self.data.get(alias) else {
            return Err(AppError::AliasNotFound(alias.into()));
        };

        fs::copy(aliased_file, target_file)?;

        println!("Replaced {} with {}", target_file, aliased_file);

        Ok(Success::FileLoaded)
    }

    fn set_file(&mut self, SetArgs { file, kind }: SetArgs) -> AppResult {
        if !fs::exists(file)? {
            return Err(AppError::InexistentFile(file.into()));
        }

        self.set_file = Some(file.to_string());

        if let SetKind::Load { alias } = kind {
            _ = self.load_file_alias(LoadArgs {
                alias,
                file: Some(file),
            })?;
        }

        Ok(Success::DataFileUpdated)
    }
}

fn main() -> AppResult {
    let args: Vec<String> = env::args().collect();

    let mut file_data = load_file_data();

    let Some(mode) = args.get(1) else {
        return print_help();
    };

    let result = match mode.as_str() {
        "--help" | "-h" => print_help(),
        "--add" | "-a" => file_data.add_file_alias(args[2..].try_into()?),
        "--set" | "-s" => file_data.set_file(args[2..].try_into()?),
        _ if !mode.starts_with("-") => file_data.load_file_alias(args[1..].into()),
        _ => Err(AppError::UnknownOption(mode.into())),
    }?;

    if let Success::FileLoaded | Success::HelpDisplayed = result {
        return Ok(result);
    }

    save_file_data(file_data)
}

fn print_help() -> AppResult {
    println!(
        r"
    Usage: mmf [OPTIONS] <ALIAS> [FILE]
    Replaces FILE with a previously ALIASed file;
    if FILE is not provided uses the default --set file.

    [OPTIONS]
    -h, --help
        Displays this help.
    -a, --add <ALIAS> <FILE> [NEW_FILE]
        Adds an ALIAS for FILE, if NEW_FILE is provided copies FILE content
        into NEW_FILE and then alias NEW_FILE instead.
    -s, --set [ALIAS] <FILE>
        Sets FILE as the default file to be replaced by mmf, if ALIAS is provided
        replaces FILE with the ALIASed file.
        "
    );
    Ok(Success::HelpDisplayed)
}
