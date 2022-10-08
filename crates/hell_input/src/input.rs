use hell_error::HellResult;
use strum::EnumCount;
use crate::keycodes::KeyCode;



#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum KeyState {
    Inactive,
    Pressed,
    Held,
    Released,
}

impl From<winit::event::ElementState> for KeyState {
    fn from(s: winit::event::ElementState) -> Self {
        match s {
            winit::event::ElementState::Pressed => KeyState::Pressed,
            winit::event::ElementState::Released => KeyState::Released,
        }
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

impl From<winit::event::ModifiersState> for ModifiersState {
    fn from(s: winit::event::ModifiersState) -> Self {
        // TODO: error handling
        ModifiersState::from_bits(s.bits()).unwrap()
    }
}

#[allow(dead_code)]
pub struct InputState {
    modifier_states: ModifiersState,
    key_states: [KeyState; KeyCode::COUNT],
}

impl InputState {
    pub fn new() -> Self {
        let modifier_states = ModifiersState::from_bits(0).unwrap();
        let key_states = [KeyState::Inactive; KeyCode::COUNT];

        Self {
            modifier_states,
            key_states,
        }
    }

    pub fn update_key_state_winit(&mut self, e: winit::event::KeyboardInput) -> HellResult<()> {
        if let Some(code) = e.virtual_keycode {
            let keycode = KeyCode::from(code);
            let state = self.key_states.get_mut(keycode as usize).unwrap();
            *state = KeyState::from(e.state);
        };

        Ok(())
    }

    pub fn update_modifiers_state(&mut self, e: winit::event::ModifiersState) {
        todo!();
    }
}



pub struct InputContext {

}

pub struct HellInput {
    active_contexts: Vec<InputContext>,
}
