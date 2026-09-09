mod impls;

pub struct AddArgs<'a> {
    pub alias: &'a String,
    pub file: &'a String,
    pub new_file: Option<&'a String>,
}

pub struct LoadArgs<'a> {
    pub alias: &'a String,
    pub file: Option<&'a String>,
}

pub struct SetArgs<'a> {
    pub file: &'a String,
    pub kind: SetKind<'a>,
}

pub enum SetKind<'a> {
    Set,
    Load { alias: &'a String },
}
