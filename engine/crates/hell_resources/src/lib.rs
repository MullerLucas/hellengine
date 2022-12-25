// crate-config: start
#![deny(warnings)]
// crate-config: end



mod resource_config;

pub mod resources;
pub mod fonts;

mod resource_manager;
pub use resource_manager::*;

