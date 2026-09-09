use crate::{
    args::{AddArgs, LoadArgs, SetArgs, SetKind},
    result::{self, AppError},
};

impl<'a> TryFrom<&'a [String]> for AddArgs<'a> {
    type Error = AppError;

    fn try_from(value: &'a [String]) -> result::Result<AddArgs<'a>> {
        if value.len() < 2 {
            return Err(AppError::NotEnoughArgs("add".into(), value.len()));
        }

        let alias = &value[0];
        let file = &value[1];
        let new_file = value.get(2);

        Ok(AddArgs {
            alias,
            file,
            new_file,
        })
    }
}

impl<'a> From<&'a [String]> for LoadArgs<'a> {
    fn from(value: &'a [String]) -> LoadArgs<'a> {
        let alias = &value[0];
        let file = value.get(1);

        LoadArgs { alias, file }
    }
}

impl<'a> TryFrom<&'a [String]> for SetArgs<'a> {
    type Error = AppError;

    fn try_from(value: &'a [String]) -> result::Result<SetArgs<'a>> {
        if value.is_empty() {
            return Err(AppError::NotEnoughArgs("set".into(), value.len()));
        }

        let first_arg = &value[0];
        let second_arg = value.get(1);

        let result = match second_arg {
            Some(file) => SetArgs {
                file,
                kind: SetKind::Load { alias: first_arg },
            },
            None => SetArgs {
                file: first_arg,
                kind: SetKind::Set,
            },
        };

        Ok(result)
    }
}
