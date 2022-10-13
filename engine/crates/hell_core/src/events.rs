// pub enum Event {
//     WindowEvent {
//         window_id: u64,
//         event: WindowEvent,
//     }
// }
//
//
// pub enum WindowEvent {
//     KeyboardInput {
//         device_id: u64,
//         input: KeyboardInput,
//         is_synthetic: bool,
//     },
//
//     ModifiersChanged(ModifiersState)
//
// }
//
//
//
//
// type ModifiersState = u64;
//
// type ScanCode = u32;
// pub enum ElementState {
//     Pressed,
//     Released,
// }
//
// type VirtualKeycode = u32;
//
// pub struct KeyboardInput {
//     deveice_id: u64,
//     scancode: ScanCode,
//     state: ElementState,
//     virtual_keycode: Option<VirtualKeycode>,
// }
