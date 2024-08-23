#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

mod builtin;
mod context;
mod raw;
mod reference;
mod resolver;
pub mod sys;
mod typed;
mod vm;

pub use builtin::*;
pub use context::*;
pub use raw::*;
pub use reference::*;
pub use typed::*;
pub use typed_jni_macros::*;
pub use vm::attach_vm;
