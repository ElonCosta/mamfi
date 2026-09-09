use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use crate::{AppResult, FILE_NAME, Success};
pub struct FileData {
    pub data: HashMap<String, String>,
    pub set_file: Option<String>,
}

pub fn load_file_data() -> FileData {
    let mut set_file: Option<String> = None;
    let mut data = HashMap::new();

    let file = File::open(FILE_NAME);

    if let Ok(file) = file {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let Ok(line) = line else {
                continue;
            };

            let bytes = line.as_bytes();

            let Some(i) = bytes.first() else {
                continue;
            };

            let bytes = &bytes[1..];

            match i {
                b'1' => {
                    let file = line[1..].trim();

                    if file.is_empty() {
                        continue;
                    };

                    set_file = Some(file.to_string());
                }
                b'2' => {
                    let key_size = bytes[0] as usize;
                    let line = &line[2..];

                    let alias = &line[..key_size].trim();
                    let path = &line[key_size..].trim();

                    data.insert(alias.to_string(), path.to_string());
                }
                _ => continue,
            }
        }
    }

    FileData { data, set_file }
}

pub fn save_file_data(file_data: FileData) -> AppResult {
    let FileData { data, set_file } = file_data;

    let file = File::create(FILE_NAME)?;

    let mut writer = BufWriter::new(file);

    if let Some(set_file) = set_file {
        writer.write_all(b"1")?;
        writer.write_all(set_file.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    for (key, path) in data.iter() {
        let key_size = key.len() as u8;

        writer.write_all(&[b'2', key_size])?;
        writer.write_all(key.as_bytes())?;
        writer.write_all(path.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(Success::DataSaved)
}
