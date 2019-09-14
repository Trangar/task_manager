#![deny(clippy::indexing_slicing)]

pub mod core;
pub mod sys;

pub use crate::core::{Context, Runtime, Task, TryTask};
pub use crate::sys::net;

pub mod prelude {
    pub use crate::core::{Context, Runtime, Task, TryTask};
    pub use crate::sys::net;
}
