use core::ops::{Deref, DerefMut};

use typed_jni_core::{ArrayElementsGuard, JNIEnv, StrongRef};

use crate::{Array, LocalObject, Object, TypedArrayExt, TypedRef, array::primitive_impls, builtin::JavaThrowable};

/// A guard for byte array elements.
pub struct BytesArrayElementsGuard<'a, R: StrongRef>(ArrayElementsGuard<'a, i8, R>);

impl<'a, R: StrongRef> BytesArrayElementsGuard<'a, R> {
    /// Commits the changes made to the array elements.
    pub fn commit(self) {
        self.0.commit()
    }
}

impl<'a, R: StrongRef> Deref for BytesArrayElementsGuard<'a, R> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.0.as_ptr() as *const u8, self.0.len()) }
    }
}

impl<'a, R: StrongRef> DerefMut for BytesArrayElementsGuard<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.0.as_ptr() as *mut u8, self.0.len()) }
    }
}

/// Extensions for primitive arrays.
///
/// This trait provides methods for working with primitive arrays.
///
/// Valid primitive types:
/// - `i8`: byte (also allow access as `u8`)
/// - `i16`: short
/// - `i32`: int
/// - `i64`: long
/// - `f32`: float
/// - `f64`: double
pub trait TypedPrimitiveArrayExt: TypedArrayExt {
    /// Creates a new primitive array of the specified type and length.
    fn typed_new_primitive_array<T: primitive_impls::PrimitiveArrayElement>(
        &self,
        len: i32,
    ) -> Result<LocalObject<'_, Array<T>>, LocalObject<'_, JavaThrowable>>;

    /// Reads a region of the array into the provided slice.
    fn typed_get_array_region<R: StrongRef, T: primitive_impls::PrimitiveArrayElement>(
        &self,
        array: &Object<R, Array<T>>,
        offset: i32,
        out: &mut [T],
    ) -> Result<(), LocalObject<'_, JavaThrowable>>;

    /// Writes a region of the array from the provided slice.
    fn typed_set_array_region<R: StrongRef, T: primitive_impls::PrimitiveArrayElement>(
        &self,
        array: &Object<R, Array<T>>,
        offset: i32,
        values: &[T],
    ) -> Result<(), LocalObject<'_, JavaThrowable>>;

    /// Get array elements.
    fn typed_get_array_elements<'env, 'a, R: StrongRef, T: primitive_impls::PrimitiveArrayElement>(
        &'env self,
        array: &'a Object<R, Array<T>>,
    ) -> Result<ArrayElementsGuard<'a, T, R>, LocalObject<'env, JavaThrowable>>
    where
        'env: 'a;

    /// Reads a region of the byte array into the provided slice.
    fn typed_get_bytes_array_region<R: StrongRef>(
        &self,
        array: &Object<R, Array<i8>>,
        offset: i32,
        out: &mut [u8],
    ) -> Result<(), LocalObject<'_, JavaThrowable>>;

    /// Writes a region of the byte array from the provided slice.
    fn typed_set_bytes_array_region<R: StrongRef>(
        &self,
        array: &Object<R, Array<i8>>,
        offset: i32,
        values: &[u8],
    ) -> Result<(), LocalObject<'_, JavaThrowable>>;

    /// Get byte array elements.
    fn typed_get_bytes_array_elements<'env, 'a, R: StrongRef>(
        &'env self,
        array: &'a Object<R, Array<i8>>,
    ) -> Result<BytesArrayElementsGuard<'a, R>, LocalObject<'env, JavaThrowable>>
    where
        'env: 'a;
}

impl<'vm> TypedPrimitiveArrayExt for JNIEnv<'vm> {
    fn typed_new_primitive_array<T: primitive_impls::PrimitiveArrayElement>(
        &self,
        len: i32,
    ) -> Result<LocalObject<'_, Array<T>>, LocalObject<'_, JavaThrowable>> {
        unsafe {
            T::new(self, len)
                .map(|arr| LocalObject::from_ref(arr))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }

    fn typed_get_array_region<R: StrongRef, T: primitive_impls::PrimitiveArrayElement>(
        &self,
        array: &Object<R, Array<T>>,
        offset: i32,
        out: &mut [T],
    ) -> Result<(), LocalObject<'_, JavaThrowable>> {
        unsafe { T::get_region(self, &**array, offset, out).map_err(|err| LocalObject::from_ref(err)) }
    }

    fn typed_set_array_region<R: StrongRef, T: primitive_impls::PrimitiveArrayElement>(
        &self,
        array: &Object<R, Array<T>>,
        offset: i32,
        values: &[T],
    ) -> Result<(), LocalObject<'_, JavaThrowable>> {
        unsafe { T::set_region(self, &**array, offset, values).map_err(|err| LocalObject::from_ref(err)) }
    }

    fn typed_get_array_elements<'env, 'a, R: StrongRef, T: primitive_impls::PrimitiveArrayElement>(
        &'env self,
        array: &'a Object<R, Array<T>>,
    ) -> Result<ArrayElementsGuard<'a, T, R>, LocalObject<'env, JavaThrowable>>
    where
        'env: 'a,
    {
        unsafe { T::get_elements(self, &**array).map_err(|err| LocalObject::from_ref(err)) }
    }

    fn typed_get_bytes_array_region<R: StrongRef>(
        &self,
        array: &Object<R, Array<i8>>,
        offset: i32,
        out: &mut [u8],
    ) -> Result<(), LocalObject<'_, JavaThrowable>> {
        unsafe {
            let out = core::slice::from_raw_parts_mut(out.as_mut_ptr() as *mut i8, out.len());

            self.typed_get_array_region(array, offset, out)
        }
    }

    fn typed_set_bytes_array_region<R: StrongRef>(
        &self,
        array: &Object<R, Array<i8>>,
        offset: i32,
        values: &[u8],
    ) -> Result<(), LocalObject<'_, JavaThrowable>> {
        unsafe {
            let values = core::slice::from_raw_parts(values.as_ptr() as *const i8, values.len());

            self.typed_set_array_region(array, offset, values)
        }
    }

    fn typed_get_bytes_array_elements<'env, 'a, R: StrongRef>(
        &'env self,
        array: &'a Object<R, Array<i8>>,
    ) -> Result<BytesArrayElementsGuard<'a, R>, LocalObject<'env, JavaThrowable>>
    where
        'env: 'a,
    {
        self.typed_get_array_elements(array).map(BytesArrayElementsGuard)
    }
}
