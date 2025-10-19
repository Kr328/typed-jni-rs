pub mod helper;

use alloc::{
    ffi::CString,
    string::{String, ToString},
};
use core::{ffi::CStr, fmt::Write};

use typed_jni_core::{FieldID, JNIEnv, MethodID, StrongRef};

use crate::{LocalObject, Signature, TypedRef, builtin::JavaThrowable};

fn build_method_signature<'s>(ret: Signature, args: impl IntoIterator<Item = Signature>) -> String {
    let mut s = String::with_capacity(16);

    write!(s, "(").unwrap();

    for arg in args {
        write!(s, "{}", arg).unwrap();
    }

    write!(s, ")").unwrap();

    write!(s, "{}", ret).unwrap();

    s
}

pub fn resolve_method<'env, const STATIC: bool, C: StrongRef, AS: IntoIterator<Item = Signature>>(
    env: &'env JNIEnv,
    cls: &C,
    name: &str,
    ret: Signature,
    args: AS,
) -> Result<MethodID<STATIC>, LocalObject<'env, JavaThrowable>> {
    // TODO: find in cache

    let name = CString::new(name).unwrap(); // TODO: throw ClassNotFoundException
    let sig = CString::new(build_method_signature(ret, args)).unwrap(); // TODO: throw ClassNotFoundException

    unsafe {
        env.get_method_id(cls, name.as_ref(), sig.as_ref())
            .map_err(|err| LocalObject::from_ref(err))
    }
}

pub fn resolve_field<'env, const STATIC: bool, C: StrongRef>(
    env: &'env JNIEnv,
    cls: &C,
    name: &str,
    sig: Signature,
) -> Result<FieldID<STATIC>, LocalObject<'env, JavaThrowable>> {
    // TODO: find in cache

    let name = CString::new(name).unwrap(); // TODO: throw ClassNotFoundException
    let sig = CString::new(sig.to_string()).unwrap(); // TODO: throw ClassNotFoundException

    unsafe {
        env.get_field_id(cls, name.as_ref(), sig.as_ref())
            .map_err(|err| LocalObject::from_ref(err))
    }
}

pub fn resolve_class_and_method_raw<'env, const STATIC: bool>(
    env: &'env JNIEnv,
    cls: &CStr,
    name: &CStr,
    sig: &CStr,
) -> Result<MethodID<STATIC>, LocalObject<'env, JavaThrowable>> {
    // TODO: find in cache

    unsafe {
        let cls = env.find_class(cls).map_err(|err| LocalObject::from_ref(err))?;

        env.get_method_id(&cls, name, sig).map_err(|err| LocalObject::from_ref(err))
    }
}
