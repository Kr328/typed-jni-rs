use typed_jni_core::{ArrayElementsGuard, JNIEnv, LocalRef, StrongRef};

use crate::PrimitiveType;

/// Function table of primitive arrays.
pub trait PrimitiveArrayElement: PrimitiveType + Sized {
    fn new_instance<'env>(env: &'env JNIEnv<'_>, len: i32) -> Result<LocalRef<'env>, LocalRef<'env>>;

    unsafe fn get_region<'env, R: StrongRef>(
        env: &'env JNIEnv<'_>,
        array: &R,
        offset: i32,
        buf: &mut [Self],
    ) -> Result<(), LocalRef<'env>>;

    unsafe fn set_region<'env, R: StrongRef>(
        env: &'env JNIEnv<'_>,
        array: &R,
        offset: i32,
        values: &[Self],
    ) -> Result<(), LocalRef<'env>>;

    unsafe fn get_elements<'env, 'a, R: StrongRef>(
        env: &'env JNIEnv<'_>,
        array: &'a R,
    ) -> Result<ArrayElementsGuard<'a, Self, R>, LocalRef<'env>>
    where
        'env: 'a;
}

macro_rules! impl_primitive_array_element {
    ($typ:ty, $new:ident, $get_region:ident, $set_region:ident, $get_elements:ident) => {
        impl PrimitiveArrayElement for $typ {
            fn new_instance<'env>(env: &'env JNIEnv<'_>, len: i32) -> Result<LocalRef<'env>, LocalRef<'env>> {
                env.$new(len)
            }

            unsafe fn get_region<'env, R: StrongRef>(
                env: &'env JNIEnv<'_>,
                array: &R,
                offset: i32,
                buf: &mut [Self],
            ) -> Result<(), LocalRef<'env>> {
                unsafe { env.$get_region(array, offset, buf) }
            }

            unsafe fn set_region<'env, R: StrongRef>(
                env: &'env JNIEnv<'_>,
                array: &R,
                offset: i32,
                values: &[Self],
            ) -> Result<(), LocalRef<'env>> {
                unsafe { env.$set_region(array, offset, values) }
            }

            unsafe fn get_elements<'env, 'a, R: StrongRef>(
                env: &'env JNIEnv<'_>,
                array: &'a R,
            ) -> Result<ArrayElementsGuard<'a, Self, R>, LocalRef<'env>>
            where
                'env: 'a,
            {
                unsafe { env.$get_elements(array) }
            }
        }
    };
}

impl_primitive_array_element!(
    bool,
    new_boolean_array,
    get_boolean_array_region,
    set_boolean_array_region,
    get_boolean_array_elements
);
impl_primitive_array_element!(
    i8,
    new_byte_array,
    get_byte_array_region,
    set_byte_array_region,
    get_byte_array_elements
);
impl_primitive_array_element!(
    u16,
    new_char_array,
    get_char_array_region,
    set_char_array_region,
    get_char_array_elements
);
impl_primitive_array_element!(
    i16,
    new_short_array,
    get_short_array_region,
    set_short_array_region,
    get_short_array_elements
);
impl_primitive_array_element!(
    i32,
    new_int_array,
    get_int_array_region,
    set_int_array_region,
    get_int_array_elements
);
impl_primitive_array_element!(
    i64,
    new_long_array,
    get_long_array_region,
    set_long_array_region,
    get_long_array_elements
);
impl_primitive_array_element!(
    f32,
    new_float_array,
    get_float_array_region,
    set_float_array_region,
    get_float_array_elements
);
impl_primitive_array_element!(
    f64,
    new_double_array,
    get_double_array_region,
    set_double_array_region,
    get_double_array_elements
);
