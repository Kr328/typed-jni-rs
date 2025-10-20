#![no_std]

//! # Basic JNI Bindings for Rust
//!
//! This crate provides basic JNI bindings for Rust.
//!
//! ## Example
//!
//! ```rust
//! use typed_jni::core::{Arg, FieldID, JNIEnv, LocalRef, MethodID};
//!
//! pub fn run_jni(env: &JNIEnv) {
//!     unsafe {
//!         let c_system = env.find_class(c"java/lang/System").unwrap();
//!
//!         let f_out: FieldID<true> = env.get_field_id(&c_system, c"out", c"Ljava/io/PrintStream;").unwrap();
//!         let o_out: Option<LocalRef> = env.get_object_field(&c_system, f_out).unwrap();
//!
//!         let c_print_stream = env.get_object_class(o_out.as_ref().unwrap());
//!         let m_println: MethodID<false> = env
//!             .get_method_id(&c_print_stream, c"println", c"(Ljava/lang/String;)V")
//!             .unwrap();
//!
//!         let s_hello = env.new_string("Hello, World!");
//!         env.call_void_method(o_out.as_ref().unwrap(), m_println, [Arg::from(&s_hello)])
//!             .unwrap();
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - `alloc`: Enables the use of `alloc` crate for dynamic memory allocation. (default)
//! - `print-throwable`: Enables the printing of throwable objects.

#[cfg(feature = "alloc")]
extern crate alloc;

mod array;
mod call;
mod field;
mod frame;
mod helper;
mod member;
mod monitor;
mod object;
mod reference;
mod register;
mod string;
pub mod sys;
mod throwable;
mod vm;

use core::{marker::PhantomData, ptr::NonNull};

pub use self::{array::*, call::*, member::*, reference::*, register::*, string::*, vm::*};
use crate::helper::call;

/// A wrapper of raw JNI environment pointer.
#[repr(transparent)]
pub struct JNIEnv<'vm> {
    env: NonNull<sys::JNINativeInterface_>,
    _vm: PhantomData<&'vm ()>,
}

impl<'vm> JNIEnv<'vm> {
    /// Creates a JNIEnv from a raw pointer.
    ///
    /// # Safety
    ///
    /// - `env` must be a valid JNI environment pointer.
    pub unsafe fn from_raw<'a>(env: *mut sys::JNIEnv) -> &'a Self {
        unsafe {
            assert!(!env.is_null());

            &*(env as *const Self)
        }
    }

    /// Returns the raw JNI environment pointer.
    pub fn as_raw_ptr(&self) -> *mut sys::JNIEnv {
        &self.env as *const _ as *mut _
    }
}

impl<'vm> JNIEnv<'vm> {
    /// Returns the version of the JNI environment.
    pub fn version(&self) -> i32 {
        unsafe { call!(self.as_raw_ptr(), GetVersion) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_equal_ptr() {
        let raw = 0x123456780usize as *mut sys::JNIEnv;
        let ctx = unsafe { JNIEnv::from_raw(raw) };
        let r_raw = ctx.as_raw_ptr();

        assert_eq!(raw, r_raw);
    }
}
