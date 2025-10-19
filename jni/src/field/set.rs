use typed_jni_core::{JNIEnv, StrongRef};

use crate::{LocalObject, Object, ObjectType, TypedRef, builtin::JavaThrowable, core::FieldID};

pub unsafe trait Value {
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

impl_value_for_object!(Option<Object<Ref, Type>>, value, { value.as_ref().map(|v| &**v) });
impl_value_for_object!(Option<&Object<Ref, Type>>, value, { value.as_ref().map(|v| &***v) });
impl_value_for_object!(Object<Ref, Type>, value, { Some(&*value) });
impl_value_for_object!(&Object<Ref, Type>, value, { Some(&**value) });
