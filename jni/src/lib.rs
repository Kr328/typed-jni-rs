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
pub use vm::attach_vm;

#[macro_export]
macro_rules! define_java_class {
    ($name:ident, $class:literal) => {
        pub struct $name;

        impl ::typed_jni::Type for $name {
            const SIGNATURE: ::typed_jni::Signature = ::typed_jni::Signature::Object($class);
        }

        impl ::typed_jni::ObjectType for $name {}
    };
}
