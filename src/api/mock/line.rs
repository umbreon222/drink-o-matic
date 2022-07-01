use crate::api::mock::{ LineHandle, LineRequestFlags };

pub struct Line { }

impl Line {
    pub fn request(&self, _args: LineRequestFlags, _value: u8, _name: &str) -> Result<LineHandle, &str> {
        Ok(LineHandle { })
    }
}