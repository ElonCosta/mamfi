use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    rc::Rc,
};

use crate::{
    AppResult, FILE_NAME, Success,
    binary_data::{BinaryReader, BinaryType, BinaryWritter},
    cli::CommandOptions,
    result,
};
pub struct FileData {
    pub root_dir: PathBuf,
    pub data: HashMap<Rc<str>, Rc<str>>,
    pub set_file: Option<Rc<str>>,
    pub as_sym_link: bool,
    pub silent: bool,
}

impl FileData {
    pub fn is_silent(&self, command_options: &CommandOptions) -> bool {
        (self.silent && !command_options.verbose()) || command_options.silent()
    }
}

pub fn load_file_data(root_dir: PathBuf) -> result::Result<FileData> {
    let mut set_file: Option<Rc<str>> = None;
    let mut data = HashMap::new();
    let mut is_sym_link: Option<bool> = None;
    let mut is_silent: Option<bool> = None;

    let file = File::open(root_dir.join(FILE_NAME));

    if let Ok(file) = file {
        let reader = BufReader::new(file);

        for line in reader.binary_data() {
            match line {
                BinaryType::SetFile(file) => set_file = Some(file),
                BinaryType::AliasEntry((alias, path)) => {
                    data.insert(alias, path);
                }
                BinaryType::Flags { sym_link, silent } => {
                    is_sym_link = Some(sym_link);
                    is_silent = Some(silent);
                }
            }
        }
    }

    Ok(FileData {
        root_dir,
        data,
        set_file,
        as_sym_link: is_sym_link.unwrap_or(false),
        silent: is_silent.unwrap_or(false),
    })
}

pub fn save_file_data(app_data: FileData) -> AppResult {
    let FileData {
        root_dir,
        data,
        set_file,
        as_sym_link,
        silent,
    } = app_data;

    let file = File::create(root_dir.join(FILE_NAME))?;

    let mut writer = BufWriter::new(file);

    if let Some(set_file) = set_file {
        writer.write_bin(BinaryType::SetFile(set_file))?;
    }

    if as_sym_link || silent {
        writer.write_bin(BinaryType::Flags {
            sym_link: as_sym_link,
            silent,
        })?;
    }

    for entry in data.into_iter() {
        writer.write_bin(BinaryType::AliasEntry(entry))?;
    }

    Ok(Success::DataSaved)
}
