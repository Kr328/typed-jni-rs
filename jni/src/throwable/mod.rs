pub(crate) mod helper;

use typed_jni_core::{JNIEnv, StrongRef};

use crate::{LocalObject, Object, TypedRef, builtin::JavaThrowable};

/// Extension methods for typed throwable maintenance.
pub trait TypedThrowableExt {
    /// Catches the throwable if any.
    fn typed_catch(&self) -> Option<LocalObject<'_, JavaThrowable>>;

    /// Throws the throwable.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether the throwable is thrown.
    fn typed_throw<R: StrongRef>(&self, throwable: &Object<R, JavaThrowable>) -> bool;
}

impl<'vm> TypedThrowableExt for JNIEnv<'vm> {
    fn typed_catch(&self) -> Option<LocalObject<'_, JavaThrowable>> {
        unsafe { self.catch().map(|v| LocalObject::from_ref(v)) }
    }

    fn typed_throw<R: StrongRef>(&self, throwable: &Object<R, JavaThrowable>) -> bool {
        if self.has_throwable() {
            return false;
        }

        unsafe {
            self.throw(&**throwable);

            true
        }
    }
}
