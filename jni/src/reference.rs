use core::marker::PhantomData;

use crate::{
    context::Context,
    sys::{jobject, jweak},
};

mod __sealed {
    pub trait Sealed {}
}

pub trait Ref: Sized + Clone + __sealed::Sealed {
    const KIND: &'static str;

    fn as_raw(&self) -> jobject;
}

pub trait StrongRef: Ref {
    fn to_global(&self) -> Global {
        Context::with_attached(|ctx| unsafe { Global::from_raw(ctx.new_global_ref(self.as_raw())) })
    }

    fn to_local<'ctx>(&self, ctx: &'ctx Context) -> Local<'ctx> {
        unsafe { Local::from_raw(ctx, ctx.new_local_ref(self.as_raw())) }
    }

    fn downgrade_weak(&self) -> Weak {
        Context::with_attached(|ctx| unsafe { Weak::from_raw(ctx.new_weak_global_ref(self.as_raw())) })
    }
}

pub trait WeakRef: Ref {
    fn upgrade_global(&self) -> Option<Global> {
        Context::with_attached(|ctx| unsafe {
            let raw = ctx.new_global_ref(self.as_raw());

            if raw.is_null() {
                None
            } else {
                Some(Global::from_raw(raw))
            }
        })
    }

    fn upgrade_local<'ctx>(&self, ctx: &'ctx Context) -> Option<Local<'ctx>> {
        unsafe {
            let raw = ctx.new_local_ref(self.as_raw());

            if raw.is_null() {
                None
            } else {
                Some(Local::from_raw(ctx, raw))
            }
        }
    }
}

#[repr(transparent)]
pub struct Global {
    raw: jobject,
}

unsafe impl Send for Global {}
unsafe impl Sync for Global {}

impl Global {
    pub unsafe fn from_raw(raw: jobject) -> Self {
        Self { raw }
    }
}

impl Clone for Global {
    fn clone(&self) -> Self {
        unsafe {
            Context::with_attached(|ctx| Self {
                raw: ctx.new_global_ref(self.raw),
            })
        }
    }
}

impl Drop for Global {
    fn drop(&mut self) {
        unsafe { Context::with_attached(|ctx| ctx.delete_global_ref(self.raw)) }
    }
}

impl __sealed::Sealed for Global {}

impl Ref for Global {
    const KIND: &'static str = "Global";

    fn as_raw(&self) -> jobject {
        self.raw
    }
}

impl StrongRef for Global {}

#[repr(transparent)]
pub struct Local<'ctx> {
    raw: jobject,
    _ctx: PhantomData<&'ctx Context>,
}

impl<'ctx> Local<'ctx> {
    pub unsafe fn from_raw(_ctx: &'ctx Context, raw: jobject) -> Self {
        Self { raw, _ctx: PhantomData }
    }
}

impl<'ctx> Clone for Local<'ctx> {
    fn clone(&self) -> Self {
        Local {
            raw: Context::with_current(|ctx| unsafe { ctx.new_local_ref(self.raw) }).unwrap(),
            _ctx: PhantomData,
        }
    }
}

impl<'ctx> Drop for Local<'ctx> {
    fn drop(&mut self) {
        Context::with_current(|ctx| unsafe { ctx.delete_local_ref(self.raw) }).unwrap();
    }
}

impl<'ctx> Ref for Local<'ctx> {
    const KIND: &'static str = "Local";

    fn as_raw(&self) -> jobject {
        self.raw
    }
}

impl<'ctx> __sealed::Sealed for Local<'ctx> {}

impl<'ctx> StrongRef for Local<'ctx> {}

#[repr(transparent)]
pub struct Weak {
    raw: jweak,
}

unsafe impl Send for Weak {}
unsafe impl Sync for Weak {}

impl Weak {
    pub unsafe fn from_raw(raw: jweak) -> Self {
        Self { raw }
    }
}

impl Clone for Weak {
    fn clone(&self) -> Self {
        unsafe {
            Context::with_attached(|ctx| Self {
                raw: ctx.new_weak_global_ref(self.raw),
            })
        }
    }
}

impl Drop for Weak {
    fn drop(&mut self) {
        unsafe { Context::with_attached(|ctx| ctx.delete_weak_global_ref(self.raw)) }
    }
}

impl Ref for Weak {
    const KIND: &'static str = "Weak";

    fn as_raw(&self) -> jobject {
        self.raw
    }
}

impl __sealed::Sealed for Weak {}

impl WeakRef for Weak {}
