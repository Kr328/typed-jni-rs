use core::{fmt::Debug, marker::PhantomData, ptr::NonNull};

use crate::{
    raw::{AsRaw, FromRaw, IntoRaw, Raw},
    sys::{_jobject, jobject},
    vm,
};

mod __sealed {
    pub trait Sealed {}
}

pub trait Ref: Raw<Raw = jobject> + AsRaw + IntoRaw + Sized + Debug + __sealed::Sealed {
    const KIND: &'static str;
}

impl<R: Ref> Raw for R {
    type Raw = jobject;
}

pub trait StrongRef: Ref {}

pub trait WeakRef: Ref {}

macro_rules! impl_basic_ref {
    ($typ:ty, $k:expr) => {
        impl<'ctx> __sealed::Sealed for $typ {}

        impl<'ctx> Ref for $typ {
            const KIND: &'static str = $k;
        }

        impl<'ctx> AsRaw for $typ {
            fn as_raw(&self) -> &Self::Raw {
                unsafe { core::mem::transmute(&self.raw) }
            }
        }

        impl<'ctx> IntoRaw for $typ {
            fn into_raw(self) -> Self::Raw {
                let r = self.raw;

                core::mem::forget(self);

                r.as_ptr()
            }
        }
    };
}

#[derive(Debug)]
pub struct Global {
    raw: NonNull<_jobject>,
}

impl_basic_ref!(Global, "Global");

impl StrongRef for Global {}

unsafe impl Send for Global {}
unsafe impl Sync for Global {}

impl FromRaw for Global {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap(),
        }
    }
}

impl Clone for Global {
    fn clone(&self) -> Self {
        unsafe {
            let attached = vm::attach();

            Self::from_raw((**attached.env()).NewGlobalRef.unwrap()(attached.env(), self.raw.as_ptr()))
        }
    }
}

impl Drop for Global {
    fn drop(&mut self) {
        unsafe {
            let attached = vm::attach();

            (**attached.env()).DeleteGlobalRef.unwrap()(attached.env(), self.raw.as_ptr());
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Local<'ctx> {
    raw: NonNull<_jobject>,
    _ctx: PhantomData<&'ctx ()>,
}

impl_basic_ref!(Local<'ctx>, "Local");

impl<'ctx> StrongRef for Local<'ctx> {}

impl<'ctx> FromRaw for Local<'ctx> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap(),
            _ctx: PhantomData,
        }
    }
}

impl<'ctx> Clone for Local<'ctx> {
    fn clone(&self) -> Self {
        unsafe {
            let env = vm::current().expect("local ref must be cloned in attached thread");

            Self::from_raw((**env).NewLocalRef.unwrap()(env, self.raw.as_ptr()))
        }
    }
}

impl<'ctx> Drop for Local<'ctx> {
    fn drop(&mut self) {
        unsafe {
            if let Some(env) = vm::current() {
                (**env).DeleteLocalRef.unwrap()(env, self.raw.as_ptr());
            }
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Trampoline<'ctx> {
    raw: NonNull<_jobject>,
    _ctx: PhantomData<&'ctx ()>,
}

impl_basic_ref!(Trampoline<'ctx>, "Trampoline");

impl<'ctx> StrongRef for Trampoline<'ctx> {}

#[derive(Debug)]
#[repr(transparent)]
pub struct Weak {
    raw: NonNull<_jobject>,
}

impl_basic_ref!(Weak, "Weak");

impl WeakRef for Weak {}

unsafe impl Send for Weak {}
unsafe impl Sync for Weak {}

impl FromRaw for Weak {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap(),
        }
    }
}

impl Drop for Weak {
    fn drop(&mut self) {
        unsafe {
            let attached = vm::attach();

            (**attached.env()).DeleteWeakGlobalRef.unwrap()(attached.env(), self.raw.as_ptr());
        }
    }
}
