use std::path::PathBuf;

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
    pub alias: String,
    pub file: String,
    pub new_file: Option<String>,
}

#[derive(Debug)]
pub struct SetFileArgs {
    pub file: String,
}

#[derive(Debug)]
pub struct LoadArgs {
    pub alias: String,
    pub file: Option<String>,
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
