#[cfg(target_arch = "wasm32")]
pub mod bot;
#[cfg(target_arch = "wasm32")]
mod dag;
#[macro_use]
pub mod data;
#[cfg(target_arch = "wasm32")]
mod map;
pub mod movegen;
pub mod tbp;

#[cfg(target_arch = "wasm32")]
pub use bot::BotConfig;

#[cfg(target_arch = "wasm32")]
mod wasm;
