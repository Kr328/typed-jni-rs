#![no_std]

//! # Basic JNI Bindings for Rust
//!
//! This crate provides basic JNI bindings for Rust.
//!
//! ## Features
//!
//! - `alloc`: Enables the use of `alloc` crate for dynamic memory allocation. (default)

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

            core::mem::transmute(env)
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
