#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

mod args;
mod builtin;
mod context;
mod ext;
mod raw;
mod reference;
mod resolver;
pub mod sys;
mod typed;
mod vm;

pub use args::*;
pub use builtin::*;
pub use context::*;
pub use raw::*;
pub use reference::*;
pub use typed::*;
pub use vm::{attach_vm, require_vm};

#[doc(hidden)]
pub const unsafe fn __class_name_to_internal_name_bytes<const N: usize>(s: &'static str) -> [u8; N] {
    let data = s.as_bytes();
    let mut ret = [0u8; N];

    let mut index = 0;
    while index < N {
        if data[index] == b'.' {
            ret[index] = b'/';
        } else {
            ret[index] = data[index];
        }

        index += 1;
    }

    ret
}

#[doc(hidden)]
pub const unsafe fn __bytes_to_str(bytes: &'static [u8]) -> &'static str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}

#[macro_export]
macro_rules! define_java_class {
    ($name:ident, $class:literal) => {
        pub struct $name;

        impl $crate::Type for $name {
            const SIGNATURE: $crate::Signature = $crate::Signature::Object({
                unsafe {
                    const REPLACED: [u8; ($class).len()] = unsafe { $crate::__class_name_to_internal_name_bytes($class) };

                    $crate::__bytes_to_str(&REPLACED)
                }
            });
        }

        impl $crate::ObjectType for $name {}
    };
}
