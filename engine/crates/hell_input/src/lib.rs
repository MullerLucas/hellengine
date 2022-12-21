// crate-config: start
#![deny(warnings)]
// crate-config: end



mod input;
mod keycodes;

pub use input::{InputManager, KeyState};
pub use keycodes::KeyCode;
