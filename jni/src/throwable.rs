use alloc::string::{String, ToString};
use core::{
    fmt::{Debug, Display, Formatter},
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    context::{Context, Method},
    reference::StrongRef,
    sys::_jmethodID,
    FromRaw, IntoRaw, Ref, RefConvert, Signature, Type, Value,
};

#[repr(transparent)]
pub struct JavaThrowable<const STATIC: bool, R: Ref>(R);
impl<const STATIC: bool, R: Ref> RefConvert for JavaThrowable<STATIC, R> {
    type Ref = R;
    type Converted<RR: Ref> = JavaThrowable<STATIC, RR>;

    fn as_raw_ref(&self) -> &Self::Ref {
        &self.0
    }

    unsafe fn from_raw_ref<RR: Ref>(r: RR) -> Self::Converted<RR> {
        JavaThrowable(r)
    }
}

impl<const STATIC: bool, R: Ref> Value for JavaThrowable<STATIC, R> {
    type Raw = R;
}

impl<const STATIC: bool, R: Ref> FromRaw for JavaThrowable<STATIC, R> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self(raw)
    }
}

impl<const STATIC: bool, R: Ref> IntoRaw for JavaThrowable<STATIC, R> {
    fn into_raw(self) -> Self::Raw {
        self.0
    }
}

impl<'a, const STATIC: bool, R: Ref> Value for &'a JavaThrowable<STATIC, R> {
    type Raw = &'a R;
}

impl<'a, const STATIC: bool, R: Ref> IntoRaw for &'a JavaThrowable<STATIC, R> {
    fn into_raw(self) -> Self::Raw {
        &self.0
    }
}

impl<R: Ref> Type for JavaThrowable<false, R> {
    const SIGNATURE: Signature = Signature::Object("java/lang/Throwable");
}

impl<R: Ref> Type for JavaThrowable<true, R> {
    const SIGNATURE: Signature = Signature::Object("java/lang/Class");
}

impl<const STATIC: bool, R: StrongRef> crate::This<STATIC> for JavaThrowable<STATIC, R> {
    type Ref = R;

    fn as_ref(&self) -> &Self::Ref {
        &self.0
    }
}

impl<R: StrongRef> crate::Class for JavaThrowable<true, R> {
    type Object<RR: Ref> = JavaThrowable<false, RR>;
}

impl<const STATIC: bool, R: StrongRef> JavaThrowable<STATIC, R> {
    pub fn to_string(&self, ctx: &Context) -> String {
        static M_TO_STRING: AtomicPtr<_jmethodID> = AtomicPtr::new(null_mut());
        let m_to_string = M_TO_STRING.load(Ordering::Relaxed);
        let m_to_string = if m_to_string.is_null() {
            match ctx
                .find_class(c"java/lang/Throwable")
                .and_then(|c| ctx.find_method(&c, c"toString", c"()Ljava/lang/String;"))
            {
                Ok(m) => {
                    M_TO_STRING.store(m.as_raw(), Ordering::Relaxed);

                    m
                }
                Err(_) => panic!("BROKEN: find throwable failed"),
            }
        } else {
            unsafe { Method::<false>::from_raw(m_to_string) }
        };

        unsafe {
            ctx.call_method(self.as_raw_ref(), m_to_string, [])
                .ok()
                .flatten()
                .map(|s| ctx.get_string(&s))
                .unwrap_or("unknown".to_string())
        }
    }
}

impl<const STATIC: bool, R: StrongRef> Debug for JavaThrowable<STATIC, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> alloc::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<const STATIC: bool, R: StrongRef> Display for JavaThrowable<STATIC, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> alloc::fmt::Result {
        Context::with_attached(|ctx| f.write_str(&JavaThrowable::to_string(self, ctx)))
    }
}

#[cfg(feature = "std")]
impl<const STATIC: bool, R: StrongRef> std::error::Error for JavaThrowable<STATIC, R> {}
