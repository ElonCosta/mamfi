#[macro_use]
mod macros;
mod impls;

use std::{io, rc::Rc};

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
    SetFile(Rc<str>),
    AliasEntry((Rc<str>, Rc<str>)),
    Flags { sym_link: bool, silent: bool },
}

pub struct BinaryContainer<S> {
    buf: S,
}
