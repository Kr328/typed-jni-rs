use core::ops::Deref;

use crate::{JNIEnv, LocalRef, StrongRef, helper::call};

#[cfg(feature = "alloc")]
impl<'vm> JNIEnv<'vm> {
    /// Create new string in jvm.
    pub fn new_string(&self, s: impl AsRef<str>) -> LocalRef<'_> {
        unsafe {
            let u16s = s.as_ref().encode_utf16().collect::<alloc::vec::Vec<_>>();

            let obj = self
                .run_catch(|| call!(self.as_raw_ptr(), NewString, u16s.as_ptr(), u16s.len() as _))
                .expect("BROKEN: Jvm throws exception while creating new string.");

            LocalRef::from_raw(self, obj)
        }
    }

    /// Get string from jvm.
    ///
    /// # Safety
    ///
    /// The `s` must be a valid string object.
    pub unsafe fn get_string<R: StrongRef>(&self, s: &R) -> alloc::string::String {
        #[cfg(debug_assertions)]
        s.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            let obj = s.as_raw_ptr();

            let length = call!(self.as_raw_ptr(), GetStringLength, obj);
            let ptr = call!(self.as_raw_ptr(), GetStringChars, obj, core::ptr::null_mut());

            let ret = alloc::string::String::from_utf16(core::slice::from_raw_parts(ptr, length as _))
                .expect("BROKEN: Jvm returns invalid UTF-16 string.");

            call!(self.as_raw_ptr(), ReleaseStringChars, obj, ptr);

            ret
        })
        .expect("BROKEN: Jvm throws exception while getting string.")
    }
}

/// A guard that releases the modified UTF-8 string when dropped.
pub struct ModifiedUTF8StrGuard<'a, R: StrongRef> {
    env: &'a JNIEnv<'a>,
    obj: &'a R,
    ptr: *const u8,
    len: i32,
}

impl<'a, R: StrongRef> Deref for ModifiedUTF8StrGuard<'a, R> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len as _) }
    }
}

impl<'a, R: StrongRef> Drop for ModifiedUTF8StrGuard<'a, R> {
    fn drop(&mut self) {
        unsafe {
            self.env
                .run_catch(|| {
                    call!(
                        self.env.as_raw_ptr(),
                        ReleaseStringUTFChars,
                        self.obj.as_raw_ptr(),
                        self.ptr as _
                    )
                })
                .expect("BROKEN: Jvm throws exception while releasing string.");
        }
    }
}

impl<'vm> JNIEnv<'vm> {
    /// Create new modified UTF-8 string in jvm.
    pub fn new_modified_utf8_string(&self, s: impl AsRef<[u8]>) -> Result<LocalRef<'_>, LocalRef<'_>> {
        assert!(s.as_ref().ends_with(b"\0"), "Modified UTF-8 string must be null-terminated.");

        unsafe {
            let obj = self.run_catch(|| call!(self.as_raw_ptr(), NewStringUTF, s.as_ref().as_ptr() as _))?;

            Ok(LocalRef::from_raw(self, obj))
        }
    }

    /// Get modified UTF-8 string from jvm.
    ///
    /// # Safety
    ///
    /// The `s` must be a valid string object.
    pub unsafe fn get_modified_utf8_string<'a, R: StrongRef>(&'a self, s: &'a R) -> ModifiedUTF8StrGuard<'a, R> {
        #[cfg(debug_assertions)]
        s.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            let obj = s.as_raw_ptr();

            let ptr = call!(self.as_raw_ptr(), GetStringUTFChars, obj, core::ptr::null_mut());
            let len = call!(self.as_raw_ptr(), GetStringUTFLength, obj);

            ModifiedUTF8StrGuard {
                env: self,
                obj: s,
                ptr: ptr as _,
                len,
            }
        })
        .expect("BROKEN: JVM throws exception while getting modified UTF-8 string.")
    }
}
