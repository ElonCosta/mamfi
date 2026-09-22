#[macro_use]
mod macros;
mod impls;

use std::io;

pub trait BinaryWritter {
    fn write_bin(&mut self, value: BinaryType) -> io::Result<()>;
}

pub trait BinaryReader<S: Sized>
where
    Self: Sized,
{
    fn binary_data(self) -> BinaryContainer<Self>;
}

#[derive(Debug)]
pub enum BinaryType {
    SetFile(String),
    AliasEntry((String, String)),
    Flags { sym_link: bool, silent: bool },
}

pub struct BinaryContainer<S> {
    buf: S,
}
