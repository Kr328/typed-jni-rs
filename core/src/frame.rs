use core::ptr::NonNull;

use crate::{JNIEnv, LocalRef, Ref, helper::call};

impl<'vm> JNIEnv<'vm> {
    /// Run a function with a pushed local frame.
    ///
    /// # Safety
    ///
    /// * The function `f` must create local references only with the given [`JNIEnv`].
    pub unsafe fn with_push_local_frame<R, F>(&self, capacity: i32, f: F) -> Result<(R, Option<LocalRef<'_>>), LocalRef<'_>>
    where
        F: for<'env> FnOnce(&'env JNIEnv<'vm>) -> (R, Option<LocalRef<'env>>),
    {
        self.run_catch(|| unsafe {
            call!(self.as_raw_ptr(), PushLocalFrame, capacity);
        })?;

        let (ret, ret_ref) = f(self);

        #[cfg(debug_assertions)]
        if let Some(ret_ref) = &ret_ref {
            ret_ref.enforce_valid_runtime(self);
        }

        let ret_obj = self.run_catch(|| unsafe {
            let ret_ref = ret_ref.map(|r| r.into_trampoline());

            call!(
                self.as_raw_ptr(),
                PopLocalFrame,
                ret_ref.as_ref().map(|f| f.as_raw_ptr()).unwrap_or(core::ptr::null_mut())
            )
        })?;

        Ok((
            ret,
            NonNull::new(ret_obj).map(|f| unsafe { LocalRef::from_raw(self, f.as_ptr()) }),
        ))
    }

    /// Ensure that the local frame has at least the specified capacity.
    pub fn ensure_local_capacity(&self, capacity: i32) -> Result<(), LocalRef<'_>> {
        self.run_catch(|| unsafe {
            call!(self.as_raw_ptr(), EnsureLocalCapacity, capacity);
        })
    }
}
