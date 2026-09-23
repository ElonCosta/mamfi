use std::{path::PathBuf, rc::Rc};

mod impls;
mod types;

#[derive(Debug)]
pub struct AppState {
    pub command: Command,
    pub command_options: CommandOptions,
    pub root_dir: PathBuf,
}

#[derive(Debug)]
pub enum Command {
    Help,
    Add(AddArgs),
    SetFile(SetFileArgs, Option<LoadArgs>),
    SetFlags,
    Load(LoadArgs),
    List,
}

#[derive(Debug)]
pub struct AddArgs {
    pub alias: Rc<str>,
    pub file: Rc<str>,
    pub new_file: Option<Rc<str>>,
}

#[derive(Debug)]
pub struct SetFileArgs {
    pub file: Rc<str>,
}

#[derive(Debug)]
pub struct LoadArgs {
    pub alias: Rc<str>,
    pub file: Option<Rc<str>>,
}
#[repr(u8)]
pub enum AppOptions {
    CurrentDirAsRoot = 1,
    AsSymLink = 2,
    AsCopy = 4,
    Silent = 8,
    Verbose = 16,
}

#[derive(Debug)]
pub struct CommandOptions(u8);
