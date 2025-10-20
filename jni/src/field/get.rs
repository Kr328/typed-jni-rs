use typed_jni_core::{FieldID, JNIEnv};

use crate::{LocalObject, ObjectType, TypedRef, builtin::JavaThrowable, core::StrongRef};

/// This trait is implemented for all types that can be the return value of a field get operation.
///
/// Supported Types:
///
/// * Primitive types: `bool`, `i8`, `u16`, `i32`, `i64`, `f32`, `f64`
/// * Object types: `LocalObject<Type>`, `Option<LocalObject<Type>>`
///
/// # Safety
///
/// This trait should not be implemented manually.
pub unsafe trait Got<'env>: Sized {
    /// Get the value of a field.
    ///
    /// # Safety
    ///
    /// * Signature of `Self` must match the signature of `field`.
    unsafe fn get_of<const STATIC: bool, R: StrongRef>(
        env: &'env JNIEnv,
        obj: &R,
        field: FieldID<STATIC>,
    ) -> Result<Self, LocalObject<'env, JavaThrowable>>;
}

macro_rules! impl_got_for_primitive {
    ($ty:ty, $get:ident) => {
        unsafe impl<'env> Got<'env> for $ty {
            unsafe fn get_of<const STATIC: bool, R: StrongRef>(
                env: &'env JNIEnv,
                obj: &R,
                field: FieldID<STATIC>,
            ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
                unsafe { env.$get(obj, field).map_err(|err| LocalObject::from_ref(err)) }
            }
        }
    };
}

impl_got_for_primitive!(bool, get_boolean_field);
impl_got_for_primitive!(i8, get_byte_field);
impl_got_for_primitive!(u16, get_char_field);
impl_got_for_primitive!(i16, get_short_field);
impl_got_for_primitive!(i32, get_int_field);
impl_got_for_primitive!(i64, get_long_field);
impl_got_for_primitive!(f32, get_float_field);
impl_got_for_primitive!(f64, get_double_field);

macro_rules! impl_got_for_object {
    ($ty:ty, $ret:ident, $transform:block) => {
        unsafe impl<'env, T: ObjectType> Got<'env> for $ty {
            unsafe fn get_of<const STATIC: bool, R: StrongRef>(
                env: &'env JNIEnv,
                obj: &R,
                field: FieldID<STATIC>,
            ) -> Result<Self, LocalObject<'env, JavaThrowable>> {
                unsafe {
                    env.get_object_field(obj, field)
                        .map(|$ret| $transform)
                        .map_err(|err| LocalObject::from_ref(err))
                }
            }
        }
    };
}

impl_got_for_object!(Option<LocalObject<'env, T>>, v, { v.map(|v| LocalObject::from_ref(v)) });
impl_got_for_object!(LocalObject<'env, T>, v, {
    LocalObject::from_ref(v.expect("get: value is null"))
});
