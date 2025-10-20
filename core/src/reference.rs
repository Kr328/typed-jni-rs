use core::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{JNIEnv, helper::call, sys, vm::JavaVM};

mod __sealed {
    pub trait Sealed {}
}

/// A reference to a Java object.
pub trait Ref: Debug + __sealed::Sealed {
    /// Returns the raw pointer to the Java object.
    fn as_raw_ptr(&self) -> sys::jobject;

    #[cfg(debug_assertions)]
    fn enforce_valid_runtime(&self, env: &JNIEnv);
}

/// A strong reference to a Java object.
pub trait StrongRef: Ref {}

/// A weak reference to a Java object.
pub trait WeakRef: Ref {}

/// TrampolineRef is a local reference to a Java object but only usable for native function implementation.
///
/// Local(trampoline) references are only valid within the same thread and are automatically
/// released when the thread returns to the Java VM.
///
/// **NOTE**: Trampoline references is the only FFI-Safe reference type.
#[repr(transparent)]
pub struct TrampolineRef<'env> {
    ptr: NonNull<sys::_jobject>,
    _env: PhantomData<&'env JNIEnv<'env>>,
}

impl<'env> __sealed::Sealed for TrampolineRef<'env> {}

impl<'env> Debug for TrampolineRef<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrampolineRef").field("ptr", &self.ptr).finish()
    }
}

impl<'env> Ref for TrampolineRef<'env> {
    fn as_raw_ptr(&self) -> sys::jobject {
        self.ptr.as_ptr()
    }

    #[cfg(debug_assertions)]
    fn enforce_valid_runtime(&self, _: &JNIEnv) {
        // no-op
    }
}

impl<'env> StrongRef for TrampolineRef<'env> {}

/// LocalRef is a local reference to a Java object.
///
/// Local references are only valid within the same thread and are automatically
/// released when the thread returns to the Java VM.
pub struct LocalRef<'env> {
    env: &'env JNIEnv<'env>,
    ptr: NonNull<sys::_jobject>,
}

impl<'env> LocalRef<'env> {
    /// Creates a new local reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the raw pointer is valid and points to a Java object.
    pub unsafe fn from_raw(env: &'env JNIEnv<'env>, raw: sys::jobject) -> Self {
        Self {
            env,
            ptr: NonNull::new(raw).expect("create local reference from null pointer"),
        }
    }

    /// Converts this local reference to a trampoline reference.
    pub fn into_trampoline(self) -> TrampolineRef<'env> {
        let r = TrampolineRef {
            ptr: self.ptr,
            _env: PhantomData,
        };

        core::mem::forget(self);

        r
    }

    /// Returns the JNIEnv associated with this local reference.
    pub fn env(&'_ self) -> &'env JNIEnv<'env> {
        self.env
    }
}

impl<'env> __sealed::Sealed for LocalRef<'env> {}

impl<'env> Debug for LocalRef<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LocalRef").field("ptr", &self.ptr).finish()
    }
}

impl<'env> Ref for LocalRef<'env> {
    fn as_raw_ptr(&self) -> sys::jobject {
        self.ptr.as_ptr()
    }

    #[cfg(debug_assertions)]
    fn enforce_valid_runtime(&self, env: &JNIEnv) {
        assert!(core::ptr::eq(self.env.vm(), env.vm()));
    }
}

impl<'env> StrongRef for LocalRef<'env> {}

impl<'env> Clone for LocalRef<'env> {
    fn clone(&self) -> Self {
        let new_ptr = unsafe { call!(self.env.as_raw_ptr(), NewLocalRef, self.ptr.as_ptr()) };

        Self {
            env: self.env,
            ptr: NonNull::new(new_ptr).expect("BROKEN: cannot create new local reference, maybe out of memory?"),
        }
    }
}

impl<'env> Drop for LocalRef<'env> {
    fn drop(&mut self) {
        unsafe {
            call!(self.env.as_raw_ptr(), DeleteLocalRef, self.ptr.as_ptr());
        }
    }
}

/// GlobalRef is a global reference to a Java object.
///
/// Global references are valid across multiple threads and are automatically
/// released when the Java VM is destroyed.
pub struct GlobalRef<'vm> {
    vm: &'vm JavaVM,
    ptr: NonNull<sys::_jobject>,
}

impl<'vm> GlobalRef<'vm> {
    /// Creates a new global reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the raw pointer is valid and points to a Java object.
    pub unsafe fn from_raw(vm: &'vm JavaVM, raw: sys::jobject) -> Self {
        Self {
            vm,
            ptr: NonNull::new(raw).expect("create global reference from null pointer"),
        }
    }

    /// Returns the JavaVM associated with this global reference.
    pub fn vm(&self) -> &'vm JavaVM {
        self.vm
    }
}

impl<'vm> __sealed::Sealed for GlobalRef<'vm> {}

impl<'vm> Debug for GlobalRef<'vm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GlobalRef").field("ptr", &self.ptr).finish()
    }
}

impl<'vm> Ref for GlobalRef<'vm> {
    fn as_raw_ptr(&self) -> sys::jobject {
        self.ptr.as_ptr()
    }

