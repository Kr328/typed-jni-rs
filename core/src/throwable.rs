use core::ptr::NonNull;

use crate::{JNIEnv, LocalRef, StrongRef, helper::call};

impl<'vm> JNIEnv<'vm> {
    /// Check if exception occurred.
    pub fn has_throwable(&self) -> bool {
        unsafe { call!(self.as_raw_ptr(), ExceptionCheck) }
    }

    /// Catch exception from jvm.
    ///
    /// # Returns
    ///
    /// Returns `None` if no exception occurred.
    pub fn catch(&self) -> Option<LocalRef<'_>> {
        unsafe {
            match NonNull::new(call!(self.as_raw_ptr(), ExceptionOccurred)) {
                None => None,
                Some(ex) => {
                    call!(self.as_raw_ptr(), ExceptionClear);

                    Some(LocalRef::from_raw(self, ex.as_ptr()))
                }
            }
        }
    }

    /// Throw exception to jvm.
    ///
    /// # Safety
    ///
    /// The `ex` must be a valid exception object.
    pub unsafe fn throw<R: StrongRef>(&self, ex: &R) {
        unsafe {
            let ret = call!(self.as_raw_ptr(), Throw, ex.as_raw_ptr());
            assert_eq!(ret, 0);
        }
    }

    /// Run function and catch exception from jvm.
    ///
    /// # Returns
    ///
    /// Returns `Ok(result)` if no exception occurred.
    /// Returns `Err(exception)` if exception occurred.
    pub fn run_catch<R, F>(&self, f: F) -> Result<R, LocalRef<'_>>
    where
        F: FnOnce() -> R,
    {
        unsafe {
            let thread_ex = call!(self.as_raw_ptr(), ExceptionOccurred);
            if !thread_ex.is_null() {
                call!(self.as_raw_ptr(), ExceptionClear);
            }

            let ret = f();

            let ret = match NonNull::new(call!(self.as_raw_ptr(), ExceptionOccurred)) {
                None => Ok(ret),
                Some(ex) => {
                    #[cfg(debug_assertions)]
                    call!(self.as_raw_ptr(), ExceptionDescribe);

                    call!(self.as_raw_ptr(), ExceptionClear);

                    Err(LocalRef::from_raw(self, ex.as_ptr()))
                }
            };

            if !thread_ex.is_null() {
                call!(self.as_raw_ptr(), Throw, thread_ex);
                call!(self.as_raw_ptr(), DeleteLocalRef, thread_ex);
            }

            ret
        }
    }
}
