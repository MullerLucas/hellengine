use hell_core::config;

// pub struct PerFrame<T> {
//     data: [T; config::FRAMES_IN_FLIGHT],
// }
//
// impl<T> PerFrame<T> {
//     pub fn new(data: [T; config::FRAMES_IN_FLIGHT]) -> Self {
//         Self {
//             data
//         }
//     }
//
//     pub fn get(&self, frame_idx: usize) -> &T {
//         &self.data[frame_idx]
//     }
//
//     pub fn get_all(&self) -> &[T] {
//         &self.data
//     }
// }

pub type PerFrame<T> = [T; config::FRAMES_IN_FLIGHT];
