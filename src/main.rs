#![feature(gen_blocks, try_blocks, string_into_chars)]
mod binary_data;
mod cli;
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
    rc::Rc,
};

use crate::{
    cli::{AddArgs, AppState, Command, CommandOptions, LoadArgs, SetFileArgs},
    data::{FileData, load_file_data, save_file_data},
    result::{AppError, Success, UpdateType},
};

const FILE_NAME: &str = ".mmf";
const HELP_TEXT: &str = include_str!("../texts/help.txt");

type AppResult = result::Result<Success>;
trait_alias! {
    PathRef: AsRef<Path> + Debug
}

/// Adds file alias to mmf, if new_file is provided copies the content of {file} to new_file
fn add_file_alias(
    data: &mut FileData,
    AddArgs {
        alias,
        file,
        new_file,
    }: AddArgs,
    options: CommandOptions,
) -> AppResult {
    let FileData { root_dir, data, .. } = data;

    let added_file: &str = &new_file.clone().unwrap_or(file.clone());
    let file: &str = &file;

    let (file_parent, file_name) = get_dir_and_name(added_file)?;

    let file_name = if let Some(file_name) = file_name
        && &file_parent == root_dir
    {
        file_name
    } else {
        added_file.into()
    };

    data.insert(alias.clone(), file_name.clone());

    let Some(new_file) = new_file else {
        return Ok(Success::DataUpdated(UpdateType::Added(alias, file_name)));
    };

    if !fs::exists(file)? {
        return Err(AppError::FileNotFound(file.to_string()));
    }

    if options.as_sym_link() {
        link_file(file, new_file.as_ref())?;
    } else {
        copy_file(file, new_file.as_ref())?;
    }

    Ok(Success::DataUpdated(UpdateType::Added(
        alias,
        new_file.clone(),
    )))
}

fn load_file_alias(
    data: &FileData,
    LoadArgs { alias, file }: LoadArgs,
    options: CommandOptions,
) -> AppResult {
    let FileData {
        root_dir,
        data,
        set_file,
        as_sym_link,
        ..
    } = data;

    let Some(target_file) = file.or(set_file.clone()) else {
        Err(AppError::NoFileSet)?
    };
    let Ok(target_file) = path::absolute(target_file.as_ref()) else {
        Err(AppError::NoFileSet)?
    };

    if !fs::exists(&target_file)? {
        Err(AppError::FileNotFound(
            target_file
                .file_name()
                .and_then(OsStr::to_str)
                .map(str::to_string)
                .unwrap(),
        ))?
    }

    let Some(aliased_file) = data.get(&alias) else {
        Err(AppError::AliasNotFound(alias))?
    };

    let aliased_path = root_dir.join(aliased_file.as_ref());

    if aliased_path == target_file {
        Err(AppError::InvalidReplace(
            target_file
                .file_name()
                .and_then(OsStr::to_str)
                .map(Rc::from)
                .unwrap(),
        ))?
    }

    let success = if (*as_sym_link || options.as_sym_link()) && !options.as_copy() {
        link_file(aliased_path, &target_file)?;

        Success::FileLinked(
            target_file
                .file_name()
                .and_then(OsStr::to_str)
                .map(Rc::from)
                .unwrap(),
            aliased_file.clone(),
        )
    } else {
        if target_file.is_symlink() {
            fs::remove_file(&target_file)?;
        }

        copy_file(aliased_path, &target_file)?;

        Success::FileReplaced(
            target_file
                .file_name()
                .and_then(OsStr::to_str)
                .map(Rc::from)
                .unwrap(),
            aliased_file.clone(),
        )
    };

    Ok(success)
}

fn set_default_file(data: &mut FileData, SetFileArgs { file }: SetFileArgs) -> AppResult {
    if !fs::exists(file.as_ref())? {
        return Err(AppError::FileNotFound(file.to_string()));
    }

    data.set_file = Some(file.clone());

    Ok(Success::DataUpdated(UpdateType::SetDefaultFile(file)))
}

fn set_flags(data: &mut FileData, options: CommandOptions) -> AppResult {
    if options.as_sym_link() && !options.as_copy() {
        data.as_sym_link = true;
    }

    if options.as_copy() {
        data.as_sym_link = false;
    }

    if options.silent() && !options.verbose() {
        data.silent = true;
    }

    if options.verbose() {
        data.silent = false;
    }

    Ok(Success::DataUpdated(UpdateType::SetDefaultOptions))
}

fn main() -> AppResult {
    let AppState {
        command,
        command_options,
        root_dir,
    }: AppState = AppState::new(env::args())?;

    let mut data = load_file_data(root_dir)?;

    let is_silent = data.is_silent(&command_options) && !command.overrides_silent();

    let result = try {
        let success = match command {
            Command::Help => Success::HelpDisplayed,
            Command::Add(add_args) => add_file_alias(&mut data, add_args, command_options)?,
            Command::SetFile(set_file_args, None) => {
                set_flags(&mut data, command_options)?;

                set_default_file(&mut data, set_file_args)?
            }
            Command::SetFile(set_file_args, Some(load_args)) => {
                load_file_alias(&data, load_args, command_options)?;

                set_default_file(&mut data, set_file_args)?
            }
            Command::SetFlags => set_flags(&mut data, command_options)?,
            Command::Load(load_args) => load_file_alias(&data, load_args, command_options)?,
            Command::List => todo!(),
        };

        if matches!(success, Success::DataUpdated(_)) {
            save_file_data(data)?;
        }

        success
    };

    if is_silent {
        return Ok(Success::Silent);
    }

    result
}

fn parent_dir<P: PathRef>(file: Option<P>) -> io::Result<PathBuf> {
    let Some(file) = file else {
        return env::current_dir();
    };

    let parent_dir = file.as_ref().parent().unwrap_or(Path::new(""));

    let dir = if parent_dir.is_absolute() {
        fs::canonicalize(parent_dir)?
    } else {
        env::current_dir()?
    };

    Ok(dir)
}

fn get_dir_and_name<P: PathRef>(file: P) -> io::Result<(PathBuf, Option<Rc<str>>)> {
    let dir = parent_dir(Some(&file))?;

    let file_name = file
        .as_ref()
        .file_name()
        .and_then(OsStr::to_str)
        .map(Rc::from);

    Ok((dir, file_name))
}

fn link_file<F: PathRef, T: PathRef>(from: F, to: T) -> result::Result<()> {
    fs::remove_file(&to)?;

    match unix::fs::symlink(&from, to) {
        Err(e) => match e.kind() {
            NotFound => Err(AppError::FileNotFound(format!("{from:?}"))),
            _ => Err(e.into()),
        },
        Ok(_) => Ok(()),
    }
}

fn copy_file<F: PathRef, T: PathRef>(from: F, to: T) -> result::Result<()> {
    match fs::copy(&from, to) {
        Err(e) => match e.kind() {
            NotFound => Err(AppError::FileNotFound(format!("{from:?}"))),
            _ => Err(e.into()),
        },
        Ok(_) => Ok(()),
    }
}
