use ash::vk;
use hell_common::HellError;


pub fn vk_to_hell_err(res: vk::Result) -> HellError {
    HellError::from(res.as_raw())
}

pub fn err_invalid_frame_idx(frame_idx: usize) -> HellError {
    HellError::from(format!("frame-idx '{}' is out of range", frame_idx))
}
