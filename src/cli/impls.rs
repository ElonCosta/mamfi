use std::{
    collections::VecDeque,
    env::{self, Args},
    fs,
    ops::{BitAnd, BitOrAssign},
    path::PathBuf,
};

use crate::{
    cli::{AddArgs, AppOptions, AppState, Command, CommandOptions, LoadArgs, SetFileArgs},
    parent_dir,
    result::{self, AppError},
};

impl AppState {
    pub fn new(value: Args) -> result::Result<Self> {
        // Skips path to executable.
        let iter = value.skip(1);

        let mut command = Mode::None;
        let mut command_options = CommandOptions(0);
        let mut args = VecDeque::new();

        for arg in extract_args(iter) {
            let arg_type = arg?;

            match arg_type {
                ArgType::Command(mode) => command = mode,
                ArgType::Option(app_options) => command_options |= app_options,
                ArgType::Arg(arg) => args.push_back(arg),
            }
        }

        let command = match command {
            Mode::Help => Command::Help,
            Mode::List => Command::List,
            Mode::Add => build_add_command(&mut args)?,
            Mode::Set => build_set_command(&mut args, &command_options)?,
            Mode::None | Mode::Load => build_load_command(&mut args)?,
        };

        let root_dir = get_root_dir(&command, &command_options)?;

        let app_state = AppState {
            command,
            command_options,
            root_dir,
        };

        Ok(app_state)
    }
}

enum ArgType {
    Command(Mode),
    Option(AppOptions),
    Arg(String),
}

enum Mode {
    Help,
    Add,
    Set,
    List,
    Load,
    None,
}

impl From<ExtractArgsError> for AppError {
    fn from(value: ExtractArgsError) -> Self {
        match value {
            ExtractArgsError::UnknownOption(option) => AppError::UnknownOption(option.to_string()),
            ExtractArgsError::UnknownCommand(command) => AppError::UnknownOption(command),
        }
    }
}

enum ExtractArgsError {
    UnknownCommand(String),
    UnknownOption(char),
}

gen fn extract_args<T>(iter: T) -> Result<ArgType, ExtractArgsError>
where
    T: Iterator<Item = String>,
{
    let mut ignore_options = false;
    for arg in iter {
        match arg.as_str() {
            "--" if !ignore_options => ignore_options = true,
            a if a.starts_with("--") && !ignore_options => {
                yield try {
                    let mode = match a {
                        "--help" => Mode::Help,
                        "--add" => Mode::Add,
                        "--set" => Mode::Set,
                        "--list" => Mode::List,
                        "--replace" => Mode::Load,
                        _ => Err(ExtractArgsError::UnknownCommand(arg))?,
                    };

                    ArgType::Command(mode)
                }
            }
            a if a.starts_with('-') && !ignore_options => {
                for option in arg.into_chars() {
                    yield try {
                        match option {
                            '-' => continue,
                            'h' => ArgType::Command(Mode::Help),
                            'a' => ArgType::Command(Mode::Add),
                            's' => ArgType::Command(Mode::Set),
                            'l' => ArgType::Command(Mode::List),
                            'd' => ArgType::Option(AppOptions::CurrentDirAsRoot),
                            'L' => ArgType::Option(AppOptions::AsSymLink),
                            'C' => ArgType::Option(AppOptions::AsCopy),
                            'S' => ArgType::Option(AppOptions::Silent),
                            'V' => ArgType::Option(AppOptions::Verbose),
                            _ => Err(ExtractArgsError::UnknownOption(option))?,
                        }
                    }
                }
            }
            _ => yield Ok(ArgType::Arg(arg)),
        };
    }
}

fn build_add_command(args: &mut VecDeque<String>) -> result::Result<Command> {
    if args.len() < 2 {
        Err(AppError::NotEnoughArgs("add", args.len()))?;
    } else if args.len() > 3 {
        Err(AppError::TooManyArgs("add", args.len()))?;
    }

    let alias = args.pop_front().unwrap();
    let file = args.pop_front().unwrap();

    let new_file = args.pop_front();

    Ok(Command::Add(AddArgs {
        alias,
        file,
        new_file,
    }))
}

fn build_set_command(
    args: &mut VecDeque<String>,
    command_options: &CommandOptions,
) -> result::Result<Command> {
    if args.is_empty() {
        if !command_options.is_empty() {
            return Ok(Command::SetFlags);
        }

        Err(AppError::NotEnoughArgs("set", args.len()))?;
    } else if args.len() > 2 {
        Err(AppError::TooManyArgs("set", args.len()))?;
    }

    let file = args.pop_back().unwrap();
    let alias = args.pop_front();

    Ok(Command::SetFile(
        SetFileArgs { file: file.clone() },
        alias.map(|a| LoadArgs {
            alias: a,
            file: Some(file),
        }),
    ))
}

fn build_load_command(args: &mut VecDeque<String>) -> result::Result<Command> {
    if args.is_empty() {
        Err(AppError::NoArgs)?;
    } else if args.len() > 2 {
        Err(AppError::TooManyArgs("replace", args.len()))?;
    }

    let alias = args.pop_front().unwrap();
    let file = args.pop_front();

    Ok(Command::Load(LoadArgs { alias, file }))
}

fn get_root_dir(command: &Command, command_options: &CommandOptions) -> result::Result<PathBuf> {
    let root_dir = if command_options.current_dir_as_root() {
        env::current_dir()?
    } else {
        let parent_dir = parent_dir(command.ref_file())?;

        if parent_dir.is_absolute() {
            fs::canonicalize(parent_dir)?
        } else {
            env::current_dir()?
        }
    };

    Ok(root_dir)
}

impl Command {
    fn ref_file(&self) -> Option<&String> {
        match self {
            Command::Add(AddArgs { file, .. })
            | Command::SetFile(SetFileArgs { file }, _)
            | Command::Load(LoadArgs {
                file: Some(file), ..
            }) => Some(file),
            _ => None,
        }
    }

    pub const fn overrides_silent(&self) -> bool {
        matches!(self, Command::List | Command::Help)
    }
}

impl CommandOptions {
    pub fn current_dir_as_root(&self) -> bool {
        self.is_on(AppOptions::CurrentDirAsRoot)
    }

    pub fn as_sym_link(&self) -> bool {
        self.is_on(AppOptions::AsSymLink)
    }

    pub fn as_copy(&self) -> bool {
        self.is_on(AppOptions::AsCopy)
    }

    pub fn silent(&self) -> bool {
        self.is_on(AppOptions::Silent)
    }

    pub fn verbose(&self) -> bool {
        self.is_on(AppOptions::Verbose)
    }

    fn is_on(&self, option: AppOptions) -> bool {
        self & option > 0
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }
}
impl BitAnd<AppOptions> for &CommandOptions {
    type Output = u8;

    fn bitand(self, rhs: AppOptions) -> Self::Output {
        self.0 & rhs as u8
    }
}

impl BitOrAssign<AppOptions> for CommandOptions {
    fn bitor_assign(&mut self, rhs: AppOptions) {
        self.0 |= rhs as u8;
    }
}
