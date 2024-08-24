use core::{marker::PhantomData, ptr};

use crate::{sys::jobject, FromRaw, IntoRaw, Raw, Ref};

#[repr(transparent)]
pub struct Nullable<T: Raw + FromRaw + IntoRaw>
where
    T::Raw: Ref,
{
    typ: PhantomData<T>,
    ptr: jobject,
}

impl<T: Raw + FromRaw + IntoRaw> Nullable<T>
where
    T::Raw: Ref,
{
    pub fn null() -> Self {
        Self {
            typ: PhantomData,
            ptr: ptr::null_mut(),
        }
    }

    pub fn value(v: T) -> Self {
        Self {
            typ: PhantomData,
            ptr: v.into_raw().into_raw(),
        }
    }

    pub fn into_inner(self) -> Option<T> {
        if self.ptr.is_null() {
            None
        } else {
            unsafe { Some(T::from_raw(T::Raw::from_raw(self.ptr))) }
        }
    }
}
