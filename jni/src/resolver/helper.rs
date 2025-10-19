use typed_jni_core::{FieldID, JNIEnv, MethodID, StrongRef};

use crate::{LocalObject, Signature, builtin::JavaThrowable};

pub fn resolve_method_by_this<'env, 'a, const STATIC: bool, T: StrongRef, AS: IntoIterator<Item = Signature>>(
    env: &'env JNIEnv,
    this: &T,
    name: &str,
    ret: Signature,
    args: AS,
) -> Result<MethodID<STATIC>, LocalObject<'env, JavaThrowable>> {
    if STATIC {
        super::resolve_method(env, this, name, ret, args)
    } else {
        let cls = env.get_object_class(this);

        super::resolve_method(env, &cls, name, ret, args)
    }
}

pub fn resolve_field_by_this<'env, 'a, const STATIC: bool, T: StrongRef>(
    env: &'env JNIEnv,
    this: &T,
    name: &str,
    sig: Signature,
) -> Result<FieldID<STATIC>, LocalObject<'env, JavaThrowable>> {
    if STATIC {
        super::resolve_field(env, this, name, sig)
    } else {
        let cls = env.get_object_class(this);

        super::resolve_field(env, &cls, name, sig)
    }
}
