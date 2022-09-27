use core::fmt;
use std::{result, error};


pub type HellResult<T> = result::Result<T, HellError>;



#[derive(fmt::Debug)]
pub struct HellError {
    msg: Option<String>,
    code: Option<i32>,
}

impl From<String> for HellError {
    fn from(val: String) -> Self {
        Self {
            msg: Some(val),
            code: None
        }
    }
}

impl From<i32> for HellError {
    fn from(val: i32) -> Self {
        Self {
            msg: None,
            code: Some(val)
        }
    }
}

impl fmt::Display for HellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "msg: '{}'; ", msg)?;
        }

        if let Some(code) = self.code {
            write!(f, "code: '{}'", code)?;
        }

        Ok(())
    }
}

impl error::Error for HellError { }






