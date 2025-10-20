use typed_jni_core::{JNIEnv, LocalRef, StrongRef};

use crate::{LocalObject, Null, Object, ObjectType, TypedRef, builtin::JavaThrowable, core::FieldID};

/// This trait is implemented for all types that can be the argument of a field set operation.
///
/// Supported Types:
///
/// * Primitive types: `bool`, `i8`, `u16`, `i32`, `i64`, `f32`, `f64`
/// * Object types: `Object<impl StrongRef, Type>`, `Option<Object<impl StrongRef, Type>>`
///
/// # Safety
///
/// This trait should not be implemented manually.
pub unsafe trait Value {
    /// Set the value of a field.
    ///
    /// # Safety
    ///
    /// * Signature of `Self` must match the signature of `field`.
    unsafe fn set_on<'env, const STATIC: bool, R: StrongRef>(
        self,
        env: &'env JNIEnv,
        this: &R,
        field: FieldID<STATIC>,
    ) -> Result<(), LocalObject<'env, JavaThrowable>>;
}

macro_rules! impl_value_for_primitive {
    ($typ:ty, $set:ident) => {
        unsafe impl Value for $typ {
            unsafe fn set_on<'env, const STATIC: bool, R: StrongRef>(
                self,
                env: &'env JNIEnv,
                this: &R,
                field: FieldID<STATIC>,
            ) -> Result<(), LocalObject<'env, JavaThrowable>> {
                unsafe { env.$set(this, field, self).map_err(|err| LocalObject::from_ref(err)) }
            }
        }
    };
}

impl_value_for_primitive!(bool, set_boolean_field);
impl_value_for_primitive!(i8, set_byte_field);
impl_value_for_primitive!(u16, set_char_field);
impl_value_for_primitive!(i16, set_short_field);
impl_value_for_primitive!(i32, set_int_field);
impl_value_for_primitive!(i64, set_long_field);
impl_value_for_primitive!(f32, set_float_field);
impl_value_for_primitive!(f64, set_double_field);

macro_rules! impl_value_for_object {
    ($typ:ty, $value:ident, $extract:block) => {
        unsafe impl<Ref: StrongRef, Type: ObjectType> Value for $typ {
            unsafe fn set_on<'env, const STATIC: bool, R: StrongRef>(
                self,
                env: &'env JNIEnv,
                this: &R,
                field: FieldID<STATIC>,
            ) -> Result<(), LocalObject<'env, JavaThrowable>> {
                unsafe {
                    let $value = self;

                    env.set_object_field(this, field, $extract)
                        .map_err(|err| LocalObject::from_ref(err))
                }
            }
        }
    };
}

impl_value_for_object!(Option<Object<Ref, Type>>, value, { value.as_deref() });
impl_value_for_object!(Option<&Object<Ref, Type>>, value, { value.as_ref().map(|v| &***v) });
impl_value_for_object!(Object<Ref, Type>, value, { Some(&*value) });
impl_value_for_object!(&Object<Ref, Type>, value, { Some(&**value) });

unsafe impl<T: ObjectType> Value for Null<T> {
    unsafe fn set_on<'env, const STATIC: bool, R: StrongRef>(
        self,
        env: &'env JNIEnv,
        this: &R,
        field: FieldID<STATIC>,
    ) -> Result<(), LocalObject<'env, JavaThrowable>> {
        unsafe {
            env.set_object_field(this, field, Option::<&LocalRef>::None)
                .map_err(|err| LocalObject::from_ref(err))
        }
    }
}
