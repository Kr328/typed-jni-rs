use core::{ptr::NonNull, sync::atomic::AtomicUsize};

use crate::{JNIEnv, helper::call, sys};

/// JavaVM is a handle to the Java VM.
#[repr(transparent)]
pub struct JavaVM {
    ptr: NonNull<sys::JNIInvokeInterface_>,
}

unsafe impl Send for JavaVM {}
unsafe impl Sync for JavaVM {}

impl JavaVM {
    /// Create a new JavaVM from a raw pointer.
    ///
    /// # Safety
    ///
    /// The raw pointer must be valid and point to a valid JavaVM.
    pub unsafe fn from_raw<'a>(raw: *mut sys::JavaVM) -> &'a Self {
        unsafe {
            assert!(!raw.is_null());

            core::mem::transmute(raw)
        }
    }

    /// Get the raw pointer to the JavaVM.
    pub fn as_raw_ptr(&self) -> *mut sys::JavaVM {
        &raw const *self as *mut _
    }
}

pub type AttachHook = fn(&JavaVM);

static ON_ATTACH: AtomicUsize = AtomicUsize::new(0);
static ON_DETACH: AtomicUsize = AtomicUsize::new(0);

impl JavaVM {
    /// Set attach hook. Given hook will be called when a new thread is attached to the Java VM.
    #[must_use]
    pub fn set_attach_hook(hook: AttachHook) -> Option<AttachHook> {
        let old = ON_ATTACH.swap(hook as usize, core::sync::atomic::Ordering::Relaxed);
        unsafe { core::mem::transmute(old) }
    }

    /// Set detach hook. Given hook will be called when a thread is detached from the Java VM.
    #[must_use]
    pub fn set_detach_hook(hook: AttachHook) -> Option<AttachHook> {
        let old = ON_DETACH.swap(hook as usize, core::sync::atomic::Ordering::Relaxed);
        unsafe { core::mem::transmute(old) }
    }

    fn run_hook(&self, hook: &AtomicUsize) {
        unsafe {
            if let Some(hook) = core::mem::transmute::<_, Option<AttachHook>>(hook.load(core::sync::atomic::Ordering::Relaxed)) {
                hook(self);
            }
        }
    }
}

#[derive(Debug)]
pub struct AttachError;

impl JavaVM {
    fn current_env(&self) -> Option<&'_ JNIEnv<'_>> {
        unsafe {
            let mut env: *mut sys::JNIEnv = core::ptr::null_mut();

            let ret = call!(self.as_raw_ptr(), GetEnv, &raw mut env as _, sys::JNI_VERSION_1_4 as _);

            if !env.is_null() && ret == sys::JNI_OK {
                Some(JNIEnv::from_raw(env))
            } else {
                None
            }
        }
    }

    /// Run the given function with the current thread's JNIEnv.
    pub fn with_current_env<F, R>(&self, f: F) -> Option<R>
    where
        F: for<'env> FnOnce(&'env JNIEnv) -> R,
    {
        self.current_env().map(f)
    }

    /// Attach the current thread to the Java VM.
    ///
    /// # Safety
    ///
    /// The JNIEnv not managed by lifetime system, please make sure it is not used after detachment.
    pub unsafe fn attach_current_thread<'a, 's: 'a>(&'s self, as_daemon: bool) -> Result<&'a JNIEnv<'a>, AttachError> {
        unsafe {
            let mut env: *mut sys::JNIEnv = core::ptr::null_mut();
            let args = sys::JavaVMAttachArgs {
                version: sys::JNI_VERSION_1_4 as _,
                name: core::ptr::null_mut(),
                group: core::ptr::null_mut(),
            };

            let ret = if as_daemon {
                call!(
                    self.as_raw_ptr(),
                    AttachCurrentThreadAsDaemon,
                    &raw mut env as _,
                    &raw const args as _
                )
            } else {
                call!(
                    self.as_raw_ptr(),
                    AttachCurrentThread,
                    &raw mut env as _,
                    &raw const args as _
                )
            };

            if ret == sys::JNI_OK {
                self.run_hook(&ON_ATTACH);

                Ok(JNIEnv::from_raw(env))
            } else {
                Err(AttachError)
            }
        }
    }

    /// Detach the current thread from the Java VM.
    pub unsafe fn detach_current_thread(&self) -> Result<(), AttachError> {
        unsafe {
            let ret = call!(self.as_raw_ptr(), DetachCurrentThread);

            if ret == sys::JNI_OK {
                self.run_hook(&ON_DETACH);

                Ok(())
            } else {
                Err(AttachError)
            }
        }
    }

    /// Run the given function with the current thread attached to the Java VM.
    pub fn with_attached_thread<F, R>(&self, as_daemon: bool, f: F) -> Result<R, AttachError>
    where
        F: for<'env> FnOnce(&'env JNIEnv) -> R,
    {
        match self.current_env() {
            Some(env) => {
                unsafe {
                    let ret = call!(env.as_raw_ptr(), PushLocalFrame, 4);
                    assert_eq!(ret, sys::JNI_OK, "BROKEN: cannot push local frame, maybe stack overflow?");
                }

                let ret = f(env);

                unsafe {
                    call!(env.as_raw_ptr(), PopLocalFrame, core::ptr::null_mut());
                }

                Ok(ret)
            }
            None => {
                let env = unsafe { self.attach_current_thread(as_daemon)? };

                let ret = f(env);

                unsafe {
                    self.detach_current_thread()
                        .expect("BROKEN: cannot detach current thread from javavm");
                }

                Ok(ret)
            }
        }
    }
}

impl<'vm> JNIEnv<'vm> {
    /// Get the JavaVM handle.
    pub fn vm(&self) -> &'vm JavaVM {
        unsafe {
            let mut vm = core::ptr::null_mut();

            let ret = call!(self.as_raw_ptr(), GetJavaVM, &raw mut vm as _);
            assert_eq!(ret, sys::JNI_OK, "cannot get JavaVM from env");

            JavaVM::from_raw(vm)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sys;

    #[test]
    fn test_javavm_ptr() {
        let raw = 0x123456780usize as *mut sys::JavaVM;
        let ctx = unsafe { JavaVM::from_raw(raw) };
        let r_raw = ctx.as_raw_ptr();

        assert_eq!(raw, r_raw);
    }
}
