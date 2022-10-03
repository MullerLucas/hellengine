use core::fmt;
use std::{result, error};


pub type HellResult<T> = result::Result<T, HellError>;

#[derive(fmt::Debug)]
pub enum HellErrorType {
    Default,
}

#[derive(fmt::Debug)]
pub struct HellError {
    child_err: Option<std::sync::Arc<dyn std::error::Error>>,
    msg: Option<String>,
    code: Option<i32>,
}

impl From<String> for HellError {
    fn from(val: String) -> Self {
        Self {
            child_err: None,
            msg: Some(val),
            code: None
        }
    }
}

impl From<&str> for HellError {
    fn from(val: &str) -> Self {
        Self {
            child_err: None,
            msg: Some(val.to_owned()),
            code: None
        }
    }
}

impl From<i32> for HellError {
    fn from(val: i32) -> Self {
        Self {
            child_err: None,
            msg: None,
            code: Some(val)
        }
    }
}

impl fmt::Display for HellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "msg: '{}'; ", msg)?;
        }

        if let Some(code) = self.code {
            write!(f, "code: '{}'", code)?;
        }

        Ok(())
    }
}

impl error::Error for HellError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match &self.child_err {
            Some(err) => Some(err),
            None => None,
        }
    }
}


// pub trait ToHellError<T, E> where E: std::error::Error {
//     fn to_hell_error(self) -> HellResult<T>;
//     // fn to_hell_error(self) -> HellResult<T> {
//     //     match self {
//     //         Ok(t) => Ok(t),
//     //         Err(e) => {
//     //             Err(HellError {
//     //
//     //             })
//     //         }
//     //     }
//     // }
// }
//
//
// impl<T, E> ToHellError for Result<T, E: std::error::Error> {
//     fn to_hell_error(self) -> HellResult<T> {
//         todo!()
//     }
// }
//
