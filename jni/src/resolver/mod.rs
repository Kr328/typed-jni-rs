pub mod helper;

use alloc::{
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use core::{ffi::CStr, fmt::Write};

use typed_jni_core::{Arg, FieldID, JNIEnv, LocalRef, MethodID, StrongRef};

use crate::{LocalObject, Signature, TypedRef, builtin::JavaThrowable};

pub fn resolve_class_and_method_raw<'env, const STATIC: bool>(
    env: &'env JNIEnv,
    cls: &CStr,
    name: &CStr,
    sig: &CStr,
) -> Result<(LocalRef<'env>, MethodID<STATIC>), LocalObject<'env, JavaThrowable>> {
    // TODO: find in cache

    unsafe {
        let cls = env.find_class(cls).map_err(|err| LocalObject::from_ref(err))?;

        let method = env.get_method_id(&cls, name, sig).map_err(|err| LocalObject::from_ref(err))?;

        Ok((cls, method))
    }
}

fn new_class_not_found_exception<'env>(env: &'env JNIEnv, msg: &str) -> LocalObject<'env, JavaThrowable> {
    let (cls, method) =
        match resolve_class_and_method_raw(env, c"java/lang/ClassNotFoundException", c"<init>", c"(Ljava/lang/String;)V") {
            Ok(v) => v,
            Err(err) => return err,
        };

    unsafe {
        match env.new_object(&cls, method, [Arg::Object(Some(&env.new_string(&msg)))]) {
            Ok(ex) => LocalObject::from_ref(ex),
            Err(err) => LocalObject::from_ref(err),
        }
    }
}

fn convert_str_to_cstring<'env>(env: &'env JNIEnv, s: impl Into<Vec<u8>>) -> Result<CString, LocalObject<'env, JavaThrowable>> {
    CString::new(s).map_err(|err| new_class_not_found_exception(env, &err.to_string()))
}

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

    let name = convert_str_to_cstring(env, name)?;
    let sig = convert_str_to_cstring(env, build_method_signature(ret, args))?;

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

    let name = convert_str_to_cstring(env, name)?;
    let sig = convert_str_to_cstring(env, sig.to_string())?;

    unsafe {
        env.get_field_id(cls, name.as_ref(), sig.as_ref())
            .map_err(|err| LocalObject::from_ref(err))
    }
}
