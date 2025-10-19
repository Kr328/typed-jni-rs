use crate::{JNIEnv, StrongRef, helper::call};

/// A guard that releases the monitor of the object when dropped.
pub struct MonitorGuard<'a, 'vm, R: StrongRef + 'a> {
    env: &'a JNIEnv<'vm>,
    obj: &'a R,
}

impl<'a, 'vm, R: StrongRef + 'a> Drop for MonitorGuard<'a, 'vm, R> {
    fn drop(&mut self) {
        unsafe {
            call!(self.env.as_raw_ptr(), MonitorExit, self.obj.as_raw_ptr());
        }
    }
}

impl<'vm> JNIEnv<'vm> {
    /// Enter the monitor of the object.
    #[must_use]
    pub fn monitor_enter<'a, R: StrongRef + 'a>(&'a self, obj: &'a R) -> MonitorGuard<'a, 'vm, R> {
        #[cfg(debug_assertions)]
        obj.enforce_valid_runtime(self);

        unsafe {
            call!(self.as_raw_ptr(), MonitorEnter, obj.as_raw_ptr());
        }

        MonitorGuard { env: self, obj }
    }
}
