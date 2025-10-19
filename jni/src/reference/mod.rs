mod new_ref;

use typed_jni_core::JNIEnv;

/// Extension methods for typed references maintenance.
pub trait TypedRefExt<'vm> {
    /// Creates a new local reference from the given typed reference.
    fn typed_new_local_ref<T: new_ref::NewRef>(&self, r: &T) -> T::LocalRef<'_>;

    /// Creates a new global reference from the given typed reference.
    fn typed_new_global_ref<T: new_ref::NewRef>(&self, r: &T) -> T::GlobalRef<'vm>;

    /// Creates a new weak global reference from the given typed reference.
    fn typed_new_weak_global_ref<T: new_ref::NewRef>(&self, r: &T) -> T::WeakGlobalRef<'vm>;
}

impl<'vm> TypedRefExt<'vm> for JNIEnv<'vm> {
    fn typed_new_local_ref<T: new_ref::NewRef>(&self, r: &T) -> T::LocalRef<'_> {
        r.new_local_ref(self)
    }

    fn typed_new_global_ref<T: new_ref::NewRef>(&self, r: &T) -> T::GlobalRef<'vm> {
        r.new_global_ref(self)
    }

    fn typed_new_weak_global_ref<T: new_ref::NewRef>(&self, r: &T) -> T::WeakGlobalRef<'vm> {
        r.new_weak_global_ref(self)
    }
}
