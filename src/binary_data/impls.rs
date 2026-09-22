use std::io::{self, BufRead, BufReader, BufWriter, Write};

use crate::binary_data::{BinaryContainer, BinaryReader, BinaryType, BinaryWritter};

impl<W: Write> BinaryWritter for BufWriter<W> {
    fn write_bin(&mut self, value: BinaryType) -> io::Result<()> {
        self.write_all(value.into_bin().as_slice())
    }
}
impl BinaryType {
    // TODO: Find a better way to do this.
    fn discriminant(&self) -> u8 {
        match self {
            BinaryType::SetFile(_) => 0,
            BinaryType::AliasEntry(_) => 1,
            BinaryType::Flags { .. } => 2,
        }
    }

    fn into_bin(self) -> Vec<u8> {
        let meta = self.discriminant();

        match self {
            Self::SetFile(file) => {
                let file_len = (file.len() as u16).to_le_bytes();

                let bin_len = 1 + 2 + file.len();

                let mut bin = Vec::with_capacity(bin_len);

                bin.push(meta);
                bin.extend_from_slice(&file_len);
                bin.extend(file.as_bytes());

                bin
            }
            Self::AliasEntry((alias, path)) => {
                let alias_len = (alias.len() as u16).to_le_bytes();
                let path_len = (path.len() as u16).to_le_bytes();

                let bin_len = 1 + 2 + alias.len() + 2 + path.len();

                let mut bin = Vec::with_capacity(bin_len);

                bin.push(meta);
                bin.extend_from_slice(&alias_len);
                bin.extend(alias.as_bytes());
                bin.extend_from_slice(&path_len);
                bin.extend(path.as_bytes());

                bin
            }
            Self::Flags { sym_link, silent } => {
                let mut flags: u8 = sym_link as u8;
                flags |= (silent as u8) << 1;

                vec![meta, flags]
            }
        }
    }
}

impl<S: Sized> BinaryReader<S> for BufReader<S> {
    fn binary_data(self) -> BinaryContainer<Self> {
        BinaryContainer { buf: self }
    }
}

impl<B: BufRead> Iterator for BinaryContainer<B> {
    type Item = BinaryType;

    fn next(&mut self) -> Option<Self::Item> {
        let meta = next_value!(u8; self.buf);

        let ident = meta & 0xf;

        let data = match ident {
            0 => {
                let set_file = String::from_utf8(next_value!(str; self.buf)).ok()?;

                BinaryType::SetFile(set_file)
            }
            1 => {
                let alias = String::from_utf8(next_value!(str; self.buf)).ok()?;
                let path = String::from_utf8(next_value!(str; self.buf)).ok()?;

                BinaryType::AliasEntry((alias, path))
            }
            2 => {
                let flags = next_value!(u8; self.buf);

                let sym_link = flags & 1 != 0;
                let silent = flags & 2 != 0;

                BinaryType::Flags { sym_link, silent }
            }
            _ => return None,
        };

        Some(data)
    }
}
