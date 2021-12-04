use crate::files::commands::BirthdaySpec;

use super::super::command::CommandState;
use super::super::Result;

impl<'a> CommandState<'a> {
    pub fn eval_birthday_spec(&mut self, spec: &BirthdaySpec) -> Result<()> {
        todo!()
    }
}
