use core::ops::{Deref, DerefMut};

use crate::{JNIEnv, LocalRef, StrongRef, helper::call, sys};

impl<'vm> JNIEnv<'vm> {
    /// Returns the length of an array.
    ///
    /// # Safety
    /// - `arr` must be a valid array.
    pub unsafe fn get_array_length<R: StrongRef>(&self, arr: &R) -> Result<i32, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        arr.enforce_valid_runtime(self);

        self.run_catch(|| unsafe { call!(self.as_raw_ptr(), GetArrayLength, arr.as_raw_ptr()) })
    }
}

impl<'vm> JNIEnv<'vm> {
    /// Creates a new array of objects.
    ///
    /// # Safety
    /// - `cls` must be a valid class.
    pub unsafe fn new_object_array<R: StrongRef>(&self, cls: &R, len: i32) -> Result<LocalRef<'_>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            call!(
                self.as_raw_ptr(),
                NewObjectArray,
                len,
                cls.as_raw_ptr(),
                core::ptr::null_mut()
            )
        })
        .map(|v| unsafe { LocalRef::from_raw(self, v) })
    }

    /// Creates a new array of objects with an initial value.
    ///
    /// # Safety
    /// - `cls` must be a valid class.
    /// - `initial` must be a valid object of `cls`.
    pub unsafe fn new_object_array_with_initial<R: StrongRef, IR: StrongRef>(
        &self,
        cls: &R,
        len: i32,
        initial: &IR,
    ) -> Result<LocalRef<'_>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        #[cfg(debug_assertions)]
        initial.enforce_valid_runtime(self);

        self.run_catch(|| unsafe { call!(self.as_raw_ptr(), NewObjectArray, len, cls.as_raw_ptr(), initial.as_raw_ptr()) })
            .map(|v| unsafe { LocalRef::from_raw(self, v) })
    }

    /// Sets an element of an object array.
    ///
    /// # Safety
    /// - `arr` must be a valid object array.
    /// - `index` must be a valid index of `arr`.
    /// - `value` must be a valid object of `cls` if it is `Some`.
    pub unsafe fn set_object_array_element<R: StrongRef, VR: StrongRef>(
        &self,
        arr: &R,
        index: i32,
        value: Option<&VR>,
    ) -> Result<(), LocalRef<'_>> {
        #[cfg(debug_assertions)]
        arr.enforce_valid_runtime(self);

        #[cfg(debug_assertions)]
        if let Some(value) = value {
            value.enforce_valid_runtime(self);
        }

        self.run_catch(|| unsafe {
            call!(
                self.as_raw_ptr(),
                SetObjectArrayElement,
                arr.as_raw_ptr(),
                index,
                value.map(|v| v.as_raw_ptr()).unwrap_or(core::ptr::null_mut())
            )
        })
    }

    /// Gets an element of an object array.
    ///
    /// # Safety
    /// - `arr` must be a valid object array.
    /// - `index` must be a valid index of `arr`.
    pub unsafe fn get_object_array_element<R: StrongRef>(
        &self,
        arr: &R,
        index: i32,
    ) -> Result<Option<LocalRef<'_>>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        arr.enforce_valid_runtime(self);

        self.run_catch(|| unsafe { call!(self.as_raw_ptr(), GetObjectArrayElement, arr.as_raw_ptr(), index) })
            .map(|v| unsafe {
                if !v.is_null() {
                    Some(LocalRef::from_raw(self, v))
                } else {
                    None
                }
            })
    }
}

/// Guard of an array elements.
pub struct ArrayElementsGuard<'a, T, R: StrongRef> {
    env: &'a JNIEnv<'a>,
    arr: &'a R,
    ptr: *mut T,
    len: i32,
    release: fn(&mut Self, bool),
}

impl<'a, T, R: StrongRef> ArrayElementsGuard<'a, T, R> {
    /// Commits the changes to the array elements.
    pub fn commit(mut self) {
        (self.release)(&mut self, true);

        core::mem::forget(self);
    }
}

impl<'a, T, R: StrongRef> Deref for ArrayElementsGuard<'a, T, R> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len as usize) }
    }
}

impl<'a, T, R: StrongRef> DerefMut for ArrayElementsGuard<'a, T, R> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len as usize) }
    }
}

impl<'a, T, R: StrongRef> Drop for ArrayElementsGuard<'a, T, R> {
    fn drop(&mut self) {
        (self.release)(self, false);
    }
}

