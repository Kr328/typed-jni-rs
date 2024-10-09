use core::{marker::PhantomData, ptr::NonNull};

use crate::{
    context::Context,
    sys::{_jobject, jobject},
    AsRaw, FromRaw, IntoRaw, Raw,
};

mod __sealed {
    pub trait Sealed {}
}

pub trait Ref: Raw<Raw = jobject> + AsRaw + IntoRaw + Sized + __sealed::Sealed {
    const KIND: &'static str;
}

impl<R: Ref> Raw for R {
    type Raw = jobject;
}

pub trait StrongRef: Ref {
    fn to_global(&self) -> Global {
        Context::with_attached(|ctx| unsafe { Global::from_raw(ctx.new_global_ref(*self.as_raw())) })
    }

    fn to_local<'ctx>(&self, ctx: &'ctx Context) -> Local<'ctx> {
        unsafe { Local::from_raw(ctx.new_local_ref(*self.as_raw())) }
    }

    fn downgrade_weak(&self) -> Weak {
        Context::with_attached(|ctx| unsafe { Weak::from_raw(ctx.new_weak_global_ref(*self.as_raw())) })
    }
}

pub trait WeakRef: Ref {
    fn upgrade_global(&self) -> Option<Global> {
        Context::with_attached(|ctx| unsafe {
            let raw = ctx.new_global_ref(*self.as_raw());

            if raw.is_null() {
                None
            } else {
                Some(Global::from_raw(raw))
            }
        })
    }

    fn upgrade_local<'ctx>(&self, ctx: &'ctx Context) -> Option<Local<'ctx>> {
        unsafe {
            let raw = ctx.new_local_ref(*self.as_raw());

            if raw.is_null() {
                None
            } else {
                Some(Local::from_raw(raw))
            }
        }
    }
}

#[repr(transparent)]
pub struct Global {
    raw: NonNull<_jobject>,
}

unsafe impl Send for Global {}
unsafe impl Sync for Global {}

impl Clone for Global {
    fn clone(&self) -> Self {
        unsafe {
            Context::with_attached(|ctx| Self {
                raw: NonNull::new(ctx.new_global_ref(self.raw.as_ptr())).unwrap(),
            })
        }
    }
}

impl Drop for Global {
    fn drop(&mut self) {
        unsafe { Context::with_attached(|ctx| ctx.delete_global_ref(self.raw.as_ptr())) }
    }
}

impl __sealed::Sealed for Global {}

impl AsRaw for Global {
    fn as_raw(&self) -> &Self::Raw {
        unsafe { core::mem::transmute(&self.raw) }
    }
}

impl FromRaw for Global {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap(),
        }
    }
}

impl IntoRaw for Global {
    fn into_raw(self) -> Self::Raw {
        let r = self.raw;

        core::mem::forget(self);

        r.as_ptr()
    }
}

impl Ref for Global {
    const KIND: &'static str = "Global";
}

impl StrongRef for Global {}

#[repr(transparent)]
pub struct Local<'ctx> {
    raw: NonNull<_jobject>,
    _ctx: PhantomData<&'ctx Context>,
}

impl<'ctx> Clone for Local<'ctx> {
    fn clone(&self) -> Self {
        let raw = Context::with_current(|ctx| unsafe { ctx.new_local_ref(self.raw.as_ptr()) }).unwrap();

        Local {
            raw: NonNull::new(raw).unwrap(),
            _ctx: PhantomData,
        }
    }
}

impl<'ctx> Drop for Local<'ctx> {
    fn drop(&mut self) {
        Context::with_current(|ctx| unsafe { ctx.delete_local_ref(self.raw.as_ptr()) }).unwrap();
    }
}

impl<'ctx> AsRaw for Local<'ctx> {
    fn as_raw(&self) -> &Self::Raw {
        unsafe { core::mem::transmute(&self.raw) }
    }
}

impl<'ctx> FromRaw for Local<'ctx> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap(),
            _ctx: PhantomData,
        }
    }
}

impl<'ctx> IntoRaw for Local<'ctx> {
    fn into_raw(self) -> Self::Raw {
        let r = self.raw;

        core::mem::forget(self);

        r.as_ptr()
    }
}

impl<'ctx> Ref for Local<'ctx> {
    const KIND: &'static str = "Local";
}

impl<'ctx> __sealed::Sealed for Local<'ctx> {}

impl<'ctx> StrongRef for Local<'ctx> {}

#[repr(transparent)]
pub struct Trampoline<'ctx> {
    raw: NonNull<_jobject>,
    _ctx: PhantomData<&'ctx Context>,
}

impl<'ctx> Ref for Trampoline<'ctx> {
    const KIND: &'static str = "Trampoline";
}

impl<'ctx> StrongRef for Trampoline<'ctx> {}

impl<'ctx> AsRaw for Trampoline<'ctx> {
    fn as_raw(&self) -> &Self::Raw {
        unsafe { core::mem::transmute(&self.raw) }
    }
}

impl<'ctx> IntoRaw for Trampoline<'ctx> {
    fn into_raw(self) -> Self::Raw {
        self.raw.as_ptr()
    }
}

impl<'ctx> __sealed::Sealed for Trampoline<'ctx> {}

#[repr(transparent)]
pub struct Weak {
    raw: NonNull<_jobject>,
}

unsafe impl Send for Weak {}
unsafe impl Sync for Weak {}

impl Clone for Weak {
    fn clone(&self) -> Self {
        unsafe {
            Context::with_attached(|ctx| Self {
                raw: NonNull::new(ctx.new_weak_global_ref(self.raw.as_ptr())).unwrap(),
            })
        }
    }
}

impl Drop for Weak {
    fn drop(&mut self) {
        unsafe { Context::with_attached(|ctx| ctx.delete_weak_global_ref(self.raw.as_ptr())) }
    }
}

impl AsRaw for Weak {
    fn as_raw(&self) -> &Self::Raw {
        unsafe { core::mem::transmute(&self.raw) }
    }
}

impl FromRaw for Weak {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap(),
        }
    }
}

impl IntoRaw for Weak {
    fn into_raw(self) -> Self::Raw {
        let r = self.raw;

        core::mem::forget(self);

        r.as_ptr()
    }
}

impl Ref for Weak {
    const KIND: &'static str = "Weak";
}

impl __sealed::Sealed for Weak {}

impl WeakRef for Weak {}
