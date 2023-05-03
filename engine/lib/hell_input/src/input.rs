use core::fmt;

use hell_core::error::HellResult;
use strum::EnumCount;
use crate::keycodes::KeyCode;



#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyState {
    NeverUsed,
    Inactive,
    Pressed,
    Held,
    Released,
}

impl KeyState {
    pub const UP_STATES:   &'static [KeyState] = &[KeyState::Released, KeyState::Inactive];
    pub const DOWN_STATES: &'static [KeyState] = &[KeyState::Pressed, KeyState::Held];

    pub fn is_up(&self) -> bool {
        Self::UP_STATES.contains(self)
    }

    pub fn is_down(&self) -> bool {
        Self::DOWN_STATES.contains(self)
    }

}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct ModifiersState: u32 {
        const SHIFT  = 0b100;
        const LSHIFT = 0b010;
        const RSHIFT = 0b001;

        const CTRL  = 0b100 << 3;
        const LCTRL = 0b010 << 3;
        const RCTRL = 0b001 << 3;

        const ALT  = 0b100 << 6;
        const LALT = 0b010 << 6;
        const RALT = 0b001 << 6;

        const SUPER  = 0b100 << 9;
        const LSUPER = 0b010 << 9;
        const RSUPER = 0b001 << 9;
    }
}




pub struct InputManager {
    modifier_states: ModifiersState,
    key_states: [KeyState; KeyCode::COUNT],
}

impl InputManager {
    pub fn new() -> Self {
        let modifier_states = ModifiersState::from_bits(0).unwrap();
        let key_states = [KeyState::NeverUsed; KeyCode::COUNT];

        Self {
            modifier_states,
            key_states,
        }
    }

    pub fn update_key_state(&mut self, keycode: KeyCode, new_state: KeyState) -> HellResult<()> {
        let state = self.key_states.get_mut(keycode as usize).unwrap();

        let new_state = match (*state, new_state) {
            (KeyState::Pressed | KeyState::Held, KeyState::Pressed) => KeyState::Held,
            (_, s) => s,
        };

        println!("INPUT: udpate key state: {:?} => {:?}", keycode, new_state);

        *state = new_state;

        Ok(())
    }

    pub fn key_state(&self, keycode: KeyCode) -> KeyState {
        self.key_states[keycode as usize]
    }

    pub fn update_modifiers_state(&mut self, new_state: ModifiersState) {
        self.modifier_states = new_state;
    }

    pub fn reset_released_keys(&mut self) {
        self.key_states.iter_mut()
            .filter(|s| **s == KeyState::Released)
            .for_each(|s| *s = KeyState::Inactive);
    }
}


impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for InputManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, state) in self.key_states.iter().enumerate() {
            if *state == KeyState::NeverUsed { continue; }
            if *state == KeyState::Inactive { continue; }

            writeln!(f, "key_state: {:?} = {:?}", (KeyCode::try_from(idx as u32).unwrap()), state)?;
        }

        Ok(())
    }
}

