use hell_core::config;

pub type PerFrame<T> = [T; config::FRAMES_IN_FLIGHT];
