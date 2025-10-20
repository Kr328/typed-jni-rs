use core::ffi::CStr;

use typed_jni_core::{Arg, JNIEnv};

use crate::{LocalObject, TypedRef, builtin::JavaThrowable, resolver::resolve_class_and_method};

pub fn new_named_exception<'env>(env: &'env JNIEnv, name: &CStr, msg: &str) -> LocalObject<'env, JavaThrowable> {
    let (cls, method) = match resolve_class_and_method(env, name, c"<init>", c"(Ljava/lang/String;)V") {
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
