use core::{ffi::CStr, ptr::NonNull};

use crate::{JNIEnv, LocalRef, StrongRef, helper::call, sys};

/// Represents a method ID.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct MethodID<const STATIC: bool> {
    ptr: NonNull<sys::_jmethodID>,
}

/// Represents a field ID.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct FieldID<const STATIC: bool> {
    ptr: NonNull<sys::_jfieldID>,
}

macro_rules! impl_common_member_id {
    ($name:ident, $raw:ty) => {
        impl<const STATIC: bool> $name<STATIC> {
            pub unsafe fn from_raw(raw: $raw) -> Self {
                Self {
                    ptr: NonNull::new(raw).expect("create id from null pointer"),
                }
            }

            pub fn as_raw_ptr(&self) -> $raw {
                self.ptr.as_ptr()
            }
        }

        unsafe impl<const STATIC: bool> Send for $name<STATIC> {}
        unsafe impl<const STATIC: bool> Sync for $name<STATIC> {}
    };
}

impl_common_member_id!(FieldID, sys::jfieldID);
impl_common_member_id!(MethodID, sys::jmethodID);

impl<'vm> JNIEnv<'vm> {
    /// Finds a class by name.
    pub fn find_class(&self, name: impl AsRef<CStr>) -> Result<LocalRef<'_>, LocalRef<'_>> {
        let cls = self.run_catch(|| unsafe { call!(self.as_raw_ptr(), FindClass, name.as_ref().as_ptr()) })?;

        unsafe { Ok(LocalRef::from_raw(self, cls)) }
    }

    /// Finds a method ID by name and signature.
    ///
    /// If `STATIC` is `true`, the method is static. Otherwise, the method is instance.
    ///
    /// # Safety
    ///
    /// `cls` must be a valid class.
    pub unsafe fn get_method_id<const STATIC: bool, R: StrongRef>(
        &self,
        cls: &R,
        name: impl AsRef<CStr>,
        sig: impl AsRef<CStr>,
    ) -> Result<MethodID<STATIC>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        let id = self.run_catch(|| unsafe {
            if STATIC {
                call!(
                    self.as_raw_ptr(),
                    GetStaticMethodID,
                    cls.as_raw_ptr(),
                    name.as_ref().as_ptr(),
                    sig.as_ref().as_ptr()
                )
            } else {
                call!(
                    self.as_raw_ptr(),
                    GetMethodID,
                    cls.as_raw_ptr(),
                    name.as_ref().as_ptr(),
                    sig.as_ref().as_ptr()
                )
            }
        })?;

        unsafe { Ok(<MethodID<STATIC>>::from_raw(id)) }
    }

    /// Finds a field ID by name and signature.
    ///
    /// If `STATIC` is `true`, the field is static. Otherwise, the field is instance.
    ///
    /// # Safety
    ///
    /// `cls` must be a valid class.
    pub unsafe fn get_field_id<const STATIC: bool, R: StrongRef>(
        &self,
        cls: &R,
        name: impl AsRef<CStr>,
        sig: impl AsRef<CStr>,
    ) -> Result<FieldID<STATIC>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        let id = self.run_catch(|| unsafe {
            if STATIC {
                call!(
                    self.as_raw_ptr(),
                    GetStaticFieldID,
                    cls.as_raw_ptr(),
                    name.as_ref().as_ptr(),
                    sig.as_ref().as_ptr()
                )
            } else {
                call!(
                    self.as_raw_ptr(),
                    GetFieldID,
                    cls.as_raw_ptr(),
                    name.as_ref().as_ptr(),
                    sig.as_ref().as_ptr()
                )
            }
        })?;

        unsafe { Ok(<FieldID<STATIC>>::from_raw(id)) }
    }
}
