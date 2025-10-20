use typed_jni_core::{Arg, JNIEnv, MethodID, StrongRef};

use crate::{LocalObject, ObjectType, TypedRef, builtin::JavaThrowable};

/// A target for a method call.
///
/// This trait is implemented for all types that can be the return  of a method call.
///
/// Supported Types:
///
/// * No return: `()`
/// * Primitive types: `bool`, `i8`, `u16`, `i32`, `i64`, `f32`, `f64`
/// * Object types: `LocalObject<Type>`, `Option<LocalObject<Type>>`
pub unsafe trait Target<'env>: Sized {
    unsafe fn call<const STATIC: bool, const N_ARGS: usize, T: StrongRef>(
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>,
        args: [Arg<'_>; N_ARGS],
    ) -> Result<Self, LocalObject<'env, JavaThrowable>>;

    unsafe fn call_variadic<'a, const STATIC: bool, T: StrongRef, Args: IntoIterator<Item = Arg<'a>>>(
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>,
        args: Args,
    ) -> Result<Self, LocalObject<'env, JavaThrowable>>;
}

macro_rules! impl_target_for_primitive {
    ($ty:ty, $call:ident, $call_variadic:ident) => {
        unsafe impl<'env> Target<'env> for $ty {
            unsafe fn call<const STATIC: bool, const N_ARGS: usize, T: StrongRef>(
                env: &'env JNIEnv,
                this: &T,
                method: MethodID<STATIC>,
                args: [Arg<'_>; N_ARGS],
            ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
                unsafe { env.$call(this, method, args).map_err(|err| LocalObject::from_ref(err)) }
            }

            unsafe fn call_variadic<'a, const STATIC: bool, T: StrongRef, Args: IntoIterator<Item = Arg<'a>>>(
                env: &'env JNIEnv,
                this: &T,
                method: MethodID<STATIC>,
                args: Args,
            ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
                unsafe {
                    env.$call_variadic(this, method, args)
                        .map_err(|err| LocalObject::from_ref(err))
                }
            }
        }
    };
}

impl_target_for_primitive!((), call_void_method, call_void_method_variadic);
impl_target_for_primitive!(bool, call_boolean_method, call_boolean_method_variadic);
impl_target_for_primitive!(i8, call_byte_method, call_byte_method_variadic);
impl_target_for_primitive!(u16, call_char_method, call_char_method_variadic);
impl_target_for_primitive!(i16, call_short_method, call_short_method_variadic);
impl_target_for_primitive!(i32, call_int_method, call_int_method_variadic);
impl_target_for_primitive!(i64, call_long_method, call_long_method_variadic);
impl_target_for_primitive!(f32, call_float_method, call_float_method_variadic);
impl_target_for_primitive!(f64, call_double_method, call_double_method_variadic);

macro_rules! impl_target_for_object {
    ($ty:ty, $ret:ident, $transform:block) => {
        unsafe impl<'env, Type: ObjectType> Target<'env> for $ty {
            unsafe fn call<const STATIC: bool, const N_ARGS: usize, T: StrongRef>(
                env: &'env JNIEnv,
                this: &T,
                method: MethodID<STATIC>,
                args: [Arg<'_>; N_ARGS],
            ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
                unsafe {
                    env.call_object_method(this, method, args)
                        .map(|$ret| $transform)
                        .map_err(|err| LocalObject::from_ref(err))
                }
            }

            unsafe fn call_variadic<'a, const STATIC: bool, T: StrongRef, Args: IntoIterator<Item = Arg<'a>>>(
                env: &'env JNIEnv,
                this: &T,
                method: MethodID<STATIC>,
                args: Args,
            ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
                unsafe {
                    env.call_object_method_variadic(this, method, args)
                        .map(|$ret| $transform)
                        .map_err(|err| LocalObject::from_ref(err))
                }
            }
        }
    };
}

impl_target_for_object!(Option<LocalObject<'env, Type>>, ret, {
    ret.map(|v| LocalObject::from_ref(v))
});
impl_target_for_object!(LocalObject<'env, Type>, ret, {
    LocalObject::from_ref(ret.expect("call returning null"))
});

pub struct NewObject<'env, T: ObjectType>(pub LocalObject<'env, T>);

unsafe impl<'env, Type: ObjectType> Target<'env> for NewObject<'env, Type> {
    unsafe fn call<const STATIC: bool, const N_ARGS: usize, T: StrongRef>(
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>, // must be Method<false>
        args: [Arg<'_>; N_ARGS],
    ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
        unsafe {
            env.new_object(&*this, core::mem::transmute(method), args)
                .map(|v| Self(LocalObject::from_ref(v)))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }

    unsafe fn call_variadic<'a, const STATIC: bool, T: StrongRef, Args: IntoIterator<Item = Arg<'a>>>(
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>, // must be Method<false>
        args: Args,
    ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
        unsafe {
            env.new_object_variadic(&*this, core::mem::transmute(method), args)
                .map(|v| Self(LocalObject::from_ref(v)))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }
}