macro_rules! define_primitive_array_ops {
    ((
        $new_name:ident, $get_elements_name:ident, $get_region_name:ident, $set_region_name:ident,
        $typ:ty,
        $new:ident,
        $get_elements:ident, $release_elements:ident,
        $get_region:ident, $set_region:ident
    )) => {
        impl<'vm> JNIEnv<'vm> {
            /// Creates a new array
            ///
            /// # Safety
            /// - `len` must be a valid length.
            pub fn $new_name(&self, len: i32) -> Result<LocalRef<'_>, LocalRef<'_>> {
                self.run_catch(|| unsafe { call!(self.as_raw_ptr(), $new, len) })
                    .map(|v| unsafe { LocalRef::from_raw(self, v) })
            }

            /// Returns a guard of an array elements.
            ///
            /// # Safety
            /// - `arr` must be a valid array.
            pub unsafe fn $get_elements_name<'s: 'a, 'a, R: StrongRef>(
                &'s self,
                arr: &'a R,
            ) -> Result<ArrayElementsGuard<'a, $typ, R>, LocalRef<'s>> {
                #[cfg(debug_assertions)]
                arr.enforce_valid_runtime(self);

                let length = unsafe { self.get_array_length(arr)? };

                self.run_catch(|| unsafe {
                    let ptr = call!(
                        self.as_raw_ptr(),
                        $get_elements,
                        arr.as_raw_ptr(),
                        core::ptr::null_mut()
                    );

                    ArrayElementsGuard {
                        env: self,
                        arr,
                        ptr,
                        len: length,
                        release: |guard, commit| {
                            call!(
                                guard.env.as_raw_ptr(),
                                $release_elements,
                                guard.arr.as_raw_ptr(),
                                guard.ptr,
                                if commit { sys::JNI_COMMIT } else { sys::JNI_ABORT }
                            )
                        },
                    }
                })
            }

            /// Copies a region of an array into a native array.
            ///
            /// # Safety
            /// - `arr` must be a valid array.
            /// - `offset` must be a valid offset of `arr`.
            pub unsafe fn $get_region_name<R: StrongRef>(
                &self,
                arr: &R,
                offset: i32,
                buf: &mut [$typ],
            ) -> Result<(), LocalRef<'_>> {
                #[cfg(debug_assertions)]
                arr.enforce_valid_runtime(self);

                self.run_catch(|| unsafe {
                    call!(
                        self.as_raw_ptr(),
                        $get_region,
                        arr.as_raw_ptr(),
                        offset,
                        buf.len() as _,
                        buf.as_mut_ptr()
                    )
                })
            }

            /// Copies a region of a native array into an array.
            ///
            /// # Safety
            /// - `arr` must be a valid array.
            /// - `offset` must be a valid offset of `arr`.
            pub unsafe fn $set_region_name<R: StrongRef>(&self, arr: &R, offset: i32, buf: &[$typ]) -> Result<(), LocalRef<'_>> {
                #[cfg(debug_assertions)]
                arr.enforce_valid_runtime(self);

                self.run_catch(|| unsafe {
                    call!(
                        self.as_raw_ptr(),
                        $set_region,
                        arr.as_raw_ptr(),
                        offset,
                        buf.len() as _,
                        buf.as_ptr()
                    )
                })
            }
        }
    };
}

define_primitive_array_ops!((
    new_boolean_array,
    get_boolean_array_elements,
    get_boolean_array_region,
    set_boolean_array_region,
    bool,
    NewBooleanArray,
    GetBooleanArrayElements,
    ReleaseBooleanArrayElements,
    GetBooleanArrayRegion,
    SetBooleanArrayRegion
));

define_primitive_array_ops!((
    new_byte_array,
    get_byte_array_elements,
    get_byte_array_region,
    set_byte_array_region,
    i8,
    NewByteArray,
    GetByteArrayElements,
    ReleaseByteArrayElements,
    GetByteArrayRegion,
    SetByteArrayRegion
));

define_primitive_array_ops!((
    new_char_array,
    get_char_array_elements,
    get_char_array_region,
    set_char_array_region,
    u16,
    NewCharArray,
    GetCharArrayElements,
    ReleaseCharArrayElements,
    GetCharArrayRegion,
    SetCharArrayRegion
));

define_primitive_array_ops!((
    new_short_array,
    get_short_array_elements,
    get_short_array_region,
    set_short_array_region,
    i16,
    NewShortArray,
    GetShortArrayElements,
    ReleaseShortArrayElements,
    GetShortArrayRegion,
    SetShortArrayRegion
));

define_primitive_array_ops!((
    new_int_array,
    get_int_array_elements,
    get_int_array_region,
    set_int_array_region,
    i32,
    NewIntArray,
    GetIntArrayElements,
    ReleaseIntArrayElements,
    GetIntArrayRegion,
    SetIntArrayRegion
));

define_primitive_array_ops!((
    new_long_array,
    get_long_array_elements,
    get_long_array_region,
    set_long_array_region,
    i64,
    NewLongArray,
    GetLongArrayElements,
    ReleaseLongArrayElements,
    GetLongArrayRegion,
    SetLongArrayRegion
));

define_primitive_array_ops!((
    new_float_array,
    get_float_array_elements,
    get_float_array_region,
    set_float_array_region,
    f32,
    NewFloatArray,
    GetFloatArrayElements,
    ReleaseFloatArrayElements,
    GetFloatArrayRegion,
    SetFloatArrayRegion
));

define_primitive_array_ops!((
    new_double_array,
    get_double_array_elements,
    get_double_array_region,
    set_double_array_region,
    f64,
    NewDoubleArray,
    GetDoubleArrayElements,
    ReleaseDoubleArrayElements,
    GetDoubleArrayRegion,
    SetDoubleArrayRegion
));