    #[cfg(debug_assertions)]
    fn enforce_valid_runtime(&self, env: &JNIEnv) {
        assert!(core::ptr::eq(self.vm, env.vm()));
    }
}

impl<'vm> StrongRef for GlobalRef<'vm> {}

impl<'vm> Clone for GlobalRef<'vm> {
    fn clone(&self) -> Self {
        self.vm
            .with_attached_thread(true, |env| {
                let new_ptr = unsafe { call!(env.as_raw_ptr(), NewGlobalRef, self.ptr.as_ptr()) };

                Self {
                    vm: self.vm,
                    ptr: NonNull::new(new_ptr).expect("BROKEN: cannot create new global reference, maybe out of memory?"),
                }
            })
            .expect("BROKEN: attach thread to javavm failed")
    }
}

impl<'vm> Drop for GlobalRef<'vm> {
    fn drop(&mut self) {
        self.vm
            .with_attached_thread(true, |env| unsafe {
                call!(env.as_raw_ptr(), DeleteGlobalRef, self.ptr.as_ptr());
            })
            .expect("BROKEN: attach thread to javavm failed")
    }
}

unsafe impl<'vm> Send for GlobalRef<'vm> {}
unsafe impl<'vm> Sync for GlobalRef<'vm> {}

/// GlobalWeakRef is a weak global reference to a Java object.
///
/// Weak global references are valid across multiple threads and are automatically
/// released when the Java VM is destroyed.
pub struct WeakGlobalRef<'vm> {
    vm: &'vm JavaVM,
    ptr: NonNull<sys::_jobject>,
}

impl<'vm> WeakGlobalRef<'vm> {
    /// Creates a new weak global reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the raw pointer is valid and points to a Java object.
    pub unsafe fn from_raw(vm: &'vm JavaVM, raw: sys::jobject) -> Self {
        Self {
            vm,
            ptr: NonNull::new(raw).expect("create weak global reference from null pointer"),
        }
    }

    /// Returns the JavaVM associated with this weak global reference.
    pub fn vm(&self) -> &'vm JavaVM {
        self.vm
    }
}

impl<'vm> __sealed::Sealed for WeakGlobalRef<'vm> {}

impl<'vm> Debug for WeakGlobalRef<'vm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GlobalWeakRef").field("ptr", &self.ptr).finish()
    }
}

impl<'vm> Ref for WeakGlobalRef<'vm> {
    fn as_raw_ptr(&self) -> sys::jobject {
        self.ptr.as_ptr()
    }

    #[cfg(debug_assertions)]
    fn enforce_valid_runtime(&self, env: &JNIEnv) {
        assert!(core::ptr::eq(self.vm, env.vm()));
    }
}

impl<'vm> WeakRef for WeakGlobalRef<'vm> {}

impl<'vm> Drop for WeakGlobalRef<'vm> {
    fn drop(&mut self) {
        self.vm
            .with_attached_thread(true, |env| unsafe {
                call!(env.as_raw_ptr(), DeleteWeakGlobalRef, self.ptr.as_ptr());
            })
            .expect("BROKEN: attach thread to javavm failed")
    }
}

unsafe impl<'vm> Send for WeakGlobalRef<'vm> {}
unsafe impl<'vm> Sync for WeakGlobalRef<'vm> {}

impl<'vm> JNIEnv<'vm> {
    /// Creates a new local reference to the given reference.
    pub fn new_local_ref<R: Ref>(&self, r: &R) -> Option<LocalRef<'_>> {
        unsafe {
            #[cfg(debug_assertions)]
            r.enforce_valid_runtime(self);

            let raw = self
                .run_catch(|| call!(self.as_raw_ptr(), NewLocalRef, r.as_raw_ptr()))
                .expect("BROKEN: cannot create new local reference, maybe out of memory?");

            Some(LocalRef::from_raw(self, NonNull::new(raw)?.as_ptr()))
        }
    }

    /// Creates a new global reference to the given reference.
    pub fn new_global_ref<R: Ref>(&self, r: &R) -> Option<GlobalRef<'vm>> {
        unsafe {
            #[cfg(debug_assertions)]
            r.enforce_valid_runtime(self);

            let raw = self
                .run_catch(|| call!(self.as_raw_ptr(), NewGlobalRef, r.as_raw_ptr()))
                .expect("BROKEN: cannot create new global reference, maybe out of memory?");

            Some(GlobalRef::from_raw(self.vm(), NonNull::new(raw)?.as_ptr()))
        }
    }

    /// Creates a new weak global reference to the given reference.
    pub fn new_weak_global_ref<R: Ref>(&self, r: &R) -> Option<WeakGlobalRef<'vm>> {
        unsafe {
            #[cfg(debug_assertions)]
            r.enforce_valid_runtime(self);

            let raw = self
                .run_catch(|| call!(self.as_raw_ptr(), NewWeakGlobalRef, r.as_raw_ptr()))
                .expect("BROKEN: cannot create new weak global reference, maybe out of memory?");

            Some(WeakGlobalRef::from_raw(self.vm(), NonNull::new(raw)?.as_ptr()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{GlobalRef, WeakGlobalRef};

    fn enforce_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_global_refs() {
        enforce_send_sync::<GlobalRef>();
        enforce_send_sync::<WeakGlobalRef>();
    }
}
