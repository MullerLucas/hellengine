use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;

use hell_core::error::HellResult;

pub struct FntFileCharRow {
    pub id: u64,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub xoffset: i32,
    pub yoffset: i32,
    pub xadvance: i32,
    pub yadvance: i32,
}

#[allow(dead_code)]
pub struct FntFile {
    chars: Vec<FntFileCharRow>
}



impl FntFile {
    #[allow(unused)]
    pub fn from_file(path: &Path) -> HellResult<Self> {
        let chars = Vec::new();

        // let file = std::fs::read_to_string(path)?;
        let buff_reader = BufReader::new(File::open(path)?);

        // TODO:
        for line in buff_reader.lines().flatten() {
            // println!("ROW: '{}'", line);
        }

        Ok(Self {
            chars
        })
    }
}
