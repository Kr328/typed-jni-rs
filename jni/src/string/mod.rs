use alloc::string::String;

use typed_jni_core::{JNIEnv, StrongRef};

use crate::{LocalObject, Object, TypedRef, builtin::JavaString};

/// Extension methods for typed string maintenance.
pub trait TypedStringExt {
    /// Creates a new string from the given string.
    fn typed_new_string(&self, s: impl AsRef<str>) -> LocalObject<'_, JavaString>;

    /// Returns the string slice of the given string object.
    fn typed_get_string(&self, s: &Object<impl StrongRef, JavaString>) -> String;
}

impl<'vm> TypedStringExt for JNIEnv<'vm> {
    fn typed_new_string(&self, s: impl AsRef<str>) -> LocalObject<'_, JavaString> {
        unsafe { LocalObject::from_ref(self.new_string(s)) }
    }

    fn typed_get_string(&self, s: &Object<impl StrongRef, JavaString>) -> String {
        unsafe { self.get_string(&**s) }
    }
}
