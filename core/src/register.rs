use core::ffi::CStr;

use crate::{JNIEnv, LocalRef, StrongRef, helper::call, sys};

pub struct NativeFunction<'a> {
    pub name: &'a CStr,
    pub signature: &'a CStr,
    pub fn_ptr: *const (),
}

impl<'vm> JNIEnv<'vm> {
    /// Registers native methods for a class.
    ///
    /// - `cls` must be a valid class.
    /// - `funcs` must be a valid slice of `NativeFunctions`.
    pub unsafe fn register_natives<const N_FUNCS: usize, R: StrongRef>(
        &self,
        cls: &R,
        funcs: [NativeFunction; N_FUNCS],
    ) -> Result<(), LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            let funcs = funcs.map(|v| sys::JNINativeMethod {
                name: v.name.as_ptr().cast_mut(),
                signature: v.signature.as_ptr().cast_mut(),
                fnPtr: v.fn_ptr as _,
            });

            call!(
                self.as_raw_ptr(),
                RegisterNatives,
                cls.as_raw_ptr(),
                funcs.as_ptr(),
                funcs.len() as _
            );
        })
    }

    /// Registers native methods for a class.
    ///
    /// - `cls` must be a valid class.
    /// - `funcs` must be a valid slice of `NativeFunctions`.
    #[cfg(feature = "alloc")]
    pub unsafe fn register_natives_variadic<R: StrongRef>(&self, cls: &R, funcs: &[NativeFunction]) -> Result<(), LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            let funcs = funcs
                .iter()
                .map(|v| sys::JNINativeMethod {
                    name: v.name.as_ptr().cast_mut(),
                    signature: v.signature.as_ptr().cast_mut(),
                    fnPtr: v.fn_ptr as _,
                })
                .collect::<alloc::vec::Vec<_>>();

            call!(
                self.as_raw_ptr(),
                RegisterNatives,
                cls.as_raw_ptr(),
                funcs.as_ptr(),
                funcs.len() as _
            );
        })
    }

    /// Unregisters native methods for a class.
    ///
    /// - `cls` must be a valid class.
    pub unsafe fn unregister_natives<R: StrongRef>(&self, cls: &R) -> Result<(), LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            call!(self.as_raw_ptr(), UnregisterNatives, cls.as_raw_ptr());
        })
    }
}
