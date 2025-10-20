use typed_jni_core::{GlobalRef, JNIEnv, LocalRef, TrampolineRef, WeakGlobalRef};

use crate::{Class, Object, ObjectType, TypedRef};

pub trait NewRef {
    type LocalRef<'env>;

    fn new_local_ref<'env>(&self, env: &'env JNIEnv) -> Self::LocalRef<'env>;

    type GlobalRef<'vm>;

    fn new_global_ref<'vm>(&self, env: &JNIEnv<'vm>) -> Self::GlobalRef<'vm>;

    type WeakGlobalRef<'vm>;

    fn new_weak_global_ref<'vm>(&self, env: &JNIEnv<'vm>) -> Self::WeakGlobalRef<'vm>;
}

macro_rules! impl_new_ref_for_strong {
    ($r:ty, $variant:ident) => {
        impl<T: ObjectType> NewRef for $variant<$r, T> {
            type LocalRef<'env> = $variant<LocalRef<'env>, T>;

            fn new_local_ref<'env>(&self, env: &'env JNIEnv) -> Self::LocalRef<'env> {
                unsafe { $variant::from_ref(env.new_local_ref(&**self).unwrap()) }
            }

            type GlobalRef<'vm> = $variant<GlobalRef<'vm>, T>;

            fn new_global_ref<'vm>(&self, env: &JNIEnv<'vm>) -> Self::GlobalRef<'vm> {
                unsafe { $variant::from_ref(env.new_global_ref(&**self).unwrap()) }
            }

            type WeakGlobalRef<'vm> = $variant<WeakGlobalRef<'vm>, T>;

            fn new_weak_global_ref<'vm>(&self, env: &JNIEnv<'vm>) -> Self::WeakGlobalRef<'vm> {
                unsafe { $variant::from_ref(env.new_weak_global_ref(&**self).unwrap()) }
            }
        }
    };
}

impl_new_ref_for_strong!(LocalRef<'_>, Object);
impl_new_ref_for_strong!(GlobalRef<'_>, Object);
impl_new_ref_for_strong!(TrampolineRef<'_>, Object);
impl_new_ref_for_strong!(LocalRef<'_>, Class);
impl_new_ref_for_strong!(GlobalRef<'_>, Class);
impl_new_ref_for_strong!(TrampolineRef<'_>, Class);

macro_rules! impl_new_ref_for_weak {
    ($r:ty, $variant:ident) => {
        impl<T: ObjectType> NewRef for $variant<$r, T> {
            type LocalRef<'env> = Option<$variant<LocalRef<'env>, T>>;

            fn new_local_ref<'env>(&self, env: &'env JNIEnv) -> Self::LocalRef<'env> {
                unsafe { env.new_local_ref(&**self).map(|r| $variant::from_ref(r)) }
            }

            type GlobalRef<'vm> = Option<$variant<GlobalRef<'vm>, T>>;

            fn new_global_ref<'vm>(&self, env: &JNIEnv<'vm>) -> Self::GlobalRef<'vm> {
                unsafe { env.new_global_ref(&**self).map(|r| $variant::from_ref(r)) }
            }

            type WeakGlobalRef<'vm> = Option<$variant<WeakGlobalRef<'vm>, T>>;

            fn new_weak_global_ref<'vm>(&self, env: &JNIEnv<'vm>) -> Self::WeakGlobalRef<'vm> {
                unsafe { env.new_weak_global_ref(&**self).map(|r| $variant::from_ref(r)) }
            }
        }
    };
}

impl_new_ref_for_weak!(WeakGlobalRef<'_>, Object);
impl_new_ref_for_weak!(WeakGlobalRef<'_>, Class);
