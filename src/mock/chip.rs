use crate::mock::Line;

pub struct Chip { }

impl Chip {
    pub fn new(_args: &str) -> Result<Self, String> {
        Ok(Chip { })
    }

    pub fn get_line(&mut self, _offset: u32) -> Result<Line, &str> {
        Ok(Line { })
    }
}