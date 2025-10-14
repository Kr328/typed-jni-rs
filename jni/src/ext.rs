use alloc::string::String;
use core::{
    ffi::CStr,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{AsRaw, Context, FromRaw, Local, Method, StrongRef, sys::_jmethodID};

pub trait ContextExt {
    fn new_null_pointer_exception(&self) -> Result<Local<'_>, Local<'_>>;
    fn new_class_cast_exception(&self) -> Result<Local<'_>, Local<'_>>;
}

macro_rules! impl_new_exception {
    ($cls:expr, $ctx:expr) => {
        unsafe {
            let cls = $ctx.find_class($cls)?;
            let method = $ctx.find_method(&cls, c"init", c"()V")?;

            $ctx.new_object(&cls, method, [])
        }
    };
}

impl ContextExt for Context {
    fn new_null_pointer_exception(&self) -> Result<Local<'_>, Local<'_>> {
        impl_new_exception!(c"java/lang/NullPointerException", self)
    }

    fn new_class_cast_exception(&self) -> Result<Local<'_>, Local<'_>> {
        impl_new_exception!(c"java/lang/ClassCastException", self)
    }
}

struct LazyMethod(AtomicPtr<_jmethodID>);

impl LazyMethod {
    pub const fn new() -> Self {
        Self(AtomicPtr::new(core::ptr::null_mut()))
    }

    pub fn get_or_init(&self, ctx: &Context, cls: &CStr, name: &CStr, sig: &CStr) -> Method<false> {
        let m = self.0.load(Ordering::Relaxed);
        if m.is_null() {
            let m = ctx
                .find_class(cls)
                .and_then(|c| ctx.find_method(&c, name, sig))
                .unwrap_or_else(|_| {
                    panic!(
                        "BROKEN: find {}.{}({}) failed",
                        cls.to_str().unwrap(),
                        name.to_str().unwrap(),
                        sig.to_str().unwrap()
                    )
                });
            self.0.store(*m.as_raw(), Ordering::Relaxed);
            m
        } else {
            unsafe { Method::<false>::from_raw(m) }
        }
    }
}

pub trait StrongRefExt: StrongRef {
    fn to_string<'ctx>(&self, ctx: &'ctx Context) -> Result<String, Local<'ctx>> {
        static M_TO_STRING: LazyMethod = LazyMethod::new();
        let m_to_string = M_TO_STRING.get_or_init(ctx, c"java/lang/Object", c"toString", c"()Ljava/lang/String;");

        unsafe {
            let s: Option<Local> = ctx.call_method(self, m_to_string, [])?;

            match s {
                Some(s) => Ok(ctx.get_string(&s)),
                None => Err(ctx.new_null_pointer_exception()?),
            }
        }
    }

    fn hash_code<'ctx>(&self, ctx: &'ctx Context) -> Result<i32, Local<'ctx>> {
        static M_HASH_CODE: LazyMethod = LazyMethod::new();
        let m_hash_code = M_HASH_CODE.get_or_init(ctx, c"java/lang/Object", c"hashCode", c"()I");

        unsafe { ctx.call_method(self, m_hash_code, []) }
    }
}

impl<R: StrongRef> StrongRefExt for R {}
