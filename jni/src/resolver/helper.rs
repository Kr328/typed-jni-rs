use alloc::{
    borrow::Cow,
    ffi::CString,
    string::{String, ToString},
};
use core::{ffi::CStr, fmt, fmt::Write};

use typed_jni_core::JNIEnv;

use crate::{LocalObject, Signature, builtin::JavaThrowable, throwable};

pub fn build_class_name<'env>(
    env: &'env JNIEnv,
    signature: Signature,
    convert_to_fully_qualified_name: bool,
) -> Result<CString, LocalObject<'env, JavaThrowable>> {
    let capacity = signature.size_hint() + 1;

    let mut name = String::with_capacity(capacity);

    if convert_to_fully_qualified_name {
        struct ConvertToFullyQualifiedNameWriter<'a> {
            name: &'a mut String,
        }

        impl<'a> Write for ConvertToFullyQualifiedNameWriter<'a> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                let mut first = true;
                for part in s.split('/') {
                    if first {
                        first = false;
                        self.name.push_str(part);
                    } else {
                        self.name.push('.');
                        self.name.push_str(part);
                    }
                }
                Ok(())
            }
        }

        let mut writer = ConvertToFullyQualifiedNameWriter { name: &mut name };

        signature.write_as_class_name_to(&mut writer).unwrap();
    } else {
        signature.write_as_class_name_to(&mut name).unwrap();
    }

    CString::new(name)
        .map_err(|err| throwable::helper::new_named_exception(env, c"java/lang/NoSuchClassException", &err.to_string()))
}

pub enum MemberKind {
    Method,
    Field,
}

pub fn build_member_name<'env, 's>(
    env: &'env JNIEnv,
    name: &'s str,
    member: MemberKind,
) -> Result<Cow<'s, CStr>, LocalObject<'env, JavaThrowable>> {
    if name.as_bytes().last() == Some(&b'\0')
        && let Ok(s) = CStr::from_bytes_with_nul(name.as_bytes())
    {
        return Ok(Cow::Borrowed(s));
    }

    CString::new(name).map(Cow::Owned).map_err(|err| {
        throwable::helper::new_named_exception(
            env,
            match member {
                MemberKind::Method => c"java/lang/NoSuchMethodException",
                MemberKind::Field => c"java/lang/NoSuchFieldException",
            },
            &err.to_string(),
        )
    })
}

pub fn build_field_signature<'env>(env: &'env JNIEnv, v: Signature) -> Result<CString, LocalObject<'env, JavaThrowable>> {
    let mut sig = String::with_capacity(v.size_hint() + 1);

    v.write_to(&mut sig).unwrap();

    CString::new(sig)
        .map_err(|err| throwable::helper::new_named_exception(env, c"java/lang/NoSuchFieldException", &err.to_string()))
}

pub fn build_method_signature<'env>(
    env: &'env JNIEnv,
    ret: Signature,
    args: impl IntoIterator<Item = Signature> + Clone,
) -> Result<CString, LocalObject<'env, JavaThrowable>> {
    let capacity = ret.size_hint() + args.clone().into_iter().map(|arg| arg.size_hint()).sum::<usize>() + 2 + 1;

    let mut sig = String::with_capacity(capacity);

    sig.push('(');
    for arg in args {
        arg.write_to(&mut sig).unwrap();
    }
    sig.push(')');

    ret.write_to(&mut sig).unwrap();

    CString::new(sig)
        .map_err(|err| throwable::helper::new_named_exception(env, c"java/lang/NoSuchMethodException", &err.to_string()))
}
