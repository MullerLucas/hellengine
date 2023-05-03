use hell_core::error::{HellError, HellErrorKind, HellErrorContent};
use num_traits::FromPrimitive;

#[repr(u32)]
#[derive(Debug, Copy, Clone, strum::EnumCount, num_derive::FromPrimitive)]
pub enum KeyCode {
    Invalid,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Alpha0,
    Alpha1,
    Alpha2,
    Alpha3,
    Alpha4,
    Alpha5,
    Alpha6,
    Alpha7,
    Alpha8,
    Alpha9,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    ControlRight,
    ControlLeft,
    AltRight,
    AltLeft,
    ShiftLeft,
    ShiftRight,
    MetaLeft,
    MetaRight,

    Space,
    Escape,
    Return,
    Backspace,
}

impl TryFrom<u32> for KeyCode {
    type Error = HellError;

    fn try_from(val: u32) -> Result<Self, Self::Error> {
        Self::from_u32(val).ok_or_else(|| HellError::new(
                HellErrorKind::GenericError,
                HellErrorContent::Message(format!("failed to convert '{}' to KeyCode", val))
            )
        )
    }
}
