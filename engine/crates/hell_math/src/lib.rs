// crate-config: start
#![deny(warnings)]
// crate-config: end



mod vec;
mod mat;


pub use vec::{Vec2, Vec3, Vec4, IVec4, UVec4};
pub use mat::Mat4;
