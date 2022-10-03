use hell_common::prelude::*;

pub fn err_invalid_frame_idx(frame_idx: usize) -> HellError {
    HellError::from_msg(HellErrorKind::RenderError, format!("frame-idx '{}' is out of range", frame_idx))
}

pub fn err_invalid_set_idx(set_idx: usize) -> HellError {
    HellError::from_msg(HellErrorKind::RenderError, format!("set-idx '{}' is out of range", set_idx))
}
