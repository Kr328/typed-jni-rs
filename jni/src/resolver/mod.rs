#[cfg(feature = "cache")]
mod cache;
pub(crate) mod helper;

use core::ffi::CStr;

use typed_jni_core::{FieldID, JNIEnv, LocalRef, MethodID, StrongRef};

use crate::{LocalObject, TypedRef, builtin::JavaThrowable};

pub fn resolve_class_and_method<'env, const STATIC: bool>(
    env: &'env JNIEnv,
    cls: &CStr,
    name: &CStr,
    sig: &CStr,
) -> Result<(LocalRef<'env>, MethodID<STATIC>), LocalObject<'env, JavaThrowable>> {
    #[cfg(feature = "cache")]
    if let Some((cls, method)) = cache::find_class_and_method::<STATIC>(env, cls, name, sig) {
        return Ok((cls, method));
    }

    unsafe {
        let cls_obj = env.find_class(cls).map_err(|err| LocalObject::from_ref(err))?;
        let method = env
            .get_method_id(&cls_obj, name, sig)
            .map_err(|err| LocalObject::from_ref(err))?;

        #[cfg(feature = "cache")]
        cache::put_class_and_method::<STATIC, _>(env, cls, name, sig, &cls_obj, method);

        Ok((cls_obj, method))
    }
}

pub fn resolve_method<'env, const STATIC: bool, C: StrongRef>(
    env: &'env JNIEnv,
    cls: &C,
    name: &CStr,
    signature: &CStr,
) -> Result<MethodID<STATIC>, LocalObject<'env, JavaThrowable>> {
    #[cfg(feature = "cache")]
    if let Some(method) = cache::find_method::<STATIC, _>(env, cls, name, signature) {
        return Ok(method);
    }

    unsafe {
        let method = env
            .get_method_id(cls, name, signature)
            .map_err(|err| LocalObject::from_ref(err))?;

        #[cfg(feature = "cache")]
        cache::put_method::<STATIC, _>(env, cls, name, signature, method);

        Ok(method)
    }
}

pub fn resolve_field<'env, const STATIC: bool, C: StrongRef>(
    env: &'env JNIEnv,
    cls: &C,
    name: &CStr,
    signature: &CStr,
) -> Result<FieldID<STATIC>, LocalObject<'env, JavaThrowable>> {
    #[cfg(feature = "cache")]
    if let Some(field) = cache::find_field::<STATIC, _>(env, cls, name, signature) {
        return Ok(field);
    }

    unsafe {
        let field = env
            .get_field_id(cls, name, signature)
            .map_err(|err| LocalObject::from_ref(err))?;

        #[cfg(feature = "cache")]
        cache::put_field::<STATIC, _>(env, cls, name, signature, field);

        Ok(field)
    }
}
